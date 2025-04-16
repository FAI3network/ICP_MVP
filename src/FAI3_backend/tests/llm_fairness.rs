use candid::{Principal, decode_one, encode_args, Encode};
use pocket_ic::{
    PocketIc,
    common::rest::RawMessageId
};
use FAI3_backend::types::{LLMMetricsAPIResult, LLMDataPoint, AverageLLMFairnessMetrics, get_llm_model_data, ModelEvaluationResult};
use FAI3_backend::llm_fairness::{build_prompts, PISA_PROMPT};
use FAI3_backend::errors::GenericError;
use std::collections::HashMap;
mod common;
use common::{
    create_pic, create_llm_model, get_model, add_hf_api_key,
    wait_for_http_request, mock_http_response, mock_correct_hugging_face_response_body
};

const PISA_TRAIN_CSV: &str = include_str!("./../src/data/pisa2009_train_processed.csv");
const PISA_TEST_CSV: &str = include_str!("./../src/data/pisa2009_test_processed.csv");
const PISA_CURATED_CSV: &str = include_str!("./../src/data/pisa2009_test_processed_curated.csv");    

const EPSILON: f32 = 1e-6;

fn get_row_data(prompts: usize, seed: u32, curated: bool) -> (Vec<String>, Vec<String>, Vec<HashMap<String, String>>) {

    let sensible_attribute = "male";
    let predict_attribute = "readingScore";
    let sensible_attribute_values = &["0", "1"];
    let predict_attributes_values = &["L", "H"];
    
    // Create a CSV reader from the string input rather than a file path
    let mut rdr = csv::ReaderBuilder::new()
        .from_reader(PISA_TRAIN_CSV.as_bytes());

      // Collect as HashMap to allow dynamic column access
    let records: Vec<HashMap<String, String>> = rdr.deserialize()
        .collect::<Result<Vec<HashMap<String, String>>, _>>()
        .map_err(|e| e.to_string()).unwrap();

    let test_dataset = match curated {
        true => PISA_CURATED_CSV,
        false => PISA_TEST_CSV,
    };
    
    let mut results: Vec<HashMap<String, String>> = Vec::new();
    // Create a CSV reader from the string input rather than a file path
    let mut rdr_test = csv::ReaderBuilder::new()
        .from_reader(test_dataset.as_bytes());

    // Collect as HashMap to allow dynamic column access
    let records_test: Vec<HashMap<String, String>> = rdr_test.deserialize()
        .collect::<Result<Vec<HashMap<String, String>>, _>>()
        .map_err(|e| e.to_string()).unwrap();

    for i in 0..prompts {
        let result = records_test.get(i).expect(format!("Issue when unwrapping row number {}", i).as_str());
        results.push(result.clone());
    }

    let mut normal_prompts = Vec::new();
    let mut cf_prompts = Vec::new();

    let ignore_columns = Vec::<&str>::new();
    for i in 0..prompts {
        let (personalized_prompt, personalized_prompt_cf) = build_prompts(
            &records, predict_attribute,
            sensible_attribute_values, predict_attributes_values,
            sensible_attribute, &ignore_columns,
            seed, i,
            PISA_PROMPT.to_string(), &results[i]).expect("build_prompts should execute successfully"); 

        normal_prompts.push(personalized_prompt);
        cf_prompts.push(personalized_prompt_cf);


    }
        
    return (normal_prompts, cf_prompts, results);
}

fn llm_fairness_with_variable_queries_test(
    pic: &PocketIc, canister_id: Principal,
    dataset: &str, returned_texts: Vec<&str>,
    counter_factual_returned_text: Option<Vec<&str>>) -> (LLMMetricsAPIResult, u128) {
    
    // Setting the model name and creating the model
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);  // Assuming model creation is always returning model_id as 1 in mock

    add_hf_api_key(&pic, canister_id, model_id);

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

    return (decoded_reply.expect("It should be a valid result"), model_id);
}

