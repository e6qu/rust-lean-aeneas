-- [tui_core]: Widget tree specification
-- Defines validity predicates for the arena-pattern widget tree
-- and proves structural properties.
import TuiCore.Types
import TuiCore.Funs

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Predicate: valid_widget_tree
-- All child indices in the widget arena are in range.
-- ============================================================

/-- A single widget has valid indices if all its child references
    point to existing entries in the arena. -/
def widget_indices_valid (w : WidgetKind) (arena_len : Nat) : Prop :=
  match w with
  | WidgetKind.TextBox _ _ => True
  | WidgetKind.ScrollableList _ _ _ => True
  | WidgetKind.StatusBar _ _ => True
  | WidgetKind.Border _ child_index => child_index < arena_len
  | WidgetKind.Container _ children => ∀ (idx : Nat), idx ∈ children → idx < arena_len

/-- All widgets in the arena have valid indices. -/
def valid_widget_tree (widgets : List WidgetKind) : Prop :=
  ∀ (w : WidgetKind), w ∈ widgets → widget_indices_valid w widgets.length

-- ============================================================
-- Depth computation
-- ============================================================

/-- Compute the depth of the widget tree rooted at index `root`.
    Uses a fuel parameter to ensure termination (arena pattern
    doesn't have structural recursion). -/
def widget_tree_depth (widgets : List WidgetKind) (root : Nat) (fuel : Nat) : Nat :=
  match fuel with
  | 0 => 0
  | fuel' + 1 =>
    match widgets[root]? with
    | none => 0
    | some w =>
      match w with
      | WidgetKind.TextBox _ _ => 1
      | WidgetKind.ScrollableList _ _ _ => 1
      | WidgetKind.StatusBar _ _ => 1
      | WidgetKind.Border _ child_index =>
        1 + widget_tree_depth widgets child_index fuel'
      | WidgetKind.Container _ children =>
        let rec max_child_depth (cs : List Nat) : Nat :=
          match cs with
          | [] => 0
          | c :: rest =>
            let d := widget_tree_depth widgets c fuel'
            let r := max_child_depth rest
            if d >= r then d else r
        1 + max_child_depth children

-- ============================================================
-- Theorem: valid tree with leaf widgets
-- ============================================================

/-- A TextBox is always valid regardless of arena size. -/
theorem textbox_always_valid (content : List U8) (cursor : Nat) (n : Nat) :
    widget_indices_valid (WidgetKind.TextBox content cursor) n := by
  simp [widget_indices_valid]

/-- A ScrollableList is always valid regardless of arena size. -/
theorem scrollable_list_always_valid (items : List (List U8)) (sel off : Nat) (n : Nat) :
    widget_indices_valid (WidgetKind.ScrollableList items sel off) n := by
  simp [widget_indices_valid]

/-- A StatusBar is always valid regardless of arena size. -/
theorem status_bar_always_valid (l r : List U8) (n : Nat) :
    widget_indices_valid (WidgetKind.StatusBar l r) n := by
  simp [widget_indices_valid]

/-- A Border is valid iff its child_index < arena_len. -/
theorem border_valid_iff (title : List U8) (child : Nat) (n : Nat) :
    widget_indices_valid (WidgetKind.Border title child) n ↔ child < n := by
  simp [widget_indices_valid]

end tui_core
