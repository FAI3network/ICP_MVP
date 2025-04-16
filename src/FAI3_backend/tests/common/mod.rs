#![allow(dead_code)]
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
    Model, ModelDetails, KeyValuePair, PrivilegedIndex,
};

// 2T cycles
pub const INIT_CYCLES: u128 = 2_000_000_000_000;

/// Retrieves the test canister WASM file. It requires that the file path is in the ENV variable TEST_WASM.
pub fn test_canister_wasm() -> Vec<u8> {
    let wasm_path = std::env::var("TEST_WASM").expect("Missing test canister wasm file");
    let canonical_path = std::fs::canonicalize(wasm_path).expect("WASM file path cannot be resolved to a canonical form");
    println!("Canonical WASM Path: {:?}", canonical_path);
    std::fs::read(canonical_path).unwrap()
}

pub fn create_classifier_model(pic: &PocketIc, canister_id: CanisterId, model_name: String) -> u128 {
    let model_details = ModelDetails {
        description: "Example model for testing".to_string(),
        framework: "Rust ML".to_string(),
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

pub fn create_llm_model(pic: &PocketIc, canister_id: CanisterId, model_name: String) -> u128 {
    let model_details = ModelDetails {
        description: "Example model for testing".to_string(),
        framework: "Rust ML".to_string(),
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

/// Adds a mock Hugging Face API key to a model
pub fn add_hf_api_key(pic: &PocketIc, canister_id: CanisterId, model_id: u128) {
    let encoded_args = encode_args(("hugging_face_api_key", "fake-hf-api-key-value")).unwrap();
    
    // Testing add_classifier_model.
    let update_call_reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "set_config",
        encoded_args
    ).expect("Failed to add a mocked Hugging Face API key");

    decode_one::<()>(&update_call_reply).expect("Failed to decode create model reply");
}

pub fn get_all_models(pic: &PocketIc, canister_id: CanisterId) -> Vec<Model> {
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

pub fn delete_model(pic: &PocketIc, canister_id: CanisterId, model_id: u128) {
    let delete_model_reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "delete_model",
        encode_one(model_id).unwrap()
    ).expect("Failed to call delete_model method");

    let decoded_reply: () = decode_one(&delete_model_reply).expect("Failed to decode delete model reply");
    assert_eq!(decoded_reply, ()); // empty reply
}

pub fn get_model(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> Model {
    // Testing get_model.
    let get_model_reply = pic.query_call(
        canister_id,
        Principal::anonymous(),
        "get_model",
        encode_one(model_id).unwrap()
    ).expect("Failed to call get_model with id = 1");

    let decoded_reply: Model = decode_one(&get_model_reply).expect("Failed to decode reply after get_model call.");
    return decoded_reply;
}

pub fn create_pic() -> (PocketIc, CanisterId) {
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
pub fn wait_for_http_request(pic: &PocketIc) {
    let mut count = 0;
    while pic.get_canister_http().is_empty() {
        println!("Waiting for HTTP request... (attempt {}, {})", count, pic.get_canister_http().len());
        pic.tick();
        count += 1;
        if count > 100 {
            panic!("Timeout waiting for HTTP request!");
        }
    }

    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 1);
}

pub fn mock_http_response(canister_http_request: &CanisterHttpRequest, body: impl Into<Vec<u8>>) -> MockCanisterHttpResponse {
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

pub fn add_dataset(
    pic: &PocketIc, canister_id: CanisterId,
    model_id: u128, features: Vec<Vec<f64>>, labels: Vec<bool>,
    predictions: Vec<bool>, privileged: Vec<KeyValuePair>, selection_labels: Vec<String>) -> Result<(), candid::Error> {

    let encoded_args = encode_args((model_id, features, labels, predictions, privileged, selection_labels)).unwrap();
    // Testing add_classifier_model.
    let create_model_reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "add_dataset",
        encoded_args
    ).expect("Failed to call add_classifier_model method");

    return decode_one(&create_model_reply);
}

pub fn calculate_statistical_parity_difference(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> Vec<PrivilegedIndex> {
    let reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "calculate_statistical_parity_difference",
        encode_args((model_id, None::<Vec<(String, (f64, bool))>>)).unwrap()
    ).expect("Failed to call calculate_statistical_parity_difference method");

    decode_one(&reply).expect("Failed to decode reply after calling calculate_statistical_parity_difference")
}

pub fn calculate_disparate_impact(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> Vec<PrivilegedIndex> {
    let reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "calculate_disparate_impact",
        encode_args((model_id, None::<Vec<(String, (f64, bool))>>)).unwrap()
    ).expect("Failed to call calculate_disparate_impact method");

    decode_one(&reply).expect("Failed to decode reply after calling calculate_disparate_impact")
}

pub fn calculate_average_odds_difference(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> Vec<PrivilegedIndex> {
    let reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "calculate_average_odds_difference",
        encode_args((model_id, None::<Vec<(String, (f64, bool))>>)).unwrap()
    ).expect("Failed to call calculate_average_odds_difference method");

    decode_one(&reply).expect("Failed to decode reply after calling calculate_average_odds_difference")
}

pub fn calculate_equal_opportunity_difference(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> Vec<PrivilegedIndex> {
    let reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "calculate_equal_opportunity_difference",
        encode_args((model_id, None::<Vec<(String, (f64, bool))>>)).unwrap()
    ).expect("Failed to call calculate_equal_opportunity_difference method");

    decode_one(&reply).expect("Failed to decode reply after calling calculate_equal_opportunity_difference")
}

pub fn calculate_accuracy(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> f32 {
    let reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "calculate_accuracy",
        encode_one(model_id).unwrap()
    ).expect("Failed to call calculate_accuracy method");

    decode_one(&reply).expect("Failed to decode reply after calling calculate_accuracy")
}

pub fn calculate_precision(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> f32 {
    let reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "calculate_precision",
        encode_one(model_id).unwrap()
    ).expect("Failed to call calculate_precision method");

    decode_one(&reply).expect("Failed to decode reply after calling calculate_precision")
}

pub fn calculate_recall(pic: &PocketIc, canister_id: CanisterId, model_id: u128) -> f32 {
    let reply = pic.update_call(
        canister_id,
        Principal::anonymous(),
        "calculate_recall",
        encode_one(model_id).unwrap()
    ).expect("Failed to call calculate_recall method");

    decode_one(&reply).expect("Failed to decode reply after calling calculate_recall")
}

pub fn mock_correct_hugging_face_response_body(generated_text: &str) -> String {
    serde_json::json!([
        {
            "generated_text": generated_text, 
            "details": {
                "finish_reason": "stop_sequence", 
                "generated_tokens": 1, 
                "seed": 1, 
                "prefill": [], 
                "tokens": [
                    {
                        "id": 1424, 
                        "text": generated_text, 
                        "logprob": -0.37548828,
                        "special": false
                    }
                ]
            }
        }
    ]).to_string()
}
