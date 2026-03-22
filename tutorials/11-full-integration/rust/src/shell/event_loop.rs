// shell/event_loop.rs — The main event loop (stub).
//
// This is the heart of the imperative shell.  On each iteration it:
//   1. Reads a terminal event (or times out to produce a Tick).
//   2. Calls the verified `app_update` to get a new state.
//   3. Calls the verified `app_view` to render the state.
//   4. Writes the view to the terminal.
//   5. Executes any pending side effects (HTTP requests, etc.).
//
// The loop itself is NOT verified.  The correctness argument is:
//   - The pure core (steps 2-3) is proved correct in Lean.
//   - The I/O layer (steps 1, 4-5) is axiomatised in IOBoundary.lean.
//   - The loop is a simple driver that connects them.

use crate::verified_core::app_state::AppState;
use crate::verified_core::app_update::app_update;
use crate::verified_core::app_view::app_view;
use crate::shell::terminal_io;
use crate::shell::http_client;

/// Default terminal width for the stub.
const DEFAULT_WIDTH: u16 = 80;
/// Default terminal height for the stub.
const DEFAULT_HEIGHT: u16 = 24;
/// Event poll timeout in milliseconds.
const TICK_TIMEOUT_MS: u64 = 100;

/// Run the main event loop.
///
/// This function takes ownership of the initial state and loops until
/// `state.running` becomes false.
///
/// Stub: runs a fixed number of iterations for testing purposes.
pub fn run(initial_state: AppState) -> AppState {
    let mut state = initial_state;
    let max_iterations = 10; // Safety limit for the stub.

    for _i in 0..max_iterations {
        if !state.running {
            break;
        }

        // Step 1: Read event from terminal.
        let event = match terminal_io::read_event(TICK_TIMEOUT_MS) {
            Some(e) => e,
            None => continue,
        };

        // Step 2: Pure update (VERIFIED).
        state = app_update(&state, &event);

        // Step 3: Pure view (VERIFIED).
        let view = app_view(&state, DEFAULT_WIDTH, DEFAULT_HEIGHT);

        // Step 4: Render to terminal (UNVERIFIED).
        terminal_io::render(&view);

        // Step 5: Execute side effects (UNVERIFIED).
        // In the real implementation we would drain `pending_side_effects`
        // and call http_client::send_llm_request for HTTP effects.
        let _ = http_client::send_llm_request("stub://endpoint", &[]);
    }

    state
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_loop_stub_terminates() {
        let state = AppState::new(2, 5);
        let final_state = run(state);
        // The stub produces Tick events which are no-ops, so state should
        // still be running (the loop ends by iteration limit).
        assert!(final_state.running);
    }
}
