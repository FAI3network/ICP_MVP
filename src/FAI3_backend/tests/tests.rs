use candid::{Principal, encode_one, decode_one, encode_args};
use ic_management_canister_types::CanisterId;
use pocket_ic::{
    PocketIc,
    common::rest::{
        CanisterHttpReply, CanisterHttpResponse,
        MockCanisterHttpResponse, CanisterHttpRequest,
    },
};
use FAI3_backend::types::{
    Model, ModelDetails, ClassifierModelData, get_classifier_model_data,
    LLMModelData, get_llm_model_data, ContextAssociationTestAPIResult
};

// 2T cycles
const INIT_CYCLES: u128 = 2_000_000_000_000;

#[test]
fn test_ping_canister() {
    let pic = PocketIc::new();

    // Create a canister and charge it with 2T cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);

    // Install the counter canister wasm file with the ping function on the canister.
    let canister_wasm = test_canister_wasm();
    pic.install_canister(canister_id, canister_wasm, vec![], None);

    // Make a call to the canister ping method.
    let reply = pic.query_call(
        canister_id,
        Principal::anonymous(),
        "ping",
        encode_one(()).unwrap()
    ).expect("Failed to call ping method");

    // Decode the Candid serialized reply to match the expected plain string.
    let decoded_reply: String = decode_one(&reply).expect("Failed to decode reply");
    assert_eq!(decoded_reply, "Canister is alive!");
}

/// Retrieves the test canister WASM file. It requires that the file path is in the ENV variable TEST_WASM.
fn test_canister_wasm() -> Vec<u8> {
    let wasm_path = std::env::var("TEST_WASM").expect("Missing test canister wasm file");
    let canonical_path = std::fs::canonicalize(wasm_path).expect("WASM file path cannot be resolved to a canonical form");
    println!("Canonical WASM Path: {:?}", canonical_path);
    std::fs::read(canonical_path).unwrap()
}

fn create_classifier_model(pic: &PocketIc, canister_id: CanisterId, model_name: String) -> u128 {
    let model_details = ModelDetails {
        description: "Example model for testing".to_string(),
        framework: "Rust ML".to_string(),
        version: "v1.0".to_string(),
        objective: "Testing functionality".to_string(),
        url: "http://example.com/testmodel".to_string()
    };

    let encoded_args = encode_args((model_name.clone(), model_details)).unwrap();

    // Testing add_classifier_model.
    let create_model_reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "add_classifier_model",
        encoded_args
    ).expect("Failed to call add_classifier_model method");

    let decoded_reply: u128 = decode_one(&create_model_reply).expect("Failed to decode create model reply");

    return decoded_reply;
}

fn create_llm_model(pic: &PocketIc, canister_id: CanisterId, model_name: String) -> u128 {
    let model_details = ModelDetails {
        description: "Example model for testing".to_string(),
        framework: "Rust ML".to_string(),
        version: "v1.0".to_string(),
        objective: "Testing functionality".to_string(),
        url: "http://example.com/testmodel".to_string()
    };

    let encoded_args = encode_args((model_name.clone(), "mistralai/Mistral-Nemo-Instruct-2407", model_details)).unwrap();

    // Testing add_classifier_model.
    let create_model_reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "add_llm_model",
        encoded_args
    ).expect("Failed to call add_llm_model method");

    let decoded_reply: u128 = decode_one(&create_model_reply).expect("Failed to decode create model reply");

    return decoded_reply;
}

fn get_all_models(pic: &PocketIc, canister_id: CanisterId) -> Vec<Model> {
    // Testing get_all_models method.
    let reply = pic.query_call(
        canister_id,
        Principal::anonymous(),
        "get_all_models",
        encode_one(()).unwrap()
    ).expect("Failed to call get_all_models method");

    // Decode the Candid serialized reply to match the expected plain string.
    return decode_one(&reply).expect("Failed to decode reply");
}

fn delete_model(pic: &PocketIc, canister_id: CanisterId, model_id: u128) {
    let delete_model_reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "delete_model",
        encode_one(model_id).unwrap()
    ).expect("Failed to call delete_model method");

    let decoded_reply: () = decode_one(&delete_model_reply).expect("Failed to decode delete model reply");
    assert_eq!(decoded_reply, ()); // empty reply
}

fn get_model(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> Model {
    // Testing get_model.
    let get_model_reply = pic.query_call(
        canister_id,
        Principal::anonymous(),
        "get_model",
        encode_one(1 as u128).unwrap()
    ).expect("Failed to call get_model with id = 1");

    let decoded_reply: Model = decode_one(&get_model_reply).expect("Failed to decode reply after get_model call.");
    return decoded_reply;
}

fn create_pic() -> (PocketIc, CanisterId) {
    let pic = PocketIc::new();

    // Create a canister and charge it with 2T cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);

    // Install the counter canister wasm file with the ping function on the canister.
    let canister_wasm = test_canister_wasm();
    pic.install_canister(canister_id, canister_wasm, vec![], None);

    (pic, canister_id)
}

/// Waits until the http request is queued
fn wait_for_http_request(pic: &PocketIc) {
    let mut count = 0;
    while pic.get_canister_http().is_empty() {
        println!("Waiting for HTTP request... (attempt {}, {})", count, pic.get_canister_http().len());
        pic.tick();
        count += 1;
        if count > 20 {
            panic!("Timeout waiting for HTTP request!");
        }
    }

    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 1);
}

fn mock_http_response(canister_http_request: &CanisterHttpRequest, body: impl Into<Vec<u8>>) -> MockCanisterHttpResponse {
    let body = body.into();
    MockCanisterHttpResponse {
        subnet_id: canister_http_request.subnet_id,
        request_id: canister_http_request.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: vec![],
            body: body.clone(),
        }),
        additional_responses: vec![],
    }
}

