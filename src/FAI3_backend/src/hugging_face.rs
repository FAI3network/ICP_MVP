use crate::CONFIGURATION;
use crate::config_management::{
    HUGGING_FACE_API_KEY_CONFIG_KEY,
};
use serde::{Deserialize, Serialize};

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
};

use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc, TransformArgs};

use num_traits::cast::ToPrimitive;
use crate::types::HuggingFaceResponseItem;

const HUGGING_FACE_ENDPOINT: &str = "https://api-inference.huggingface.co/models";
const HUGGING_FACE_INFERENCE_PROVIDER_URL: &str = "https://router.huggingface.co";

trait InferenceProvider {
    fn name(&self) -> &str;
    fn generate_payload(&self, llm_model: String, input_text: String, parameters: HuggingFaceRequestParameters) -> Result<Vec<u8>, String>;
    fn get_response_text(&self, response_body: &Vec<u8>) -> Result<String, String>;
    fn endpoint_url(&self, llm_model: String) -> String;
}

struct NovitaProvider {}

// Structure when no provider is used.
// It calls Hugging Face API
struct NoneProvider {}

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

// OpenAI compatible API request, for inference providers
#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    seed: Option<u32>,
    do_sample: Option<bool>,
    stream: bool,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIMessage {
    role: String,
    content: String,
}

// This struct is legacy code and is not really used in the code.
#[derive(Serialize, Deserialize)]
struct Context {
    bucket_start_time_index: usize,
    closing_price_index: usize,
}

// Necessary function to remove the non-determinism "id" and "created" values
// It replaces them with ""
#[ic_cdk::query]
fn transform_hf_response(raw: TransformArgs) -> HttpResponse {
    // This might not be necessary, but we are overriding the headers
    // Just in case they return anything variable
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let body: Vec::<u8>;
    let status = raw.response.status.clone();
    if status != 200_u16 {
        ic_cdk::api::print(format!("Transform function: received an error from Hugging Face: err = {:?}", raw));  
        return raw.response;
    }

    // TODO: this only works for Novita?
    let res = raw.response;
    if let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(&res.body) {
        // id and created field are variable, so this converts them to ""
        if let Some(obj) = json.as_object_mut() {
            if let Some(id) = obj.get_mut("id") {
                *id = serde_json::Value::String("".to_string());
            }
            if let Some(created) = obj.get_mut("created") {
                *created = serde_json::Value::Number(serde_json::Number::from(0));
            }
        }
        body = serde_json::to_vec(&json).unwrap_or(res.body);
    } else {
        body = res.body;
    }

    let res = HttpResponse {
        status,
        body,
        headers,
        ..Default::default()
    };

    res
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
pub async fn call_hugging_face(input_text: String, llm_model: String, seed: u32, hf_parameters: Option<HuggingFaceRequestParameters>, inference_provider: &Option<String>) -> Result<String, String> {

    let hugging_face_bearer_token = CONFIGURATION.with(|config| {
        let config_tree = config.borrow();

        let not_found_error_message = format!("{} config key should be set.", HUGGING_FACE_API_KEY_CONFIG_KEY.to_string());
        return config_tree.get(&HUGGING_FACE_API_KEY_CONFIG_KEY.to_string()).expect(not_found_error_message.as_str());
    });

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

    let provider: Box<dyn InferenceProvider> = if let Some(_provider) = inference_provider.clone() {
        ic_cdk::println!("configured provider: {}", _provider);
        match _provider.as_str() {
            "novita" => Box::new(NovitaProvider{}),
            p => {
                ic_cdk::println!("No known provider {}, using NoneProvider", p);
                Box::new(NoneProvider{})
            }
        }
    } else {
        Box::new(NoneProvider{})
    };

    ic_cdk::println!("Using {} provider", provider.name());

    // 1) Generate payload
    let json_payload = provider.generate_payload(llm_model.clone(), input_text, parameters)?;

    // ic_cdk::println!("{}", String::from_utf8(json_payload.clone()).unwrap());
 

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
    let url = provider.endpoint_url(llm_model);

    let mut transform = None;

    if let Some(_provider) = inference_provider.clone() {
        ic_cdk::println!("using transform function");
        // From: https://github.com/dfinity/examples/blob/master/rust/send_http_post/src/send_http_post_backend/src/lib.rsL80
        let context = Context {
            bucket_start_time_index: 0,
            closing_price_index: 4,
        };

        // TODO: continue from here: https://github.com/dfinity/examples/blob/master/rust/send_http_post/src/send_http_post_backend/src/lib.rs#L145
        transform = Some(TransformContext {
            context: serde_json::to_vec(&context).unwrap(),
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform_hf_response".to_string(),
            }),         
        });
    }
    
    ic_cdk::println!("Endpoint url: {}", url);
    ic_cdk::println!("json payload: {}", String::from_utf8_lossy(&json_payload));
    
    let request_arg = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::POST,
        headers,
        body: Some(json_payload),
        max_response_bytes: Some(2_000_000),
        transform,
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

    ic_cdk::println!("Json response: {}", String::from_utf8_lossy(&response.body));

    // 1) Parse raw bytes into a `serde_json::Value`
    let json_val: serde_json::Value =
        serde_json::from_slice(&response.body).map_err(|e| e.to_string())?;

    ic_cdk::println!("HF response: {}", &json_val);

    return provider.get_response_text(&response.body);
}
