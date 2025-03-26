use candid::{Principal, decode_one, encode_args};
use FAI3_backend::types::ContextAssociationTestAPIResult;
mod common;
use common::{
    create_pic, create_llm_model,
    wait_for_http_request, mock_http_response, mock_correct_hugging_face_response_body
};


fn cat_with_two_queries_test(first_returned_text: &str, second_returned_text: &str) -> ContextAssociationTestAPIResult {
    let (pic, canister_id) = create_pic();
    
    // Creating model
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);

    // Calling context_association_test
    let max_queries: usize = 2;
    let seed: u32 = 1;
    let shuffle_questions: bool = false;
    let encoded_args = encode_args((model_id, max_queries, seed, shuffle_questions)).unwrap();
    // Submit an update call to the test canister making a canister http outcall
    // and mock a canister http outcall response.
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "context_association_test",
        encoded_args,
    ).expect("context_association_test call should succeed");

    // We need a pair of ticks for the test canister method to make the http outcall
    // and for the management canister to start processing the http outcall.
    wait_for_http_request(&pic);
    
    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_hf_response_body = mock_correct_hugging_face_response_body(first_returned_text);
    let mock_canister_http_response = mock_http_response(canister_http_request, mock_hf_response_body);
    pic.mock_canister_http_response(mock_canister_http_response);

    wait_for_http_request(&pic);

    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_hf_response_body = mock_correct_hugging_face_response_body(second_returned_text);
    let mock_canister_http_response = mock_http_response(canister_http_request, mock_hf_response_body);
    pic.mock_canister_http_response(mock_canister_http_response);
    
    // There should be no more pending canister http outcalls.
    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 0);

    // Now the test canister will receive the http outcall response
    // and reply to the ingress message from the test driver.
    let reply = pic.await_call(call_id).unwrap();
    let decoded_reply: Result<ContextAssociationTestAPIResult, String> = decode_one(&reply).expect("Failed to decode context association test reply");

    return decoded_reply.expect("It should be a valid CAT result");
}

#[test]
/// CAT test should return successfully, but show 2 errors in the response
fn test_llm_cat_test_wrong_hugging_face_responses() {
    let (pic, canister_id) = create_pic();
    
    // Creating model
    let model_name = String::from("Test Model");
    let model_id: u128 = create_llm_model(&pic, canister_id, model_name.clone());
    assert_eq!(model_id, 1);

    // Calling context_association_test
    let max_queries: usize = 2;
    let seed: u32 = 1;
    let shuffle_questions: bool = true;
    let encoded_args = encode_args((model_id, max_queries, seed, shuffle_questions)).unwrap();
    // Submit an update call to the test canister making a canister http outcall
    // and mock a canister http outcall response.
    let call_id = pic.submit_call(
        canister_id,
        Principal::anonymous(),
        "context_association_test",
        encoded_args,
    ).expect("context_association_test call should succeed");

    // We need a pair of ticks for the test canister method to make the http outcall
    // and for the management canister to start processing the http outcall.
    wait_for_http_request(&pic);
    
    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_canister_http_response = mock_http_response(canister_http_request, b"invalid json");
    pic.mock_canister_http_response(mock_canister_http_response);

    wait_for_http_request(&pic);

    let canister_http_requests = pic.get_canister_http();
    let canister_http_request = &canister_http_requests[0];

    let mock_canister_http_response = mock_http_response(canister_http_request, b"invalid json");
    pic.mock_canister_http_response(mock_canister_http_response);
    
    // There should be no more pending canister http outcalls.
    let canister_http_requests = pic.get_canister_http();
    assert_eq!(canister_http_requests.len(), 0);

    // Now the test canister will receive the http outcall response
    // and reply to the ingress message from the test driver.
    let reply = pic.await_call(call_id).unwrap();
    let decoded_reply: Result<ContextAssociationTestAPIResult, String> = decode_one(&reply).expect("Failed to decode context association test reply");

    let cat_result = decoded_reply.expect("It should be a valid CAT result");
    
    assert_eq!(cat_result.error_count, 2);
    assert_eq!(cat_result.general_n, 0);
}

#[test]
/// CAT test should return successfully, but show 2 errors in the response
fn test_llm_cat_test_invalid_hugging_face_responses() {
    let cat_result = cat_with_two_queries_test("hello", "hello");

    // Invalid (but correctly formatted) text responses
    // are not classified as errors, but as 'other'
    assert_eq!(cat_result.error_count, 0);
    assert_eq!(cat_result.general_n, 2);
    assert_eq!(cat_result.general.other, 2);
    assert_eq!(cat_result.general.stereotype, 0);
    assert_eq!(cat_result.general.anti_stereotype, 0);
    assert_eq!(cat_result.general.neutral, 0);
    assert_eq!(cat_result.general_lms, 0.0 as f32);
    // ss and icat_score_general are not defined if no valid response was given
}

#[test]
fn test_llm_cat_perfect_balance_test() {
    // stereotype, anti-stereotype
    let cat_result = cat_with_two_queries_test("3", "1");
    
    assert_eq!(cat_result.error_count, 0);
    assert_eq!(cat_result.general_n, 2);
    assert_eq!(cat_result.general.stereotype, 1);
    assert_eq!(cat_result.general.anti_stereotype, 1);
    assert_eq!(cat_result.general.other, 0);
    assert_eq!(cat_result.general.neutral, 0);
    assert_eq!(cat_result.general_ss, 50.0 as f32);
    assert_eq!(cat_result.general_lms, 100.0 as f32);
    assert_eq!(cat_result.icat_score_general, 100.0 as f32);
}
