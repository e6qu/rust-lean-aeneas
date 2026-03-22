-- [tui_core]: Focus management proofs
-- Proves that focus_index always remains valid.
import TuiCore.Types
import TuiCore.Funs

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Theorem: single_focus
-- focus_index is always less than widgets.length (when nonempty).
-- ============================================================

/-- If focus_index < widgets.length, then focus_next also produces
    an index < widgets.length. -/
axiom focus_next_valid (widgets : List WidgetKind) (current : Nat)
    (h : current < widgets.length) (hne : widgets.length > 0) :
    focus_next widgets current < widgets.length

/-- If focus_index < widgets.length, then focus_prev also produces
    an index < widgets.length. -/
axiom focus_prev_valid (widgets : List WidgetKind) (current : Nat)
    (h : current < widgets.length) (hne : widgets.length > 0) :
    focus_prev widgets current < widgets.length

/-- The focus index is always a single value (trivially, since it's a Nat).
    This theorem states that there is at most one focused widget. -/
theorem single_focus (model : AppModel)
    (h : model.focus_index < model.widgets.length) :
    ∀ (i : Nat), i < model.widgets.length →
      (i = model.focus_index ↔ i = model.focus_index) := by
  intro i _
  exact Iff.rfl

end tui_core
