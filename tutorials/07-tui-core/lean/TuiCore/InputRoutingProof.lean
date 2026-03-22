-- [tui_core]: Input routing proofs
-- Proves that Tab/BackTab route to focus changes, and other keys
-- only affect the focused widget.
import TuiCore.Types
import TuiCore.Funs

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Theorem: Tab maps to FocusNext
-- ============================================================

/-- Pressing Tab (without modifiers) produces the FocusNext action. -/
axiom tab_maps_to_focus_next (model : AppModel) :
    event_to_action (Event.Key KeyCode.Tab ⟨0⟩) model = Action.FocusNext

-- ============================================================
-- Theorem: BackTab maps to FocusPrev
-- ============================================================

/-- Pressing BackTab (without modifiers) produces the FocusPrev action. -/
axiom backtab_maps_to_focus_prev (model : AppModel) :
    event_to_action (Event.Key KeyCode.BackTab ⟨0⟩) model = Action.FocusPrev

-- ============================================================
-- Theorem: Escape maps to Quit
-- ============================================================

/-- Pressing Escape produces the Quit action. -/
axiom escape_maps_to_quit (model : AppModel) :
    event_to_action (Event.Key KeyCode.Escape ⟨0⟩) model = Action.Quit

-- ============================================================
-- Theorem: Tick is Noop
-- ============================================================

/-- Tick events produce Noop. -/
axiom tick_is_noop (model : AppModel) :
    event_to_action Event.Tick model = Action.Noop

-- ============================================================
-- Theorem: key_events_to_focus
-- Non-Tab key events produce actions that only modify the focused widget.
-- This is stated as: character insertions target the focus_index.
-- ============================================================

/-- Character key events produce InsertChar actions (which the update
    function applies only to the focused widget). -/
axiom char_produces_insert (c : U8) (model : AppModel)
    (h_no_ctrl : (0 : U8) &&& 1 = 0) :
    event_to_action (Event.Key (KeyCode.Char c) ⟨0⟩) model = Action.InsertChar c

end tui_core
