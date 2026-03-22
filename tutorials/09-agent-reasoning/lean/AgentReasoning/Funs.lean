-- AgentReasoning/Funs.lean
-- Simulated Aeneas output: function translations for the agent reasoning engine.
import AgentReasoning.Types
import Aeneas

open Primitives
open agent_reasoning

namespace agent_reasoning

/-! ## State machine -/

/-- Pure state-machine transition. Returns `none` for invalid transitions. -/
def agent_transition (phase : AgentPhase) (event : AgentEvent)
    : Option (AgentPhase × AgentAction) :=
  match phase, event with
  | .Idle, .UserMessage            => some (.Thinking, .SendToLlm)
  | .Thinking, .LlmResponse       => some (.Composing, .Noop)
  | .Thinking, .ToolCallNeeded    => some (.CallingTool, .ExecuteTool)
  | .Thinking, .ThinkingDone      => some (.Composing, .Noop)
  | .CallingTool, .ToolResult     => some (.AwaitingToolResult, .Noop)
  | .AwaitingToolResult, .ToolResult => some (.Thinking, .SendToLlm)
  | .Composing, .ComposeDone      => some (.Done, .EmitResponse)
  -- Cancel / Timeout from non-terminal states
  | .Idle, .Cancel                 => some (.Error, .LogEntry)
  | .Idle, .Timeout                => some (.Error, .LogEntry)
  | .Thinking, .Cancel             => some (.Error, .LogEntry)
  | .Thinking, .Timeout            => some (.Error, .LogEntry)
  | .CallingTool, .Cancel          => some (.Error, .LogEntry)
  | .CallingTool, .Timeout         => some (.Error, .LogEntry)
  | .AwaitingToolResult, .Cancel   => some (.Error, .LogEntry)
  | .AwaitingToolResult, .Timeout  => some (.Error, .LogEntry)
  | .Composing, .Cancel            => some (.Error, .LogEntry)
  | .Composing, .Timeout           => some (.Error, .LogEntry)
  -- Terminal states and all other combinations
  | _, _                           => none

/-- Returns `true` if the phase is terminal. -/
def is_terminal (phase : AgentPhase) : Bool :=
  match phase with
  | .Done  => true
  | .Error => true
  | _      => false

/-! ## Reasoning chain -/

/-- Map a step to its phase-order tag. -/
def chain_step_order (step : Step) : U32 :=
  match step with
  | .Observe _ => ⟨0⟩
  | .Think _   => ⟨1⟩
  | .Decide _  => ⟨2⟩
  | .Act _     => ⟨3⟩

/-- Check that a reasoning chain is well-formed (monotonically non-decreasing order). -/
def is_chain_well_formed_aux : List Step → U32 → Bool
  | [], _ => true
  | s :: rest, prev =>
    let ord := chain_step_order s
    if ord.val < prev.val then false
    else is_chain_well_formed_aux rest ord

def is_chain_well_formed (chain : List Step) : Bool :=
  match chain with
  | [] => true
  | s :: rest => is_chain_well_formed_aux rest (chain_step_order s)

/-- Append a step only if the result remains well-formed. Returns the new chain or none. -/
def append_step (chain : List Step) (step : Step) : Option (List Step) :=
  match chain.getLast? with
  | none => some [step]
  | some last =>
    if (chain_step_order step).val ≥ (chain_step_order last).val then
      some (chain ++ [step])
    else
      none

/-! ## Tool registry -/

/-- Linear search for a tool by name_id. -/
def find_tool_aux : List ToolSpec → U32 → Nat → Option Nat
  | [], _, _ => none
  | spec :: rest, name_id, idx =>
    if spec.name_id == name_id then some idx
    else find_tool_aux rest name_id (idx + 1)

def find_tool (registry : List ToolSpec) (name_id : U32) : Option Nat :=
  find_tool_aux registry name_id 0

/-- Check whether a required parameter is satisfied. -/
def param_satisfied (param : ToolParam) (args : ToolCallArgs) : Bool :=
  args.param_values.any fun (nid, kind) => nid == param.name_id && kind == param.kind

/-- Check that an argument refers to a parameter in the spec. -/
def arg_in_spec (arg_name_id : U32) (arg_kind : ParamKind) (spec : ToolSpec) : Bool :=
  spec.params.any fun p => p.name_id == arg_name_id && p.kind == arg_kind

/-- Validate a tool call against its specification. -/
def validate_tool_call (spec : ToolSpec) (args : ToolCallArgs) : Bool :=
  (spec.params.all fun p => !p.required || param_satisfied p args) &&
  (args.param_values.all fun (nid, kind) => arg_in_spec nid kind spec)

/-! ## Retry -/

/-- Create the initial retry state. -/
def initial_retry_state (max_attempts : U32) (base_delay_ms : U32) (max_delay_ms : U32)
    : RetryState :=
  { attempt := ⟨0⟩, delay_ms := base_delay_ms, max_attempts, base_delay_ms, max_delay_ms }

/-- Advance to the next retry. Returns none if max exceeded. -/
def next_retry (state : RetryState) : Option RetryState :=
  if state.attempt.val ≥ state.max_attempts.val then none
  else
    let new_attempt : U32 := ⟨state.attempt.val + 1⟩
    let doubled := state.delay_ms.val * 2
    let new_delay : U32 := ⟨min doubled state.max_delay_ms.val⟩
    some { state with attempt := new_attempt, delay_ms := new_delay }

/-- Returns true if another retry is allowed. -/
def should_retry (state : RetryState) : Bool :=
  state.attempt.val < state.max_attempts.val

/-! ## Guardrails -/

def check_message_length (message_len : U32) (config : GuardrailConfig) : Bool :=
  message_len.val ≤ config.max_message_len.val

def check_recursion_depth (depth : U32) (config : GuardrailConfig) : Bool :=
  depth.val ≤ config.max_recursion_depth.val

def check_reasoning_steps (step_count : U32) (config : GuardrailConfig) : Bool :=
  step_count.val < config.max_reasoning_steps.val

def all_guards_pass (message_len : U32) (recursion_depth : U32) (step_count : U32)
    (config : GuardrailConfig) : Bool :=
  check_message_length message_len config &&
  check_recursion_depth recursion_depth config &&
  check_reasoning_steps step_count config

/-! ## Agent step and run -/

/-- Process a single event, producing a new snapshot. -/
def agent_step (config : AgentConfig) (snapshot : AgentSnapshot) (event : AgentEvent)
    : Option AgentSnapshot :=
  if snapshot.step_count.val ≥ config.max_steps.val then none
  else
    match agent_transition snapshot.phase event with
    | none => none
    | some (next_phase, action) =>
      let new_count : U32 := ⟨snapshot.step_count.val + 1⟩
      some {
        phase := next_phase
        reasoning_chain := snapshot.reasoning_chain
        step_count := new_count
        retry_state := snapshot.retry_state
        last_action := action
      }

/-- Run the agent over a finite list of events.
    Structurally recursive on the event list — always terminates. -/
def agent_run (config : AgentConfig) (snapshot : AgentSnapshot)
    : List AgentEvent → AgentSnapshot
  | [] => snapshot
  | event :: rest =>
    if is_terminal snapshot.phase then snapshot
    else if snapshot.step_count.val ≥ config.max_steps.val then snapshot
    else
      match agent_step config snapshot event with
      | none => snapshot
      | some next => agent_run config next rest

end agent_reasoning