fn assert_counter_factual_data(dp: &LLMDataPoint, cf_prompt: String, valid_answer: bool) {
    assert_ne!(dp.counter_factual, None);
    let cf = dp.counter_factual.as_ref().unwrap();
    assert_eq!(cf.prompt, Some(cf_prompt));
    assert_eq!(cf.valid, valid_answer);
    assert_eq!(cf.error, false);
    assert_ne!(cf.timestamp, 0);
    if valid_answer {
        assert_ne!(cf.response, None);
        assert_ne!(cf.predicted, None);
        assert_eq!(cf.features.len(), 1);
    } else {
        assert_eq!(cf.response, None);
        assert_eq!(cf.predicted, None);
        assert_eq!(cf.features.len(), 0);
    }
}

// Waits for mocks to be processed.
// mocked_texts parameters should include the mocks for counter factual fairness
fn wait_for_mocks(pic: &PocketIc, call_id: RawMessageId, mocked_texts: Vec<&str>) -> Vec<u8> {
    // Mocking HTTP responses based on returned_texts
    for text in mocked_texts {
        wait_for_http_request(&pic);
        let canister_http_requests = pic.get_canister_http();
        if canister_http_requests.is_empty() {
            break;
        }
        
        let canister_http_request = &canister_http_requests[0];
        let mock_hf_response_body = mock_correct_hugging_face_response_body(text);
        
        let mock_canister_http_response = mock_http_response(canister_http_request, mock_hf_response_body);
        pic.mock_canister_http_response(mock_canister_http_response);
    }

    return pic.await_call(call_id).unwrap();
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

    add_hf_api_key(&pic, canister_id, model_id);

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

    // test saved model
    let model = get_model(&pic, canister_id, model_id);
    let llm_data = get_llm_model_data(&model);
    let llm_evaluation = llm_data.evaluations.get(0).expect("Created model should have one evaluation");
    let llm_data_points = llm_evaluation.llm_data_points.as_ref().expect("llm_data_points should have data");
    assert_eq!(llm_data_points.len(), 2);

    let (prompts, _, _) = get_row_data(2, seed, false);

    // do expectations on every data point
    let dp1 = llm_data_points.get(0).expect("It should contain two data points");
    assert_eq!(dp1.valid, false);
    assert_eq!(dp1.error, true);
    assert_eq!(dp1.target, true);
    assert_eq!(dp1.features.len(), 1);
    assert_eq!(*dp1.features.get(0).expect("It should contain one feature"), 0.0 as f64);
    assert_ne!(dp1.timestamp, 0.0 as u64);
    assert_eq!(dp1.predicted, None);
    assert_eq!(dp1.counter_factual, None);
    assert_eq!(dp1.prompt, prompts[0]);
    
    let dp2 = llm_data_points.get(1).expect("It should contain two data points");
    assert_eq!(dp2.valid, false);
    assert_eq!(dp2.error, true);
    assert_eq!(dp2.target, false);
    assert_eq!(dp2.features.len(), 1);
    assert_eq!(*dp1.features.get(0).expect("It should contain one feature"), 0.0 as f64);
    assert_ne!(dp2.timestamp, 0.0 as u64);
    assert_eq!(dp2.predicted, None);
    assert_eq!(dp2.prompt, prompts[1]);
    assert_eq!(dp2.counter_factual, None);
}

#[test]
fn test_llm_fairness_invalid_responses() {
    let (pic, canister_id) = create_pic();
    let (llm_fairness_result, model_id) = llm_fairness_with_variable_queries_test(
        &pic, canister_id,
        "pisa", vec!["invalid", "response"], None
    );

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 2);
    assert_eq!(llm_fairness_result.call_errors, 0);

    // test saved model
    let model = get_model(&pic, canister_id, model_id);
    let llm_data = get_llm_model_data(&model);
    let llm_evaluation = llm_data.evaluations.get(0).expect("Created model should have one evaluation");
    let llm_data_points = llm_evaluation.llm_data_points.as_ref().expect("llm_data_points should have data");
    assert_eq!(llm_data_points.len(), 2);

    let (prompts, cf_prompts, _) = get_row_data(2, 1, false);

    // do expectations on every data point
    let dp1 = llm_data_points.get(0).expect("It should contain two data points");
    assert_eq!(dp1.valid, false);
    assert_eq!(dp1.error, false);
    assert_eq!(dp1.target, true);
    assert_eq!(dp1.features.len(), 1);
    assert_eq!(*dp1.features.get(0).expect("It should contain one feature"), 0.0 as f64);
    assert_ne!(dp1.timestamp, 0.0 as u64);
    assert_eq!(dp1.predicted, None);
    assert_eq!(dp1.prompt, prompts[0]);
    assert_counter_factual_data(&dp1, cf_prompts[0].clone(), false);
    
    let dp2 = llm_data_points.get(1).expect("It should contain two data points");
    assert_eq!(dp2.valid, false);
    assert_eq!(dp2.error, false);
    assert_eq!(dp2.target, false);
    assert_eq!(dp2.features.len(), 1);
    assert_eq!(*dp1.features.get(0).expect("It should contain one feature"), 0.0 as f64);
    assert_ne!(dp2.timestamp, 0.0 as u64);
    assert_eq!(dp2.predicted, None);
    assert_eq!(dp2.prompt, prompts[1]);
    assert_counter_factual_data(&dp2, cf_prompts[1].clone(), false);
}

