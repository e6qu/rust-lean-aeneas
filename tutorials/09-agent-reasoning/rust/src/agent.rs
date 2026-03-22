//! Agent orchestration: config, snapshot, step, and bounded run loop.
//!
//! `agent_step` applies one event to the current snapshot, checking guards
//! and updating the reasoning chain.  `agent_run` iterates over a finite
//! event list, stopping when the agent reaches a terminal phase or runs out
//! of events.  Because the event list is finite, `agent_run` always terminates.

use crate::agent_state::{AgentAction, AgentEvent, AgentPhase, agent_transition, is_terminal};
use crate::guardrails::GuardrailConfig;
use crate::reasoning::Step;
use crate::retry::RetryState;

/// Static configuration for an agent run.
pub struct AgentConfig {
    pub max_steps: u32,
    pub guardrails: GuardrailConfig,
    pub retry_config: RetryState,
}

/// A complete snapshot of the agent's state at one point in time.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AgentSnapshot {
    pub phase: AgentPhase,
    pub reasoning_chain: Vec<Step>,
    pub step_count: u32,
    pub retry_state: RetryState,
    pub last_action: AgentAction,
}

/// Process a single event, producing a new snapshot.
///
/// Returns `None` if the transition is invalid or if the step-count guard
/// is violated (the agent has exceeded `max_steps`).
pub fn agent_step(
    config: &AgentConfig,
    snapshot: &AgentSnapshot,
    event: &AgentEvent,
) -> Option<AgentSnapshot> {
    // Guard: step count must be below max
    if snapshot.step_count >= config.max_steps {
        return None;
    }

    // Attempt the state-machine transition
    let (next_phase, action) = agent_transition(snapshot.phase, event)?;

    let new_step_count = snapshot.step_count + 1;
    Some(AgentSnapshot {
        phase: next_phase,
        reasoning_chain: snapshot.reasoning_chain.clone(),
        step_count: new_step_count,
        retry_state: snapshot.retry_state.clone(),
        last_action: action,
    })
}

