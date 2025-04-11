use crate::{
    check_cycles_before_action, CONFIGURATION,
};
use crate::errors::GenericError;
use crate::admin_management::only_admin;

pub const HUGGING_FACE_API_KEY_CONFIG_KEY: &str = "hugging_face_api_key";

#[ic_cdk::update]
pub fn set_config(config_key: String, config_value: String) {
    check_cycles_before_action();
    only_admin();

    CONFIGURATION.with(|config| {
        let mut config_tree = config.borrow_mut();
        config_tree.insert(config_key, config_value);
    });
}

#[ic_cdk::query]
pub fn get_config(config_key: String) -> Result<String, GenericError> {
    check_cycles_before_action();
    only_admin();

    let result = CONFIGURATION.with(|config| {
        let config_tree = config.borrow();
        config_tree.get(&config_key)
    });

    match result {
        Some(value) => return Ok(value),
        None => return Err(GenericError::new(GenericError::CONFIGURATION_KEY_NOT_FOUND, "Key not set")),
    }
}
