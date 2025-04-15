use candid::{Principal, decode_one, encode_args};
use FAI3_backend::types::{get_llm_model_data, LanguageEvaluationResult, LanguageEvaluationMetrics};
use FAI3_backend::errors::GenericError;
mod common;
use common::{
    create_pic, create_llm_model, get_model,
    wait_for_http_request, mock_http_response, mock_correct_hugging_face_response_body, wait_for_mocks_strings,
};
use FAI3_backend::llm_language_evaluations::LanguageEvaluationAnswer;

fn json_answer(answer: &str) -> String {
    serde_json::to_string(&LanguageEvaluationAnswer {
        choice: answer.to_string()
    }).unwrap()
}

#[test]
fn test_language_evaluations_with_multiple_languages() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    let seed: u32 = 0;
    let max_queries: usize = 10;
    
   // Calculate average metrics
    let languages = vec!["es", "en"];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    let mocked_texts = vec![
        json_answer("La Respuesta A y B, son Correctas"),
        json_answer("8"), // incorrect answer
        json_answer("La Respuesta A y B, son Correctas."),
        json_answer("Todos los grados de yododeficiencia (leve, moderada o severa), pueden potencialmente causar daño neurológico en el feto"),
        json_answer("La Respuesta C y D, son Correctas"),
        json_answer("Hans Peters"),
        json_answer("Protein"), // incorrect answer
        json_answer("Anther is kidney shaped"),
        json_answer("i and ii"),
        json_answer("In pectoral girdle"),
    ];
    
    let reply = wait_for_mocks_strings(&pic, call_id, &mocked_texts);
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");

    assert!(result.is_ok(), "Result should be ok");

    let result: LanguageEvaluationResult = result.unwrap();

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
        assert_eq!(result.data_points.len(), 10);
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
    let seed: u32 = 0;
    let max_queries: usize = 5;
    
   // Calculate average metrics
    let languages = vec!["en"];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    let mocked_texts = vec![
        json_answer("Hans Peters"),
        json_answer("Chitin"),
        json_answer("Anther is kidney shaped"),
        json_answer("i and ii"),
        json_answer("In pectoral girdle"),
        
    ];
    
    let reply = wait_for_mocks_strings(&pic, call_id, &mocked_texts);
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");

    assert!(result.is_ok(), "Result should be ok");

    let result: LanguageEvaluationResult = result.unwrap();

    fn assert_perfect_score_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 5);
        assert_eq!(metrics.correct_responses, 5);
        assert_eq!(metrics.incorrect_responses, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 0);
        
        assert_eq!(metrics.overall_accuracy, Some(1.0));
        assert_eq!(metrics.accuracy_on_valid_responses, Some(1.0));
    }

    fn assert_perfect_score_result(result: &LanguageEvaluationResult) {
        assert_eq!(result.data_points.len(), 5);
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

    assert_perfect_score_result(&result);
    
    // checking it was saved correctly
    let model = get_model(&pic, canister_id, model_id);
    let llm_model_data = get_llm_model_data(&model);

    assert_eq!(llm_model_data.language_evaluations.len(), 1);
    let evaluation = llm_model_data.language_evaluations.get(0).unwrap();
    assert_perfect_score_result(&evaluation);
}

#[test]
fn test_language_evaluations_non_perfect_score() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    let seed: u32 = 0;
    let max_queries: usize = 5;
    
   // Calculate average metrics
    let languages = vec!["en"];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    let mocked_texts = vec![
        json_answer("Hans Peters"),
        json_answer("Protein"), // incorrect answer
        json_answer("Anther is kidney shaped"),
        json_answer("i, ii and iii"), // incorrect answer
        json_answer("In pectoral girdle"),
        
    ];
    
    let reply = wait_for_mocks_strings(&pic, call_id, &mocked_texts);
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");

    assert!(result.is_ok(), "Result should be ok");

    let result: LanguageEvaluationResult = result.unwrap();

    fn assert_non_perfect_score_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 5);
        assert_eq!(metrics.correct_responses, 3);
        assert_eq!(metrics.incorrect_responses, 2);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 0);
        
        assert_eq!(metrics.overall_accuracy, Some(0.6));
        assert_eq!(metrics.accuracy_on_valid_responses, Some(0.6));
    }

    fn assert_non_perfect_score_result(result: &LanguageEvaluationResult) {
        assert_eq!(result.data_points.len(), 5);
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

    assert_non_perfect_score_result(&result);
    
    // checking it was saved correctly
    let model = get_model(&pic, canister_id, model_id);
    let llm_model_data = get_llm_model_data(&model);

    assert_eq!(llm_model_data.language_evaluations.len(), 1);
    let evaluation = llm_model_data.language_evaluations.get(0).unwrap();
    assert_non_perfect_score_result(&evaluation);
}

