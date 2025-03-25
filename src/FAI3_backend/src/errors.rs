use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Debug, CandidType, Serialize, Deserialize)]
pub struct GenericError {
    pub category: u16,
    pub code: u16,
    pub message: String,
    pub details: Vec<(String, String)>, // Additional context as key-value pairs
}

#[allow(dead_code)]
impl GenericError {
    // Error categories
    // Generic error: 000
    // Input error: 100
    // Authorization error: 200
    // Resource error: 300
    // External resource error: 400
    // Internal error: 500

    // Specific errors within categories
    pub const EMPTY_INPUT: u16 = 101;
    pub const INVALID_FORMAT: u16 = 102;

    pub const NOT_FOUND: u16 = 301;
    pub const ALREADY_EXISTS: u16 = 302;
    pub const INVALID_RESOURCE_FORMAT: u16 = 103;

    // External resource errors
    pub const EXTERNAL_RESOURCE_GENERIC_ERROR: u16 = 400;
    pub const HUGGING_FACE_ERROR_RATE_REACHED: u16 = 401;

    pub const GENERIC_SYSTEM_FAILURE: u16 = 500;
    
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        let category: u16 = (code / 100) * 100;
        Self {
            category,
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    // Adds extra details to the error
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.push((key.into(), value.into()));
        self
    }
}
