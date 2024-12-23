mod tests;


use candid::{CandidType, Deserialize as CandidDeserialize, Principal};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use ic_cdk::api::management_canister::http_request::{
    http_request, HttpHeader, HttpMethod, HttpRequestArg, HttpResponse,
};
use ic_cdk_macros::*;
use ic_cdk::export::candid::Principal;
use ic_cdk::api::call::RejectionCode;

// Cycles management

const CYCLE_THRESHOLD: u64 = 1_000_000_000;

#[ic_cdk::query]
fn check_cycles() -> u64 {
    ic_cdk::api::canister_balance()
}

#[ic_cdk::update]
fn stop_if_low_cycles() {
    let cycles = ic_cdk::api::canister_balance();
    if cycles < CYCLE_THRESHOLD {
        ic_cdk::trap("Cycle balance too low, stopping execution to avoid canister deletion.");
    }
}

fn check_cycles_before_action() {
    stop_if_low_cycles();
}

// ---------- Data Structures ---------- //

// For canister data structures, use Candid's versions: CandidType + CandidDeserialize
#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct DataPoint {
    data_point_id: u128,
    target: bool,
    privileged: bool,
    predicted: bool,
    features: Vec<f64>,
    timestamp: u64,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Metrics {
    statistical_parity_difference: Option<f32>,
    disparate_impact: Option<f32>,
    average_odds_difference: Option<f32>,
    equal_opportunity_difference: Option<f32>,
    accuracy: Option<f32>,
    precision: Option<f32>,
    recall: Option<f32>,
    timestamp: u64,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct ModelDetails {
    description: String,
    framework: String,
    version: String,
    objective: String,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct Model {
    model_id: u128,
    model_name: String,
    user_id: Principal,
    data_points: Vec<DataPoint>,
    metrics: Metrics,
    details: ModelDetails,
    metrics_history: Vec<Metrics>,
}

#[derive(CandidType, CandidDeserialize, Clone, Debug)]
pub struct User {
    user_id: Principal,
    models: HashMap<u128, Model>,
}

// ---------- Global State ---------- //

thread_local! {
    static USERS: RefCell<HashMap<Principal, User>> = RefCell::new(HashMap::new());
    static NEXT_MODEL_ID: RefCell<u128> = RefCell::new(1);
    static NEXT_DATA_POINT_ID: RefCell<u128> = RefCell::new(1);
}

// ---------- Operations ---------- //

#[ic_cdk::update]
fn add_dataset(
    model_id: u128,
    features: Vec<Vec<f64>>,
    labels: Vec<bool>,
    predictions: Vec<bool>,
    privilege_indices: Vec<u128>,
) {
    check_cycles_before_action();

    // Verify that all columns have consistent lengths
    let data_length = labels.len();
    if predictions.len() != data_length {
        ic_cdk::api::trap("Error: Lengths of labels and predictions must be equal.");
    }
    for feature_column in &features {
        if feature_column.len() != data_length {
            ic_cdk::api::trap("Error: All feature columns must have the same length as labels.");
        }
    }

    let caller = ic_cdk::api::caller();
    let timestamp = ic_cdk::api::time();

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).expect("User not found");

        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != caller {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        NEXT_DATA_POINT_ID.with(|next_data_point_id| {
            let mut next_data_point_id = next_data_point_id.borrow_mut();

            for i in 0..data_length {
                let mut feature_vector = Vec::new();
                for feature_column in &features {
                    feature_vector.push(feature_column[i]);
                }

                // Determine if the data point is privileged
                let mut privileged = false;
                for &index in &privilege_indices {
                    let idx = index as usize;
                    if idx < feature_vector.len() && feature_vector[idx] > 0.0 {
                        privileged = true;
                        break;
                    }
                }

                let data_point = DataPoint {
                    data_point_id: *next_data_point_id,
                    target: labels[i],
                    privileged,
                    predicted: predictions[i],
                    features: feature_vector.clone(),
                    timestamp,
                };

                model.data_points.push(data_point);
                *next_data_point_id += 1;
            }
        });
    });
}

#[ic_cdk::update]
fn add_model(model_name: String, model_details: ModelDetails) -> u128 {
    check_cycles_before_action();

    if model_name.trim().is_empty() {
        ic_cdk::api::trap("Error: Model name cannot be empty or null.");
    }

    let caller = ic_cdk::api::caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(caller).or_insert(User {
            user_id: caller,
            models: HashMap::new(),
        });

        NEXT_MODEL_ID.with(|next_model_id| {
            let model_id = *next_model_id.borrow();
            user.models.insert(
                model_id,
                Model {
                    model_id,
                    model_name: model_name.clone(),
                    user_id: caller,
                    data_points: Vec::new(),
                    metrics: Metrics {
                        statistical_parity_difference: None,
                        disparate_impact: None,
                        average_odds_difference: None,
                        equal_opportunity_difference: None,
                        accuracy: None,
                        recall: None,
                        precision: None,
                        timestamp: 0,
                    },
                    details: model_details,
                    metrics_history: Vec::new(),
                },
            );
            *next_model_id.borrow_mut() += 1;
            model_id
        })
    })
}

