/// Roles in a chat conversation.
/// Uses Vec<u8> throughout instead of String for Aeneas compatibility.
#[derive(Clone, PartialEq, Debug)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Information about a tool call made by the assistant.
#[derive(Clone, PartialEq, Debug)]
pub struct ToolCallInfo {
    pub id: Vec<u8>,
    pub function_name: Vec<u8>,
    pub arguments: Vec<u8>,
}

/// Information about the result of executing a tool.
#[derive(Clone, PartialEq, Debug)]
pub struct ToolResultInfo {
    pub tool_call_id: Vec<u8>,
    pub content: Vec<u8>,
}

/// A message in a chat conversation.
///
/// `RoleMessage` carries a role (System, User, or Assistant) and content bytes.
/// `ToolCall` represents the assistant requesting a tool invocation.
/// `ToolResult` represents the result of a tool execution.
#[derive(Clone, PartialEq, Debug)]
pub enum ChatMessage {
    RoleMessage(Role, Vec<u8>),
    ToolCall(ToolCallInfo),
    ToolResult(ToolResultInfo),
}

/// Extract the logical role of a message.
///
/// - `RoleMessage` returns its own role.
/// - `ToolCall` is always from the `Assistant`.
/// - `ToolResult` is logically a `User`-side contribution (tool output fed back).
pub fn message_role(msg: &ChatMessage) -> Role {
    match msg {
        ChatMessage::RoleMessage(role, _) => role.clone(),
        ChatMessage::ToolCall(_) => Role::Assistant,
        ChatMessage::ToolResult(_) => Role::User,
    }
}

/// Return the content bytes of a message (for token estimation).
pub fn message_content(msg: &ChatMessage) -> &[u8] {
    match msg {
        ChatMessage::RoleMessage(_, content) => content,
        ChatMessage::ToolCall(info) => &info.arguments,
        ChatMessage::ToolResult(info) => &info.content,
    }
}
