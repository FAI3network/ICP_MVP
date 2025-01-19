use crate::{check_cycles_before_action, only_admin, USERS, NEXT_MODEL_ID, Model, ModelDetails, User, Metrics, DataPoint};
use candid::Principal;
use std::cell::RefCell;
use std::collections::HashMap;

#[ic_cdk::update]
pub fn add_model(model_name: String, model_details: ModelDetails) -> u128 {
    only_admin();
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
                        accuracy: None,
                        recall: None,
                        precision: None,
                        timestamp: 0,
                    },
                    details: ModelDetails {
                        description: model_details.description,
                        framework: model_details.framework,
                        version: model_details.version,
                        objective: model_details.objective,
                        url: model_details.url,
                    },
                    metrics_history: Vec::new(),
                },
            );
            *next_model_id.borrow_mut() += 1;
            model_id
        })
    })
}

#[ic_cdk::update]
pub fn delete_model(model_id: u128) {
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

#[ic_cdk::query]
pub fn get_all_models() -> Vec<Model> {
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
pub fn get_model_data_points(model_id: u128) -> Vec<DataPoint> {
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
pub fn get_model_metrics(model_id: u128) -> Metrics {
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
pub fn get_model(model_id: u128) -> Model {
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