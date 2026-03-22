-- AgentReasoning/Types.lean
-- Simulated Aeneas output: inductive types for the agent reasoning engine.
import Aeneas

open Primitives

namespace agent_reasoning

/-- The seven phases of the agent lifecycle. -/
inductive AgentPhase where
  | Idle
  | Thinking
  | CallingTool
  | AwaitingToolResult
  | Composing
  | Done
  | Error
deriving DecidableEq, Repr

/-- External stimuli that drive the agent forward. -/
inductive AgentEvent where
  | UserMessage
  | LlmResponse
  | ToolCallNeeded
  | ToolResult
  | Timeout
  | Cancel
  | ComposeDone
  | ThinkingDone
deriving DecidableEq, Repr

/-- Observable outputs produced by transitions. -/
inductive AgentAction where
  | SendToLlm
  | ExecuteTool
  | EmitResponse
  | LogEntry
  | Noop
deriving DecidableEq, Repr

/-- A decision produced during the Decide phase. -/
inductive Decision where
  | CallTool (tool_name_idx : U32) (args_hash : U32)
  | Respond
  | AskClarification
  | GiveUp
deriving DecidableEq, Repr

/-- One step in the chain-of-thought reasoning. -/
inductive Step where
  | Observe (id : U32)
  | Think (id : U32)
  | Decide (d : Decision)
  | Act (id : U32)
deriving DecidableEq, Repr

/-- The kind of a tool parameter. -/
inductive ParamKind where
  | StringParam
  | IntParam
  | BoolParam
deriving DecidableEq, Repr

/-- A single parameter in a tool specification. -/
structure ToolParam where
  name_id : U32
  kind : ParamKind
  required : Bool
deriving DecidableEq, Repr

/-- The specification of a tool that the agent may call. -/
structure ToolSpec where
  name_id : U32
  description_id : U32
  params : List ToolParam
deriving DecidableEq, Repr

/-- Concrete arguments supplied in a tool call. -/
structure ToolCallArgs where
  param_values : List (U32 × ParamKind)
deriving Repr

/-- Configuration of safety limits. -/
structure GuardrailConfig where
  max_message_len : U32
  max_recursion_depth : U32
  max_reasoning_steps : U32
deriving DecidableEq, Repr

/-- Retry state with exponential backoff. -/
structure RetryState where
  attempt : U32
  delay_ms : U32
  max_attempts : U32
  base_delay_ms : U32
  max_delay_ms : U32
deriving DecidableEq, Repr

/-- Static configuration for an agent run. -/
structure AgentConfig where
  max_steps : U32
  guardrails : GuardrailConfig
  retry_config : RetryState
deriving Repr

/-- A complete snapshot of the agent's state at one point in time. -/
structure AgentSnapshot where
  phase : AgentPhase
  reasoning_chain : List Step
  step_count : U32
  retry_state : RetryState
  last_action : AgentAction
deriving Repr

end agent_reasoning
