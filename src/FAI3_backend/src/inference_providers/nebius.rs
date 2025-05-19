pub struct NebiusProvider {}

use super::traits::InferenceProvider;
use super::lib::{
    HuggingFaceRequestParameters,
    HUGGING_FACE_INFERENCE_PROVIDER_URL,
    OpenAIRequest,
    OpenAIMessage,
};

use super::together::TogetherAIResponse;

impl InferenceProvider for NebiusProvider {
    fn name(&self) -> &str {
        return "nebius";
    }
    
    fn generate_payload(&self, llm_model: String, input_text: String, parameters: HuggingFaceRequestParameters) -> Result<Vec<u8>, String> {
        let payload = OpenAIRequest {
            model: llm_model,
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
        // Since Nebius uses same format as TogetherAI, we can reuse the same response structure
        let json_val: serde_json::Value =
            serde_json::from_slice(&response_body).map_err(|e| e.to_string())?;
        let nebius_response: TogetherAIResponse = // Reusing TogetherAI response structure
            serde_json::from_value(json_val).map_err(|e| e.to_string())?;
        let choice = nebius_response
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
        return format!("{}/{}", &HUGGING_FACE_INFERENCE_PROVIDER_URL.to_string(), "nebius/v1/chat/completions");
    }
}
