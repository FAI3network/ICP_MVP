use candid::{Principal, decode_one, encode_args};
use FAI3_backend::types::LLMMetricsAPIResult;
mod common;
use common::{
    create_pic, create_llm_model,
    wait_for_http_request, mock_http_response, mock_correct_hugging_face_response_body
};

fn llm_fairness_with_variable_queries_test(
    dataset: &str, returned_texts: Vec<&str>,
    counter_factual_returned_text: Option<Vec<&str>>) -> LLMMetricsAPIResult {
    let (pic, canister_id) = create_pic();

    // Setting the model name and creating the model
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);  // Assuming model creation is always returning model_id as 1 in mock

    // Preparing the request with the dynamic number of max_queries based on returned_texts length
    let max_queries: usize = returned_texts.len();  // Now this is dynamic
    let seed: u32 = 1;
    let encoded_args = encode_args((model_id, dataset, max_queries, seed)).unwrap();
    
    // Submitting the computational request
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "calculate_llm_metrics",
        encoded_args,
    ).expect("calculate_llm_metrics call should succeed");

    // Mocking HTTP responses based on returned_texts
    let mut text_idx = 0;
    for text in returned_texts {
        for i in 0..2 { // Looping twice for each response simulation
            wait_for_http_request(&pic);
            let canister_http_requests = pic.get_canister_http();
            if canister_http_requests.is_empty() {
                break;
            }
            let canister_http_request = &canister_http_requests[0];

            let mut mock_hf_response_body = mock_correct_hugging_face_response_body(text);
            if i == 1 {
                // If a counter factual array was passed, then we get the text from there for the CF response
                if let Some(cf_array) = &counter_factual_returned_text {
                    mock_hf_response_body = mock_correct_hugging_face_response_body(cf_array[text_idx]);
                }
            }
            
            let mock_canister_http_response = mock_http_response(canister_http_request, mock_hf_response_body);
            pic.mock_canister_http_response(mock_canister_http_response);
        }
        text_idx += 1;
    }

    // Verifying end condition: no pending HTTP outcalls
    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 0);

    // Awaiting and decoding the response
    let reply = pic.await_call(call_id).unwrap();
    let decoded_reply: Result<LLMMetricsAPIResult, String> = decode_one(&reply).expect("Failed to decode context association test reply");

    return decoded_reply.expect("It should be a valid result");
}

#[test]
fn test_llm_fairness_wrong_json_response() {
    let first_returned_text = "not-a-json";
    let second_returned_text = "not-a-json-2";
    
    let (pic, canister_id) = create_pic();
    
    // Creating model
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);

    // Calling context_association_test
    let max_queries: usize = 2;
    let seed: u32 = 1;
    let encoded_args = encode_args((model_id, "pisa", max_queries, seed)).unwrap();
    // Submit an update call to the test canister making a canister http outcall
    // and mock a canister http outcall response.
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "calculate_llm_metrics",
        encoded_args,
    ).expect("calculate_llm_metrics call should succeed");

    // Counter factual testing is not called when there is an error in the first call
    
    // We need a pair of ticks for the test canister method to make the http outcall
    // and for the management canister to start processing the http outcall.
    wait_for_http_request(&pic);    
    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_canister_http_response = mock_http_response(canister_http_request, first_returned_text);
    pic.mock_canister_http_response(mock_canister_http_response);


    wait_for_http_request(&pic);

    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_canister_http_response = mock_http_response(canister_http_request, second_returned_text);
    pic.mock_canister_http_response(mock_canister_http_response);
    
    // There should be no more pending canister http outcalls.
    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 0);

    // Now the test canister will receive the http outcall response
    // and reply to the ingress message from the test driver.
    let reply = pic.await_call(call_id).unwrap();
    let decoded_reply: Result<LLMMetricsAPIResult, String> = decode_one(&reply).expect("Failed to decode context association test reply");

    let llm_fairness_result = decoded_reply.expect("It should be a valid result");

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 2);
}

#[test]
fn test_llm_fairness_invalid_responses() {
    let llm_fairness_result = llm_fairness_with_variable_queries_test("pisa", vec!["invalid", "response"], None);

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 2);
    assert_eq!(llm_fairness_result.call_errors, 0);
}

#[test]
fn test_llm_fairness_happy_path() {
    let llm_fairness_result = llm_fairness_with_variable_queries_test("pisa", vec!["H", "L"], None);

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    let counter_factual = llm_fairness_result.counter_factual.expect("It should have a counter factual object");

    assert_eq!(counter_factual.change_rate_overall, 0.0);
    assert_eq!(counter_factual.sensible_attribute, "male");
    assert_eq!(counter_factual.change_rate_sensible_attributes, vec![0.0, 0.0]);
}

#[test]
fn test_llm_counterfactual_fairness_worst_case() {
    let llm_fairness_result = llm_fairness_with_variable_queries_test("pisa_test", vec!["H", "L"], Some(vec!["L", "H"]));

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    let counter_factual = llm_fairness_result.counter_factual.expect("It should have a counter factual object");

    assert_eq!(counter_factual.change_rate_overall, 1.0);
    assert_eq!(counter_factual.sensible_attribute, "male");
    assert_eq!(counter_factual.change_rate_sensible_attributes, vec![1.0, 1.0]);
}

#[test]
fn test_llm_counterfactual_fairness_50_percent_change() {
    let llm_fairness_result = llm_fairness_with_variable_queries_test("pisa_test", vec!["H", "L"], Some(vec!["L", "L"]));

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    let counter_factual = llm_fairness_result.counter_factual.expect("It should have a counter factual object");

    assert_eq!(counter_factual.change_rate_overall, 0.5);
    assert_eq!(counter_factual.sensible_attribute, "male");
    assert_eq!(counter_factual.change_rate_sensible_attributes, vec![0.0, 1.0]);
}

#[test]
fn test_llm_fairness_metrics_with_pisa_test() {
    let llm_fairness_result = llm_fairness_with_variable_queries_test("pisa_test", vec![
        "H", 
        "L", 
        "H", 
        "H", 
        "L", 
        "H", 
        "H", 
        "H", 
        "H", 
        "L", 
        "H", 
        "L", 
        "L", 
        "H", 
        "H", 
        "L", 
        "H", 
        "H", 
        "H", 
        "L"
    ], None);

    assert_eq!(llm_fairness_result.queries, 20);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    assert!(llm_fairness_result.metrics.accuracy.expect("It should return accuracy") - 0.7 < 1e-6);
    assert!(llm_fairness_result.metrics.precision.expect("It should return precision") - 0.6923076923076923 < 1e-6);
    assert!(llm_fairness_result.metrics.recall.expect("It should return recall") - 0.8181818181818182 < 1e-6);
    
    assert!( (llm_fairness_result.metrics.average_metrics.disparate_impact.expect("It should return disparate_impact") - 0.763888888888889).abs() < 1e-6);
    assert!( (llm_fairness_result.metrics.average_metrics.average_odds_difference.expect("It should return average_odds_difference") - 0.10357143).abs()  < 1e-6);
    assert!( (llm_fairness_result.metrics.average_metrics.equal_opportunity_difference.expect("It should return equal_opportunity_difference") - (-0.1071428571428571)).abs() < 1e-6);
    assert!( (llm_fairness_result.metrics.average_metrics.statistical_parity_difference.expect("It should return statistical_parity_difference") - (-0.1717171717171717)).abs() < 1e-6);
}
