//! Semantic actions produced by event-to-action mapping.

/// Actions that the app model can process.
#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    Noop,
    InsertChar(u8),
    DeleteChar,
    MoveCursor(i32),
    ScrollUp,
    ScrollDown,
    FocusNext,
    FocusPrev,
    Submit,
    Quit,
    Redraw,
}
