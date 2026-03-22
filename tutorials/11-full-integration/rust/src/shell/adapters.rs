// shell/adapters.rs — Type mapping between shell I/O types and core types.
//
// The verified core uses its own event/state types (AppEvent, PaneId, etc.).
// The shell uses library-specific types (crossterm::Event, ureq::Response).
// This module bridges the two.
//
// These conversions are NOT verified — they are part of the trusted shell.
// The I/O axioms in Lean assume that the adapters faithfully translate
// between the two type vocabularies.

use crate::verified_core::integration_types::AppEvent;

/// Key code constants (matching a subset of crossterm key codes).
pub const KEY_ENTER: u32 = 13;
pub const KEY_ESC: u32 = 27;
pub const KEY_BACKSPACE: u32 = 127;
pub const KEY_TAB: u32 = 9;

/// Convert a raw terminal key code and modifiers into an `AppEvent`.
///
/// In a real implementation `raw_key` comes from `crossterm::event::KeyEvent`.
///
/// Stub: performs a direct numeric mapping.
pub fn adapt_key_event(raw_key: u32, _modifiers: u32) -> AppEvent {
    match raw_key {
        KEY_ENTER => AppEvent::UserSubmitMessage,
        KEY_ESC => AppEvent::Quit,
        KEY_TAB => AppEvent::SwitchPane(1), // cycle panes (simplified)
        _ => AppEvent::KeyPress(raw_key),
    }
}

/// Convert a raw terminal resize event into an `AppEvent`.
pub fn adapt_resize_event(width: u16, height: u16) -> AppEvent {
    AppEvent::Resize(width as u32, height as u32)
}

/// Convert an HTTP response body into an `AppEvent`.
///
/// In a real implementation this parses the LLM JSON response to extract
/// the content and maps it to `LlmResponseReceived`.
///
/// Stub: always produces a response for agent 0 with content_id 0.
pub fn adapt_http_response(_body: &[u8], agent_id: u32) -> AppEvent {
    // Real implementation would parse the response JSON.
    AppEvent::LlmResponseReceived(agent_id, 0)
}

/// Convert an HTTP error into an `AppEvent`.
///
/// Stub: produces a `Tick` (error handling elided).
pub fn adapt_http_error(_error_code: u32) -> AppEvent {
    AppEvent::Tick
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_becomes_submit() {
        assert_eq!(adapt_key_event(KEY_ENTER, 0), AppEvent::UserSubmitMessage);
    }

    #[test]
    fn esc_becomes_quit() {
        assert_eq!(adapt_key_event(KEY_ESC, 0), AppEvent::Quit);
    }

    #[test]
    fn printable_becomes_keypress() {
        assert_eq!(
            adapt_key_event(b'a' as u32, 0),
            AppEvent::KeyPress(b'a' as u32)
        );
    }

    #[test]
    fn resize_adapter() {
        assert_eq!(adapt_resize_event(120, 40), AppEvent::Resize(120, 40));
    }
}
