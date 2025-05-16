pub struct NovitaProvider {}

use serde::{Deserialize, Serialize};

use super::traits::InferenceProvider;
use super::lib::{
    HuggingFaceRequestParameters,
    HUGGING_FACE_INFERENCE_PROVIDER_URL,
    OpenAIRequest,
    OpenAIMessage,
};

// Novita JSON response
#[derive(Serialize, Deserialize)]
struct NovitaResponse {
    choices: Vec<NovitaChoice>,
    created: i64,
    id: String,
    model: String,
    object: String,
    system_fingerprint: String,
    usage: NovitaUsage,
}

#[derive(Serialize, Deserialize)]
struct NovitaChoice {
    content_filter_results: NovitaContentFilterResults,
    finish_reason: String,
    index: i32,
    message: NovitaMessage,
}

#[derive(Serialize, Deserialize)]
struct NovitaContentFilterResults {
    hate: NovitaFilterResult,
    jailbreak: NovitaJailbreakResult,
    profanity: NovitaFilterResult,
    self_harm: NovitaFilterResult,
    sexual: NovitaFilterResult,
    violence: NovitaFilterResult,
}

#[derive(Serialize, Deserialize)]
struct NovitaFilterResult {
    filtered: bool,
    #[serde(default)]
    detected: bool,
}

#[derive(Serialize, Deserialize)]
struct NovitaJailbreakResult {
    detected: bool,
    filtered: bool,
}

#[derive(Serialize, Deserialize)]
struct NovitaMessage {
    content: String,
    role: String,
}

#[derive(Serialize, Deserialize)]
struct NovitaUsage {
    completion_tokens: i32,
    completion_tokens_details: Option<serde_json::Value>,
    prompt_tokens: i32,
    prompt_tokens_details: Option<serde_json::Value>,
    total_tokens: i32,
}

impl InferenceProvider for NovitaProvider {
    fn name(&self) -> &str {
        return "novita";
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
        // 1) Parse raw bytes into a `serde_json::Value`
        let json_val: serde_json::Value =
            serde_json::from_slice(&response_body).map_err(|e| e.to_string())?;
        
        // 2) Now parse that `json_val` into a vector of your items
        let hf_response: NovitaResponse =
            serde_json::from_value(json_val).map_err(|e| e.to_string())?;

        // 3) Extract the text from the first item, or default
        let choice = hf_response
            .choices.get(0);

        if let None = choice {
            return Err("choices fields is empty".to_string());
        }

        Ok(choice
           .unwrap()
           .message
           .content.clone())
    }

    fn endpoint_url(&self, _llm_model: String) -> String {
        return format!("{}/{}", &HUGGING_FACE_INFERENCE_PROVIDER_URL.to_string(), "novita/v3/openai/chat/completions");
    }
}
