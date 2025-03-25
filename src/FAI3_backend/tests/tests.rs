use candid::{Principal, encode_one, decode_one, encode_args, types::number::Nat};
use ic_management_canister_types::CanisterId;
use pocket_ic::PocketIc;
use FAI3_backend::types::{Model, ModelDetails, ClassifierModelData, get_classifier_model_data};

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
    println!("{:?}", wasm_path);
    let canonical_path = std::fs::canonicalize(wasm_path).expect("WASM file path cannot be resolved to a canonical form");
    println!("Canonical WASM Path: {:?}", canonical_path);
    std::fs::read(canonical_path).unwrap()
}

fn create_test_model(pic: &PocketIc, canister_id: CanisterId, model_name: String, model_type: &str) -> u128 {
    let method_to_call = match model_type {
        "classifier" => "add_classifier_model",
        "llm" => "add_llm_model",
        _ => panic!("Unknown model_type"),
    };
    
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
        method_to_call,
        encoded_args
    ).expect(format!("Failed to call {} method", method_to_call).as_str());

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

#[test]
/// Tests the creation, get and deletion of a model
fn test_classifier_model_happy_path() {
    let pic = PocketIc::new();

    // Create a canister and charge it with 2T cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, INIT_CYCLES);

    // Install the counter canister wasm file with the ping function on the canister.
    let canister_wasm = test_canister_wasm();
    pic.install_canister(canister_id, canister_wasm, vec![], None);

    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 0);

    // Testing add_classifier_model.
    // Define model name and model details for testing add_classifier_model.
    let model_name = String::from("Test Model");
    let model_id: u128 = create_test_model(&pic, canister_id, model_name.clone(), "classifier");
    assert_eq!(model_id, 1);

    // Re-query the get_all_models method to check that a model has been added.
    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 1);

    // Testing get_model
    let decoded_reply: Model = get_model(&pic, canister_id, 1 as u128);

    assert_eq!(decoded_reply.model_id, 1);
    assert_eq!(decoded_reply.model_name, model_name);

    // test model details
    assert_eq!(decoded_reply.details.description, "Example model for testing");
    assert_eq!(decoded_reply.details.framework, "Rust ML");
    assert_eq!(decoded_reply.details.version, "v1.0");
    assert_eq!(decoded_reply.details.objective, "Testing functionality");
    assert_eq!(decoded_reply.details.url, "http://example.com/testmodel");

    // test classifier data
    let classifier_data: ClassifierModelData = get_classifier_model_data(&decoded_reply);

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
