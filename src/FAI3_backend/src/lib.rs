use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult;
use std::cell::RefCell;
use std::collections::HashMap;

/* #[ic_cdk::update]
async fn add_example_data_points(model_id: u128) -> CallResult<()> {
    check_cycles_before_action();

    let mut random_bytes = Vec::new();
    let mut bit_index = 0;

    for _ in 0..100 {
        // Check if we need to fetch new random bytes before accessing the array
        if bit_index >= random_bytes.len() * 8 {
            let (new_bytes,): (Vec<u8>,) =
                ic_cdk::call(Principal::management_canister(), "raw_rand", ()).await?;
            random_bytes = new_bytes;
            bit_index = 0;
        }

        // Safely get the next random bit
        let privileged = get_random_bit(&random_bytes, &mut bit_index);
        let actual = get_random_bit(&random_bytes, &mut bit_index);
        let predicted = get_random_bit(&random_bytes, &mut bit_index);

        add_data_point(model_id, privileged, actual, predicted);
    }

    Ok(())
} */

fn get_random_bit(bytes: &[u8], index: &mut usize) -> bool {
    let byte_index = *index / 8;
    let bit_index = *index % 8;

    if byte_index >= bytes.len() {
        return false;
    }

    *index += 1;

    (bytes[byte_index] & (1 << bit_index)) != 0
}

// Cycles management

const CYCLE_THRESHOLD: u64 = 1_000_000_000;

#[ic_cdk::query]
fn check_cycles() -> u64 {
    ic_cdk::api::canister_balance() // Returns the current cycle balance
}

#[ic_cdk::update]
fn stop_if_low_cycles() {
    let cycles: u64 = ic_cdk::api::canister_balance();
    if cycles < CYCLE_THRESHOLD {
        ic_cdk::trap("Cycle balance too low, stopping execution to avoid canister deletion.");
    }
}

fn check_cycles_before_action() {
    stop_if_low_cycles();
}

