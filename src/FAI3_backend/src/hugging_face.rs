use serde::{Deserialize, Serialize};
use ic_cdk::api::management_canister::http_request::{
  http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
};
use num_traits::cast::ToPrimitive;
use ic_cdk_macros::*;
use crate::types::HuggingFaceResponseItem;


#[derive(Serialize, Deserialize)]
struct HuggingFaceRequest {
    inputs: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HuggingFaceResponse {
    generated_text: Option<String>,
}

const HUGGING_FACE_ENDPOINT: &str = "https://api-inference.huggingface.co/models/gpt2";
const HUGGING_FACE_BEARER_TOKEN: &str = "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";

#[update]
pub async fn call_hugging_face(input_text: String) -> Result<String, String> {
    // 1) Prepare JSON payload
    let payload = HuggingFaceRequest { inputs: input_text };
    let json_payload =
        serde_json::to_vec(&payload).map_err(|e| format!("Failed to serialize payload: {}", e))?;

    // 2) Prepare headers
    let headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", HUGGING_FACE_BEARER_TOKEN),
        },
    ];

    // 3) Construct the argument
    //    - Wrap json_payload in Some(...)
    //    - Provide max_response_bytes (e.g., 2 MB)
    let request_arg = CanisterHttpRequestArgument {
        url: HUGGING_FACE_ENDPOINT.to_string(),
        method: HttpMethod::POST,
        headers,
        body: Some(json_payload),
        max_response_bytes: Some(2_000_000),
        transform: None,
    };

    // 4) Make the outcall. The second param is cycles to spend (0 if none).
    let (response_tuple,): (HttpResponse,) = http_request(request_arg, 20000000000)
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