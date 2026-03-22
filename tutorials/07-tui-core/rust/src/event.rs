//! Event types: KeyCode, Modifiers, Event.

/// Key codes for terminal input.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyCode {
    Char(u8),
    Enter,
    Backspace,
    Tab,
    BackTab,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Delete,
    PageUp,
    PageDown,
}

/// Modifier keys as a bitflag byte.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Modifiers {
    pub bits: u8,
}

impl Modifiers {
    pub const NONE: u8 = 0;
    pub const CTRL: u8 = 1;
    pub const ALT: u8 = 2;
    pub const SHIFT: u8 = 4;

    pub fn none() -> Self {
        Modifiers { bits: Self::NONE }
    }

    pub fn ctrl(&self) -> bool {
        (self.bits & Self::CTRL) != 0
    }

    pub fn alt(&self) -> bool {
        (self.bits & Self::ALT) != 0
    }

    pub fn shift(&self) -> bool {
        (self.bits & Self::SHIFT) != 0
    }
}

/// Input events with no I/O dependency.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Event {
    Key(KeyCode, Modifiers),
    Resize(u16, u16),
    Tick,
}
