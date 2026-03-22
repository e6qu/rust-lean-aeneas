// shell/terminal_io.rs — Terminal I/O via crossterm (stub).
//
// In a real build this module wraps crossterm to:
//   - Read keyboard/mouse/resize events from the terminal.
//   - Write rendered cells (from ViewTree) to the terminal.
//
// This is part of the UNVERIFIED shell.  The Lean proofs axiomatise its
// behaviour in IOBoundary.lean.

use crate::verified_core::app_view::ViewTree;
use crate::verified_core::integration_types::AppEvent;

/// Read one event from the terminal.
///
/// In a real implementation this calls `crossterm::event::read()` with a
/// timeout and converts the raw event into an `AppEvent` via `adapters`.
///
/// Stub: always returns `Tick`.
pub fn read_event(_timeout_ms: u64) -> Option<AppEvent> {
    // Real implementation:
    //   if crossterm::event::poll(Duration::from_millis(timeout_ms))? {
    //       let raw = crossterm::event::read()?;
    //       Ok(adapters::adapt_terminal_event(raw))
    //   } else {
    //       Ok(Some(AppEvent::Tick))
    //   }
    Some(AppEvent::Tick)
}

/// Render a `ViewTree` to the terminal.
///
/// Iterates over cells and issues `crossterm::cursor::MoveTo` +
/// `crossterm::style::Print` for each one.
///
/// Stub: does nothing.
pub fn render(_view: &ViewTree) {
    // Real implementation:
    //   for cell in &view.cells {
    //       execute!(stdout, MoveTo(cell.x, cell.y), Print(cell.ch as char))?;
    //   }
    //   stdout.flush()?;
}

/// Enter raw mode and alternate screen.
///
/// Stub: does nothing.
pub fn enter_raw_mode() {
    // crossterm::terminal::enable_raw_mode().unwrap();
    // crossterm::execute!(stdout, EnterAlternateScreen).unwrap();
}

/// Leave raw mode and alternate screen.
///
/// Stub: does nothing.
pub fn leave_raw_mode() {
    // crossterm::execute!(stdout, LeaveAlternateScreen).unwrap();
    // crossterm::terminal::disable_raw_mode().unwrap();
}