#[test]
fn test_language_evaluations_with_invalid_answers() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    let seed: u32 = 0;
    let max_queries: usize = 5;
    
   // Calculate average metrics
    let languages = vec!["en"];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    let mocked_texts = vec![
        json_answer("invalid answer"),
        json_answer("invalid answer"),
        json_answer("invalid answer"),
        json_answer("invalid answer"),
        json_answer("invalid answer"),        
    ];
    
    let reply = wait_for_mocks_strings(&pic, call_id, &mocked_texts);
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");

    assert!(result.is_ok(), "Result should be ok");

    let result: LanguageEvaluationResult = result.unwrap();

    fn assert_invalid_score_metrics(metrics: &LanguageEvaluationMetrics) {
        assert_eq!(metrics.n, 5);
        assert_eq!(metrics.correct_responses, 0);
        assert_eq!(metrics.incorrect_responses, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.invalid_responses, 5);
        
        assert_eq!(metrics.overall_accuracy, Some(0.0));
        assert_eq!(metrics.accuracy_on_valid_responses, None);
    }

    fn assert_invalid_score_result(result: &LanguageEvaluationResult) {
        assert_eq!(result.data_points.len(), 5);
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

    assert_invalid_score_result(&result);
    
    // checking it was saved correctly
    let model = get_model(&pic, canister_id, model_id);
    let llm_model_data = get_llm_model_data(&model);

    assert_eq!(llm_model_data.language_evaluations.len(), 1);
    let evaluation = llm_model_data.language_evaluations.get(0).unwrap();
    assert_invalid_score_result(&evaluation);
}

#[test]
fn test_hugging_face_invalid_json_responses() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    let seed: u32 = 1;
    let max_queries: usize = 2;
    
   // Calculate average metrics
    let languages = vec!["en"];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    for _ in 0..2 {
        wait_for_http_request(&pic);
        let canister_http_requests = pic.get_canister_http();
        
        let canister_http_request = &canister_http_requests[0];

        let hf_response = "invalid hf response";        
        let mock_canister_http_response = mock_http_response(canister_http_request, hf_response);
        pic.mock_canister_http_response(mock_canister_http_response);
    }

    let reply = pic.await_call(call_id).unwrap();
    
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");
    assert!(result.is_ok(), "Result is ok");
    let result: LanguageEvaluationResult = result.unwrap();

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
    let seed: u32 = 1;
    let max_queries: usize = 2;
    
   // Calculate average metrics
    let languages = vec!["en"];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, languages, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_evaluate_languages",
        encoded_args,
    ).expect("llm_evaluate_languages call should be submitted.");

    for _ in 0..2 {
        wait_for_http_request(&pic);
        let canister_http_requests = pic.get_canister_http();
        
        let canister_http_request = &canister_http_requests[0];

        // In this case the HF API json response is valid,
        // but the contents of the LLM response is not valid json
        let hf_response = mock_correct_hugging_face_response_body("invalid llm json response");
        
        let mock_canister_http_response = mock_http_response(canister_http_request, hf_response);
        pic.mock_canister_http_response(mock_canister_http_response);
    }

    let reply = pic.await_call(call_id).unwrap();
    
    let result: Result<LanguageEvaluationResult, GenericError> = decode_one(&reply).expect("Failed to decode llm_evaluate_languages reply.");
    assert!(result.is_ok(), "Result is ok");
    let result: LanguageEvaluationResult = result.unwrap();

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
