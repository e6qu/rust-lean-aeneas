// shell/main.rs — Application entry point (stub).
//
// In a real build this would be the binary target.  It initialises the
// terminal, creates the initial AppState, and enters the event loop.
//
// This file is NOT verified by Aeneas.  It exists purely to document the
// intended startup sequence.

// NOTE: This is a stub.  To actually run, you would:
//   1. Add crossterm and ureq to [dependencies] in Cargo.toml.
//   2. Enable raw-mode, alternate screen.
//   3. Call `event_loop::run(state)`.
//
// Pseudo-code:
//
//   fn main() -> Result<(), Box<dyn std::error::Error>> {
//       // Parse configuration.
//       let agent_count = 3;
//       let turn_budget = 100;
//
//       // Enter raw mode.
//       crossterm::terminal::enable_raw_mode()?;
//       let mut stdout = std::io::stdout();
//       crossterm::execute!(stdout, EnterAlternateScreen)?;
//
//       // Create initial state.
//       let state = AppState::new(agent_count, turn_budget);
//
//       // Run the event loop (pure core + I/O shell).
//       event_loop::run(state, &mut stdout)?;
//
//       // Restore terminal.
//       crossterm::execute!(stdout, LeaveAlternateScreen)?;
//       crossterm::terminal::disable_raw_mode()?;
//       Ok(())
//   }

/// Stub entry point for documentation purposes.
pub fn main_stub() {
    // This function intentionally does nothing.
    // See the pseudo-code above for the real implementation.
}