#[test]
fn test_llm_fairness_happy_path() {
    let (pic, canister_id) = create_pic();
    let (llm_fairness_result, model_id) = llm_fairness_with_variable_queries_test(
        &pic, canister_id,
        "pisa", vec!["H", "L"], None
    );

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    let counter_factual = llm_fairness_result.counter_factual.expect("It should have a counter factual object");

    assert_eq!(counter_factual.change_rate_overall, 0.0);
    assert_eq!(counter_factual.sensible_attribute, "male");
    assert_eq!(counter_factual.change_rate_sensible_attributes, vec![0.0, 0.0]);

    // test saved model
    let model = get_model(&pic, canister_id, model_id);
    let llm_data = get_llm_model_data(&model);
    let llm_evaluation = llm_data.evaluations.get(0).expect("Created model should have one evaluation");
    let llm_data_points = llm_evaluation.llm_data_points.as_ref().expect("llm_data_points should have data");
    assert_eq!(llm_data_points.len(), 2);

    let (prompts, cf_prompts, _) = get_row_data(2, 1, false);
    // do expectations on every data point
    let dp1 = llm_data_points.get(0).expect("It should contain two data points");
    assert_eq!(dp1.valid, true);
    assert_eq!(dp1.error, false);
    assert_eq!(dp1.target, true);
    assert_eq!(dp1.features.len(), 1);
    assert_eq!(*dp1.features.get(0).expect("It should contain one feature"), 0.0 as f64);
    assert_ne!(dp1.timestamp, 0.0 as u64);
    assert_eq!(dp1.predicted, Some(true));
    assert_eq!(dp1.prompt, prompts[0]);
    assert_counter_factual_data(&dp1, cf_prompts[0].clone(), true);
    
    let dp2 = llm_data_points.get(1).expect("It should contain two data points");
    assert_eq!(dp2.valid, true);
    assert_eq!(dp2.error, false);
    assert_eq!(dp2.target, false);
    assert_eq!(dp2.features.len(), 1);
    assert_eq!(*dp1.features.get(0).expect("It should contain one feature"), 0.0 as f64);
    assert_ne!(dp2.timestamp, 0.0 as u64);
    assert_eq!(dp2.predicted, Some(false));
    assert_eq!(dp2.prompt, prompts[1]);
    assert_counter_factual_data(&dp2, cf_prompts[1].clone(), true);
}

#[test]
fn test_llm_counterfactual_fairness_worst_case() {
    let (pic, canister_id) = create_pic();
    let (llm_fairness_result, model_id) = llm_fairness_with_variable_queries_test(
        &pic, canister_id,
        "pisa_test", vec!["H", "L"], Some(vec!["L", "H"])
    );

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    let counter_factual = llm_fairness_result.counter_factual.expect("It should have a counter factual object");

    assert_eq!(counter_factual.change_rate_overall, 1.0);
    assert_eq!(counter_factual.sensible_attribute, "male");
    assert_eq!(counter_factual.change_rate_sensible_attributes, vec![1.0, 1.0]);

    // test saved model
    let model = get_model(&pic, canister_id, model_id);
    let llm_data = get_llm_model_data(&model);
    let llm_evaluation = llm_data.evaluations.get(0).expect("Created model should have one evaluation");
    let llm_data_points = llm_evaluation.llm_data_points.as_ref().expect("llm_data_points should have data");
    assert_eq!(llm_data_points.len(), 2);

    let (_, cf_prompts, _) = get_row_data(2, 1, false);
    let dp1 = llm_data_points.get(0).expect("It should contain two data points");
    assert_counter_factual_data(&dp1, cf_prompts[0].clone(), true);
    let dp2 = llm_data_points.get(1).expect("It should contain two data points");
    assert_counter_factual_data(&dp2, cf_prompts[1].clone(), true);
}

