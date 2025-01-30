use crate::Model;
use candid::Principal;

pub fn is_owner(model: &Model, caller: Principal) {
    if model.owners.iter().all(|id| *id != caller) {
        ic_cdk::api::trap("Unauthorized: You are not the owner of this model");
    }
}
