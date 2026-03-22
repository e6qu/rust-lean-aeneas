use crate::message_types::{ChatMessage, Role, message_role};
use crate::token_estimate::estimate_tokens;

/// A managed conversation with a maximum context window.
///
/// The first message is always the system prompt. Subsequent messages
/// alternate between user and assistant roles (with tool calls/results
/// interspersed on the assistant/user sides respectively).
#[derive(Clone, Debug)]
pub struct Conversation {
    pub messages: Vec<ChatMessage>,
    pub max_context_tokens: u32,
}

/// Errors when manipulating a conversation.
#[derive(Clone, PartialEq, Debug)]
pub enum ConvError {
    NotSystemFirst,
    InvalidAlternation,
}

impl Conversation {
    /// Create a new conversation with the given system prompt and token budget.
    pub fn new(system_msg: Vec<u8>, max_context: u32) -> Self {
        let messages: Vec<ChatMessage> = vec![ChatMessage::RoleMessage(Role::System, system_msg)];
        Conversation {
            messages,
            max_context_tokens: max_context,
        }
    }

    /// Append a message to the conversation.
    ///
    /// Validates:
    /// - If the conversation is empty (should never happen after `new`), the
    ///   message must be a System message.
    /// - Otherwise, the new message must respect alternation: after the system
    ///   message, roles should alternate between User and Assistant. ToolCall
    ///   messages count as Assistant; ToolResult messages count as User.
    pub fn append(&mut self, msg: ChatMessage) -> Result<(), ConvError> {
        if self.messages.is_empty() {
            // First message must be system
            let role = message_role(&msg);
            if role != Role::System {
                return Err(ConvError::NotSystemFirst);
            }
            self.messages.push(msg);
            return Ok(());
        }

        let last_role = message_role(&self.messages[self.messages.len() - 1]);
        let new_role = message_role(&msg);

        // After system, first real message must be User
        // After User, next must be Assistant (or ToolCall)
        // After Assistant (or ToolCall), next must be User (or ToolResult)
        // ToolResult after ToolCall is also valid (both are user-side after assistant-side)
        let valid = match last_role {
            Role::System => new_role == Role::User,
            Role::User => new_role == Role::Assistant,
            Role::Assistant => new_role == Role::User,
        };

        if !valid {
            return Err(ConvError::InvalidAlternation);
        }

        self.messages.push(msg);
        Ok(())
    }

    /// Trim the conversation so the estimated token count fits within
    /// `max_context_tokens`.
    ///
    /// Strategy: repeatedly remove `messages[1]` (the oldest non-system
    /// message) until the estimate fits or only the system message remains.
    ///
    /// Uses a while loop for Aeneas compatibility.
    pub fn trim_to_context(&mut self) {
        while self.messages.len() > 1 && estimate_tokens(&self.messages) > self.max_context_tokens {
            self.messages.remove(1);
        }
    }

    /// Return the estimated token count for the current messages.
    pub fn total_estimated_tokens(&self) -> u32 {
        estimate_tokens(&self.messages)
    }
}
