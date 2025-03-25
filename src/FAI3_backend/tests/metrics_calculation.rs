use candid::{Principal, decode_one, encode_args};
use FAI3_backend::types::{
    Model, ClassifierModelData, get_classifier_model_data, KeyValuePair,
};
mod common;
use common::{
    create_pic, create_classifier_model, get_model, delete_model, get_all_models, add_dataset,
    calculate_accuracy, calculate_average_odds_difference, calculate_disparate_impact, calculate_equal_opportunity_difference,
    calculate_precision, calculate_recall, calculate_statistical_parity_difference
};

#[test]
/// Tests the creation, get and deletion of a model
fn test_add_dataset_and_calculate_metrics() {
    let (pic, canister_id) = create_pic();

    // Testing add_classifier_model.
    // Define model name and model details for testing add_classifier_model.
    let model_name = String::from("Test Model");
    let model_id: u128 = create_classifier_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);

    // Testing get_model
    let model: Model = get_model(&pic, canister_id, 1 as u128);

    // Using a similar than the pisa set that is on data
    let features : Vec<Vec<f64>> = vec![
        vec![
            0.0,0.0, 0.0,1.0,1.0,0.0,1.0,1.0,1.0,1.0,
            0.0,0.0, 0.0,1.0,1.0,1.0,0.0,1.0,1.0,0.0,
        ]
    ];
    let labels : Vec<bool> = vec![true, false, false, true, true, true, true, true, false, false, true, true, false, false, true, false, false, true, true, false];
    let predictions: Vec<bool> = vec![true, false, true, true, false, true, true, true, true, false, true, false, false, true, true, false, true, true, true, false];
    let privileged: Vec<KeyValuePair> = vec![
        KeyValuePair {
            key: String::from("male"),
            value: 0,
        }
    ];
    assert_eq!(features[0].len(), labels.len());
    assert_eq!(labels.len(), predictions.len());

    let data_len = labels.len();
    
    let response = add_dataset(&pic, canister_id, model.model_id, features.clone(), labels.clone(), predictions.clone(), privileged, Vec::new());

    if let Err(e) = response {
        panic!("add_dataset failed with error {}", e.to_string());
    }

    // checking metrics were added
    let model = get_model(&pic, canister_id, model_id);

    // test classifier data
    let classifier_data: ClassifierModelData = get_classifier_model_data(&model);

    assert_eq!(classifier_data.data_points.len(), data_len);

    let saved_labels: Vec<bool> = classifier_data.data_points.iter().map(| dp | dp.target ).collect();
    let saved_predictions: Vec<bool> = classifier_data.data_points.iter().map(| dp | dp.predicted ).collect();
    let saved_features: Vec<f64> = classifier_data.data_points.iter().map(| dp | dp.features[0] ).collect();

    assert_eq!(labels, saved_labels);
    assert_eq!(predictions, saved_predictions);
    assert_eq!(features[0], saved_features);
    
    // call calculate metrics
    let statistical_parity_difference = calculate_statistical_parity_difference(&pic, canister_id, model_id);
    let disparate_impact = calculate_disparate_impact(&pic, canister_id, model_id);
    let average_odds_difference = calculate_average_odds_difference(&pic, canister_id, model_id);
    let equal_opportunity_difference= calculate_equal_opportunity_difference(&pic, canister_id, model_id);
    let accuracy = calculate_accuracy(&pic, canister_id, model_id);
    let precision = calculate_precision(&pic, canister_id, model_id);
    let recall = calculate_recall(&pic, canister_id, model_id);
    assert!( (accuracy - 0.7).abs() < 1e-6);
    assert!( (precision - 0.6923076923076923).abs() < 1e-6);
    assert!( (recall - 0.8181818181818182).abs() < 1e-6);
    
    assert!( (disparate_impact[0].value - 0.763888888888889).abs() < 1e-6);
    assert!( (average_odds_difference[0].value - 0.10357143).abs()  < 1e-6);
    assert!( (equal_opportunity_difference[0].value - (-0.1071428571428571)).abs() < 1e-6);
    assert!( (statistical_parity_difference[0].value - (-0.1717171717171717)).abs() < 1e-6);
    // Testing delete model
    delete_model(&pic, canister_id, 1 as u128);

    // Now no model should be present
    let all_models: Vec<Model> = get_all_models(&pic, canister_id);
    assert_eq!(all_models.len(), 0);  
}
