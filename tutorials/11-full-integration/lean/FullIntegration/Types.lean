-- FullIntegration/Types.lean
-- Lean translation of the verified core types.
--
-- In a real Aeneas pipeline this file is auto-generated from the Rust source.
-- Here we write it by hand to match the Rust definitions exactly.

namespace FullIntegration

/-- All external stimuli normalised into a single event type. -/
inductive AppEvent where
  | KeyPress    : UInt32 → AppEvent
  | Resize      : UInt32 → UInt32 → AppEvent
  | UserSubmitMessage : AppEvent
  | SwitchPane  : UInt32 → AppEvent
  | SwitchAgent : UInt32 → AppEvent
  | AgentEvent  : UInt32 → UInt32 → AppEvent
  | OrchestratorTick : AppEvent
  | LlmResponseReceived : UInt32 → UInt32 → AppEvent
  | ToolResultReceived : UInt32 → UInt32 → UInt32 → AppEvent
  | Tick : AppEvent
  | Quit : AppEvent
  deriving Repr, BEq, DecidableEq

/-- Identifies one of the four application panes. -/
inductive PaneId where
  | ChatInput
  | ConversationView
  | AgentStatusPanel
  | DebugReasoningPanel
  deriving Repr, BEq, DecidableEq

/-- Convert a numeric id to a PaneId. -/
def PaneId.fromUInt32 (id : UInt32) : Option PaneId :=
  if id == 0 then some .ChatInput
  else if id == 1 then some .ConversationView
  else if id == 2 then some .AgentStatusPanel
  else if id == 3 then some .DebugReasoningPanel
  else none

/-- Convert a PaneId to a numeric id. -/
def PaneId.toUInt32 : PaneId → UInt32
  | .ChatInput           => 0
  | .ConversationView    => 1
  | .AgentStatusPanel    => 2
  | .DebugReasoningPanel => 3

/-- A single entry in the conversation log. -/
structure ConversationEntry where
  agent_id  : UInt32
  role      : UInt32
  content_id : UInt32
  timestamp : UInt32
  deriving Repr, BEq, DecidableEq

/-- The complete application state. -/
structure AppState where
  active_pane    : PaneId
  selected_agent : UInt32
  debug_visible  : Bool
  running        : Bool
  input_buffer   : List UInt8
  conversations  : List ConversationEntry
  agent_count    : UInt32
  turn_count     : UInt32
  turn_budget    : UInt32
  message_queue  : List (UInt32 × UInt32 × UInt32)
  error_message  : Option UInt32
  next_timestamp : UInt32
  deriving Repr, BEq

/-- A single character cell in the rendered output. -/
structure ViewCell where
  x  : UInt16
  y  : UInt16
  ch : UInt8
  deriving Repr, BEq

/-- The complete rendered view. -/
structure ViewTree where
  cells : List ViewCell
  deriving Repr, BEq

/-- Number of defined PaneId variants. -/
def PANE_COUNT : UInt32 := 4

end FullIntegration
