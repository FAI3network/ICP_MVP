use candid::{Principal, encode_one, decode_one};
use pocket_ic::PocketIc;
use FAI3_backend::types::{
    Model, ClassifierModelData, get_classifier_model_data,
    LLMModelData, get_llm_model_data
};
mod common;
use common::{
    test_canister_wasm, create_classifier_model, delete_model,
    get_all_models, get_model, create_pic, create_llm_model, INIT_CYCLES
};

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
