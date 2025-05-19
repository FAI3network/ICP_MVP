use serde::{Deserialize, Serialize};

pub const HUGGING_FACE_ENDPOINT: &str = "https://api-inference.huggingface.co/models";
pub const HUGGING_FACE_INFERENCE_PROVIDER_URL: &str = "https://router.huggingface.co";

// OpenAI compatible API request, for inference providers
#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub max_tokens: Option<u32>,
    pub seed: Option<u32>,
    pub do_sample: Option<bool>,
    pub stream: bool,
    pub temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HuggingFaceRequestParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<char>>,
    pub max_new_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub return_full_text: Option<bool>,
    pub decoder_input_details: Option<bool>,
    pub details: Option<bool>,
    pub seed: Option<u32>,
    pub do_sample: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct HuggingFaceRequest {
    pub inputs: String,
    pub parameters: Option<HuggingFaceRequestParameters>,
}
