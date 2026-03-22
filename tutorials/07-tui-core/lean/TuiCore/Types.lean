-- [tui_core]: type definitions
-- This file simulates the Aeneas-generated type definitions for the
-- TUI core data structures.
import Aeneas

open Aeneas Primitives

namespace tui_core

/-- A rectangle in terminal coordinates. -/
structure Rect where
  x : U16
  y : U16
  width : U16
  height : U16
deriving Inhabited, BEq, DecidableEq

/-- A single position in the terminal grid. -/
structure Position where
  x : U16
  y : U16
deriving Inhabited, BEq, DecidableEq

/-- A rendered cell with position, character byte, and style byte. -/
structure Cell where
  pos : Position
  ch : U8
  style : U8
deriving Inhabited

/-- Key codes for terminal input. -/
inductive KeyCode where
  | Char (c : U8)
  | Enter
  | Backspace
  | Tab
  | BackTab
  | Escape
  | Left
  | Right
  | Up
  | Down
  | Home
  | End
  | Delete
  | PageUp
  | PageDown
deriving Inhabited, BEq, DecidableEq

/-- Modifier keys as a bitflag byte. -/
structure Modifiers where
  bits : U8
deriving Inhabited, BEq, DecidableEq

/-- Input events with no I/O dependency. -/
inductive Event where
  | Key (code : KeyCode) (mods : Modifiers)
  | Resize (w : U16) (h : U16)
  | Tick
deriving Inhabited

/-- Direction for splitting a rectangle. -/
inductive SplitDir where
  | Horizontal
  | Vertical
deriving Inhabited, BEq, DecidableEq

/-- Arena-pattern widget kinds. Children are referenced by index (Nat)
    into a flat list of widgets. -/
inductive WidgetKind where
  | TextBox (content : List U8) (cursor : Nat)
  | ScrollableList (items : List (List U8)) (selected : Nat) (scroll_offset : Nat)
  | StatusBar (left_text : List U8) (right_text : List U8)
  | Border (title : List U8) (child_index : Nat)
  | Container (dir : SplitDir) (children : List Nat)
deriving Inhabited

/-- Semantic actions produced by event-to-action mapping. -/
inductive Action where
  | Noop
  | InsertChar (c : U8)
  | DeleteChar
  | MoveCursor (delta : I32)
  | ScrollUp
  | ScrollDown
  | FocusNext
  | FocusPrev
  | Submit
  | Quit
  | Redraw
deriving Inhabited, BEq, DecidableEq

/-- The entire UI state as a flat arena. -/
structure AppModel where
  widgets : List WidgetKind
  focus_index : Nat
  areas : List Rect
  screen_width : U16
  screen_height : U16
deriving Inhabited

end tui_core
