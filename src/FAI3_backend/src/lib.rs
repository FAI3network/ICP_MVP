// mod tests;
pub(crate) mod cycles_management;
mod types;
mod admin_management;
mod data_management;
mod model;
mod metrics_calculation;
mod hugging_face;
mod utils;

use candid::Principal;

use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;

use cycles_management::check_cycles_before_action;
use model::get_model;
use types::{DataPoint, Metrics, Model, ModelDetails, User, AverageMetrics};
use admin_management::only_admin;
use utils::is_owner;
use metrics_calculation::{calculate_confusion_matrix, calculate_group_counts, calculate_overall_confusion_matrix, calculate_true_positive_false_negative};

thread_local! {
    static ADMINS: RefCell<Vec<Principal>> = RefCell::new(vec![Principal::from_text("f5hu5-c5eqs-4m2bm-fxb27-5mnk2-lpbva-l3tb5-7xv5p-w65wt-a3uyd-lqe").unwrap(), Principal::from_text("o7e2a-bbdxy-oigax-2ci44-naapw-oyqcx-uwf67-4b6ch-zrfis-bfa5n-wae").unwrap()]);
    // static USERS: RefCell<HashMap<Principal, User>> = RefCell::new(HashMap::new());
    static MODELS: RefCell<HashMap<u128, Model>> = RefCell::new(HashMap::new());
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