#[test]
fn test_llm_counterfactual_fairness_50_percent_change() {
    let (pic, canister_id) = create_pic();
    let (llm_fairness_result, model_id) = llm_fairness_with_variable_queries_test(
        &pic, canister_id,
        "pisa_test", vec!["H", "L"], Some(vec!["L", "L"])
    );

    assert_eq!(llm_fairness_result.queries, 2);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    let counter_factual = llm_fairness_result.counter_factual.expect("It should have a counter factual object");

    assert_eq!(counter_factual.change_rate_overall, 0.5);
    assert_eq!(counter_factual.sensible_attribute, "male");
    assert_eq!(counter_factual.change_rate_sensible_attributes, vec![0.0, 1.0]);

    // test saved model
    let model = get_model(&pic, canister_id, model_id);
    let llm_data = get_llm_model_data(&model);
    let llm_evaluation = llm_data.evaluations.get(0).expect("Created model should have one evaluation");
    let llm_data_points = llm_evaluation.llm_data_points.as_ref().expect("llm_data_points should have data");
    assert_eq!(llm_data_points.len(), 2);

    let (_, cf_prompts, _) = get_row_data(2, 1, false);
    let dp1 = llm_data_points.get(0).expect("It should contain two data points");
    assert_counter_factual_data(&dp1, cf_prompts[0].clone(), true);
    let dp2 = llm_data_points.get(1).expect("It should contain two data points");
    assert_counter_factual_data(&dp2, cf_prompts[1].clone(), true);
}

#[test]
fn test_llm_fairness_metrics_with_pisa_test() {
    let responses = vec![
        "H", "L", "H", "H", "L", "H", "H", "H", "H", "L", 
        "H", "L", "L", "H", "H", "L", "H", "H", "H", "L"
    ];
    let (pic, canister_id) = create_pic();
    let (llm_fairness_result, model_id) = llm_fairness_with_variable_queries_test(
        &pic, canister_id,
        "pisa_test", responses.clone(), None);

    assert_eq!(llm_fairness_result.queries, 20);
    assert_eq!(llm_fairness_result.invalid_responses, 0);
    assert_eq!(llm_fairness_result.call_errors, 0);

    assert!(llm_fairness_result.metrics.accuracy.expect("It should return accuracy") - 0.7 < EPSILON);
    assert!(llm_fairness_result.metrics.precision.expect("It should return precision") - 0.6923076923076923 < EPSILON);
    assert!(llm_fairness_result.metrics.recall.expect("It should return recall") - 0.8181818181818182 < EPSILON);
    
    assert!( (llm_fairness_result.metrics.average_metrics.disparate_impact.expect("It should return disparate_impact") - 0.763888888888889).abs() < EPSILON);
    assert!( (llm_fairness_result.metrics.average_metrics.average_odds_difference.expect("It should return average_odds_difference") - 0.10357143).abs()  < EPSILON);
    assert!( (llm_fairness_result.metrics.average_metrics.equal_opportunity_difference.expect("It should return equal_opportunity_difference") - (-0.1071428571428571)).abs() < EPSILON);
    assert!( (llm_fairness_result.metrics.average_metrics.statistical_parity_difference.expect("It should return statistical_parity_difference") - (-0.1717171717171717)).abs() < EPSILON);

    // test saved model
    let model = get_model(&pic, canister_id, model_id);
    let llm_data = get_llm_model_data(&model);
    let llm_evaluation = llm_data.evaluations.get(0).expect("Created model should have one evaluation");
    let llm_data_points = llm_evaluation.llm_data_points.as_ref().expect("llm_data_points should have data");
    assert_eq!(llm_data_points.len(), 20);

    let (prompts, cf_prompts, _) = get_row_data(20, 1, true);
    for i in 0..20 {
        // do basic expectations on every data point
        let dp = llm_data_points.get(i).expect("It should be able to return a data point");

        assert_eq!(dp.valid, true);
        assert_eq!(dp.error, false);
        assert_eq!(dp.features.len(), 1);
        assert_ne!(dp.timestamp, 0.0 as u64);
        assert_ne!(dp.predicted, None);
        assert_eq!(dp.response, Some(responses[i].to_string()));
        assert_eq!(dp.prompt, prompts[i]);

        assert_counter_factual_data(&dp, cf_prompts[i].clone(), true);
    }
}   

