// app_update.rs — Pure Elm-architecture update function.
//
// `app_update` takes the current state and an event and returns a new state.
// It never performs I/O.  The shell feeds events in and reads side-effect
// descriptors out of the returned state.

use crate::verified_core::app_state::{AppState, ConversationEntry};
use crate::verified_core::integration_types::{AppEvent, PaneId};
use crate::verified_core::message_bridge;

/// Apply `event` to `state`, producing a new `AppState`.
///
/// This is the single entry-point that the shell calls on every iteration
/// of the event loop.  All logic branches are pure.
pub fn app_update(state: &AppState, event: &AppEvent) -> AppState {
    match event {
        AppEvent::KeyPress(key) => handle_keypress(state, *key),
        AppEvent::Resize(_w, _h) => {
            // Resize only affects the view layer; state is unchanged.
            state.clone()
        }
        AppEvent::UserSubmitMessage => handle_submit(state),
        AppEvent::SwitchPane(pane_id) => handle_switch_pane(state, *pane_id),
        AppEvent::SwitchAgent(agent_id) => handle_switch_agent(state, *agent_id),
        AppEvent::AgentEvent(_agent_id, _code) => {
            // In the full system this would dispatch to the agent reasoning
            // engine.  Stub: state unchanged.
            state.clone()
        }
        AppEvent::OrchestratorTick => handle_orchestrator_tick(state),
        AppEvent::LlmResponseReceived(agent_id, content_id) => {
            handle_llm_response(state, *agent_id, *content_id)
        }
        AppEvent::ToolResultReceived(agent_id, _tool_id, content_id) => {
            // Treat tool results like LLM responses for conversation logging.
            handle_llm_response(state, *agent_id, *content_id)
        }
        AppEvent::Tick => handle_tick(state),
        AppEvent::Quit => {
            let mut new_state = state.clone();
            new_state.running = false;
            new_state
        }
    }
}

// ── Private handler functions ──────────────────────────────────────────────

/// Handle a key press.
///
/// If the input pane is active, the key is appended to the input buffer
/// (simplified: we treat the u32 as an ASCII byte).  Otherwise the key
/// is ignored.
fn handle_keypress(state: &AppState, key: u32) -> AppState {
    let mut new_state = state.clone();
    if state.active_pane == PaneId::ChatInput {
        // Backspace (key == 8 or 127)
        if key == 8 || key == 127 {
            new_state.input_buffer.pop();
        } else if key < 128 {
            // Printable ASCII range — append to buffer.
            new_state.input_buffer.push(key as u8);
        }
        // Keys >= 128 are special keys; ignored in this simplified model.
    }
    new_state
}

/// Handle the user pressing Enter on a non-empty input buffer.
///
/// Extracts the input, creates a conversation entry and a message-bus
/// envelope, then clears the buffer.
fn handle_submit(state: &AppState) -> AppState {
    if state.input_buffer.is_empty() {
        return state.clone();
    }

    let mut new_state = state.clone();

    // Allocate a content_id from the timestamp counter.
    let content_id = new_state.next_timestamp;

    // Build the conversation entry for the user's message.
    let entry = message_bridge::message_to_conversation_entry(
        u32::MAX, // sender = human user (u32::MAX is the reserved user id)
        content_id,
        new_state.next_timestamp,
    );
    new_state.conversations.push(entry);
    new_state.next_timestamp += 1;

    // Build a message-bus triple: user -> coordinator (agent 0).
    let (sender, recipient, cid) =
        message_bridge::user_input_to_message(content_id, 0);
    new_state.message_queue.push((sender, recipient, cid));

    // Clear the input buffer.
    new_state.input_buffer.clear();
    new_state.error_message = None;

    new_state
}

/// Switch focus to a different pane, if the id is valid.
fn handle_switch_pane(state: &AppState, pane_id: u32) -> AppState {
    if !message_bridge::is_valid_pane_id(pane_id) {
        return state.clone();
    }
    let mut new_state = state.clone();
    if let Some(pane) = PaneId::from_u32(pane_id) {
        new_state.active_pane = pane;
    }
    new_state
}

