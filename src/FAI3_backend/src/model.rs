use crate::types::get_classifier_model_data;
use crate::types::{ClassifierModelData, LLMModelData, ModelDetailsHistory, ModelType};
use crate::{
    check_cycles_before_action, is_owner, only_admin, AverageMetrics, DataPoint, Metrics, Model,
    ModelDetails, MODELS, NEXT_MODEL_ID,
};
use candid::Principal;
use std::vec;

#[ic_cdk::update]
pub fn add_classifier_model(model_name: String, model_details: ModelDetails) -> u128 {
    only_admin();
    check_cycles_before_action();

    if model_name.trim().is_empty() {
        ic_cdk::api::trap("Error: Model name cannot be empty or null.");
    }

    let caller: Principal = ic_cdk::api::caller();

    let id = MODELS.with(|models| {
        return NEXT_MODEL_ID.with(|id| {
            let current_id = *id.borrow().get();

            models.borrow_mut().insert(
                current_id,
                Model {
                    model_id: current_id,
                    model_name: model_name.clone(),
                    owners: vec![caller],
                    details: model_details.clone(),
                    details_history: vec![ModelDetailsHistory {
                        name: model_name,
                        details: model_details,
                        version: 0,
                        timestamp: ic_cdk::api::time(),
                    }],
                    model_type: ModelType::Classifier(ClassifierModelData {
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
                        metrics_history: Vec::new(),
                    }),
                    cached_thresholds: None,
                    cached_selections: None,
                    version: 0,
                },
            );

            id.borrow_mut().set(current_id + 1).unwrap();

            current_id
        });
    });
    id
}

#[ic_cdk::update]
pub fn add_llm_model(
    model_name: String,
    hugging_face_url: String,
    model_details: ModelDetails,
    inference_provider: Option<String>,
) -> u128 {
    only_admin();
    check_cycles_before_action();

    if model_name.trim().is_empty() {
        ic_cdk::api::trap("Error: Model name cannot be empty or null.");
    }

    let caller: Principal = ic_cdk::api::caller();

    let id = MODELS.with(|models| {
        return NEXT_MODEL_ID.with(|id| {
            let current_id = *id.borrow().get();

            models.borrow_mut().insert(
                current_id,
                Model {
                    model_id: current_id,
                    model_name: model_name.clone(),
                    owners: vec![caller],
                    details: model_details.clone(),
                    details_history: vec![ModelDetailsHistory {
                        name: model_name,
                        details: model_details,
                        version: 0,
                        timestamp: ic_cdk::api::time(),
                    }],
                    model_type: ModelType::LLM(LLMModelData {
                        cat_metrics: None,
                        cat_metrics_history: Vec::new(),
                        hugging_face_url,
                        evaluations: Vec::new(),
                        average_fairness_metrics: None,
                        language_evaluations: Vec::new(),
                        inference_provider,
                    }),
                    cached_thresholds: None,
                    cached_selections: None,
                    version: 0,
                },
            );

            id.borrow_mut().set(current_id + 1).unwrap();

            current_id
        });
    });
    id
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
pub fn get_all_models(limit: usize, _offset: usize, model_type: Option<String>) -> Vec<Model> {
    check_cycles_before_action();

    return MODELS.with(|models| {
        let models = models.borrow();
        return models
            .values()
            .filter(|model| {
                ic_cdk::println!("Filtering");
                match &model_type {
                    Some(ref mt) if mt == "llm" => matches!(model.model_type, ModelType::LLM(_)),
                    Some(ref mt) if mt == "classifier" => {
                        matches!(model.model_type, ModelType::Classifier(_))
                    }
                    // if model type is not "llm" or "classifier", it matches everything
                    _ => true,
                }
            })
            .take(limit)
            .map(|model| model.prune())
            .collect();
    });
}

#[ic_cdk::query]
pub fn get_model_data_points(model_id: u128) -> Vec<DataPoint> {
    check_cycles_before_action();

    MODELS.with(|models| {
        let models = models.borrow();
        let model = models.get(&model_id).expect("Model not found");
        return get_classifier_model_data(&model).data_points;
    })
}

#[ic_cdk::query]
pub fn get_model_metrics(model_id: u128) -> Metrics {
    check_cycles_before_action();

    MODELS.with(|models| {
        let model = models.borrow().get(&model_id).expect("Model not found");
        return get_classifier_model_data(&model).metrics;
    })
}

/// Returns a model
/// For limitations and data size, it won't return LLM data_points
/// And it won't return LLM metrics history
#[ic_cdk::query]
pub fn get_model(model_id: u128) -> Model {
    let model = MODELS.with(|models| {
        models
            .borrow()
            .get(&model_id)
            .expect("Model not found")
            .clone()
    });

    return model.prune();
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
pub fn update_model(
    model_id: u128,
    model_name: String,
    model_details: ModelDetails,
    edit: bool,
) -> bool {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();
    let mut status = false;

    MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("Model not found");
        is_owner(&model, caller);
        model.model_name = model_name.clone();
        model.details = model_details.clone();

        let timestamp: u64 = ic_cdk::api::time();

        let latest_details = if edit {
            model.details_history.pop().unwrap()
        } else {
            model.details_history.last().unwrap().clone()
        };

        let latest_version = if edit {
            latest_details.version
        } else {
            model.version = latest_details.version + 1;
            model.version
        };

        model.details_history.push(ModelDetailsHistory {
            name: model_name,
            details: model_details,
            version: latest_version,
            timestamp,
        });

        models.insert(model_id, model);

        status = true;
    });

    status
}

#[ic_cdk::query]
pub fn get_llm_model_data_id(model_id: u128) -> LLMModelData {
    MODELS.with(|models| {
        let models = models.borrow();
        let model = models.get(&model_id).expect("Model not found");
        match model.model_type {
            ModelType::LLM(ref model_data) => model_data.clone(),
            _ => panic!("A classifier model was expected, got another type of model instead"),
        }
    })
}

#[ic_cdk::query]
pub fn get_details_history(model_id: u128) -> Vec<ModelDetailsHistory> {
    check_cycles_before_action();

    MODELS.with(|models| {
        let models = models.borrow();
        let model = models.get(&model_id).expect("Model not found");
        model.details_history.clone()
    })
}
