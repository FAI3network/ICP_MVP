use crate::CONFIGURATION;
use crate::config_management::HUGGING_FACE_API_KEY_CONFIG_KEY;
use serde::{Deserialize, Serialize};

use super::inference_providers::{
    InferenceProvider,
    novita::NovitaProvider,
    together::TogetherAIProvider,
    nebius::NebiusProvider,
    none::NoneProvider,
    lib::HuggingFaceRequestParameters,
};

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
};

use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc, TransformArgs};

use num_traits::cast::ToPrimitive;

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

    let res = raw.response;
    if let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(&res.body) {
        // id and created field are variable, so this converts them to ""
        if let Some(obj) = json.as_object_mut() {
            // Novita, Nebius, TogetherAI use this struture
            // Cleaning "id" and "created" fields
            if let Some(id) = obj.get_mut("id") {
                *id = serde_json::Value::String("".to_string());
            }
            if let Some(created) = obj.get_mut("created") {
                ic_cdk::println!("Changing created");
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
            "togetherai" => Box::new(TogetherAIProvider{}),
            "nebius" => Box::new(NebiusProvider{}),
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

    // From: https://github.com/dfinity/examples/blob/master/rust/send_http_post/src/send_http_post_backend/src/lib.rsL80
    let context = Context {
        bucket_start_time_index: 0,
        closing_price_index: 4,
    };
    let transform = Some(TransformContext {
        context: serde_json::to_vec(&context).unwrap(),
        function: TransformFunc(candid::Func {
            principal: ic_cdk::api::id(),
            method: "transform_hf_response".to_string(),
        }),         
    });

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