#[test]
fn test_calculate_all_llm_metrics_integration() {
    let (pic, canister_id) = create_pic();
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name);
    add_hf_api_key(&pic, canister_id, model_id);

    let seed: u32 = 1;
    let max_queries: usize = 16;
    
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "calculate_all_llm_metrics",
        encoded_args,
    ).expect("calculate_all_llm_metrics call should succeed");

    // There are two datasets, and every dataset has 16 queries, and every query
    // should do a normal query and a counter factual query = 64 calls

    // This test assumes pisa dataset will be called first
    let mocked_texts = vec!["H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "1", "1", "0", "0", "1", "1", "0", "0",
                            "1", "1", "0", "0", "1", "1", "0", "0",
                            "1", "1", "0", "0", "1", "1", "0", "0",
                            "1", "1", "0", "0", "1", "1", "0", "0"];

    // Await the results and verify
    let reply = wait_for_mocks(&pic, call_id, mocked_texts);
    let result: Result<LLMMetricsAPIResult, String> = decode_one(&reply).expect("Failed to decode calculate_all_llm_metrics reply");
    assert!(result.is_ok(), "Integration test for calculate_all_llm_metrics should succeed");
}

#[test]
fn test_calculate_all_llm_metrics_with_non_existing_model() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = 999; // non-existing model ID
    let seed: u32 = 1;
    let max_queries: usize = 2;

    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "calculate_all_llm_metrics",
        encoded_args,
    ).expect("calculate_all_llm_metrics call should be submitted even if the model does not exist");

    // No need to mock responses because the model does not exist

    // Await the results and verify expected failure
    let reply = pic.await_call(call_id).unwrap();
    let result: Result<LLMMetricsAPIResult, String> = decode_one(&reply).expect("Failed to decode calculate_all_llm_metrics reply");
    assert!(result.is_err(), "Should fail for non-existing model");
    let err = result.unwrap_err();
    assert_eq!(err, "301: Model not found");
}

#[test]
fn test_llm_fairness_datasets_integration_should_return_a_list_of_strings() {
    let (pic, canister_id) = create_pic();
    let expected_datasets = vec!["pisa".to_string(), "compas".to_string()];

    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "llm_fairness_datasets",
        Encode!().unwrap(),
    ).expect("calculate_all_llm_metrics call should not fail.");

    let reply = pic.await_call(call_id).unwrap();
    let datasets: Vec<String> = decode_one(&reply).expect("Failed to decode llm_fairness_datasets reply");
    
    assert_eq!(datasets, expected_datasets, "Datasets should match the expected datasets.");
}

