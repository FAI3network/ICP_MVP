use candid::{Principal, decode_one, encode_args};
use FAI3_backend::types::{get_llm_model_data, LanguageEvaluationResult, LanguageEvaluationMetrics, JobType};
use FAI3_backend::errors::GenericError;
use pocket_ic::PocketIc;
mod common;
use common::{
    create_pic, create_llm_model, get_model, add_hf_api_key,
    wait_for_http_request, mock_http_response, mock_correct_hugging_face_response_body,
     wait_for_job_to_finish,
};
use FAI3_backend::llm_language_evaluations::LanguageEvaluationAnswer;
use std::time::Duration;

fn json_answer(answer: &str) -> String {
    serde_json::to_string(&LanguageEvaluationAnswer {
        choice: answer.to_string()
    }).unwrap()
}

fn evaluate_languages(pic: &PocketIc, canister_id: Principal, model_id: u128,  languages: &Vec<&str>, max_queries: usize, mocked_texts: &Vec<String>, use_correcet_hf_response_body: bool) -> LanguageEvaluationResult {
    let seed: u32 = 0;
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");
    
    let reply = pic.await_call(call_id).unwrap();
    let result: Result<u128, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");

    assert!(result.is_ok(), "Result for job_id should be ok");

    let job_id = result.unwrap();

    for i in 0..mocked_texts.len() {
        pic.advance_time(Duration::from_secs(2));
        // We need a pair of ticks for the test canister method to make the http outcall
        // and for the management canister to start processing the http outcall.
        wait_for_http_request(&pic);
        
        let canister_http_requests = pic.get_canister_http();
        let canister_http_request = &canister_http_requests[0];

        let mock_hf_response_body = match use_correcet_hf_response_body {
            true => mock_correct_hugging_face_response_body(mocked_texts.get(i).unwrap()),
            false => mocked_texts.get(i).unwrap().clone(),
        };
        let mock_canister_http_response = mock_http_response(canister_http_request, mock_hf_response_body);
        pic.mock_canister_http_response(mock_canister_http_response);

        pic.tick();
        pic.tick();
    }
    
    pic.advance_time(Duration::from_secs(2));
    
    // There should be no more pending canister http outcalls.
    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 0);

    // Now we wait until the job finishes
    let job = wait_for_job_to_finish(&pic, canister_id, job_id)
        .expect("Job should finish on time");
    
    let language_evaluation_id = if let JobType::LanguageEvaluation { language_model_evaluation_id } = job.job_type {
        language_model_evaluation_id
    } else {
        panic!("job_type should be LanguageEvaluation");
    };

    let encoded_args = encode_args((model_id, language_evaluation_id)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "get_language_evaluation",
        encoded_args,
    ).expect("get_language_evaluation call should be submitted.");

    let reply = pic.await_call(call_id).unwrap();
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode get_language_evaluation reply.");

    assert!(result.is_ok(), "Result for LanguageEvaluationResult should be ok");

    let result: LanguageEvaluationResult = result.unwrap();

    return result;
}

#[test]
fn test_language_evaluations_with_multiple_languages() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);
    
    let max_queries: usize = 10;
    
   // Calculate average metrics
    let languages: Vec<&str> = vec!["es", "en"];

    let mocked_texts = vec![
        json_answer("d"),
        json_answer("a"), 
        json_answer("a"), // incorrect answer (es)
        json_answer("a"), // incorrect answer (en)
        json_answer("d"),
        json_answer("b"),
        json_answer("d"), 
        json_answer("a"),
        json_answer("b"),
        json_answer("c"),
    ];

    let result = evaluate_languages(&pic, canister_id, model_id, &languages, max_queries, &mocked_texts, true);


    fn assert_overall_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 10);
        assert_eq!(metrics.correct_responses, 8);
        assert_eq!(metrics.incorrect_responses, 2);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 0);
        
        assert_eq!(metrics.overall_accuracy, Some(0.8));
        assert_eq!(metrics.accuracy_on_valid_responses, Some(0.8));
    }

    fn assert_language_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 5);
        assert_eq!(metrics.correct_responses, 4);
        assert_eq!(metrics.incorrect_responses, 1);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 0);
        
        assert_eq!(metrics.overall_accuracy, Some(0.8));
        assert_eq!(metrics.accuracy_on_valid_responses, Some(0.8));
    }

    fn assert_score_result(result: &LanguageEvaluationResult) {
        assert_eq!(result.metrics.n, 10);
        assert_eq!(result.language_model_evaluation_id, 1);
        assert_eq!(result.max_queries, 10);
        assert_eq!(result.languages, vec!["es", "en"]);
        assert_overall_metrics(&result.metrics);
        
        // check metrics for en language
        assert_eq!(result.metrics_per_language.len(), 2);

        let (es_label, es_metrics) = result.metrics_per_language.get(0).unwrap();
        assert_eq!("es", es_label);
        assert_language_metrics(&es_metrics);
        let (en_label, en_metrics) = result.metrics_per_language.get(1).unwrap();
        assert_eq!("en", en_label);
        assert_language_metrics(&en_metrics);
    }

    assert_score_result(&result);
    
    // checking it was saved correctly
    let model = get_model(&pic, canister_id, model_id);
    let llm_model_data = get_llm_model_data(&model);

    assert_eq!(llm_model_data.language_evaluations.len(), 1);
    let evaluation = llm_model_data.language_evaluations.get(0).unwrap();
    assert_score_result(&evaluation);
}