/// Run the agent over a finite list of events.
///
/// Stops when:
/// - The agent reaches a terminal phase (`Done` or `Error`), or
/// - All events have been consumed, or
/// - A step fails (invalid transition or guard violation).
///
/// Because the event list is finite and each iteration consumes one event,
/// this function always terminates.
pub fn agent_run(
    config: &AgentConfig,
    initial: AgentSnapshot,
    events: &[AgentEvent],
) -> AgentSnapshot {
    let mut snapshot = initial;
    let mut i: usize = 0;
    while i < events.len() {
        if is_terminal(snapshot.phase) {
            break;
        }
        if snapshot.step_count >= config.max_steps {
            break;
        }
        match agent_step(config, &snapshot, &events[i]) {
            Some(next) => {
                snapshot = next;
            }
            None => {
                break;
            }
        }
        i += 1;
    }
    snapshot
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardrails::GuardrailConfig;
    use crate::retry::initial_retry_state;

    fn test_config() -> AgentConfig {
        AgentConfig {
            max_steps: 100,
            guardrails: GuardrailConfig {
                max_message_len: 4096,
                max_recursion_depth: 10,
                max_reasoning_steps: 50,
            },
            retry_config: initial_retry_state(3, 100, 1000),
        }
    }

    fn idle_snapshot() -> AgentSnapshot {
        AgentSnapshot {
            phase: AgentPhase::Idle,
            reasoning_chain: Vec::new(),
            step_count: 0,
            retry_state: initial_retry_state(3, 100, 1000),
            last_action: AgentAction::Noop,
        }
    }

    #[test]
    fn test_agent_step_idle_to_thinking() {
        let cfg = test_config();
        let snap = idle_snapshot();
        let next = agent_step(&cfg, &snap, &AgentEvent::UserMessage).unwrap();
        assert_eq!(next.phase, AgentPhase::Thinking);
        assert_eq!(next.last_action, AgentAction::SendToLlm);
        assert_eq!(next.step_count, 1);
    }

    #[test]
    fn test_agent_step_invalid_transition() {
        let cfg = test_config();
        let snap = idle_snapshot();
        let next = agent_step(&cfg, &snap, &AgentEvent::LlmResponse);
        assert!(next.is_none());
    }

    #[test]
    fn test_agent_step_exceeds_max_steps() {
        let cfg = AgentConfig {
            max_steps: 1,
            guardrails: GuardrailConfig {
                max_message_len: 4096,
                max_recursion_depth: 10,
                max_reasoning_steps: 50,
            },
            retry_config: initial_retry_state(3, 100, 1000),
        };
        let snap = AgentSnapshot {
            step_count: 1,
            ..idle_snapshot()
        };
        let next = agent_step(&cfg, &snap, &AgentEvent::UserMessage);
        assert!(next.is_none());
    }

    #[test]
    fn test_agent_run_happy_path() {
        let cfg = test_config();
        let snap = idle_snapshot();
        let events = vec![
            AgentEvent::UserMessage,   // Idle -> Thinking
            AgentEvent::LlmResponse,   // Thinking -> Composing
            AgentEvent::ComposeDone,   // Composing -> Done
        ];
        let result = agent_run(&cfg, snap, &events);
        assert_eq!(result.phase, AgentPhase::Done);
        assert_eq!(result.last_action, AgentAction::EmitResponse);
        assert_eq!(result.step_count, 3);
    }

    #[test]
    fn test_agent_run_stops_at_terminal() {
        let cfg = test_config();
        let snap = idle_snapshot();
        let events = vec![
            AgentEvent::UserMessage,
            AgentEvent::LlmResponse,
            AgentEvent::ComposeDone,
            AgentEvent::UserMessage,   // should be ignored — already Done
        ];
        let result = agent_run(&cfg, snap, &events);
        assert_eq!(result.phase, AgentPhase::Done);
        assert_eq!(result.step_count, 3);
    }

    #[test]
    fn test_agent_run_tool_call_path() {
        let cfg = test_config();
        let snap = idle_snapshot();
        let events = vec![
            AgentEvent::UserMessage,     // Idle -> Thinking
            AgentEvent::ToolCallNeeded,  // Thinking -> CallingTool
            AgentEvent::ToolResult,      // CallingTool -> AwaitingToolResult
            AgentEvent::ToolResult,      // AwaitingToolResult -> Thinking
            AgentEvent::LlmResponse,     // Thinking -> Composing
            AgentEvent::ComposeDone,     // Composing -> Done
        ];
        let result = agent_run(&cfg, snap, &events);
        assert_eq!(result.phase, AgentPhase::Done);
        assert_eq!(result.step_count, 6);
    }

    #[test]
    fn test_agent_run_cancel() {
        let cfg = test_config();
        let snap = idle_snapshot();
        let events = vec![
            AgentEvent::UserMessage,
            AgentEvent::Cancel,
        ];
        let result = agent_run(&cfg, snap, &events);
        assert_eq!(result.phase, AgentPhase::Error);
    }

    #[test]
    fn test_agent_run_empty_events() {
        let cfg = test_config();
        let snap = idle_snapshot();
        let result = agent_run(&cfg, snap, &[]);
        assert_eq!(result.phase, AgentPhase::Idle);
        assert_eq!(result.step_count, 0);
    }

    #[test]
    fn test_agent_run_bounded_by_max_steps() {
        let cfg = AgentConfig {
            max_steps: 2,
            guardrails: GuardrailConfig {
                max_message_len: 4096,
                max_recursion_depth: 10,
                max_reasoning_steps: 50,
            },
            retry_config: initial_retry_state(3, 100, 1000),
        };
        let snap = idle_snapshot();
        let events = vec![
            AgentEvent::UserMessage,   // step 1
            AgentEvent::LlmResponse,   // step 2
            AgentEvent::ComposeDone,   // would be step 3 — blocked
        ];
        let result = agent_run(&cfg, snap, &events);
        assert_eq!(result.step_count, 2);
        // Should have stopped before processing ComposeDone
        assert_eq!(result.phase, AgentPhase::Composing);
    }

    #[test]
    fn test_agent_run_invalid_event_stops() {
        let cfg = test_config();
        let snap = idle_snapshot();
        // LlmResponse is invalid when Idle
        let events = vec![AgentEvent::LlmResponse];
        let result = agent_run(&cfg, snap, &events);
        assert_eq!(result.phase, AgentPhase::Idle);
        assert_eq!(result.step_count, 0);
    }
}
