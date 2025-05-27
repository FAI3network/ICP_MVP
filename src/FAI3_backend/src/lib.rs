mod admin_management;
mod config_management;
pub mod context_association_test;
pub(crate) mod cycles_management;
mod data_management;
pub mod errors;
mod hugging_face;
pub mod inference_providers;
mod job_management;
pub mod llm_fairness;
pub mod llm_language_evaluations;
mod metrics_calculation;
mod model;
pub mod types;
mod utils;

use candid::Principal;
use errors::GenericError;

use ic_cdk_macros::*;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    Cell, DefaultMemoryImpl, StableBTreeMap,
};
use std::cell::RefCell;

use admin_management::only_admin;
use cycles_management::check_cycles_before_action;
use types::{AverageMetrics, DataPoint, Job, Metrics, Model, ModelDetails};
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

    static NEXT_LLM_LANGUAGE_EVALUATION_ID: RefCell<Cell<u128, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
            1
        ).unwrap()
    );

    static JOBS: RefCell<StableBTreeMap<u128, Job, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8)))
        )
    );

    static NEXT_JOB_ID: RefCell<Cell<u128, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))),
            1
        ).unwrap()
    );

    static NEXT_CONTEXT_ASSOCIATION_TEST_ID: RefCell<Cell<u128, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        Cell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
            1
        ).unwrap()
    );
}

#[ic_cdk::init]
fn init() {
    let deployer = ic_cdk::caller();
    ADMINS.with(|admins| admins.borrow_mut().insert(deployer, ()));
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
        None => Err(GenericError::new(
            GenericError::NOT_FOUND,
            "Model not found",
        )),
    }
}
