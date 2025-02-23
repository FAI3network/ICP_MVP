use crate::{
    check_cycles_before_action, is_owner, only_admin, AverageMetrics, DataPoint, Metrics, Model,
    ModelDetails, MODELS, NEXT_MODEL_ID,
};
use candid::Principal;
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
        NEXT_MODEL_ID.with(|id| {
            let current_id = *id.borrow().get();

            models.borrow_mut().insert(
                current_id,
                Model {
                    model_id: current_id,
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

            id.borrow_mut().set(current_id + 1).unwrap();

            current_id
        })
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
        models
            .borrow()
            .get(&model_id)
            .expect("Model not found")
            .metrics
            .clone()
    })
}

#[ic_cdk::query]
pub fn get_model(model_id: u128) -> Model {
    MODELS.with(|models| {
        models
            .borrow()
            .get(&model_id)
            .expect("Model not found")
            .clone()
    })
}

#[ic_cdk::update]
pub fn add_owner(model_id: u128, new_owner: Principal) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");
        is_owner(&model, caller);
        model.owners.push(new_owner);
        models.insert(model_id, model);
    });
}

#[ic_cdk::query]
pub fn get_owners(model_id: u128) -> Vec<Principal> {
    check_cycles_before_action();
    
    MODELS.with(|models| {
        let models = models.borrow();
        let model = models.get(&model_id).expect("Model not found");
        model.owners.clone()
    })
}

#[ic_cdk::update]
pub fn update_model(model_id: u128, model_name: String, model_details: ModelDetails) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");
        is_owner(&model, caller);
        model.model_name = model_name;
        model.details = model_details;
        models.insert(model_id, model);
    });
}