#[ic_cdk::update]
fn add_data_point(
    model_id: u128,
    target: bool,
    privileged: bool,
    predicted: bool,
    features: Vec<f64>,
) {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();
    let timestamp = ic_cdk::api::time();

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).expect("User not found");

        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");
        if model.user_id != caller {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        NEXT_DATA_POINT_ID.with(|next_data_point_id| {
            let data_point_id = *next_data_point_id.borrow();

            let data_point = DataPoint {
                data_point_id,
                target,
                privileged,
                predicted,
                features,
                timestamp,
            };

            model.data_points.push(data_point);
            *next_data_point_id.borrow_mut() += 1;
        });
    });
}

#[ic_cdk::update]
fn delete_model(model_id: u128) {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).expect("User not found");
        if let Some(model) = user.models.get(&model_id) {
            if model.user_id != caller {
                ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
            }
        }
        user.models
            .remove(&model_id)
            .expect("Model not found or not owned by user");
    });
}

#[ic_cdk::update]
fn delete_data_point(model_id: u128, data_point_id: u128) {
    check_cycles_before_action();
    let caller = ic_cdk::api::caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");
        if model.user_id != caller {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }
        let data_point_index = model
            .data_points
            .iter()
            .position(|d| d.data_point_id == data_point_id)
            .expect("Data point not found");
        model.data_points.remove(data_point_index);
    });
}

// ---------- Metric Calculations ---------- //

#[ic_cdk::update]
fn calculate_statistical_parity_difference(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (priv_count, unpriv_count, priv_pos, unpriv_pos) =
            calculate_group_counts(&model.data_points);

        if priv_count == 0 || unpriv_count == 0 {
            ic_cdk::api::trap("Cannot calculate statistical parity difference: One of the groups has no data points.");
        }

        let privileged_prob = priv_pos as f32 / priv_count as f32;
        let unprivileged_prob = unpriv_pos as f32 / unpriv_count as f32;

        let result = unprivileged_prob - privileged_prob;
        model.metrics.statistical_parity_difference = Some(result);
        model.metrics.timestamp = ic_cdk::api::time();
        result
    })
}

#[ic_cdk::update]
fn calculate_disparate_impact(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (priv_count, unpriv_count, priv_pos, unpriv_pos) =
            calculate_group_counts(&model.data_points);

        if priv_count == 0 || unpriv_count == 0 {
            ic_cdk::api::trap("Cannot calculate statistical parity difference: One of the groups has no data points.");
        }

        let privileged_probability = priv_pos as f32 / priv_count as f32;
        let unprivileged_probability = unpriv_pos as f32 / unpriv_count as f32;

        assert!(
            privileged_probability > 0.0,
            "Privileged group has no positive outcomes"
        );

        let result = unprivileged_probability / privileged_probability;
        model.metrics.disparate_impact = Some(result);
        model.metrics.timestamp = ic_cdk::api::time();
        result
    })
}

#[ic_cdk::update]
fn calculate_average_odds_difference(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (
            p_tp, p_fp, p_tn, p_fn,
            u_tp, u_fp, u_tn, u_fn
        ) = calculate_confusion_matrix(&model.data_points);

        let p_pos_total = p_tp + p_fn;
        let u_pos_total = u_tp + u_fn;
        let p_neg_total = p_fp + p_tn;
        let u_neg_total = u_fp + u_tn;

        if p_pos_total == 0 || u_pos_total == 0 || p_neg_total == 0 || u_neg_total == 0 {
            ic_cdk::api::trap("Cannot calculate average odds difference: One of the groups has no data points or no positives/negatives.");
        }

        let p_tpr = p_tp as f32 / p_pos_total as f32;
        let u_tpr = u_tp as f32 / u_pos_total as f32;
        let p_fpr = p_fp as f32 / p_neg_total as f32;
        let u_fpr = u_fp as f32 / u_neg_total as f32;

        let result = ((u_fpr - p_fpr) + (u_tpr - p_tpr)) / 2.0;
        model.metrics.average_odds_difference = Some(result);
        model.metrics.timestamp = ic_cdk::api::time();
        result
    })
}

