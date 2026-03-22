use crate::message_types::{ChatMessage, Role, message_role};

/// A simplified tool definition for the request.
#[derive(Clone, PartialEq, Debug)]
pub struct ToolDef {
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub parameters_schema: Vec<u8>,
}

/// A well-formed LLM API request.
///
/// Temperature is fixed-point: value 100 means 1.0, value 200 means 2.0.
/// This avoids floating point, which Aeneas cannot handle.
#[derive(Clone, PartialEq, Debug)]
pub struct Request {
    pub model: Vec<u8>,
    pub messages: Vec<ChatMessage>,
    pub temperature: u32,
    pub max_tokens: u32,
    pub tools: Vec<ToolDef>,
}

/// Errors that can occur when building a request.
#[derive(Clone, PartialEq, Debug)]
pub enum RequestError {
    EmptyMessages,
    NoSystemMessage,
    TemperatureTooHigh,
    MaxTokensZero,
}

/// Build a validated LLM request.
///
/// Validates:
/// - `messages` is non-empty
/// - The first message has role `System`
/// - `temperature` <= 200 (i.e., 2.0 in fixed-point with scale 100)
/// - `max_tokens` > 0
///
/// If all checks pass, returns `Ok(Request)`. Otherwise returns the
/// appropriate `RequestError`.
pub fn build_request(
    model: &[u8],
    messages: &[ChatMessage],
    temperature: u32,
    max_tokens: u32,
    tools: &[ToolDef],
) -> Result<Request, RequestError> {
    if messages.is_empty() {
        return Err(RequestError::EmptyMessages);
    }

    let first_role = message_role(&messages[0]);
    if first_role != Role::System {
        return Err(RequestError::NoSystemMessage);
    }

    if temperature > 200 {
        return Err(RequestError::TemperatureTooHigh);
    }

    if max_tokens == 0 {
        return Err(RequestError::MaxTokensZero);
    }

    // Clone all inputs into the request. Aeneas-friendly: no references stored.
    let mut msg_vec: Vec<ChatMessage> = Vec::new();
    let mut i: usize = 0;
    while i < messages.len() {
        msg_vec.push(messages[i].clone());
        i += 1;
    }

    let mut tool_vec: Vec<ToolDef> = Vec::new();
    let mut j: usize = 0;
    while j < tools.len() {
        tool_vec.push(tools[j].clone());
        j += 1;
    }

    Ok(Request {
        model: model.to_vec(),
        messages: msg_vec,
        temperature,
        max_tokens,
        tools: tool_vec,
    })
}
