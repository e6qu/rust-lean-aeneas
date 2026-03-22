use crate::message_types::{ChatMessage, message_content};

/// Approximate bytes per token. The standard heuristic is ~4 bytes per token
/// for English text. This is a constant so the Lean translation is trivial.
pub const BYTES_PER_TOKEN: u32 = 4;

/// Per-message overhead in tokens (accounts for role tags, separators, etc.).
pub const MESSAGE_OVERHEAD: u32 = 4;

/// Estimate the token count for a single message.
///
/// Formula: content_length / BYTES_PER_TOKEN + MESSAGE_OVERHEAD
/// Uses integer division (floor).
pub fn estimate_tokens_single(msg: &ChatMessage) -> u32 {
    let content = message_content(msg);
    let content_len = content.len() as u32;
    content_len / BYTES_PER_TOKEN + MESSAGE_OVERHEAD
}

/// Estimate the total token count for a slice of messages.
///
/// Sums `estimate_tokens_single` over all messages.
/// Uses an explicit while loop for Aeneas compatibility.
pub fn estimate_tokens(messages: &[ChatMessage]) -> u32 {
    let mut total: u32 = 0;
    let mut i: usize = 0;
    while i < messages.len() {
        total += estimate_tokens_single(&messages[i]);
        i += 1;
    }
    total
}