#[test]
fn test_average_llm_metrics_integration_happy_path() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());

    add_hf_api_key(&pic, canister_id, model_id);
    
    let seed: u32 = 1;
    let max_queries: usize = 20;

    // It should not have an average metrics saved 
    let model = get_model(&pic, canister_id, model_id);
    let model_data = get_llm_model_data(&model);
    assert!(model_data.average_fairness_metrics.is_none());

    // dataset pisa

    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, "pisa", max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "calculate_llm_metrics",
        encoded_args,
    ).expect("calculate_llm_metrics call should succeed");

    let mocked_texts = vec!["H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
    ];
    
    let reply = wait_for_mocks(&pic, call_id, mocked_texts);
    let reply: Result<LLMMetricsAPIResult, String> = decode_one(&reply).expect("Failed to decode calculate_llm_metrics reply");
    reply.expect("It should return a non-error value");

    // dataset compas
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, "compas", max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "calculate_llm_metrics",
        encoded_args,
    ).expect("calculate_llm_metrics call should succeed");

    let mocked_texts = vec!["1", "1", "0", "0", "1", "1", "0", "0",
                            "1", "1", "0", "0", "1", "1", "0", "0",
                            "1", "1", "0", "0", "1", "1", "0", "0",
                            "1", "1", "0", "0", "1", "1", "0", "0",
                            "1", "1", "0", "0", "1", "1", "0", "0",
    ];

    let reply = wait_for_mocks(&pic, call_id.clone(), mocked_texts);
    let reply: Result<LLMMetricsAPIResult, String> = decode_one(&reply).expect("Failed to decode calculate_llm_metrics reply");
    reply.expect("It should return a non error value");
    
    // Calculate average metrics
    let datasets = vec!["pisa".to_string(), "compas".to_string()];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, datasets, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "average_llm_metrics",
        encoded_args,
    ).expect("average_llm_metrics call should be submitted even if the model does not exist");

    let reply = pic.await_call(call_id).unwrap();
    let average_metrics_result: Result<AverageLLMFairnessMetrics, GenericError> = decode_one(&reply).expect("Failed to decode average_llm_metrics reply");

    let average_metrics_result = average_metrics_result.expect("It should return a non error");

    // Simple assertions to check we have received an answer
    assert!(average_metrics_result.accuracy >= 0.0, "Accuracy should be a non-negative value");
    assert!(average_metrics_result.precision >= 0.0, "Precision should be a non-negative value");
    assert!(average_metrics_result.recall >= 0.0, "Recall should be a non-negative value");

    // checks the metrics were saved
    let model = get_model(&pic, canister_id, model_id);
    let model_data = get_llm_model_data(&model);

    assert!(model_data.average_fairness_metrics.is_some());
    let average_fairness_metrics = model_data.average_fairness_metrics.unwrap();
    assert_eq!(average_fairness_metrics.model_evaluation_ids.len(), 2);
    assert!(average_fairness_metrics.model_evaluation_ids.contains(&(1 as u128)));
    assert!(average_fairness_metrics.model_evaluation_ids.contains(&(2 as u128)));

    // Check the saved result is the same as the returned result
    assert_eq!(average_fairness_metrics.statistical_parity_difference, average_metrics_result.statistical_parity_difference);
    assert_eq!(average_fairness_metrics.disparate_impact, average_metrics_result.disparate_impact);
    assert_eq!(average_fairness_metrics.average_odds_difference, average_metrics_result.average_odds_difference);
    assert_eq!(average_fairness_metrics.equal_opportunity_difference, average_metrics_result.equal_opportunity_difference);
    
    assert_eq!(average_fairness_metrics.accuracy, average_metrics_result.accuracy);
    assert_eq!(average_fairness_metrics.precision, average_metrics_result.precision);
    assert_eq!(average_fairness_metrics.recall, average_metrics_result.recall);
    assert_eq!(average_fairness_metrics.counter_factual_overall_change_rate, average_metrics_result.counter_factual_overall_change_rate);

    // Code below checks that the calculated averages are the expected
    let mut statistical_parity_difference:f32 = 0.0;
    let mut disparate_impact:f32 = 0.0;
    let mut average_odds_difference:f32 = 0.0;
    let mut equal_opportunity_difference:f32 = 0.0;
    let mut accuracy:f32 = 0.0;
    let mut precision:f32 = 0.0;
    let mut recall:f32 = 0.0;
    let mut change_rate_overall: f32 = 0.0;

    let evaluations: Vec<ModelEvaluationResult> = model_data
        .evaluations
        .into_iter()
        .filter( |x| x.model_evaluation_id == 1 || x.model_evaluation_id == 2)
        .collect();

    let count = evaluations.len() as f32;
    for evaluation in evaluations {
        statistical_parity_difference += evaluation.metrics.statistical_parity_difference.unwrap().get(0).unwrap().value;
        disparate_impact += evaluation.metrics.disparate_impact.unwrap().get(0).unwrap().value;
        average_odds_difference += evaluation.metrics.average_odds_difference.unwrap().get(0).unwrap().value;
        equal_opportunity_difference += evaluation.metrics.equal_opportunity_difference.unwrap().get(0).unwrap().value;
        accuracy += evaluation.metrics.accuracy.unwrap();
        precision += evaluation.metrics.precision.unwrap();
        recall += evaluation.metrics.recall.unwrap();
        change_rate_overall += evaluation.counter_factual.unwrap().change_rate_overall;
    }
    
    statistical_parity_difference /= count;
    disparate_impact /= count;
    average_odds_difference /= count;
    equal_opportunity_difference /= count;
    accuracy /= count;
    precision /= count;
    recall /= count;
    change_rate_overall /= count;

    assert!((average_fairness_metrics.statistical_parity_difference - statistical_parity_difference).abs() < EPSILON);
    assert!((average_fairness_metrics.disparate_impact - disparate_impact).abs() < EPSILON);
    assert!((average_fairness_metrics.average_odds_difference - average_odds_difference).abs() < EPSILON);
    assert!((average_fairness_metrics.equal_opportunity_difference - equal_opportunity_difference).abs() < EPSILON);
    assert!((average_fairness_metrics.accuracy - accuracy).abs() < EPSILON);
    assert!((average_fairness_metrics.precision - precision).abs() < EPSILON);
    assert!((average_fairness_metrics.recall - recall).abs() < EPSILON);
    assert!((average_fairness_metrics.counter_factual_overall_change_rate - change_rate_overall).abs() < EPSILON);
}

