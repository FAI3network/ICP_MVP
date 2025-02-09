// mod tests;
pub(crate) mod cycles_management;
mod types;
mod admin_management;
mod data_management;
mod model;
mod metrics_calculation;
mod hugging_face;

use candid::Principal;

use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;

use cycles_management::check_cycles_before_action;
use types::{DataPoint, Metrics, Model, ModelDetails, User, AverageMetrics};
use admin_management::only_admin;
use metrics_calculation::{calculate_confusion_matrix, calculate_group_counts, calculate_overall_confusion_matrix, calculate_true_positive_false_negative};

thread_local! {
    static ADMINS: RefCell<Vec<Principal>> = RefCell::new(Vec::new());
    static USERS: RefCell<HashMap<Principal, User>> = RefCell::new(HashMap::new());
    static NEXT_MODEL_ID: RefCell<u128> = RefCell::new(1);
    static NEXT_DATA_POINT_ID: RefCell<u128> = RefCell::new(1);
}

#[ic_cdk::init]
fn init() {
    let deployer = ic_cdk::caller();
    ADMINS.with(|admins| {
        admins.borrow_mut().push(deployer);
    });
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
