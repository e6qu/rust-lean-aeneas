// integration_types.rs — Central event and pane types for the full application.
//
// `AppEvent` normalises every external stimulus (key presses, network responses,
// timer ticks) into a single algebraic type.  The pure `app_update` function
// pattern-matches on `AppEvent` to produce a new `AppState`.

/// All external stimuli that the application can receive.
///
/// The imperative shell converts raw I/O (crossterm events, HTTP responses)
/// into `AppEvent` values before handing them to the verified core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEvent {
    /// A key press.  The `u32` is a simplified key code (ASCII value or
    /// special-key sentinel).
    KeyPress(u32),

    /// Terminal window resized to (width, height).
    Resize(u32, u32),

    /// User pressed Enter on a non-empty input buffer.
    UserSubmitMessage,

    /// Switch focus to pane `id`.
    SwitchPane(u32),

    /// Switch the active agent view.
    SwitchAgent(u32),

    /// An event originating from a specific agent.
    /// (agent_id, event_code)
    AgentEvent(u32, u32),

    /// The orchestrator's periodic heartbeat.
    OrchestratorTick,

    /// An LLM response arrived for an agent.
    /// (agent_id, content_id)
    LlmResponseReceived(u32, u32),

    /// A tool call completed.
    /// (agent_id, tool_id, result_content_id)
    ToolResultReceived(u32, u32, u32),

    /// Generic timer tick (for animations, cursor blink, etc.).
    Tick,

    /// Graceful shutdown.
    Quit,
}

/// Identifies one of the four application panes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PaneId {
    /// The text-input area where the user types messages.
    ChatInput,
    /// The scrollable conversation history.
    ConversationView,
    /// The panel showing agent status / phases.
    AgentStatusPanel,
    /// The debug / reasoning-trace panel.
    DebugReasoningPanel,
}

impl PaneId {
    /// Convert a numeric id to a `PaneId`, returning `None` for out-of-range
    /// values.  This is used by `SwitchPane` event handling.
    pub fn from_u32(id: u32) -> Option<PaneId> {
        match id {
            0 => Some(PaneId::ChatInput),
            1 => Some(PaneId::ConversationView),
            2 => Some(PaneId::AgentStatusPanel),
            3 => Some(PaneId::DebugReasoningPanel),
            _ => None,
        }
    }

    /// Convert back to a numeric id.
    pub fn to_u32(self) -> u32 {
        match self {
            PaneId::ChatInput => 0,
            PaneId::ConversationView => 1,
            PaneId::AgentStatusPanel => 2,
            PaneId::DebugReasoningPanel => 3,
        }
    }
}
