use candid::{Principal, decode_one, encode_args};
use pocket_ic::PocketIc;
use FAI3_backend::types::{LLMMetricsAPIResult, LLMDataPoint, get_llm_model_data};
use FAI3_backend::llm_fairness::{build_prompts, PISA_PROMPT};
use std::collections::HashMap;
mod common;
use common::{
    create_pic, create_llm_model, get_model,
    wait_for_http_request, mock_http_response, mock_correct_hugging_face_response_body
};

const PISA_TRAIN_CSV: &str = include_str!("./../src/data/pisa2009_train_processed.csv");
const PISA_TEST_CSV: &str = include_str!("./../src/data/pisa2009_test_processed.csv");
const PISA_CURATED_CSV: &str = include_str!("./../src/data/pisa2009_test_processed_curated.csv");    

fn get_row_data(prompts: usize, seed: u32, curated: bool) -> (Vec<String>, Vec<String>, Vec<HashMap<String, String>>) {

    let sensible_attribute = "male";
    let predict_attribute = "readingScore";
    let sensible_attribute_values = &["0", "1"];
    let predict_attributes_values = &["L", "H"];
    let reader_score_column = "readingScore";
    
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
    
    for i in 0..prompts {
        println!("Armando para query number {}", i);
        let (personalized_prompt, personalized_prompt_cf) = build_prompts(
            &records, predict_attribute,
            sensible_attribute_values, predict_attributes_values,
            sensible_attribute, reader_score_column, seed, i,
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

    assert!(llm_fairness_result.metrics.accuracy.expect("It should return accuracy") - 0.7 < 1e-6);
    assert!(llm_fairness_result.metrics.precision.expect("It should return precision") - 0.6923076923076923 < 1e-6);
    assert!(llm_fairness_result.metrics.recall.expect("It should return recall") - 0.8181818181818182 < 1e-6);
    
    assert!( (llm_fairness_result.metrics.average_metrics.disparate_impact.expect("It should return disparate_impact") - 0.763888888888889).abs() < 1e-6);
    assert!( (llm_fairness_result.metrics.average_metrics.average_odds_difference.expect("It should return average_odds_difference") - 0.10357143).abs()  < 1e-6);
    assert!( (llm_fairness_result.metrics.average_metrics.equal_opportunity_difference.expect("It should return equal_opportunity_difference") - (-0.1071428571428571)).abs() < 1e-6);
    assert!( (llm_fairness_result.metrics.average_metrics.statistical_parity_difference.expect("It should return statistical_parity_difference") - (-0.1717171717171717)).abs() < 1e-6);

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
