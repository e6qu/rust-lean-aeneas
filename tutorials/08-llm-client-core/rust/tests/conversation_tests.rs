use llm_client_core::message_types::*;
use llm_client_core::conversation::*;

fn user_msg(s: &str) -> ChatMessage {
    ChatMessage::RoleMessage(Role::User, s.as_bytes().to_vec())
}

fn assistant_msg(s: &str) -> ChatMessage {
    ChatMessage::RoleMessage(Role::Assistant, s.as_bytes().to_vec())
}

#[test]
fn test_new_conversation() {
    let conv = Conversation::new(b"You are helpful.".to_vec(), 1000);
    assert_eq!(conv.messages.len(), 1);
    assert_eq!(
        conv.messages[0],
        ChatMessage::RoleMessage(Role::System, b"You are helpful.".to_vec())
    );
}

#[test]
fn test_append_valid_alternation() {
    let mut conv = Conversation::new(b"sys".to_vec(), 10000);
    assert!(conv.append(user_msg("hello")).is_ok());
    assert!(conv.append(assistant_msg("hi")).is_ok());
    assert!(conv.append(user_msg("how are you?")).is_ok());
    assert_eq!(conv.messages.len(), 4);
}

#[test]
fn test_append_invalid_user_after_user() {
    let mut conv = Conversation::new(b"sys".to_vec(), 10000);
    assert!(conv.append(user_msg("hello")).is_ok());
    let result = conv.append(user_msg("hello again"));
    assert_eq!(result, Err(ConvError::InvalidAlternation));
}

#[test]
fn test_append_invalid_assistant_first() {
    let mut conv = Conversation::new(b"sys".to_vec(), 10000);
    let result = conv.append(assistant_msg("hi"));
    assert_eq!(result, Err(ConvError::InvalidAlternation));
}

#[test]
fn test_append_tool_call_after_user() {
    let mut conv = Conversation::new(b"sys".to_vec(), 10000);
    assert!(conv.append(user_msg("search for cats")).is_ok());
    // ToolCall has role Assistant, so it should be valid after User
    let tool_call = ChatMessage::ToolCall(ToolCallInfo {
        id: b"1".to_vec(),
        function_name: b"search".to_vec(),
        arguments: b"cats".to_vec(),
    });
    assert!(conv.append(tool_call).is_ok());
}

#[test]
fn test_append_tool_result_after_tool_call() {
    let mut conv = Conversation::new(b"sys".to_vec(), 10000);
    assert!(conv.append(user_msg("search for cats")).is_ok());
    let tool_call = ChatMessage::ToolCall(ToolCallInfo {
        id: b"1".to_vec(),
        function_name: b"search".to_vec(),
        arguments: b"cats".to_vec(),
    });
    assert!(conv.append(tool_call).is_ok());
    // ToolResult has role User, valid after Assistant (ToolCall)
    let tool_result = ChatMessage::ToolResult(ToolResultInfo {
        tool_call_id: b"1".to_vec(),
        content: b"Found 42 cats".to_vec(),
    });
    assert!(conv.append(tool_result).is_ok());
}

#[test]
fn test_trim_to_context() {
    // System message is small, then add many user/assistant messages
    let mut conv = Conversation::new(b"sys".to_vec(), 20);
    // Each message ~(content_len/4 + 4) tokens
    // "sys" = 3 bytes => 3/4+4 = 4 tokens
    // Add pairs that exceed the budget
    let _ = conv.append(user_msg("a]long message that has many bytes in it"));
    let _ = conv.append(assistant_msg("another long response with lots of text"));
    let _ = conv.append(user_msg("yet another message"));
    let _ = conv.append(assistant_msg("and the final response"));

    let before = conv.messages.len();
    conv.trim_to_context();
    // After trimming, should have fewer messages and fit within budget
    assert!(conv.messages.len() <= before);
    assert!(conv.total_estimated_tokens() <= conv.max_context_tokens || conv.messages.len() <= 2);
}

#[test]
fn test_trim_preserves_system() {
    let mut conv = Conversation::new(b"system prompt".to_vec(), 5);
    let _ = conv.append(user_msg("hello"));
    let _ = conv.append(assistant_msg("world"));
    conv.trim_to_context();
    // System message should always be preserved
    assert!(conv.messages.len() >= 1);
    assert_eq!(
        conv.messages[0],
        ChatMessage::RoleMessage(Role::System, b"system prompt".to_vec())
    );
}

#[test]
fn test_total_estimated_tokens() {
    let conv = Conversation::new(b"sys".to_vec(), 10000);
    // "sys" = 3 bytes => 3/4 + 4 = 4 tokens
    assert_eq!(conv.total_estimated_tokens(), 4);
}