/// Switch the selected agent, if the id is in range.
fn handle_switch_agent(state: &AppState, agent_id: u32) -> AppState {
    if !message_bridge::is_valid_agent_id(agent_id, state.agent_count) {
        return state.clone();
    }
    let mut new_state = state.clone();
    new_state.selected_agent = agent_id;
    new_state
}

/// Process one orchestrator tick.
///
/// Increments the turn counter (if under budget) and delivers the next
/// message from the queue, creating a conversation entry for the delivery.
fn handle_orchestrator_tick(state: &AppState) -> AppState {
    let mut new_state = state.clone();
    if new_state.turn_count >= new_state.turn_budget {
        // Budget exhausted — no further orchestration.
        return new_state;
    }
    new_state.turn_count += 1;

    // Deliver one message from the queue.
    if let Some((sender, _recipient, content_id)) = new_state.message_queue.first().cloned() {
        new_state.message_queue.remove(0);
        let entry = ConversationEntry {
            agent_id: sender,
            role: 2, // system (delivery notification)
            content_id,
            timestamp: new_state.next_timestamp,
        };
        new_state.conversations.push(entry);
        new_state.next_timestamp += 1;
    }

    new_state
}

/// Handle an LLM response arriving for an agent.
///
/// Appends the response as a conversation entry from the given agent.
fn handle_llm_response(state: &AppState, agent_id: u32, content_id: u32) -> AppState {
    let mut new_state = state.clone();
    let entry = ConversationEntry {
        agent_id,
        role: 1, // assistant
        content_id,
        timestamp: new_state.next_timestamp,
    };
    new_state.conversations.push(entry);
    new_state.next_timestamp += 1;
    new_state
}

