use serde::{Deserialize, Serialize};

use crate::types::HuggingFaceResponseItem;

use super::traits::InferenceProvider;
use super::lib::{
    HuggingFaceRequestParameters,
    HuggingFaceRequest,
    HUGGING_FACE_ENDPOINT,
};

pub struct NoneProvider {}

#[derive(Serialize, Deserialize, Debug)]
pub struct HuggingFaceResponse {
    generated_text: Option<String>,
}

impl InferenceProvider for NoneProvider {
    fn name(&self) -> &str {
        return "none (Hugging Face API)";
    }
    
    fn generate_payload(&self, _llm_model: String, input_text: String, parameters: HuggingFaceRequestParameters) -> Result<Vec<u8>, String> {
        let payload = HuggingFaceRequest {
            inputs: input_text.clone(),
            parameters: Some(parameters), 
        };

        let json_payload =
            serde_json::to_vec(&payload).map_err(|e| format!("Failed to serialize payload: {}", e))?;

        return Ok(json_payload);
    }

    fn get_response_text(&self, response_body: &Vec<u8>) -> Result<String, String> {
        // 1) Parse raw bytes into a `serde_json::Value`
        let json_val: serde_json::Value =
            serde_json::from_slice(&response_body).map_err(|e| e.to_string())?;
        
        // 2) Now parse that `json_val` into a vector of your items
        let hf_response: Vec<HuggingFaceResponseItem> =
            serde_json::from_value(json_val).map_err(|e| e.to_string())?;

        // 3) Extract the text from the first item, or default
        let items = hf_response.get(0);

        if let None = items {
            return Err("No generated text".to_string());
        }

        return Ok(items
                  .and_then(|item| item.generated_text.clone())
                  .unwrap_or_else(|| "No generated_text".to_string()));
    }

    fn endpoint_url(&self, llm_model: String) -> String {
        return format!("{}/{}", HUGGING_FACE_ENDPOINT, llm_model);
    }
}
