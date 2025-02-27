use crate::{
    check_cycles_before_action, only_admin, LLMModel,
    ModelDetails, LLM_MODELS, NEXT_MODEL_ID
};
use candid::Principal;
use std::vec;
use crate::types::{ContextAssociationTestDataPoint, ContextAssociationTestMetricsBag};
use crate::utils::is_llm_owner;

#[ic_cdk::update]
pub fn add_llm_model(model_name: String, hf_url: String, model_details: ModelDetails) -> u128 {
    only_admin();
    check_cycles_before_action();

    if model_name.trim().is_empty() {
        ic_cdk::api::trap("Error: LLMModel name cannot be empty or null.");
    }

    let caller: Principal = ic_cdk::api::caller();

    LLM_MODELS.with(|models| {
        NEXT_MODEL_ID.with(|id| {
            let current_id = *id.borrow().get();

            models.borrow_mut().insert(
                current_id,
                LLMModel {
                    model_id: current_id,
                    model_name,
                    hf_url,
                    owners: vec![caller],
                    cat_metrics: None,
                    details: model_details,
                    cat_metrics_history: Vec::new(),
                },
            );

            id.borrow_mut().set(current_id + 1).unwrap();

            current_id
        })
    })
}

#[ic_cdk::update]
pub fn delete_llm_model(model_id: u128) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();

    LLM_MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let model = models.get(&model_id).expect("LLMModel not found");
        is_llm_owner(&model, caller);
        models.remove(&model_id);
    });
}

#[ic_cdk::query]
pub fn get_all_llm_models() -> Vec<LLMModel> {
    check_cycles_before_action();

    LLM_MODELS.with(|models| {
        let models = models.borrow();
        models.values().map(
            |model| {
                let mut m2 = model.clone();
                // keys below are excluded to avoid sending unnecessary information
                m2.cat_metrics = None;
                m2.cat_metrics_history = Vec::new();
                m2
            }
        ).collect()
    })
}

#[ic_cdk::query]
pub fn get_context_association_test_metrics_data_points(model_id: u128) -> Option<Vec<ContextAssociationTestDataPoint>> {
    check_cycles_before_action();

    LLM_MODELS.with(|models| {
        let models = models.borrow();
        let model = models.get(&model_id).expect("LLMModel not found");
        match model.cat_metrics {
            Some(metrics) => Some(metrics.data_points.clone()),
            None => None,
        }
    })
}

#[ic_cdk::query]
pub fn get_context_association_test_metrics(model_id: u128) -> Option<ContextAssociationTestMetricsBag> {
    check_cycles_before_action();

    LLM_MODELS.with(|models| {
        models
            .borrow()
            .get(&model_id)
            .expect("LLMModel not found")
            .cat_metrics
            .clone()
    })
}

#[ic_cdk::query]
pub fn get_context_association_test_metrics_history(model_id: u128) -> Vec<ContextAssociationTestMetricsBag> {
    check_cycles_before_action();

    LLM_MODELS.with(|models| {
        models
            .borrow()
            .get(&model_id)
            .expect("LLMModel not found")
            .cat_metrics_history
            .clone()
    })
}

#[ic_cdk::query]
pub fn get_llm_model(model_id: u128) -> LLMModel {
    LLM_MODELS.with(|models| {
        models
            .borrow()
            .get(&model_id)
            .expect("LLMModel not found")
            .clone()
    })
}

#[ic_cdk::update]
pub fn add_llm_owner(model_id: u128, new_owner: Principal) {
    check_cycles_before_action();
    let caller: Principal = ic_cdk::api::caller();

    LLM_MODELS.with(|models| {
        let mut models = models.borrow_mut();
        let mut model = models.get(&model_id).expect("LLMModel not found");
        is_llm_owner(&model, caller);
        model.owners.push(new_owner);
        models.insert(model_id, model);
    });
}

#[ic_cdk::query]
pub fn get_llm_owners(model_id: u128) -> Vec<Principal> {
    check_cycles_before_action();
    
    LLM_MODELS.with(|models| {
        let models = models.borrow();
        let model = models.get(&model_id).expect("LLMModel not found");
        model.owners.clone()
    })
}