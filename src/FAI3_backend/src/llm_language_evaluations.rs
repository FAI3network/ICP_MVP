use ic_cdk_macros::*;
use crate::hugging_face::{call_hugging_face, HuggingFaceRequestParameters};
use crate::types::{LanguageEvaluationResult, LanguageEvaluationMetrics, ModelType, LLMModelData, LanguageEvaluationDataPoint, get_llm_model_data};
use crate::{check_cycles_before_action, NEXT_LLM_LANGUAGE_EVALUATION_ID, get_model_from_memory, only_admin};
use crate::utils::{is_owner, seeded_vector_shuffle};
use crate::errors::GenericError;
use crate::MODELS;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::collections::HashSet;

const KALEIDOSKOPE_CSV: &str = include_str!("data/kaleidoscope.csv");

const SYSTEM_PROMPT: &str = "You are a helpful assistant who answers multiple-choice questions. For each question,
output your final answer in JSON format with the following structure: {\"choice\":
\"The correct option\"}. ONLY output this format exactly. Do
not include any additional text or explanations outside the JSON structure.";

fn build_prompt(question: &String, options: &Vec<String>, seed: u32) -> String {
    let mut prompt = String::with_capacity(
        SYSTEM_PROMPT.len() + 
        question.len() + 
        options.iter().map(|s| s.len() + 1).sum::<usize>() + 
        4  // For extra newlines
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

async fn run_evaluate_languages(model_data: &LLMModelData, languages: &Vec<String>, seed: u32, max_queries: usize) -> Result<LanguageEvaluationResult, String> {
    let mut data_points: Vec<LanguageEvaluationDataPoint> = Vec::new();

    let mut metrics = HashMap::<String, LanguageEvaluationMetrics>::new(); 
    let mut overall_metrics = LanguageEvaluationMetrics::new();

    // Create a metrics object for every separated language
    for lang in languages {
        metrics.insert(lang.clone(), LanguageEvaluationMetrics::new());
    }
    
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

    // The number of max_queries is divided among the number of languages
    // Using integer division
    let original_max_queries = max_queries;
    let max_queries = max_queries / languages.len();
    if original_max_queries > 0 && max_queries == 0 {
        return Err("Wrong max_queries value. It should be at least the number of languages, or zero.".to_string());
    }
    
    for lang in languages {
        let mut queries : usize = 0;
        ic_cdk::println!("Processing `{}` language", lang);
        let mut rdr = csv::ReaderBuilder::new()
            .from_reader(KALEIDOSKOPE_CSV.as_bytes());
        
        for result in rdr.deserialize::<HashMap<String, String>>() {
            let result = result.map_err(|e| e.to_string())?;

            let language: &String = result.get("language")
                .expect("It should be able to get the language field.");
            
            if language != lang {
                // Only process records for current language
                continue;
            }

            ic_cdk::println!("Executing query {}/{} of language {}", queries, max_queries, language);
            
            let question: &String = result.get("question")
                .expect("It should be able to get the question field.");
            let answer: usize = result.get("answer")
                .and_then(|ans| ans.parse::<usize>().ok())
                .expect("Answer field should be a valid usize index");
            let options: Vec<String> = result.get("options")
                .map(|opt_str| {
                    ic_cdk::println!("{}", &opt_str);
                    // Split by space and clean up quotes from each element
                    opt_str.trim_matches(|c| c == '[' || c == ']')
                        .split('\'')
                        .filter(|s| !s.trim().is_empty() && s.trim() != " ")
                        .map(|s| s.trim().to_string())
                        .collect()
                })
                .expect("It should be able to parse the options field.");
            let text_answer: String = options.get(answer)
                .expect("Answer should exist in the options vector")
                .to_string();
            ic_cdk::println!("Valid answer: {}", text_answer);

            let lang_metrics: &mut LanguageEvaluationMetrics = metrics.get_mut(language).expect("Value for language should exist");

            let prompt: String = build_prompt(&question, &options, seed * (queries as u32));
            
            let res = call_hugging_face(prompt.clone(), model_data.hugging_face_url.clone(), seed, Some(hf_parameters.clone())).await;

            let trimmed_response = match res {
                Ok(response) => crate::utils::clean_llm_response(&response),
                Err(e) => {
                    ic_cdk::println!("Error calling Hugging Face API: {}", e);

                    overall_metrics.add_error();
                    lang_metrics.add_error();

                    data_points.push(LanguageEvaluationDataPoint {
                        prompt: prompt.clone(),
                        response: None,
                        valid: false,
                        error: true,
                        correct_answer: text_answer.clone(),
                    });
                    
                    queries += 1;
                    if max_queries > 0 && queries >= max_queries {
                        break;
                    }
                    
                    continue; // Skip this iteration and continue with the next question
                }
            };

            ic_cdk::println!("Cleaned response: {}", &trimmed_response);

            let evaluation_answer = match serde_json::from_str::<LanguageEvaluationAnswer>(&trimmed_response) {
                Ok(json) => {
                    ic_cdk::println!("Parsed JSON response: {:?}", &json);
                    json
                },
                Err(e) => {
                    ic_cdk::println!("Failed to parse JSON: {}. Skipping to next row.", e);
                    overall_metrics.add_invalid();
                    lang_metrics.add_invalid();

                    data_points.push(LanguageEvaluationDataPoint {
                        prompt: prompt.clone(),
                        response: None,
                        valid: false,
                        error: false,
                        correct_answer: text_answer.clone(),
                    });
                    
                    queries += 1;
                    if max_queries > 0 && queries >= max_queries {
                        break;
                    }
                    continue;
                }
            };

            let llm_answer = evaluation_answer.choice.trim();

            data_points.push(LanguageEvaluationDataPoint {
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

            queries += 1;
            if max_queries > 0 && queries >= max_queries {
                break;
            }
        }
    }

    overall_metrics.calculate_rates();
    for lang_metrics in metrics.values_mut() {
        lang_metrics.calculate_rates();
    }

    // Done in this way so the result order is deterministic
    let mut metrics_per_language = Vec::<(String, LanguageEvaluationMetrics)>::new();
    for lang in languages {
        metrics_per_language.push((lang.clone(), metrics.get(lang).unwrap().clone()));
    }
    
    let result = NEXT_LLM_LANGUAGE_EVALUATION_ID.with( |id| {
        let current_id = *id.borrow().get();
        
        let evaluation = LanguageEvaluationResult {
            language_model_evaluation_id: current_id,
            timestamp: ic_cdk::api::time(),
            languages: languages.clone(),
            prompt_templates: vec![("overall".to_string(), SYSTEM_PROMPT.to_string())],
            data_points,
            metrics: overall_metrics,
            metrics_per_language,
            max_queries: original_max_queries,
        };

        id.borrow_mut().set(current_id + 1).unwrap();

        evaluation
    });

    return Ok(result);
}

/// Evaluates languages for a LLM. It returns the LanguageEvaluationResult,
/// and it also saves the result into the model data.
#[update]
pub async fn llm_evaluate_languages(model_id: u128, languages: Vec<String>, max_queries: usize, seed: u32) -> Result<LanguageEvaluationResult, GenericError> {
    only_admin();
    check_cycles_before_action();

    if languages.len() == 0 {
        return Err(GenericError::new(GenericError::INVALID_ARGUMENT, "You should select at least one language."));
    }
    let valid_values: HashSet<&str> = ["ar", "bn", "de", "en", "es", "fa", "fr", "hi", "hr", "hu", "lt", "nl", "pt", "ru", "sr", "uk"].into_iter().collect();
    let all_valid = languages.iter().all(|x| valid_values.contains(x.as_str()));
    if !all_valid {
        return Err(GenericError::new(GenericError::INVALID_ARGUMENT, "An invalid language was selected."));
    }
    
    let caller = ic_cdk::api::caller();

    ic_cdk::println!("Calling llm_evaluate_languages for model {}", model_id);

    let model = get_model_from_memory(model_id);
    if let Err(err) = model {
        return Err(err);
    }
    let model = model.unwrap();
    is_owner(&model, caller);

    if let ModelType::LLM(model_type_data) = model.model_type {
        let result = run_evaluate_languages(&model_type_data, &languages, seed, max_queries).await;

        ic_cdk::println!("Language evaluation finished successfully");
        
        return match result {
            Ok(language_evaluation_result) => {
                MODELS.with(|models| {
                    let mut models = models.borrow_mut();
                    let mut model = models.get(&model_id).expect("Model not found");

                    let mut model_data = get_llm_model_data(&model);
                    
                    model_data.language_evaluations.push(language_evaluation_result.clone());
                    
                    model.model_type = ModelType::LLM(model_data);
                    models.insert(model_id, model);
                });

                ic_cdk::println!("Language evaluation saved successfully");

                Ok(language_evaluation_result)
            },
            Err(err_message) => {
                Err(GenericError::new(GenericError::GENERIC_SYSTEM_FAILURE, err_message))
            }
        };
    } else {
        return Err(GenericError::new(GenericError::INVALID_MODEL_TYPE, "Model should be a LLM"));
    }
}

#[query]
pub async fn get_language_evaluation_data_points(llm_model_id: u128, language_evaluation_id: u128, limit: u32, offset: usize) -> Result<(Vec<LanguageEvaluationDataPoint>, usize), GenericError>  {
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
        let language_evaluation = model_data.language_evaluations
            .into_iter()
            .find(|le: &LanguageEvaluationResult| le.language_model_evaluation_id == language_evaluation_id)
            .expect("Context association test with passed index should exist.");

        let data_points: &Vec<LanguageEvaluationDataPoint> = language_evaluation.data_points.as_ref();

        let data_points_total_length = data_points.len();

        // Get a slice of data points based on offset and limit
        let start = offset;
        let end = (offset + limit as usize).min(data_points.len());
        
        // Clone the selected range of data points
        let data_points = data_points[start..end].to_vec();
        
        return Ok((data_points, data_points_total_length));
    } else {
        return Err(GenericError::new(GenericError::INVALID_MODEL_TYPE, "Model should be an LLM."));
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
            String::from("D) Madrid")
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
