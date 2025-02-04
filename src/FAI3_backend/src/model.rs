use crate::{check_cycles_before_action, only_admin, MODELS, NEXT_MODEL_ID, Model, ModelDetails, User, Metrics, DataPoint, AverageMetrics, is_owner};
use candid::Principal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::vec;

#[ic_cdk::update]
pub fn add_model(model_name: String, model_details: ModelDetails) -> u128 {
    only_admin();
    check_cycles_before_action();

    if model_name.trim().is_empty() {
        ic_cdk::api::trap("Error: Model name cannot be empty or null.");
    }

    let caller: Principal = ic_cdk::api::caller();

    MODELS.with(|models| {
        // let mut models: std::cell::RefMut<'_, HashMap<u128, Model>> = models.borrow_mut();
        // let model_id: u128 = NEXT_MODEL_ID.with(|next_model_id: &RefCell<u128>| {
        //     let model_id: u128 = *next_model_id.borrow();
        //     *next_model_id.borrow_mut() += 1;
        //     model_id
        // });~

        NEXT_MODEL_ID.with(|id| id.borrow_mut().set(id.borrow().get() + 1).unwrap());

        let model_id = NEXT_MODEL_ID.with(|id| *id.borrow().get());

        models.borrow_mut().insert(
            model_id,
            Model {
                model_id: model_id,
                model_name,
                owners: vec![caller],
                data_points: Vec::new(),
                metrics: Metrics {
                    statistical_parity_difference: None,
                    disparate_impact: None,
                    average_odds_difference: None,
                    equal_opportunity_difference: None,
                    average_metrics: AverageMetrics {
                        statistical_parity_difference: None,
                        disparate_impact: None,
                        average_odds_difference: None,
                        equal_opportunity_difference: None,
                    },
                    accuracy: None,
                    recall: None,
                    precision: None,
                    timestamp: 0,
                },
                details: model_details,
                metrics_history: Vec::new(),
            },
        );

        model_id
    })
}

#[ic_cdk::update]
pub fn delete_model(model_id: u128) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let model = models.get(&model_id).expect("Model not found");
        is_owner(&model, caller);
        models.remove(&model_id);
    });
}

#[ic_cdk::query]
pub fn get_all_models() -> Vec<Model> {
    check_cycles_before_action();

    MODELS.with(|models| {
        let models = models.borrow();
        models.values().map(|model| model.clone()).collect()
    })
}

#[ic_cdk::query]
pub fn get_model_data_points(model_id: u128) -> Vec<DataPoint> {
    check_cycles_before_action();

    MODELS.with(|models| {
        let models = models.borrow();
        let model = models.get(&model_id).expect("Model not found");
        model.data_points.clone()
    })
}

#[ic_cdk::query]
pub fn get_model_metrics(model_id: u128) -> Metrics {
    check_cycles_before_action();

    MODELS.with(|models| {
        models.borrow().get(&model_id).expect("Model not found").metrics.clone()
    })
}

#[ic_cdk::query]
pub fn get_model(model_id: u128) -> Model {
    MODELS.with(|models| {
        models.borrow().get(&model_id).expect("Model not found").clone()
    })
}

#[ic_cdk::update]
pub fn add_owner(model_id: u128, new_owner: Principal) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();

    MODELS.with(|models| {
        let models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");
        is_owner(&model, caller);
        model.owners.push(new_owner);
    });
}