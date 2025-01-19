use crate::{ ADMINS, check_cycles_before_action };
use candid::Principal;

#[ic_cdk::query]
fn is_admin() -> bool {
    ADMINS.with(|admins| {
        let admins = admins.borrow();
        admins.contains(&ic_cdk::api::caller())
    })
}

pub(crate) fn only_admin() {
    if !is_admin() {
        ic_cdk::api::trap("Unauthorized: You are not an admin");
    }
}

#[ic_cdk::update]
fn add_admin(admin: String) {
    only_admin();
    check_cycles_before_action();
    ADMINS.with(|admins| {
        admins
            .borrow_mut()
            .push(Principal::from_text(admin).unwrap());
    });
}

#[ic_cdk::query]
fn get_admins() -> Vec<Principal> {
    ADMINS.with(|admins| admins.borrow().clone())
}
