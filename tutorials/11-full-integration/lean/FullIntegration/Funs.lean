-- FullIntegration/Funs.lean
-- Lean translation of the verified core functions.
--
-- In a real Aeneas pipeline this file is auto-generated.  Here we write
-- it by hand to match the Rust implementations.

import FullIntegration.Types

namespace FullIntegration

/- ── Message bridge functions ─────────────────────────────────────────── -/

/-- Build a message-bus triple from a user submission. -/
def user_input_to_message (content_id : UInt32) (coordinator_id : UInt32)
    : UInt32 × UInt32 × UInt32 :=
  (UInt32.ofNat 4294967295, coordinator_id, content_id)  -- u32::MAX

/-- Build a ConversationEntry from raw fields. -/
def message_to_conversation_entry (sender : UInt32) (content_id : UInt32)
    (timestamp : UInt32) : ConversationEntry :=
  let role := if sender == UInt32.ofNat 4294967295 then 0 else 1
  { agent_id := sender, role := role, content_id := content_id, timestamp := timestamp }

/-- Check whether a numeric pane id is valid. -/
def is_valid_pane_id (id : UInt32) : Bool :=
  id < PANE_COUNT

/-- Check whether an agent id is within range. -/
def is_valid_agent_id (id : UInt32) (agent_count : UInt32) : Bool :=
  id < agent_count

/- ── AppState constructor ─────────────────────────────────────────────── -/

/-- Create an initial AppState. -/
def AppState.new (agent_count : UInt32) (turn_budget : UInt32) : AppState :=
  { active_pane    := .ChatInput
    selected_agent := 0
    debug_visible  := false
    running        := true
    input_buffer   := []
    conversations  := []
    agent_count    := agent_count
    turn_count     := 0
    turn_budget    := turn_budget
    message_queue  := []
    error_message  := none
    next_timestamp := 0 }

/- ── Handler functions ────────────────────────────────────────────────── -/

/-- Handle a key press on the input pane. -/
def handle_keypress (state : AppState) (key : UInt32) : AppState :=
  if state.active_pane != .ChatInput then state
  else if key == 8 || key == 127 then
    { state with input_buffer := state.input_buffer.dropLast }
  else if key < 128 then
    { state with input_buffer := state.input_buffer ++ [UInt8.ofNat key.toNat] }
  else state

/-- Handle user submit. -/
def handle_submit (state : AppState) : AppState :=
  if state.input_buffer.isEmpty then state
  else
    let content_id := state.next_timestamp
    let entry := message_to_conversation_entry 0 content_id state.next_timestamp
    let msg := user_input_to_message content_id 0
    { state with
      conversations  := state.conversations ++ [entry]
      next_timestamp := state.next_timestamp + 1
      message_queue  := state.message_queue ++ [msg]
      input_buffer   := []
      error_message  := none }

/-- Handle pane switch. -/
def handle_switch_pane (state : AppState) (pane_id : UInt32) : AppState :=
  if ¬(is_valid_pane_id pane_id) then state
  else match PaneId.fromUInt32 pane_id with
    | some p => { state with active_pane := p }
    | none   => state

/-- Handle agent switch. -/
def handle_switch_agent (state : AppState) (agent_id : UInt32) : AppState :=
  if ¬(is_valid_agent_id agent_id state.agent_count) then state
  else { state with selected_agent := agent_id }

/-- Handle orchestrator tick. -/
def handle_orchestrator_tick (state : AppState) : AppState :=
  if state.turn_count >= state.turn_budget then state
  else
    let state := { state with turn_count := state.turn_count + 1 }
    match state.message_queue with
    | [] => state
    | (sender, _, content_id) :: rest =>
      let entry : ConversationEntry :=
        { agent_id := sender, role := 2, content_id := content_id,
          timestamp := state.next_timestamp }
      { state with
        message_queue  := rest
        conversations  := state.conversations ++ [entry]
        next_timestamp := state.next_timestamp + 1 }

/-- Handle LLM response. -/
def handle_llm_response (state : AppState) (agent_id : UInt32)
    (content_id : UInt32) : AppState :=
  let entry : ConversationEntry :=
    { agent_id := agent_id, role := 1, content_id := content_id,
      timestamp := state.next_timestamp }
  { state with
    conversations  := state.conversations ++ [entry]
    next_timestamp := state.next_timestamp + 1 }

/-- Handle generic tick (no-op). -/
def handle_tick (state : AppState) : AppState := state

/- ── Main update function ─────────────────────────────────────────────── -/

/-- Pure Elm-architecture update: dispatch on event type. -/
def app_update (state : AppState) (event : AppEvent) : AppState :=
  match event with
  | .KeyPress key            => handle_keypress state key
  | .Resize _ _              => state
  | .UserSubmitMessage       => handle_submit state
  | .SwitchPane pid          => handle_switch_pane state pid
  | .SwitchAgent aid         => handle_switch_agent state aid
  | .AgentEvent _ _          => state
  | .OrchestratorTick        => handle_orchestrator_tick state
  | .LlmResponseReceived a c => handle_llm_response state a c
  | .ToolResultReceived a _ c => handle_llm_response state a c
  | .Tick                    => handle_tick state
  | .Quit                    => { state with running := false }

end FullIntegration