/// Generic tick handler (cursor blink, animations).
///
/// In this simplified model a tick does nothing to the logical state.
fn handle_tick(state: &AppState) -> AppState {
    state.clone()
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> AppState {
        AppState::new(3, 10)
    }

    // ── KeyPress ───────────────────────────────────────────────────────

    #[test]
    fn keypress_appends_to_buffer() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::KeyPress(b'h' as u32));
        assert_eq!(s2.input_buffer, vec![b'h']);
        let s3 = app_update(&s2, &AppEvent::KeyPress(b'i' as u32));
        assert_eq!(s3.input_buffer, vec![b'h', b'i']);
    }

    #[test]
    fn keypress_backspace_removes_last() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::KeyPress(b'a' as u32));
        let s3 = app_update(&s2, &AppEvent::KeyPress(127)); // backspace
        assert!(s3.input_buffer.is_empty());
    }

    #[test]
    fn keypress_ignored_when_not_input_pane() {
        let mut s = fresh_state();
        s.active_pane = PaneId::ConversationView;
        let s2 = app_update(&s, &AppEvent::KeyPress(b'x' as u32));
        assert!(s2.input_buffer.is_empty());
    }

    // ── UserSubmitMessage ──────────────────────────────────────────────

    #[test]
    fn submit_empty_is_noop() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::UserSubmitMessage);
        assert_eq!(s2.conversations.len(), 0);
        assert_eq!(s2.message_queue.len(), 0);
    }

    #[test]
    fn submit_creates_conversation_entry_and_message() {
        let mut s = fresh_state();
        s.input_buffer = vec![b'h', b'i'];
        let s2 = app_update(&s, &AppEvent::UserSubmitMessage);
        assert_eq!(s2.conversations.len(), 1);
        assert_eq!(s2.conversations[0].role, 0); // user
        assert_eq!(s2.message_queue.len(), 1);
        assert!(s2.input_buffer.is_empty());
    }

    #[test]
    fn submit_clears_error() {
        let mut s = fresh_state();
        s.input_buffer = vec![b'x'];
        s.error_message = Some(42);
        let s2 = app_update(&s, &AppEvent::UserSubmitMessage);
        assert_eq!(s2.error_message, None);
    }

    // ── SwitchPane ─────────────────────────────────────────────────────

    #[test]
    fn switch_pane_valid() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::SwitchPane(2));
        assert_eq!(s2.active_pane, PaneId::AgentStatusPanel);
    }

    #[test]
    fn switch_pane_invalid_is_noop() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::SwitchPane(99));
        assert_eq!(s2.active_pane, PaneId::ChatInput);
    }

    // ── SwitchAgent ────────────────────────────────────────────────────

    #[test]
    fn switch_agent_valid() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::SwitchAgent(2));
        assert_eq!(s2.selected_agent, 2);
    }

    #[test]
    fn switch_agent_out_of_range() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::SwitchAgent(99));
        assert_eq!(s2.selected_agent, 0);
    }

    // ── OrchestratorTick ───────────────────────────────────────────────

    #[test]
    fn orchestrator_tick_increments_turn() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::OrchestratorTick);
        assert_eq!(s2.turn_count, 1);
    }

    #[test]
    fn orchestrator_tick_delivers_message() {
        let mut s = fresh_state();
        s.message_queue.push((0, 1, 42));
        let s2 = app_update(&s, &AppEvent::OrchestratorTick);
        assert!(s2.message_queue.is_empty());
        // Delivery creates a conversation entry.
        assert_eq!(s2.conversations.len(), 1);
        assert_eq!(s2.conversations[0].content_id, 42);
    }

    #[test]
    fn orchestrator_tick_respects_budget() {
        let mut s = fresh_state();
        s.turn_count = 10; // at budget
        s.message_queue.push((0, 1, 7));
        let s2 = app_update(&s, &AppEvent::OrchestratorTick);
        // No delivery because budget exhausted.
        assert_eq!(s2.message_queue.len(), 1);
        assert_eq!(s2.turn_count, 10);
    }

    // ── LlmResponseReceived ───────────────────────────────────────────

    #[test]
    fn llm_response_appends_entry() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::LlmResponseReceived(1, 55));
        assert_eq!(s2.conversations.len(), 1);
        assert_eq!(s2.conversations[0].agent_id, 1);
        assert_eq!(s2.conversations[0].role, 1); // assistant
        assert_eq!(s2.conversations[0].content_id, 55);
    }

    // ── ToolResultReceived ─────────────────────────────────────────────

    #[test]
    fn tool_result_appends_entry() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::ToolResultReceived(2, 7, 88));
        assert_eq!(s2.conversations.len(), 1);
        assert_eq!(s2.conversations[0].content_id, 88);
    }

    // ── Tick ───────────────────────────────────────────────────────────

    #[test]
    fn tick_is_noop() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::Tick);
        assert_eq!(s, s2);
    }

    // ── Quit ───────────────────────────────────────────────────────────

    #[test]
    fn quit_sets_running_false() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::Quit);
        assert!(!s2.running);
    }

    // ── Resize ─────────────────────────────────────────────────────────

    #[test]
    fn resize_is_noop_on_state() {
        let s = fresh_state();
        let s2 = app_update(&s, &AppEvent::Resize(120, 40));
        assert_eq!(s, s2);
    }

    // ── State consistency ──────────────────────────────────────────────

    #[test]
    fn timestamps_are_monotonic() {
        let mut s = fresh_state();
        s.input_buffer = vec![b'a'];
        let s2 = app_update(&s, &AppEvent::UserSubmitMessage);
        let s3 = app_update(&s2, &AppEvent::LlmResponseReceived(1, 100));
        assert!(s3.conversations[1].timestamp > s3.conversations[0].timestamp);
    }

    #[test]
    fn running_preserved_by_non_quit_events() {
        let s = fresh_state();
        let events = vec![
            AppEvent::KeyPress(b'a' as u32),
            AppEvent::Tick,
            AppEvent::OrchestratorTick,
            AppEvent::SwitchPane(1),
            AppEvent::SwitchAgent(0),
            AppEvent::Resize(80, 24),
        ];
        for e in &events {
            let s2 = app_update(&s, e);
            assert!(s2.running, "running should be true after {:?}", e);
        }
    }
}
