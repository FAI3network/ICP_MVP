use ic_cdk_macros::*;
use ic_cdk::api::call::{msg_cycles_accept, msg_cycles_available};


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

pub(crate) fn check_cycles_before_action() {
    stop_if_low_cycles();
}

/// Accepts whatever cycles were sent with this call and returns how many were accepted.
#[update]
fn add_funds() -> u64 {
    // How many cycles the caller attached to *this* call
    let available = msg_cycles_available();
    if available > 0 {
        // Accept them all into our canister's balance
        let accepted = msg_cycles_accept(available);
        ic_cdk::println!("Accepted {} cycles into the canister", accepted);
        accepted
    } else {
        0
    }
}