#[test]
/// Tests the creation, get and deletion of a model
fn test_classifier_model_crud() {
    let (pic, canister_id) = create_pic();

    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 0);

    // Testing add_classifier_model.
    // Define model name and model details for testing add_classifier_model.
    let model_name = String::from("Test Model");
    let model_id: u128 = create_classifier_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);

    // Re-query the get_all_models method to check that a model has been added.
    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 1);

    // Testing get_model
    let model: Model = get_model(&pic, canister_id, 1 as u128);
    assert_eq!(model.model_id, 1);
    assert_eq!(model.model_name, model_name);

    // test model details
    assert_eq!(model.details.description, "Example model for testing");
    assert_eq!(model.details.framework, "Rust ML");
    assert_eq!(model.details.version, "v1.0");
    assert_eq!(model.details.objective, "Testing functionality");
    assert_eq!(model.details.url, "http://example.com/testmodel");

    // test classifier data
    let classifier_data: ClassifierModelData = get_classifier_model_data(&model);

    // Ensure classifier data points are initialized as expected
    assert!(classifier_data.data_points.is_empty(), "Initial data points should be empty.");
    assert!(classifier_data.metrics_history.is_empty(), "Initial metrics history should be empty.");

    // Testing initial state of metrics
    let actually_metrics = classifier_data.metrics;
    assert!(actually_metrics.statistical_parity_difference.is_none(), "Initial statistical parity difference should be None.");
    assert!(actually_metrics.disparate_impact.is_none(), "Initial disparate impact should be None.");
    assert!(actually_metrics.average_odds_difference.is_none(), "Initial average odds difference should be None.");
    assert!(actually_metrics.equal_opportunity_difference.is_none(), "Initial equal opportunity difference should be None.");
    assert!(actually_metrics.average_metrics.statistical_parity_difference.is_none(), "Initial average metrics for statistical parity difference should be None.");
    assert!(actually_metrics.average_metrics.disparate_impact.is_none(), "Initial average metrics for disparate impact should be None.");
    assert!(actually_metrics.average_metrics.average_odds_difference.is_none(), "Initial average metrics for average odds difference should be None.");
    assert!(actually_metrics.average_metrics.equal_opportunity_difference.is_none(), "Initial average metrics for equal opportunity difference should be None.");
    assert!(actually_metrics.accuracy.is_none(), "Initial accuracy should be None.");
    assert!(actually_metrics.recall.is_none(), "Initial recall should be None.");
    assert!(actually_metrics.precision.is_none(), "Initial precision should be None.");
    assert_eq!(actually_metrics.timestamp, 0, "Initial timestamp should be 0.");

    // Testing delete model
    delete_model(&pic, canister_id, 1 as u128);

    // Now no model should be present
    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 0);  
}

#[test]
/// Tests the creation, get and deletion of a model
fn test_llm_model_crud() {
    let (pic, canister_id) = create_pic();
    
    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 0);

    // Creating model
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);

    // Re-query the get_all_models method to check that a model has been added.
    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 1);

    // Testing get_model
    let model: Model = get_model(&pic, canister_id, 1 as u128);
    assert_eq!(model.model_id, 1);
    assert_eq!(model.model_name, model_name);

    // test model details
    assert_eq!(model.details.description, "Example model for testing");
    assert_eq!(model.details.framework, "Rust ML");
    assert_eq!(model.details.version, "v1.0");
    assert_eq!(model.details.objective, "Testing functionality");
    assert_eq!(model.details.url, "http://example.com/testmodel");

    let llm_data: LLMModelData = get_llm_model_data(&model);
    // Ensure LLM model data is initialized as expected
    assert!(llm_data.cat_metrics.is_none(), "Initial CAT metrics should be None.");
    assert!(llm_data.cat_metrics_history.is_empty(), "Initial CAT metrics history should be empty.");
    assert_eq!(llm_data.hugging_face_url, "mistralai/Mistral-Nemo-Instruct-2407", "Hugging Face URL should be initialized.");

    // Testing delete model
    delete_model(&pic, canister_id, 1 as u128);

    // Now no model should be present
    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 0);
}

#[test]
/// CAT test should return successfully, but show 2 errors in the response
fn test_llm_cat_test_wrong_hugging_face_responses() {
    let (pic, canister_id) = create_pic();
    
    // Creating model
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);

    // Calling context_association_test
    let max_queries: usize = 2;
    let seed: u32 = 1;
    let shuffle_questions: bool = true;
    let encoded_args = encode_args((model_id, max_queries, seed, shuffle_questions)).unwrap();
    // Submit an update call to the test canister making a canister http outcall
    // and mock a canister http outcall response.
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "context_association_test",
        encoded_args,
    ).expect("context_association_test call should succeed");

    // We need a pair of ticks for the test canister method to make the http outcall
    // and for the management canister to start processing the http outcall.
    wait_for_http_request(&pic);
    
    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_canister_http_response = mock_http_response(canister_http_request, b"hello");
    pic.mock_canister_http_response(mock_canister_http_response);

    wait_for_http_request(&pic);

    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_canister_http_response = mock_http_response(canister_http_request, b"hello");
    pic.mock_canister_http_response(mock_canister_http_response);
    
    // There should be no more pending canister http outcalls.
    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 0);

    // Now the test canister will receive the http outcall response
    // and reply to the ingress message from the test driver.
    let reply = pic.await_call(call_id).unwrap();
    let decoded_reply: Result<ContextAssociationTestAPIResult, String> = decode_one(&reply).expect("Failed to decode context association test reply");

    let cat_result = decoded_reply.expect("It should be a valid CAT result");
    
    assert_eq!(cat_result.error_count, 2);
    assert_eq!(cat_result.general_n, 0);
}