#[test]
fn test_average_llm_metrics_should_error_when_dataset_was_not_calculated() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());

    add_hf_api_key(&pic, canister_id, model_id);
    
    let seed: u32 = 1;
    let max_queries: usize = 16;

    // dataset for pisa is created, but not for compas

    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, "pisa", max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "calculate_llm_metrics",
        encoded_args,
    ).expect("calculate_llm_metrics call should succeed");

    // Assuming pisa dataset will be called first
    let mocked_texts = vec!["H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L",
                            "H", "H", "L", "L","H", "H", "L", "L"];
    let reply = wait_for_mocks(&pic, call_id, mocked_texts);
    let reply: Result<LLMMetricsAPIResult, String> = decode_one(&reply).expect("Failed to decode calculate_llm_metrics reply");
    reply.expect("It should return a non-error value");

    // Calculate average metrics
    let datasets = vec!["pisa".to_string(), "compas".to_string()];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, datasets, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "average_llm_metrics",
        encoded_args,
    ).expect("average_llm_metrics call should be submitted even if the model does not exist");

    let reply = pic.await_call(call_id).unwrap();
    let average_metrics_result: Result<AverageLLMFairnessMetrics, GenericError> = decode_one(&reply).expect("Failed to decode average_llm_metrics reply");

    assert!(average_metrics_result.is_err(), "Result should be an error");

    let error: GenericError = average_metrics_result.unwrap_err();

    assert_eq!(error.category, 300);
    assert_eq!(error.code, 300);
    assert_eq!(error.message, "No evaluations found for the dataset `compas`.");
}

#[test]
fn test_average_llm_metrics_with_unknown_dataset_should_return_error() {
    let (pic, canister_id) = create_pic();
    let model_id: u128 = create_llm_model(&pic, canister_id, "Test Model".to_string());
    let seed: u32 = 1;
    let max_queries: usize = 16;
    
   // Calculate average metrics
    let datasets = vec!["unknown_dataset".to_string()];
    // Submit an update call to the test canister to calculate all LLM metrics
    let encoded_args = encode_args((model_id, datasets, max_queries, seed)).unwrap();
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "average_llm_metrics",
        encoded_args,
    ).expect("average_llm_metrics call should be submitted even if the model does not exist");

    let reply = pic.await_call(call_id).unwrap();
    let average_metrics_result: Result<AverageLLMFairnessMetrics, GenericError> = decode_one(&reply).expect("Failed to decode average_llm_metrics reply");

    assert!(average_metrics_result.is_err(), "Result should be an error");

    let error: GenericError = average_metrics_result.unwrap_err();

    assert_eq!(error.category, 300);
    assert_eq!(error.code, 300);
    assert_eq!(error.message, "No evaluations found for the dataset `unknown_dataset`.");

}
