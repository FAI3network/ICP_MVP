use crate::errors::GenericError;
use crate::hugging_face::call_hugging_face;
use crate::inference_providers::lib::HuggingFaceRequestParameters;
use crate::job_management::{internal_job_complete, internal_job_fail, internal_job_in_progress, create_job_with_job_type, bootstrap_job_queue, job_should_be_stopped, internal_job_stop};
use crate::types::{
    get_llm_model_data, LanguageEvaluationDataPoint, LanguageEvaluationMetrics,
    LanguageEvaluationResult, ModelType, Job, JobType, HuggingFaceConfig,
};
use crate::utils::{is_owner, seeded_vector_shuffle};
use crate::MODELS;
use crate::{
    check_cycles_before_action, get_model_from_memory, only_admin, NEXT_LLM_LANGUAGE_EVALUATION_ID,
};
use crate::config_management::{
    HUGGING_FACE_API_KEY_CONFIG_KEY,
    internal_get_config,
};
use candid::CandidType;
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

const KALEIDOSKOPE_CSV: &str = include_str!("data/kaleidoscope.csv");

const SYSTEM_PROMPT: &str =
    "You are a helpful assistant who answers multiple-choice questions. For each question,
output your final answer in JSON format with the following structure: {\"choice\":
\"The correct option\"}. ONLY output this format exactly. Do
not include any additional text or explanations outside the JSON structure.";

fn build_prompt(question: &String, options: &Vec<String>, seed: u32) -> String {
    let mut prompt = String::with_capacity(
        SYSTEM_PROMPT.len()
            + question.len()
            + options.iter().map(|s| s.len() + 1).sum::<usize>()
            + 4, // For extra newlines
    );

    prompt.push_str(SYSTEM_PROMPT);
    prompt.push_str("\n\n");
    prompt.push_str(question);
    prompt.push_str("\n");

    // Create and shuffle option indices
    let mut option_indices: Vec<usize> = (0..options.len()).collect();
    option_indices = seeded_vector_shuffle(option_indices, seed);

    // Add shuffled options
    for &idx in &option_indices {
        prompt.push_str(&options[idx]);
        prompt.push_str("\n");
    }

    prompt
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageEvaluationAnswer {
    pub choice: String,
}

async fn run_evaluate_languages(
    hf_data: &HuggingFaceConfig,
    language_evaluation: &mut LanguageEvaluationResult,
    job: &Job,
) -> Result<(), String> {
    let overall_metrics = &mut language_evaluation.metrics;
    let seed = language_evaluation.seed;
    let max_queries = language_evaluation.max_queries;
    let current_query = job.progress.completed;
    let hf_parameters = HuggingFaceRequestParameters {
        max_new_tokens: None,
        stop: None,
        temperature: Some(0.3),
        decoder_input_details: Some(false),
        details: Some(false),
        return_full_text: Some(false),
        seed: Some(seed),
        do_sample: Some(false),
    };

    let mut queries: usize = 0; // query counter
    let mut rdr = csv::ReaderBuilder::new().from_reader(KALEIDOSKOPE_CSV.as_bytes());

    for result in rdr.deserialize::<HashMap<String, String>>() {
        let result = result.map_err(|e| e.to_string())?;

        if queries < current_query {
            queries += 1;
            continue;
        }

        let language: &String = result
            .get("language")
            .expect("It should be able to get the language field.");

        // ignore languages that were not scheduled
        if !language_evaluation.languages.contains(&language) {
            continue;
        }

        ic_cdk::println!(
            "Executing query {}/{} with language {}",
            current_query,
            max_queries,
            language
        );

        let question: &String = result
            .get("question")
            .expect("It should be able to get the question field.");
        
        let answer: usize = result
            .get("answer")
            .and_then(|ans| ans.parse::<usize>().ok())
            .expect("Answer field should be a valid usize index");
        
        let options: Vec<String> = result
            .get("options")
            .map(|opt_str| {
                ic_cdk::println!("{}", &opt_str);
                // Split by space and clean up quotes from each element
                opt_str
                    .trim_matches(|c| c == '[' || c == ']')
                    .split('\'')
                    .filter(|s| !s.trim().is_empty() && s.trim() != " ")
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .expect("It should be able to parse the options field.");
        
        let text_answer: String = options
            .get(answer)
            .expect("Answer should exist in the options vector")
            .to_string();
        
        ic_cdk::println!("Valid answer: {}", text_answer);

        let lang_metrics: &mut LanguageEvaluationMetrics = &mut language_evaluation.metrics_per_language
            .iter_mut()
            .find(|lang_tuple| lang_tuple.0 == *language)
            .expect(format!("Value for language {} should exist", &language).as_str())
            .1;

        let prompt: String = build_prompt(&question, &options, seed * (queries as u32));

        let res = call_hugging_face(
            prompt.clone(),
            hf_data.hugging_face_url.clone(),
            seed,
            Some(hf_parameters.clone()),
            &hf_data.inference_provider,
        ).await;

        let trimmed_response = match res {
            Ok(response) => crate::utils::clean_llm_response(&response),
            Err(e) => {
                ic_cdk::println!("Error calling Hugging Face API: {}", e);

                overall_metrics.add_error();
                lang_metrics.add_error();

                language_evaluation.data_points.push(LanguageEvaluationDataPoint {
                    prompt: prompt.clone(),
                    response: None,
                    valid: false,
                    error: true,
                    correct_answer: text_answer.clone(),
                });

                return Ok(());
            }
        };

        ic_cdk::println!("Cleaned response: {}", &trimmed_response);

        let evaluation_answer =  match serde_json::from_str::<LanguageEvaluationAnswer>(&trimmed_response) {
            Ok(json) => {
                ic_cdk::println!("Parsed JSON response: {:?}", &json);
                json
            }
            Err(e) => {
                ic_cdk::println!("Failed to parse JSON: {}. Skipping to next row.", e);
                overall_metrics.add_invalid();
                lang_metrics.add_invalid();

                language_evaluation.data_points.push(LanguageEvaluationDataPoint {
                    prompt: prompt.clone(),
                    response: None,
                    valid: false,
                    error: false,
                    correct_answer: text_answer.clone(),
                });

                return Ok(());
            }
        };

        let llm_answer = evaluation_answer.choice.trim();

        language_evaluation.data_points.push(LanguageEvaluationDataPoint {
            prompt: prompt.clone(),
            response: Some(llm_answer.to_string()),
            valid: false,
            error: false,
            correct_answer: text_answer.clone(),
        });

        if llm_answer.to_lowercase() == text_answer.trim().to_lowercase() {
            overall_metrics.add_correct();
            lang_metrics.add_correct();
        } else {
            // Check if it belongs to any of the options, otherwise it's classified as invalid
            let mut belongs_to_an_option = false;
            for option in &options {
                if llm_answer.to_lowercase() == option.trim().to_lowercase() {
                    belongs_to_an_option = true;
                    break;
                }
            }
            if belongs_to_an_option {
                overall_metrics.add_incorrect();
                lang_metrics.add_incorrect();
            } else {
                overall_metrics.add_invalid();
                lang_metrics.add_invalid();
            }
        }

        
        return Ok(());
    }

    return Ok(());
}

/// Evaluates languages for a LLM. It returns the LanguageEvaluationResult,
/// and it also saves the result into the model data.
#[update]
pub async fn llm_evaluate_languages(
    model_id: u128,
    languages: Vec<String>,
    max_queries: usize,
    seed: u32,
) -> Result<u128, GenericError> {
    only_admin();
    check_cycles_before_action();

    if languages.len() == 0 {
        return Err(GenericError::new(
            GenericError::INVALID_ARGUMENT,
            "You should select at least one language.",
        ));
    }
    let valid_values: HashSet<&str> = [
        "ar", "bn", "de", "en", "es", "fa", "fr", "hi", "hr", "hu", "lt", "nl", "pt", "ru", "sr",
        "uk",
    ]
    .into_iter()
    .collect();
    let all_valid = languages.iter().all(|x| valid_values.contains(x.as_str()));
    if !all_valid {
        return Err(GenericError::new(
            GenericError::INVALID_ARGUMENT,
            "An invalid language was selected.",
        ));
    }

    let caller = ic_cdk::api::caller();

    let model = get_model_from_memory(model_id);
    if let Err(err) = model {
        return Err(err);
    }
    let model = model.unwrap();
    is_owner(&model, caller);

    if let ModelType::LLM(_) = model.model_type {
        let created_job = MODELS.with(|models| {
            let mut models = models.borrow_mut();
            let mut model = models.get(&model_id).expect("Model not found");

            let mut model_data = get_llm_model_data(&model);

            let job_id = NEXT_LLM_LANGUAGE_EVALUATION_ID.with(|id| {
                let mut next_data_point_id = id.borrow_mut();

                let job_queries_target = if max_queries == 0 {
                    let counts = get_language_evaluation_counts();
                    let mut sum = 0;
                    for lang in &languages {
                        sum += counts.per_language.get(lang)
                            .expect(format!("Language {} should exist in counts per language", lang).as_str());
                    }
                    sum
                } else {
                    max_queries
                };

                let job_id = create_job_with_job_type(model_id, JobType::LanguageEvaluation {
                    language_model_evaluation_id: *next_data_point_id.get(),
                }, job_queries_target);

                let mut metrics_per_language: Vec<(String, LanguageEvaluationMetrics)>  = vec![];

                for lang in &languages {
                    metrics_per_language.push((lang.clone(), Default::default()));
                }

                model_data.language_evaluations.push(LanguageEvaluationResult {
                    language_model_evaluation_id: *next_data_point_id.get(),
                    timestamp: ic_cdk::api::time(),
                    languages: languages.clone(),
                    prompt_templates: vec![("overall".to_string(), SYSTEM_PROMPT.to_string())],
                    data_points: Vec::new(),
                    metrics: Default::default(),
                    metrics_per_language,
                    max_queries,
                    seed,
                    finished: false,
                    canceled: false,
                    job_id: Some(job_id),        
                });

                let current_id = *next_data_point_id.get();
                next_data_point_id.set(current_id + 1).unwrap();

                return job_id;
            });

            model.model_type = ModelType::LLM(model_data);
            models.insert(model_id, model);
            
            return job_id;
        });

        bootstrap_job_queue();

        return Ok(created_job);
            
    } else {
        return Err(GenericError::new(
            GenericError::INVALID_MODEL_TYPE,
            "Model should be a LLM",
        ));
    }
}

pub async fn process_next_query(llm_model_id: u128, language_model_evaluation_id: u128, job: &Job) -> Result<bool, String> {
    ic_cdk::println!("Processing llm language evaluation query");
    let model = get_model_from_memory(llm_model_id);
    if let Err(err) = model {
        return Err(err.to_string());
    }
    let model = model.unwrap();

    let mut model_data = get_llm_model_data(&model);

    let hf_data = HuggingFaceConfig {
        hugging_face_url: model_data.hugging_face_url.clone(),
        inference_provider: model_data.inference_provider.clone(),
    };

    // Get evaluation from this model
    let language_evaluation_index = model_data.language_evaluations.iter().position(|evaluation| 
        evaluation.language_model_evaluation_id == language_model_evaluation_id
    );

    if language_evaluation_index.is_none() {
        let error = "Invalid index for language evaluation";
        ic_cdk::eprintln!("Error: {}", error);
        internal_job_fail(job.id, llm_model_id, Some(error.to_string()));
        return Ok(true);
    }
    let language_evaluation_index = language_evaluation_index.unwrap();
    
    let mut language_evaluation = model_data.language_evaluations
        .iter_mut()
        .find(|e| e.language_model_evaluation_id == language_model_evaluation_id)
        .expect("Language evaluation should exist");

    // Check if the job is already finished
    if language_evaluation.finished || language_evaluation.canceled {
        ic_cdk::println!("Language evaluation already finished. Exiting...");
        return Ok(true); // Nothing more to do
    }

    if job_should_be_stopped(job.id) {
        ic_cdk::eprintln!("Job has been stopped while running. Marking LanguageEvaluationResult as finished and cancelled.");
        internal_job_stop(job.id, job.model_id);
        
        MODELS.with(|models| {
            let mut models = models.borrow_mut();
            let mut model = models.get(&llm_model_id).expect("Model not found");
            let mut model_data = get_llm_model_data(&model);

            model_data.language_evaluations[language_evaluation_index].canceled = true;
            model_data.language_evaluations[language_evaluation_index].finished = true;
            
            model.model_type = ModelType::LLM(model_data);
            models.insert(llm_model_id, model);
        });
        
        return Ok(true);
    }

    // Get API key for Hugging Face
    let _api_key = internal_get_config(HUGGING_FACE_API_KEY_CONFIG_KEY.to_string())?;

    // checks if the run has finished and set it as complete
    if job.progress.completed >= job.progress.target {
        // Job is complete
        MODELS.with(|models| {
            let mut models = models.borrow_mut();
            let mut model = models.get(&llm_model_id).expect("Model not found");
            let mut model_data = get_llm_model_data(&model);

            let evaluation = &mut model_data.language_evaluations[language_evaluation_index];
            evaluation.finished = true;

            // calculating metrics
            evaluation.metrics.calculate_rates();
            
            for (_, lang_metrics) in &mut evaluation.metrics_per_language {
                let metrics: &mut LanguageEvaluationMetrics = lang_metrics;
                metrics.calculate_rates();
            }

            model.model_type = ModelType::LLM(model_data);
            models.insert(llm_model_id, model);
        });

        internal_job_complete(job.id, llm_model_id);

        ic_cdk::println!("Job {} is complete.", job.id);
        
        return Ok(true);
    }
    

    let result =
        run_evaluate_languages(&hf_data, &mut language_evaluation, &job).await;

    match result {
        Err(err_str) => {
            // A grave error happened. Setting the job as failed
            ic_cdk::eprintln!("An error happened: '{}'. Stopping job.", &err_str);
            internal_job_fail(job.id, llm_model_id, Some(err_str));

            // setting evaluation as canceled and finished
            MODELS.with(|models| {
                let mut models = models.borrow_mut();
                let mut model = models.get(&llm_model_id).expect("Model not found");
                let mut model_data = get_llm_model_data(&model);

                let evaluation = &mut model_data.language_evaluations[language_evaluation_index];
                evaluation.finished = true;
                evaluation.canceled = true;

                model.model_type = ModelType::LLM(model_data);
                models.insert(llm_model_id, model);
            });

            return Ok(true);
        },
        Ok(()) => {
            let invalid_responses = language_evaluation.metrics.invalid_responses;
            let errors = language_evaluation.metrics.error_count;
            
            // saving modified data
           MODELS.with(|models| {
                let mut models = models.borrow_mut();
                let mut model = models.get(&llm_model_id).expect("Model not found");
                let mut model_data = get_llm_model_data(&model);

               model_data.language_evaluations[language_evaluation_index] = language_evaluation.clone();

                model.model_type = ModelType::LLM(model_data);
                models.insert(llm_model_id, model);
           });

            ic_cdk::println!("Saving progress: {}", job.progress.completed + 1);
            
            internal_job_in_progress(job.id, llm_model_id,
                                     job.progress.completed + 1,
                                     invalid_responses as usize,
                                     errors as usize);

            return Ok(false);
        },
    };
}

#[query]
pub async fn get_language_evaluation_data_points(
    llm_model_id: u128,
    language_evaluation_id: u128,
    limit: u32,
    offset: usize,
) -> Result<(Vec<LanguageEvaluationDataPoint>, usize), GenericError> {
    only_admin();
    check_cycles_before_action();

    let caller = ic_cdk::api::caller();

    // Check the model exists and is a LLM
    let model = get_model_from_memory(llm_model_id);
    if let Err(err) = model {
        return Err(err);
    }
    let model = model.unwrap();
    is_owner(&model, caller);

    if let ModelType::LLM(model_data) = model.model_type {
        let language_evaluation = model_data
            .language_evaluations
            .into_iter()
            .find(|le: &LanguageEvaluationResult| {
                le.language_model_evaluation_id == language_evaluation_id
            })
            .expect("Context association test with passed index should exist.");

        let data_points: &Vec<LanguageEvaluationDataPoint> =
            language_evaluation.data_points.as_ref();

        let data_points_total_length = data_points.len();

        // Get a slice of data points based on offset and limit
        let start = offset;
        let end = (offset + limit as usize).min(data_points.len());

        // Clone the selected range of data points
        let data_points = data_points[start..end].to_vec();

        return Ok((data_points, data_points_total_length));
    } else {
        return Err(GenericError::new(
            GenericError::INVALID_MODEL_TYPE,
            "Model should be an LLM.",
        ));
    }
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
pub struct LanguageEvaluationCounts {
    pub total_count: usize,
    pub per_language: HashMap<String, usize>,
}

/// Returns the number of elements in Language Evaluations, total and per language
/// # Returns
/// * `LanguageEvaluationCounts` - Total count and counts per language
#[query]
pub fn get_language_evaluation_counts() -> LanguageEvaluationCounts {
    let mut rdr = csv::ReaderBuilder::new().from_reader(KALEIDOSKOPE_CSV.as_bytes());

    let mut per_language = HashMap::new();
    let mut total_count = 0;

    for result in rdr.deserialize::<HashMap<String, String>>() {
        if let Ok(record) = result {
            if let Some(language) = record.get("language") {
                *per_language.entry(language.clone()).or_insert(0) += 1;
                total_count += 1;
            }
        }
    }

    LanguageEvaluationCounts {
        total_count,
        per_language,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        // Test inputs
        let question = String::from("What is the capital of France?");
        let options = vec![
            String::from("A) Paris"),
            String::from("B) London"),
            String::from("C) Berlin"),
            String::from("D) Madrid"),
        ];
        let seed = 42;

        // Build the prompt
        let result = build_prompt(&question, &options, seed);

        // Verify the prompt contains all required elements
        assert!(result.contains(SYSTEM_PROMPT));
        assert!(result.contains("What is the capital of France?"));

        // Verify all options are present
        for option in options.iter() {
            assert!(result.contains(option));
        }

        // Verify basic structure (contains newlines between sections)
        assert!(result.contains("\n\n"));
    }
}