#[ic_cdk::update]
fn calculate_equal_opportunity_difference(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (p_tp, p_fn, u_tp, u_fn) = calculate_true_positive_false_negative(&model.data_points);

        let p_pos_total = p_tp + p_fn;
        let u_pos_total = u_tp + u_fn;

        if p_pos_total == 0 || u_pos_total == 0 {
            ic_cdk::api::trap("Cannot calculate equal opportunity difference: One of the groups has no positive data points.");
        }

        let p_tpr = p_tp as f32 / p_pos_total as f32;
        let u_tpr = u_tp as f32 / u_pos_total as f32;

        let result = u_tpr - p_tpr;
        model.metrics.equal_opportunity_difference = Some(result);
        model.metrics_history.push(model.metrics.clone());
        model.metrics.timestamp = ic_cdk::api::time();
        result
    })
}

#[ic_cdk::update]
fn calculate_accuracy(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user.models.get_mut(&model_id).expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (tp, tn, fp, fn_) = calculate_overall_confusion_matrix(&model.data_points);
        let total = tp + tn + fp + fn_;
        if total == 0 {
            ic_cdk::api::trap("No data points to calculate accuracy");
        }

        let accuracy = (tp + tn) as f32 / total as f32;
        model.metrics.accuracy = Some(accuracy);
        model.metrics.timestamp = ic_cdk::api::time();
        accuracy
    })
}

#[ic_cdk::update]
fn calculate_precision(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user.models.get_mut(&model_id).expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (tp, _, fp, _) = calculate_overall_confusion_matrix(&model.data_points);
        let denominator = tp + fp;
        if denominator == 0 {
            ic_cdk::api::trap("No positive predictions to calculate precision");
        }

        let precision = tp as f32 / denominator as f32;
        model.metrics.precision = Some(precision);
        model.metrics.timestamp = ic_cdk::api::time();
        precision
    })
}

#[ic_cdk::update]
fn calculate_recall(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user.models.get_mut(&model_id).expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (tp, _, _, fn_) = calculate_overall_confusion_matrix(&model.data_points);
        let denominator = tp + fn_;
        if denominator == 0 {
            ic_cdk::api::trap("No actual positives to calculate recall");
        }

        let recall = tp as f32 / denominator as f32;
        model.metrics.recall = Some(recall);
        model.metrics.timestamp = ic_cdk::api::time();
        recall
    })
}

#[ic_cdk::update]
fn calculate_all_metrics(model_id: u128) -> (f32, f32, f32, f32, f32, f32, f32) {
    let spd = calculate_statistical_parity_difference(model_id);
    let di = calculate_disparate_impact(model_id);
    let aod = calculate_average_odds_difference(model_id);
    let eod = calculate_equal_opportunity_difference(model_id);
    let acc = calculate_accuracy(model_id);
    let prec = calculate_precision(model_id);
    let rec = calculate_recall(model_id);

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&ic_cdk::api::caller()).expect("User not found");
        let model = user.models.get_mut(&model_id).expect("Model not found or not owned by user");
        model.metrics.timestamp = ic_cdk::api::time();
        model.metrics_history.push(model.metrics.clone());
    });

    (spd, di, aod, eod, acc, prec, rec)
}

// ---------- GETTERS ---------- //

#[ic_cdk::update]
fn test_function() -> bool {
    true
}

#[ic_cdk::query]
fn get_all_models() -> Vec<Model> {
    check_cycles_before_action();
    USERS.with(|users| {
        users
            .borrow()
            .values()
            .flat_map(|user| user.models.values().cloned())
            .collect()
    })
}

#[ic_cdk::query]
fn get_model_data_points(model_id: u128) -> Vec<DataPoint> {
    check_cycles_before_action();
    USERS.with(|users| {
        let users = users.borrow();
        for user in users.values() {
            if let Some(model) = user.models.get(&model_id) {
                return model.data_points.clone();
            }
        }
        ic_cdk::api::trap("Model not found");
    })
}

#[ic_cdk::query]
fn get_model_metrics(model_id: u128) -> Metrics {
    check_cycles_before_action();
    USERS.with(|users| {
        let users = users.borrow();
        for user in users.values() {
            if let Some(model) = user.models.get(&model_id) {
                return model.metrics.clone();
            }
        }
        ic_cdk::api::trap("Model not found");
    })
}

#[ic_cdk::query]
fn get_model(model_id: u128) -> Model {
    USERS.with(|users| {
        let users = users.borrow();
        for user in users.values() {
            if let Some(model) = user.models.get(&model_id) {
                return model.clone();
            }
        }
        ic_cdk::api::trap("Model not found");
    })
}

// ---------- HELPER FUNCTIONS ---------- //