#[test]
fn test_language_evaluations_perfect_score() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);

    let max_queries: usize = 5;
    
   // Calculate average metrics
    let languages = vec!["en"];
    
    let mocked_texts = vec![
        json_answer("a"),
        json_answer("c"),
        json_answer("b"),
        json_answer("a"),
        json_answer("c"),  
    ];

    let result = evaluate_languages(&pic, canister_id, model_id, &languages, max_queries, &mocked_texts, true);

    fn assert_perfect_score_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 5);
        assert_eq!(metrics.correct_responses, 5);
        assert_eq!(metrics.incorrect_responses, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 0);
        
        assert_eq!(metrics.overall_accuracy, Some(1.0));
        assert_eq!(metrics.accuracy_on_valid_responses, Some(1.0));
    }

    fn assert_perfect_score_result(result: &LanguageEvaluationResult, data_points_len: usize) {
        assert_eq!(result.data_points.len(), data_points_len);
        assert_eq!(result.language_model_evaluation_id, 1);
        assert_eq!(result.max_queries, 5);
        assert_eq!(result.languages, vec!["en"]);
        assert_perfect_score_metrics(&result.metrics);
        
        // check metrics for en language
        assert_eq!(result.metrics_per_language.len(), 1);
        let (en_label, en_metrics) = result.metrics_per_language.get(0).unwrap();
        assert_eq!("en", en_label);
        assert_perfect_score_metrics(&en_metrics);
    }

    assert_perfect_score_result(&result, 5);
    
    // checking it was saved correctly
    let model = get_model(&pic, canister_id, model_id);
    let llm_model_data = get_llm_model_data(&model);

    assert_eq!(llm_model_data.language_evaluations.len(), 1);
    let evaluation = llm_model_data.language_evaluations.get(0).unwrap();
    assert_perfect_score_result(&evaluation, 0);
}

#[test]
fn test_language_evaluations_non_perfect_score() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);

    let max_queries: usize = 5;
    
   // Calculate average metrics
    let languages = vec!["en"];

    let mocked_texts = vec![
        json_answer("a"),
        json_answer("d"), // incorrect answer
        json_answer("b"),
        json_answer("b"), // incorrect answer
        json_answer("c"),
        
    ];

    let result = evaluate_languages(&pic, canister_id, model_id, &languages, max_queries, &mocked_texts, true);

    fn assert_non_perfect_score_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 5);
        assert_eq!(metrics.correct_responses, 3);
        assert_eq!(metrics.incorrect_responses, 2);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 0);
        
        assert_eq!(metrics.overall_accuracy, Some(0.6));
        assert_eq!(metrics.accuracy_on_valid_responses, Some(0.6));
    }

    fn assert_non_perfect_score_result(result: &LanguageEvaluationResult, data_points_len: usize) {
        assert_eq!(result.data_points.len(), data_points_len);
        assert_eq!(result.language_model_evaluation_id, 1);
        assert_eq!(result.max_queries, 5);
        assert_eq!(result.languages, vec!["en"]);
        assert_non_perfect_score_metrics(&result.metrics);
        
        // check metrics for en language
        assert_eq!(result.metrics_per_language.len(), 1);
        let (en_label, en_metrics) = result.metrics_per_language.get(0).unwrap();
        assert_eq!("en", en_label);
        assert_non_perfect_score_metrics(&en_metrics);
    }

    assert_non_perfect_score_result(&result, 5);
    
    // checking it was saved correctly
    let model = get_model(&pic, canister_id, model_id);
    let llm_model_data = get_llm_model_data(&model);

    assert_eq!(llm_model_data.language_evaluations.len(), 1);
    let evaluation = llm_model_data.language_evaluations.get(0).unwrap();
    assert_non_perfect_score_result(&evaluation, 0);
}

