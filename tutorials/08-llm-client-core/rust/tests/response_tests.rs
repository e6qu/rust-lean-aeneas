use llm_client_core::response::*;

/// Helper to write a u32 in big-endian.
fn write_u32_be(val: u32) -> Vec<u8> {
    vec![
        ((val >> 24) & 0xFF) as u8,
        ((val >> 16) & 0xFF) as u8,
        ((val >> 8) & 0xFF) as u8,
        (val & 0xFF) as u8,
    ]
}

/// Helper to write a length-prefixed byte vector.
fn write_lp(data: &[u8]) -> Vec<u8> {
    let mut result = write_u32_be(data.len() as u32);
    result.extend_from_slice(data);
    result
}

#[test]
fn test_parse_text_response() {
    let mut buf: Vec<u8> = Vec::new();
    buf.push(0); // finish_reason = Stop
    buf.extend_from_slice(&write_u32_be(100)); // prompt_tokens
    buf.extend_from_slice(&write_u32_be(50)); // completion_tokens
    buf.extend_from_slice(&write_u32_be(1)); // content_count
    buf.push(0); // content_type = Text
    buf.extend_from_slice(&write_lp(b"Hello, world!"));

    let result = parse_response(&buf);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.finish_reason, FinishReason::Stop);
    assert_eq!(resp.usage.prompt_tokens, 100);
    assert_eq!(resp.usage.completion_tokens, 50);
    assert_eq!(resp.content.len(), 1);
    match &resp.content[0] {
        ResponseContent::Text(t) => assert_eq!(t, b"Hello, world!"),
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_parse_tool_use_response() {
    let mut buf: Vec<u8> = Vec::new();
    buf.push(2); // finish_reason = ToolUse
    buf.extend_from_slice(&write_u32_be(80)); // prompt_tokens
    buf.extend_from_slice(&write_u32_be(20)); // completion_tokens
    buf.extend_from_slice(&write_u32_be(1)); // content_count
    buf.push(1); // content_type = ToolUse
    buf.extend_from_slice(&write_lp(b"call_1"));
    buf.extend_from_slice(&write_lp(b"search"));
    buf.extend_from_slice(&write_lp(b"{\"q\":\"test\"}"));

    let result = parse_response(&buf);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.finish_reason, FinishReason::ToolUse);
    assert_eq!(resp.content.len(), 1);
    match &resp.content[0] {
        ResponseContent::ToolUse(info) => {
            assert_eq!(info.id, b"call_1");
            assert_eq!(info.function_name, b"search");
            assert_eq!(info.arguments, b"{\"q\":\"test\"}");
        }
        _ => panic!("Expected ToolUse content"),
    }
}

#[test]
fn test_parse_empty_data() {
    let result = parse_response(&[]);
    assert_eq!(result, Err(ResponseParseError::InvalidFormat));
}

#[test]
fn test_parse_invalid_finish_reason() {
    let buf = vec![99u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let result = parse_response(&buf);
    assert_eq!(result, Err(ResponseParseError::InvalidFinishReason));
}

#[test]
fn test_parse_truncated_data() {
    let buf = vec![0u8, 0, 0]; // Too short for usage fields
    let result = parse_response(&buf);
    assert_eq!(result, Err(ResponseParseError::InvalidFormat));
}
