use serde::{Deserialize, Serialize};

pub struct TogetherAIProvider {}

use super::traits::InferenceProvider;
use super::lib::{
    HuggingFaceRequestParameters,
    HUGGING_FACE_INFERENCE_PROVIDER_URL,
    OpenAIRequest,
    OpenAIMessage,
};

// TogetherAI JSON response
#[derive(Serialize, Deserialize)]
pub struct TogetherAIResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub prompt: Option<Vec<String>>,
    pub choices: Vec<TogetherAIChoice>,
    pub usage: TogetherAIUsage,
}

#[derive(Serialize, Deserialize)]
pub struct TogetherAIChoice {
    pub finish_reason: String,
    pub seed: Option<u32>,
    pub logprobs: Option<serde_json::Value>,
    pub index: i32,
    pub message: TogetherAIMessage,
}

#[derive(Serialize, Deserialize)]
pub struct TogetherAIMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct TogetherAIUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

impl InferenceProvider for TogetherAIProvider {
    fn name(&self) -> &str {
        return "togetherai";
    }
    
    fn generate_payload(&self, llm_model: String, input_text: String, parameters: HuggingFaceRequestParameters) -> Result<Vec<u8>, String> {
        let payload = OpenAIRequest {
            model: llm_model.to_lowercase(),
            messages: vec![
                OpenAIMessage {
                    role: "user".to_string(),
                    content: input_text,
                }
            ],
            stream: false,
            max_tokens: Some(5000),
            seed: parameters.seed,
            do_sample: Some(false),
            temperature: Some(0.0),
        };

        return serde_json::to_vec(&payload).map_err(|e| format!("Failed to serialize payload: {}", e));
    }

    fn get_response_text(&self, response_body: &Vec<u8>) -> Result<String, String> {
        // Parse raw bytes into TogetherAI response format
        let json_val: serde_json::Value =
            serde_json::from_slice(&response_body).map_err(|e| e.to_string())?;
        
        let together_response: TogetherAIResponse =
            serde_json::from_value(json_val).map_err(|e| e.to_string())?;

        // Extract the text from the first choice
        let choice = together_response
            .choices.get(0);

        if let None = choice {
            return Err("choices field is empty".to_string());
        }

        Ok(choice
           .unwrap()
           .message
           .content.clone())
    }

    fn endpoint_url(&self, _llm_model: String) -> String {
        return format!("{}/{}", &HUGGING_FACE_INFERENCE_PROVIDER_URL.to_string(), "together/v1/chat/completions");
    }
}
