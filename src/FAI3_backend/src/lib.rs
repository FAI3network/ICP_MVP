pub(crate) mod cycles_management;
pub mod types;
mod admin_management;
mod data_management;
mod model;
mod metrics_calculation;
mod hugging_face;
pub mod context_association_test;
pub mod llm_fairness;
mod utils;
pub mod errors;
mod config_management;

use errors::GenericError;
use candid::Principal;

use ic_cdk_macros::*;
use std::cell::RefCell;
use ic_stable_structures::{StableBTreeMap, Cell, memory_manager::{MemoryManager, MemoryId, VirtualMemory}, DefaultMemoryImpl};

use cycles_management::check_cycles_before_action;
use types::{DataPoint, Metrics, Model, ModelDetails, AverageMetrics};
use admin_management::only_admin;
use utils::is_owner;

// thread_local! {
//     static ADMINS: RefCell<Vec<Principal>> = RefCell::new(Vec::new());
//     static MODELS: RefCell<HashMap<u128, Model>> = RefCell::new(HashMap::new());
//     static NEXT_MODEL_ID: RefCell<u128> = RefCell::new(1);
//     static NEXT_DATA_POINT_ID: RefCell<u128> = RefCell::new(1);
// }

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ADMINS: RefCell<StableBTreeMap<Principal, (), VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static MODELS: RefCell<StableBTreeMap<u128, Model, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    static NEXT_MODEL_ID: RefCell<Cell<u128, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            1
        ).unwrap()
    );

    static NEXT_DATA_POINT_ID: RefCell<Cell<u128, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
            1
        ).unwrap()
    );

    static NEXT_LLM_DATA_POINT_ID: RefCell<Cell<u128, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            1
        ).unwrap()
    );

    static NEXT_LLM_MODEL_EVALUATION_ID: RefCell<Cell<u128, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            1
        ).unwrap()
    );

    static CONFIGURATION: RefCell<StableBTreeMap<String, String, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
        )
    );
}

#[ic_cdk::init]
fn init() {
    let deployer = ic_cdk::caller();
    ADMINS.with(|admins| 
        admins.borrow_mut().insert(deployer, ())
    );
}

#[ic_cdk::query]
fn whoami() -> Principal {
    ic_cdk::api::caller()
}

/// Simple test query
#[query]
fn ping() -> String {
    "Canister is alive!".to_string()
}

pub(crate) fn get_model_from_memory(model_id: u128) -> Result<Model, GenericError> {
    let model = MODELS.with(|models| {
        let models = models.borrow_mut();
        models.get(&model_id)
    });

    match model {
        Some(model) => Ok(model),
        None => Err(GenericError::new(GenericError::NOT_FOUND, "Model not found"))
    }
}
