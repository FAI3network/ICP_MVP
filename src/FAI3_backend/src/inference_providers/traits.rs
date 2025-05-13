use super::lib::HuggingFaceRequestParameters;

pub trait InferenceProvider {
    fn name(&self) -> &str;
    fn generate_payload(&self, llm_model: String, input_text: String, parameters: HuggingFaceRequestParameters) -> Result<Vec<u8>, String>;
    fn get_response_text(&self, response_body: &Vec<u8>) -> Result<String, String>;
    fn endpoint_url(&self, llm_model: String) -> String;
}
