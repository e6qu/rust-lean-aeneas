use llm_client_core::message_types::*;
use llm_client_core::request::*;

fn system_msg(s: &str) -> ChatMessage {
    ChatMessage::RoleMessage(Role::System, s.as_bytes().to_vec())
}

fn user_msg(s: &str) -> ChatMessage {
    ChatMessage::RoleMessage(Role::User, s.as_bytes().to_vec())
}

#[test]
fn test_build_request_success() {
    let msgs = vec![system_msg("You are helpful."), user_msg("Hello")];
    let result = build_request(b"gpt-4", &msgs, 100, 1024, &[]);
    assert!(result.is_ok());
    let req = result.unwrap();
    assert_eq!(req.model, b"gpt-4");
    assert_eq!(req.messages.len(), 2);
    assert_eq!(req.temperature, 100);
    assert_eq!(req.max_tokens, 1024);
}

#[test]
fn test_build_request_empty_messages() {
    let msgs: Vec<ChatMessage> = vec![];
    let result = build_request(b"gpt-4", &msgs, 100, 1024, &[]);
    assert_eq!(result, Err(RequestError::EmptyMessages));
}

#[test]
fn test_build_request_no_system_message() {
    let msgs = vec![user_msg("Hello")];
    let result = build_request(b"gpt-4", &msgs, 100, 1024, &[]);
    assert_eq!(result, Err(RequestError::NoSystemMessage));
}

#[test]
fn test_build_request_temperature_too_high() {
    let msgs = vec![system_msg("sys")];
    let result = build_request(b"gpt-4", &msgs, 201, 1024, &[]);
    assert_eq!(result, Err(RequestError::TemperatureTooHigh));
}

#[test]
fn test_build_request_temperature_boundary() {
    let msgs = vec![system_msg("sys")];
    // 200 should be accepted (2.0)
    let result = build_request(b"gpt-4", &msgs, 200, 1024, &[]);
    assert!(result.is_ok());
}

#[test]
fn test_build_request_max_tokens_zero() {
    let msgs = vec![system_msg("sys")];
    let result = build_request(b"gpt-4", &msgs, 100, 0, &[]);
    assert_eq!(result, Err(RequestError::MaxTokensZero));
}

#[test]
fn test_build_request_with_tools() {
    let msgs = vec![system_msg("sys"), user_msg("call a tool")];
    let tools = vec![ToolDef {
        name: b"search".to_vec(),
        description: b"Search the web".to_vec(),
        parameters_schema: b"{}".to_vec(),
    }];
    let result = build_request(b"gpt-4", &msgs, 50, 512, &tools);
    assert!(result.is_ok());
    let req = result.unwrap();
    assert_eq!(req.tools.len(), 1);
    assert_eq!(req.tools[0].name, b"search");
}