fn calculate_group_counts(data_points: &Vec<DataPoint>) -> (i128, i128, i128, i128) {
    let mut privileged_count = 0_i128;
    let mut unprivileged_count = 0_i128;
    let mut privileged_pos = 0_i128;
    let mut unprivileged_pos = 0_i128;

    for point in data_points {
        if point.privileged {
            privileged_count += 1;
            if point.predicted {
                privileged_pos += 1;
            }
        } else {
            unprivileged_count += 1;
            if point.predicted {
                unprivileged_pos += 1;
            }
        }
    }

    (privileged_count, unprivileged_count, privileged_pos, unprivileged_pos)
}

fn calculate_confusion_matrix(
    data_points: &Vec<DataPoint>,
) -> (i128, i128, i128, i128, i128, i128, i128, i128) {
    let (mut p_tp, mut p_fp, mut p_tn, mut p_fn) = (0, 0, 0, 0);
    let (mut u_tp, mut u_fp, mut u_tn, mut u_fn) = (0, 0, 0, 0);

    for point in data_points {
        match (point.privileged, point.target, point.predicted) {
            (true, true, true) => p_tp += 1,
            (true, true, false) => p_fn += 1,
            (true, false, true) => p_fp += 1,
            (true, false, false) => p_tn += 1,
            (false, true, true) => u_tp += 1,
            (false, true, false) => u_fn += 1,
            (false, false, true) => u_fp += 1,
            (false, false, false) => u_tn += 1,
        }
    }

    (p_tp, p_fp, p_tn, p_fn, u_tp, u_fp, u_tn, u_fn)
}

fn calculate_overall_confusion_matrix(data_points: &Vec<DataPoint>) -> (i128, i128, i128, i128) {
    let (mut tp, mut tn, mut fp, mut fn_) = (0, 0, 0, 0);

    for point in data_points {
        match (point.target, point.predicted) {
            (true, true) => tp += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
            (true, false) => fn_ += 1,
        }
    }

    (tp, tn, fp, fn_)
}

fn calculate_true_positive_false_negative(
    data_points: &Vec<DataPoint>,
) -> (i128, i128, i128, i128) {
    let (mut p_tp, mut p_fn) = (0, 0);
    let (mut u_tp, mut u_fn) = (0, 0);

    for point in data_points {
        match (point.privileged, point.target, point.predicted) {
            (true, true, true) => p_tp += 1,
            (true, true, false) => p_fn += 1,
            (false, true, true) => u_tp += 1,
            (false, true, false) => u_fn += 1,
            _ => {}
        }
    }

    (p_tp, p_fn, u_tp, u_fn)
}

// ---------- Hugging Face HTTPS Outcall ---------- //

// For Hugging Face requests, we use serde's Serialize/Deserialize
#[derive(Serialize, Deserialize)]
struct HuggingFaceRequest {
    inputs: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HuggingFaceResponse {
    generated_text: Option<String>,
}

// Example placeholders
const HUGGING_FACE_ENDPOINT: &str = "https://api-inference.huggingface.co/models/gpt2";
const HUGGING_FACE_BEARER_TOKEN: &str = "hf_YourHuggingFaceAPITokenHere";

#[update]
async fn call_hugging_face(input_text: String) -> Result<String, String> {
    let payload = HuggingFaceRequest {
        inputs: input_text.clone(),
    };

    let json_payload = serde_json::to_vec(&payload)
        .map_err(|e| format!("Failed to serialize payload: {}", e))?;

    let headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", HUGGING_FACE_BEARER_TOKEN),
        },
    ];

    let request = HttpRequest {
        url: HUGGING_FACE_ENDPOINT.to_string(),
        method: HttpMethod::POST,
        headers,
        body: json_payload,
        // We omit the transform, for a direct response
        transform: None,
    };

    let response: HttpResponse = http_request(request)
        .await
        .map_err(|(code, msg)| format!("HTTP request failed. Code: {}. Msg: {}", code, msg))?;

    if response.status_code != 200 {
        return Err(format!(
            "Hugging Face API returned status {}: {}",
            response.status_code,
            String::from_utf8_lossy(&response.body)
        ));
    }

    let parsed: serde_json::Value = serde_json::from_slice(&response.body)
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    let hugging_face_resp: HuggingFaceResponse = serde_json::from_value(parsed.clone())
        .map_err(|_| format!("Unexpected response format: {}", parsed))?;

    let text = hugging_face_resp
        .generated_text
        .unwrap_or_else(|| "No generated_text field found.".to_string());

    Ok(text)
}

#[query]
fn ping() -> String {
    "Canister is alive!".to_string()
}
