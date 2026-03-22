//! Decision logic: deterministic action selection based on context.
//!
//! `decide_next_action` examines the current `DecisionContext` and produces
//! a `Decision` without side effects.

use crate::reasoning::Decision;

/// Everything the decision function needs to know about the current state.
pub struct DecisionContext {
    /// Tool name_ids that are available in the registry.
    pub available_tools: Vec<u32>,
    /// Number of messages in the conversation so far.
    pub conversation_len: u32,
    /// An opaque tag for the most recent response kind (0 = none).
    pub last_response_kind: u32,
    /// How many tool results are still pending.
    pub pending_tool_results: u32,
    /// Steps executed so far.
    pub step_count: u32,
    /// Maximum allowed steps.
    pub max_steps: u32,
}

/// Deterministic decision function.
///
/// Rules (evaluated in order):
/// 1. If `step_count >= max_steps` -> `GiveUp`
/// 2. If there are pending tool results -> `AskClarification` (wait for them)
/// 3. If `conversation_len == 0` -> `AskClarification` (need user input)
/// 4. If tools are available and `last_response_kind == 1` (tool-use signal)
///    -> `CallTool` with the first available tool
/// 5. Otherwise -> `Respond`
pub fn decide_next_action(ctx: &DecisionContext) -> Decision {
    if ctx.step_count >= ctx.max_steps {
        return Decision::GiveUp;
    }
    if ctx.pending_tool_results > 0 {
        return Decision::AskClarification;
    }
    if ctx.conversation_len == 0 {
        return Decision::AskClarification;
    }
    if !ctx.available_tools.is_empty() && ctx.last_response_kind == 1 {
        return Decision::CallTool {
            tool_name_idx: ctx.available_tools[0],
            args_hash: 0,
        };
    }
    Decision::Respond
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_give_up_at_max_steps() {
        let ctx = DecisionContext {
            available_tools: vec![1],
            conversation_len: 5,
            last_response_kind: 0,
            pending_tool_results: 0,
            step_count: 10,
            max_steps: 10,
        };
        assert_eq!(decide_next_action(&ctx), Decision::GiveUp);
    }

    #[test]
    fn test_ask_clarification_pending_tools() {
        let ctx = DecisionContext {
            available_tools: vec![],
            conversation_len: 3,
            last_response_kind: 0,
            pending_tool_results: 2,
            step_count: 0,
            max_steps: 100,
        };
        assert_eq!(decide_next_action(&ctx), Decision::AskClarification);
    }

    #[test]
    fn test_ask_clarification_empty_conversation() {
        let ctx = DecisionContext {
            available_tools: vec![1],
            conversation_len: 0,
            last_response_kind: 0,
            pending_tool_results: 0,
            step_count: 0,
            max_steps: 100,
        };
        assert_eq!(decide_next_action(&ctx), Decision::AskClarification);
    }

    #[test]
    fn test_call_tool_when_signaled() {
        let ctx = DecisionContext {
            available_tools: vec![42, 99],
            conversation_len: 5,
            last_response_kind: 1,
            pending_tool_results: 0,
            step_count: 2,
            max_steps: 100,
        };
        assert_eq!(
            decide_next_action(&ctx),
            Decision::CallTool { tool_name_idx: 42, args_hash: 0 }
        );
    }

    #[test]
    fn test_respond_default() {
        let ctx = DecisionContext {
            available_tools: vec![1],
            conversation_len: 5,
            last_response_kind: 0,
            pending_tool_results: 0,
            step_count: 3,
            max_steps: 100,
        };
        assert_eq!(decide_next_action(&ctx), Decision::Respond);
    }
}
