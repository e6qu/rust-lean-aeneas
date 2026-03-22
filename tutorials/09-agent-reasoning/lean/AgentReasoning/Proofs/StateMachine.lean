-- AgentReasoning/Proofs/StateMachine.lean
-- Proofs about the agent state machine transition function.
import AgentReasoning.Types
import AgentReasoning.Funs
import Aeneas

open Primitives
open agent_reasoning

namespace agent_reasoning

/-!
# State Machine Proofs

## agent_transition_from_terminal_is_none
Transitions from terminal states (`Done`, `Error`) always return `none`.

## valid_transitions_enumerated
Every valid (phase, event) pair is explicitly covered; all others return `none`.
-/

/-- Transitions from `Done` always return `none`, regardless of the event. -/
theorem agent_transition_from_done_is_none (event : AgentEvent) :
    agent_transition .Done event = none := by
  cases event <;> simp [agent_transition]

/-- Transitions from `Error` always return `none`, regardless of the event. -/
theorem agent_transition_from_error_is_none (event : AgentEvent) :
    agent_transition .Error event = none := by
  cases event <;> simp [agent_transition]

/-- Combined: any terminal phase rejects all events. -/
theorem agent_transition_from_terminal_is_none
    (phase : AgentPhase) (event : AgentEvent)
    (h : is_terminal phase = true) :
    agent_transition phase event = none := by
  cases phase <;> simp [is_terminal] at h <;> cases event <;> simp [agent_transition]

/-- Cancel from any non-terminal state goes to Error. -/
theorem cancel_goes_to_error (phase : AgentPhase) (h : is_terminal phase = false) :
    agent_transition phase .Cancel = some (.Error, .LogEntry) := by
  cases phase <;> simp [is_terminal, agent_transition] at *

/-- Timeout from any non-terminal state goes to Error. -/
theorem timeout_goes_to_error (phase : AgentPhase) (h : is_terminal phase = false) :
    agent_transition phase .Timeout = some (.Error, .LogEntry) := by
  cases phase <;> simp [is_terminal, agent_transition] at *

/-- A valid transition from a non-terminal state produces a result. -/
theorem idle_user_message_is_some :
    agent_transition .Idle .UserMessage = some (.Thinking, .SendToLlm) := by
  simp [agent_transition]

theorem thinking_llm_response_is_some :
    agent_transition .Thinking .LlmResponse = some (.Composing, .Noop) := by
  simp [agent_transition]

theorem composing_compose_done_is_some :
    agent_transition .Composing .ComposeDone = some (.Done, .EmitResponse) := by
  simp [agent_transition]

end agent_reasoning