// Structs

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct DataPoint {
    data_point_id: u128,
    target: bool,
    privileged: bool,
    predicted: bool,
    features: Vec<f64>,       // New Cols as features
    timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Metrics {
    statistical_parity_difference: Option<f32>,
    disparate_impact: Option<f32>,
    average_odds_difference: Option<f32>,
    equal_opportunity_difference: Option<f32>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Model {
    model_id: u128,
    model_name: String,
    user_id: Principal,
    data_points: Vec<DataPoint>,
    metrics: Metrics,
    details: ModelDetails,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ModelDetails {
    description: String,
    framework: String,
    version: String,
    objective: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct User {
    user_id: Principal,
    models: HashMap<u128, Model>,
}

thread_local! {
    static USERS: RefCell<HashMap<Principal, User>> = RefCell::new(HashMap::new());
    static NEXT_MODEL_ID: RefCell<u128> = RefCell::new(1);
    static NEXT_DATA_POINT_ID: RefCell<u128> = RefCell::new(1);
}

// Operations

#[ic_cdk::update]
fn add_dataset(
    model_id: u128,
    features: Vec<Vec<f64>>,
    gender: Vec<String>,
    labels: Vec<bool>,
    predictions: Vec<bool>,
) {
    check_cycles_before_action();

    // Verificar que la longitud de todas las columnas sea consistente
    let data_length = labels.len();
    if gender.len() != data_length || predictions.len() != data_length {
        ic_cdk::api::trap("Error: Las longitudes de gender, labels y predictions deben ser iguales.");
    }
    for feature_column in &features {
        if feature_column.len() != data_length {
            ic_cdk::api::trap("Error: Todas las columnas de features deben tener la misma longitud que labels.");
        }
    }

    let caller: Principal = ic_cdk::api::caller();
    let timestamp: u64 = ic_cdk::api::time();

    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users.get_mut(&caller).expect("User not found");

        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != caller {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        NEXT_DATA_POINT_ID.with(|next_data_point_id: &RefCell<u128>| {
            let mut next_data_point_id = next_data_point_id.borrow_mut();

            for i in 0..data_length {
                let privileged = gender[i].to_lowercase() == "male"; // Asumimos que "male" es privilegiado

                let mut feature_vector = Vec::new();
                for feature_column in &features {
                    feature_vector.push(feature_column[i]);
                }

                let data_point = DataPoint {
                    data_point_id: *next_data_point_id,
                    target: labels[i],
                    privileged,
                    predicted: predictions[i],
                    features: feature_vector.clone(), // Asegurarse de agregar las características
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

    let caller: Principal = ic_cdk::api::caller();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users.entry(caller).or_insert(User {
            user_id: caller,
            models: HashMap::new(),
        });

        NEXT_MODEL_ID.with(|next_model_id: &RefCell<u128>| {
            let model_id: u128 = *next_model_id.borrow();
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
                    },
                    details: ModelDetails {
                        description: model_details.description,
                        framework: model_details.framework,
                        version: model_details.version,
                        objective: model_details.objective,
                    }
                },
            );
            *next_model_id.borrow_mut() += 1;
            model_id
        })
    })
}

/* #[ic_cdk::update]
fn add_data_point(model_id: u128, target: bool, privileged: bool, predicted: bool) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();
    let timestamp: u64 = ic_cdk::api::time();

    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users.get_mut(&caller).expect("User not found");

        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");
        if model.user_id != caller {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        NEXT_DATA_POINT_ID.with(|next_data_point_id: &RefCell<u128>| {
            let data_point_id: u128 = *next_data_point_id.borrow();

            let data_point: DataPoint = DataPoint {
                data_point_id,
                target,
                privileged,
                predicted,
                timestamp,
            };

            model.data_points.push(data_point);
            *next_data_point_id.borrow_mut() += 1;
        });
    });
} */

#[ic_cdk::update]
fn add_data_point(model_id: u128, target: bool, privileged: bool, predicted: bool, features: Vec<f64>) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();
    let timestamp: u64 = ic_cdk::api::time();

    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users.get_mut(&caller).expect("User not found");

        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");
        if model.user_id != caller {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        NEXT_DATA_POINT_ID.with(|next_data_point_id: &RefCell<u128>| {
            let data_point_id: u128 = *next_data_point_id.borrow();

            let data_point: DataPoint = DataPoint {
                data_point_id,
                target,
                privileged,
                predicted,
                features, // Añadir las características del punto de datos
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
    let caller: Principal = ic_cdk::api::caller();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users.get_mut(&caller).expect("User not found");
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
    let caller: Principal = ic_cdk::api::caller();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
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

#[ic_cdk::update]
fn calculate_statistical_parity_difference(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (
            privileged_count,
            unprivileged_count,
            privileged_positive_count,
            unprivileged_positive_count,
        ) = calculate_group_counts(&model.data_points);

        assert!(
            privileged_count > 0 && unprivileged_count > 0,
            "No data for one of the groups"
        );

        let privileged_probability: f32 =
            privileged_positive_count as f32 / privileged_count as f32;
        let unprivileged_probability: f32 =
            unprivileged_positive_count as f32 / unprivileged_count as f32;

        let result: f32 = unprivileged_probability - privileged_probability;
        model.metrics.statistical_parity_difference = Some(result);
        result
    })
}

#[ic_cdk::update]
fn calculate_disparate_impact(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (
            privileged_count,
            unprivileged_count,
            privileged_positive_count,
            unprivileged_positive_count,
        ) = calculate_group_counts(&model.data_points);

        assert!(
            privileged_count > 0 && unprivileged_count > 0,
            "No data for one of the groups"
        );

        let privileged_probability: f32 =
            privileged_positive_count as f32 / privileged_count as f32;
        let unprivileged_probability: f32 =
            unprivileged_positive_count as f32 / unprivileged_count as f32;

        assert!(
            privileged_probability > 0.0,
            "Privileged group has no positive outcomes"
        );

        let result: f32 = unprivileged_probability / privileged_probability;
        model.metrics.disparate_impact = Some(result);
        result
    })
}

#[ic_cdk::update]
fn calculate_average_odds_difference(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (
            privileged_tp,
            privileged_fp,
            privileged_tn,
            privileged_fn,
            unprivileged_tp,
            unprivileged_fp,
            unprivileged_tn,
            unprivileged_fn,
        ) = calculate_confusion_matrix(&model.data_points);

        let privileged_tpr: f32 = privileged_tp as f32 / (privileged_tp + privileged_fn) as f32;
        let unprivileged_tpr: f32 =
            unprivileged_tp as f32 / (unprivileged_tp + unprivileged_fn) as f32;
        let privileged_fpr: f32 = privileged_fp as f32 / (privileged_fp + privileged_tn) as f32;
        let unprivileged_fpr: f32 =
            unprivileged_fp as f32 / (unprivileged_fp + unprivileged_tn) as f32;

        let result: f32 =
            ((unprivileged_fpr - privileged_fpr) + (unprivileged_tpr - privileged_tpr)) / 2.0;
        model.metrics.average_odds_difference = Some(result);
        result
    })
}

#[ic_cdk::update]
fn calculate_equal_opportunity_difference(model_id: u128) -> f32 {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let mut users: std::cell::RefMut<'_, HashMap<Principal, User>> = users.borrow_mut();
        let user: &mut User = users
            .get_mut(&ic_cdk::api::caller())
            .expect("User not found");
        let model: &mut Model = user
            .models
            .get_mut(&model_id)
            .expect("Model not found or not owned by user");

        if model.user_id != ic_cdk::api::caller() {
            ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
        }

        let (privileged_tp, privileged_fn, unprivileged_tp, unprivileged_fn) =
            calculate_true_positive_false_negative(&model.data_points);

        let privileged_tpr: f32 = privileged_tp as f32 / (privileged_tp + privileged_fn) as f32;
        let unprivileged_tpr: f32 =
            unprivileged_tp as f32 / (unprivileged_tp + unprivileged_fn) as f32;

        let result: f32 = unprivileged_tpr - privileged_tpr;
        model.metrics.equal_opportunity_difference = Some(result);
        result
    })
}