#[test]
fn test_language_evaluations_with_invalid_answers() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);

    let max_queries: usize = 5;
    
   // Calculate average metrics
    let languages = vec!["en"];

    let mocked_texts = vec![
        json_answer("invalid answer"),
        json_answer("invalid answer"),
        json_answer("invalid answer"),
        json_answer("invalid answer"),
        json_answer("invalid answer"),        
    ];

     let result = evaluate_languages(&pic, canister_id, model_id, &languages, max_queries, &mocked_texts, true);

    fn assert_invalid_score_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 5);
        assert_eq!(metrics.correct_responses, 0);
        assert_eq!(metrics.incorrect_responses, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 5);
        
        assert_eq!(metrics.overall_accuracy, Some(0.0));
        assert_eq!(metrics.accuracy_on_valid_responses, None);
    }

    fn assert_invalid_score_result(result: &LanguageEvaluationResult, data_points_len: usize) {
        assert_eq!(result.data_points.len(), data_points_len);
        assert_eq!(result.language_model_evaluation_id, 1);
        assert_eq!(result.max_queries, 5);
        assert_eq!(result.languages, vec!["en"]);
        assert_invalid_score_metrics(&result.metrics);
        
        // check metrics for en language
        assert_eq!(result.metrics_per_language.len(), 1);
        let (en_label, en_metrics) = result.metrics_per_language.get(0).unwrap();
        assert_eq!("en", en_label);
        assert_invalid_score_metrics(&en_metrics);
    }

    assert_invalid_score_result(&result, 5);
    
    // checking it was saved correctly
    let model = get_model(&pic, canister_id, model_id);
    let llm_model_data = get_llm_model_data(&model);

    assert_eq!(llm_model_data.language_evaluations.len(), 1);
    let evaluation = llm_model_data.language_evaluations.get(0).unwrap();
    assert_invalid_score_result(&evaluation, 0);
}

/// This tests what happens when Hugging Face return an invalid formatted json
/// This means that the structure of Hugging Face response is wrong / cannot be parsed.
#[test]
fn test_hugging_face_invalid_json_responses() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);
    // let seed: u32 = 1;
    let max_queries: usize = 2;
    
   // Calculate average metrics
    let languages = vec!["en"];

    let mocked_texts = vec![
        "invalid hf response".to_string(),
        "invalid hf response".to_string(),
    ];

    let result = evaluate_languages(&pic, canister_id, model_id, &languages, max_queries, &mocked_texts, false);

    let metrics = result.metrics;
    assert_eq!(metrics.n, 2);
    assert_eq!(metrics.correct_responses, 0);
    assert_eq!(metrics.incorrect_responses, 0);
    assert_eq!(metrics.error_count, 2);
    assert_eq!(metrics.invalid_responses, 0);

    assert_eq!(metrics.overall_accuracy, None);
    assert_eq!(metrics.accuracy_on_valid_responses, None);

    assert_eq!(result.metrics_per_language.len(), 1);
    let (_, metrics) = result.metrics_per_language.get(0).unwrap();
    assert_eq!(metrics.n, 2);
    assert_eq!(metrics.correct_responses, 0);
    assert_eq!(metrics.incorrect_responses, 0);
    assert_eq!(metrics.error_count, 2);
    assert_eq!(metrics.invalid_responses, 0);

    assert_eq!(metrics.overall_accuracy, None);
    assert_eq!(metrics.accuracy_on_valid_responses, None);
}

#[test]
fn test_language_evaluations_with_invalid_json() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);

    let max_queries: usize = 2;
    
   // Calculate average metrics
    let languages = vec!["en"];

    let mocked_texts = vec![
        "invalid llm json response".to_string(),
        "invalid llm json response".to_string(),
    ];

    let result = evaluate_languages(&pic, canister_id, model_id, &languages, max_queries, &mocked_texts, true);

    let metrics = result.metrics;
    assert_eq!(metrics.n, 2);
    assert_eq!(metrics.correct_responses, 0);
    assert_eq!(metrics.incorrect_responses, 0);
    assert_eq!(metrics.error_count, 0);
    assert_eq!(metrics.invalid_responses, 2);
    
    assert_eq!(metrics.overall_accuracy, Some(0.0));
    assert_eq!(metrics.accuracy_on_valid_responses, None);
    
    assert_eq!(result.metrics_per_language.len(), 1);
    let (_, metrics) = result.metrics_per_language.get(0).unwrap();
    assert_eq!(metrics.n, 2);
    assert_eq!(metrics.correct_responses, 0);
    assert_eq!(metrics.incorrect_responses, 0);
    assert_eq!(metrics.error_count, 0);
    assert_eq!(metrics.invalid_responses, 2);
    
    assert_eq!(metrics.overall_accuracy, Some(0.0));
    assert_eq!(metrics.accuracy_on_valid_responses, None);
}

#[test]
fn test_language_evaluations_with_unknown_languages() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);
    let seed: u32 = 1;
    let max_queries: usize = 1;
    
   // Calculate average metrics
    let languages = vec!["en", "unknown"];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    let reply = pic.await_call(call_id).unwrap();
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");

    assert!(result.is_err(), "Result should be an error");

    let error: GenericError = result.unwrap_err();

    assert_eq!(error.code, GenericError::INVALID_ARGUMENT);
    assert_eq!(error.message, "An invalid language was selected.");

}

#[test]
fn test_language_evaluations_with_0_languages_should_error() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    add_hf_api_key(&pic, canister_id, model_id);
    let seed: u32 = 1;
    let max_queries: usize = 1;
    
   // Calculate average metrics
    let languages = Vec::<String>::new();
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    let reply = pic.await_call(call_id).unwrap();
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");

    assert!(result.is_err(), "Result should be an error");

    let error: GenericError = result.unwrap_err();

    assert_eq!(error.code, GenericError::INVALID_ARGUMENT);
    assert_eq!(error.message, "You should select at least one language.");

}
