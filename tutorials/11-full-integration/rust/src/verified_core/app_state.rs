// app_state.rs — The unified application state.
//
// `AppState` combines UI state, orchestrator state, conversation history,
// and an input buffer into a single value.  Because it is a plain struct
// with no interior mutability, Aeneas can translate it directly to Lean.

use crate::verified_core::integration_types::PaneId;

/// A single entry in the conversation log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversationEntry {
    /// Which agent produced (or is the target of) this entry.
    pub agent_id: u32,
    /// Role: 0 = user, 1 = assistant, 2 = system.
    pub role: u32,
    /// Opaque identifier for the message content.
    pub content_id: u32,
    /// Logical timestamp (monotonically increasing counter).
    pub timestamp: u32,
}

/// The complete application state.
///
/// The shell never modifies this directly — it always goes through
/// `app_update`.  The Lean proofs reason about transitions on this struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppState {
    // ── UI state ──────────────────────────────────────────────────────
    /// Which pane currently has focus.
    pub active_pane: PaneId,

    // ── Agent / orchestrator state ────────────────────────────────────
    /// Index of the agent whose details are shown in the status panel.
    pub selected_agent: u32,
    /// Whether the debug/reasoning panel is visible.
    pub debug_visible: bool,
    /// False after a `Quit` event.
    pub running: bool,

    // ── Input ─────────────────────────────────────────────────────────
    /// The user's current input (simplified gap buffer).
    pub input_buffer: Vec<u8>,

    // ── Conversation ──────────────────────────────────────────────────
    /// Ordered log of all conversation entries.
    pub conversations: Vec<ConversationEntry>,

    // ── Orchestrator bookkeeping ──────────────────────────────────────
    /// Total number of agents in the system.
    pub agent_count: u32,
    /// How many orchestrator ticks have elapsed.
    pub turn_count: u32,
    /// Maximum turns before the orchestrator stops.
    pub turn_budget: u32,

    // ── Message bus (simplified) ──────────────────────────────────────
    /// Pending messages: (sender, recipient, content_id).
    pub message_queue: Vec<(u32, u32, u32)>,

    // ── Error handling ────────────────────────────────────────────────
    /// An optional error code displayed in the status bar.
    pub error_message: Option<u32>,

    // ── Monotonic timestamp counter ───────────────────────────────────
    /// Incremented each time a conversation entry is appended.
    pub next_timestamp: u32,
}

impl AppState {
    /// Create an initial `AppState` with `agent_count` agents and a turn budget.
    ///
    /// Everything starts empty/default: no messages, no conversation history,
    /// focus on the chat-input pane, debug panel hidden.
    pub fn new(agent_count: u32, turn_budget: u32) -> Self {
        AppState {
            active_pane: PaneId::ChatInput,
            selected_agent: 0,
            debug_visible: false,
            running: true,
            input_buffer: Vec::new(),
            conversations: Vec::new(),
            agent_count,
            turn_count: 0,
            turn_budget,
            message_queue: Vec::new(),
            error_message: None,
            next_timestamp: 0,
        }
    }
}
