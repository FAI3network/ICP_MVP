use crate::CONFIGURATION;
use crate::config_management::HUGGING_FACE_API_KEY_CONFIG_KEY;
use serde::{Deserialize, Serialize};

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
};

use num_traits::cast::ToPrimitive;
use crate::types::HuggingFaceResponseItem;

const HUGGING_FACE_ENDPOINT: &str = "https://api-inference.huggingface.co/models";

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
    inputs: String,
    parameters: Option<HuggingFaceRequestParameters>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HuggingFaceResponse {
    generated_text: Option<String>,
}

/// Calls Hugging Face, returning the HF response.
///
/// # Parameters
/// - `input_text: String`: prompt to be passed to the LLM.
/// - `llm_model: String`: name of the model. It's the string that goes after 'https://api-inference.huggingface.co/models' in the URL.
/// - `seed: u32`: seed param for Hugging Face.
///
/// # Returns
/// - `Result<String, String>`: if successful, it returns the model answer, without the prompt text. Otherwise, it returns an error description.
///
pub async fn call_hugging_face(input_text: String, llm_model: String, seed: u32, hf_parameters: Option<HuggingFaceRequestParameters>) -> Result<String, String> {

    let default_parameters = HuggingFaceRequestParameters {
        max_new_tokens: Some(100),
        stop: Some(vec!['1', '2', '3']),
        temperature: Some(0.3),
        decoder_input_details: Some(false),
        details: Some(false),
        return_full_text: Some(false),
        seed: Some(seed),
        do_sample: Some(false),
    };

    let mut parameters = default_parameters;

    if let Some(p) = hf_parameters {
        parameters = p;
    }
    
    // 1) Prepare JSON payload
    let payload = HuggingFaceRequest {
        inputs: input_text,
        parameters: Some(parameters), 
    };
    let json_payload =
        serde_json::to_vec(&payload).map_err(|e| format!("Failed to serialize payload: {}", e))?;

    // ic_cdk::println!("{}", String::from_utf8(json_payload.clone()).unwrap());

    let hugging_face_bearer_token = CONFIGURATION.with(|config| {
        let config_tree = config.borrow();

        let not_found_error_message = format!("{} config key should be set.", HUGGING_FACE_API_KEY_CONFIG_KEY.to_string());
        return config_tree.get(&HUGGING_FACE_API_KEY_CONFIG_KEY.to_string()).expect(not_found_error_message.as_str());
    });

    // 2) Prepare headers
    let headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", hugging_face_bearer_token),
        },
    ];

    // 3) Construct the argument
    //    - Wrap json_payload in Some(...)
    //    - Provide max_response_bytes (e.g., 2 MB)
    let request_arg = CanisterHttpRequestArgument {
        url: format!("{}/{}", HUGGING_FACE_ENDPOINT.to_string(), llm_model),
        method: HttpMethod::POST,
        headers,
        body: Some(json_payload),
        max_response_bytes: Some(2_000_000),
        transform: None,
    };

    // 4) Make the outcall. The second param is cycles to spend (0 if none).
    let (response_tuple,): (HttpResponse,) = http_request(request_arg, 30000000000)
        .await
        .map_err(|(code, msg)| format!("HTTP request failed. Code: {:?}, Msg: {}", code, msg))?;
    let response = response_tuple;

    // Convert the `Nat` status code to u64
    let status_u64: u64 = response.status.0.to_u64().unwrap_or(0);
    if status_u64 != 200 {
        return Err(format!(
            "Hugging Face returned status {}: {}",
            status_u64,
            String::from_utf8_lossy(&response.body),
        ));
    }

    // 1) Parse raw bytes into a `serde_json::Value`
    let json_val: serde_json::Value =
        serde_json::from_slice(&response.body).map_err(|e| e.to_string())?;

    // 2) Now parse that `json_val` into a vector of your items
    let hf_response: Vec<HuggingFaceResponseItem> =
        serde_json::from_value(json_val).map_err(|e| e.to_string())?;

    // 3) Extract the text from the first item, or default
    let text: String = hf_response
        .get(0)
        .and_then(|item| item.generated_text.clone())
        .unwrap_or_else(|| "No generated_text".to_string());

    // 4) Return a `String` in `Ok(...)`
    Ok(text)
}
