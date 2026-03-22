//! Agent state machine: phases, events, actions, and pure transition function.
//!
//! The agent lifecycle is modelled as a 7-state deterministic state machine.
//! Only valid (phase, event) pairs produce a transition; everything else
//! returns `None`.

/// The seven phases of the agent lifecycle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentPhase {
    Idle,
    Thinking,
    CallingTool,
    AwaitingToolResult,
    Composing,
    Done,
    Error,
}

/// External stimuli that drive the agent forward.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AgentEvent {
    UserMessage,
    LlmResponse,
    ToolCallNeeded,
    ToolResult,
    Timeout,
    Cancel,
    ComposeDone,
    ThinkingDone,
}

/// Observable outputs produced by transitions.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AgentAction {
    SendToLlm,
    ExecuteTool,
    EmitResponse,
    LogEntry,
    Noop,
}

/// Pure state-machine transition.
///
/// Returns `Some((next_phase, action))` for valid transitions, `None` otherwise.
/// Terminal states (`Done`, `Error`) reject all events except that `Cancel` and
/// `Timeout` from any non-terminal state transition to `Error`.
pub fn agent_transition(phase: AgentPhase, event: &AgentEvent) -> Option<(AgentPhase, AgentAction)> {
    match (phase, event) {
        // --- happy path ---
        (AgentPhase::Idle, AgentEvent::UserMessage) => {
            Some((AgentPhase::Thinking, AgentAction::SendToLlm))
        }
        (AgentPhase::Thinking, AgentEvent::LlmResponse) => {
            Some((AgentPhase::Composing, AgentAction::Noop))
        }
        (AgentPhase::Thinking, AgentEvent::ToolCallNeeded) => {
            Some((AgentPhase::CallingTool, AgentAction::ExecuteTool))
        }
        (AgentPhase::Thinking, AgentEvent::ThinkingDone) => {
            Some((AgentPhase::Composing, AgentAction::Noop))
        }
        (AgentPhase::CallingTool, AgentEvent::ToolResult) => {
            Some((AgentPhase::AwaitingToolResult, AgentAction::Noop))
        }
        (AgentPhase::AwaitingToolResult, AgentEvent::ToolResult) => {
            Some((AgentPhase::Thinking, AgentAction::SendToLlm))
        }
        (AgentPhase::Composing, AgentEvent::ComposeDone) => {
            Some((AgentPhase::Done, AgentAction::EmitResponse))
        }

        // --- error transitions from any non-terminal state ---
        (AgentPhase::Done, _) => None,
        (AgentPhase::Error, _) => None,
        (_, AgentEvent::Cancel) => Some((AgentPhase::Error, AgentAction::LogEntry)),
        (_, AgentEvent::Timeout) => Some((AgentPhase::Error, AgentAction::LogEntry)),

        // --- everything else is invalid ---
        _ => None,
    }
}

/// Returns `true` if the phase is terminal (no further transitions possible).
pub fn is_terminal(phase: AgentPhase) -> bool {
    matches!(phase, AgentPhase::Done | AgentPhase::Error)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_user_message() {
        let r = agent_transition(AgentPhase::Idle, &AgentEvent::UserMessage);
        assert_eq!(r, Some((AgentPhase::Thinking, AgentAction::SendToLlm)));
    }

    #[test]
    fn test_thinking_llm_response() {
        let r = agent_transition(AgentPhase::Thinking, &AgentEvent::LlmResponse);
        assert_eq!(r, Some((AgentPhase::Composing, AgentAction::Noop)));
    }

    #[test]
    fn test_thinking_tool_call_needed() {
        let r = agent_transition(AgentPhase::Thinking, &AgentEvent::ToolCallNeeded);
        assert_eq!(r, Some((AgentPhase::CallingTool, AgentAction::ExecuteTool)));
    }

    #[test]
    fn test_awaiting_tool_result() {
        let r = agent_transition(AgentPhase::AwaitingToolResult, &AgentEvent::ToolResult);
        assert_eq!(r, Some((AgentPhase::Thinking, AgentAction::SendToLlm)));
    }

    #[test]
    fn test_composing_done() {
        let r = agent_transition(AgentPhase::Composing, &AgentEvent::ComposeDone);
        assert_eq!(r, Some((AgentPhase::Done, AgentAction::EmitResponse)));
    }

    #[test]
    fn test_done_rejects_all() {
        let events = [
            AgentEvent::UserMessage, AgentEvent::LlmResponse, AgentEvent::ToolCallNeeded,
            AgentEvent::ToolResult, AgentEvent::Timeout, AgentEvent::Cancel,
            AgentEvent::ComposeDone, AgentEvent::ThinkingDone,
        ];
        for ev in &events {
            assert_eq!(agent_transition(AgentPhase::Done, ev), None);
        }
    }

    #[test]
    fn test_error_rejects_all() {
        let events = [
            AgentEvent::UserMessage, AgentEvent::LlmResponse, AgentEvent::ToolCallNeeded,
            AgentEvent::ToolResult, AgentEvent::Timeout, AgentEvent::Cancel,
            AgentEvent::ComposeDone, AgentEvent::ThinkingDone,
        ];
        for ev in &events {
            assert_eq!(agent_transition(AgentPhase::Error, ev), None);
        }
    }

    #[test]
    fn test_cancel_from_non_terminal() {
        let phases = [
            AgentPhase::Idle, AgentPhase::Thinking, AgentPhase::CallingTool,
            AgentPhase::AwaitingToolResult, AgentPhase::Composing,
        ];
        for p in &phases {
            let r = agent_transition(*p, &AgentEvent::Cancel);
            assert_eq!(r, Some((AgentPhase::Error, AgentAction::LogEntry)));
        }
    }

    #[test]
    fn test_timeout_from_non_terminal() {
        let phases = [
            AgentPhase::Idle, AgentPhase::Thinking, AgentPhase::CallingTool,
            AgentPhase::AwaitingToolResult, AgentPhase::Composing,
        ];
        for p in &phases {
            let r = agent_transition(*p, &AgentEvent::Timeout);
            assert_eq!(r, Some((AgentPhase::Error, AgentAction::LogEntry)));
        }
    }

    #[test]
    fn test_invalid_transition_returns_none() {
        assert_eq!(agent_transition(AgentPhase::Idle, &AgentEvent::LlmResponse), None);
        assert_eq!(agent_transition(AgentPhase::Composing, &AgentEvent::UserMessage), None);
    }

    #[test]
    fn test_is_terminal() {
        assert!(is_terminal(AgentPhase::Done));
        assert!(is_terminal(AgentPhase::Error));
        assert!(!is_terminal(AgentPhase::Idle));
        assert!(!is_terminal(AgentPhase::Thinking));
        assert!(!is_terminal(AgentPhase::CallingTool));
        assert!(!is_terminal(AgentPhase::AwaitingToolResult));
        assert!(!is_terminal(AgentPhase::Composing));
    }
}
