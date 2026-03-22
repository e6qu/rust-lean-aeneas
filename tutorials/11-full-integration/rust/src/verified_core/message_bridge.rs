// message_bridge.rs — Type-conversion bridge between components.
//
// These small pure functions translate between the different type vocabularies
// used by the TUI, orchestrator, and LLM subsystems.  In the Lean proofs we
// show that every bridge function preserves validity (well-formedness).

use crate::verified_core::app_state::ConversationEntry;

/// The number of defined `PaneId` variants.
pub const PANE_COUNT: u32 = 4;

/// Build a message-bus triple from a user submission.
///
/// The user is always sender `u32::MAX` (a reserved id meaning "human user"),
/// and the message is addressed to the coordinator agent.
///
/// Returns `(sender, recipient, content_id)`.
pub fn user_input_to_message(content_id: u32, coordinator_id: u32) -> (u32, u32, u32) {
    (u32::MAX, coordinator_id, content_id)
}

/// Build a `ConversationEntry` from raw fields.
///
/// `sender` is mapped to `agent_id` and `role` is set to 0 (user) when the
/// sender is `u32::MAX`, otherwise 1 (assistant).
pub fn message_to_conversation_entry(
    sender: u32,
    content_id: u32,
    timestamp: u32,
) -> ConversationEntry {
    let role = if sender == u32::MAX { 0 } else { 1 };
    ConversationEntry {
        agent_id: sender,
        role,
        content_id,
        timestamp,
    }
}

/// Check whether a numeric pane id corresponds to a valid `PaneId` variant.
pub fn is_valid_pane_id(id: u32) -> bool {
    id < PANE_COUNT
}

/// Check whether an agent id is within the valid range `[0, agent_count)`.
pub fn is_valid_agent_id(id: u32, agent_count: u32) -> bool {
    id < agent_count
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_input_to_message_sender_is_max() {
        let (sender, recipient, cid) = user_input_to_message(42, 0);
        assert_eq!(sender, u32::MAX);
        assert_eq!(recipient, 0);
        assert_eq!(cid, 42);
    }

    #[test]
    fn conversation_entry_user_role() {
        let entry = message_to_conversation_entry(u32::MAX, 10, 0);
        assert_eq!(entry.role, 0);
    }

    #[test]
    fn conversation_entry_assistant_role() {
        let entry = message_to_conversation_entry(1, 10, 0);
        assert_eq!(entry.role, 1);
    }

    #[test]
    fn pane_id_validity() {
        assert!(is_valid_pane_id(0));
        assert!(is_valid_pane_id(3));
        assert!(!is_valid_pane_id(4));
        assert!(!is_valid_pane_id(u32::MAX));
    }

    #[test]
    fn agent_id_validity() {
        assert!(is_valid_agent_id(0, 3));
        assert!(is_valid_agent_id(2, 3));
        assert!(!is_valid_agent_id(3, 3));
        assert!(!is_valid_agent_id(0, 0));
    }
}