#[ic_cdk::update]
fn calculate_all_metrics(model_id: u128) -> (f32, f32, f32, f32) {
    (
        calculate_statistical_parity_difference(model_id),
        calculate_disparate_impact(model_id),
        calculate_average_odds_difference(model_id),
        calculate_equal_opportunity_difference(model_id),
    )
}


#[ic_cdk::update]
fn test_function() -> bool {
    true
}

// Getters
#[ic_cdk::query]
fn get_all_models() -> Vec<Model> {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let users: std::cell::Ref<'_, HashMap<Principal, User>> = users.borrow();
        users
            .values()
            .flat_map(|user| user.models.values().cloned())
            .collect()
    })
}

#[ic_cdk::query]
fn get_model_data_points(model_id: u128) -> Vec<DataPoint> {
    check_cycles_before_action();
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let users: std::cell::Ref<'_, HashMap<Principal, User>> = users.borrow();

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
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let users: std::cell::Ref<'_, HashMap<Principal, User>> = users.borrow();

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
    USERS.with(|users: &RefCell<HashMap<Principal, User>>| {
        let users: std::cell::Ref<'_, HashMap<Principal, User>> = users.borrow();

        for user in users.values() {
            if let Some(model) = user.models.get(&model_id) {
                return model.clone();
            }
        }

        ic_cdk::api::trap("Model not found");
    })
}

// Helper functions

fn calculate_group_counts(data_points: &Vec<DataPoint>) -> (i128, i128, i128, i128) {
    let mut privileged_count: i128 = 0;
    let mut unprivileged_count: i128 = 0;
    let mut privileged_positive_count: i128 = 0;
    let mut unprivileged_positive_count: i128 = 0;

    for point in data_points {
        if point.privileged {
            privileged_count += 1;
            if point.predicted {
                privileged_positive_count += 1;
            }
        } else {
            unprivileged_count += 1;
            if point.predicted {
                unprivileged_positive_count += 1;
            }
        }
    }

    (
        privileged_count,
        unprivileged_count,
        privileged_positive_count,
        unprivileged_positive_count,
    )
}

fn calculate_confusion_matrix(
    data_points: &Vec<DataPoint>,
) -> (i128, i128, i128, i128, i128, i128, i128, i128) {
    let (mut privileged_tp, mut privileged_fp, mut privileged_tn, mut privileged_fn) = (0, 0, 0, 0);
    let (mut unprivileged_tp, mut unprivileged_fp, mut unprivileged_tn, mut unprivileged_fn) =
        (0, 0, 0, 0);

    for point in data_points {
        match (point.privileged, point.target, point.predicted) {
            (true, true, true) => privileged_tp += 1,
            (true, true, false) => privileged_fn += 1,
            (true, false, true) => privileged_fp += 1,
            (true, false, false) => privileged_tn += 1,
            (false, true, true) => unprivileged_tp += 1,
            (false, true, false) => unprivileged_fn += 1,
            (false, false, true) => unprivileged_fp += 1,
            (false, false, false) => unprivileged_tn += 1,
        }
    }

    (
        privileged_tp,
        privileged_fp,
        privileged_tn,
        privileged_fn,
        unprivileged_tp,
        unprivileged_fp,
        unprivileged_tn,
        unprivileged_fn,
    )
}

fn calculate_true_positive_false_negative(
    data_points: &Vec<DataPoint>,
) -> (i128, i128, i128, i128) {
    let (mut privileged_tp, mut privileged_fn) = (0, 0);
    let (mut unprivileged_tp, mut unprivileged_fn) = (0, 0);

    for point in data_points {
        match (point.privileged, point.target, point.predicted) {
            (true, true, true) => privileged_tp += 1,
            (true, true, false) => privileged_fn += 1,
            (false, true, true) => unprivileged_tp += 1,
            (false, true, false) => unprivileged_fn += 1,
            _ => {}
        }
    }

    (
        privileged_tp,
        privileged_fn,
        unprivileged_tp,
        unprivileged_fn,
    )
}