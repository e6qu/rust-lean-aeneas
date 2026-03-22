-- [tui_core]: function definitions
-- This file simulates the Aeneas-generated function translations.
import TuiCore.Types

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Geometry functions
-- ============================================================

/-- Check whether a position lies within a rectangle. -/
def rect_contains (r : Rect) (p : Position) : Bool :=
  p.x.val >= r.x.val && p.x.val < r.x.val + r.width.val &&
  p.y.val >= r.y.val && p.y.val < r.y.val + r.height.val

/-- Total area of a rectangle as a natural number. -/
def rect_area (r : Rect) : Nat :=
  r.width.val * r.height.val

/-- Check whether two rectangles overlap. -/
def rect_intersects (a b : Rect) : Bool :=
  a.x.val < b.x.val + b.width.val && b.x.val < a.x.val + a.width.val &&
  a.y.val < b.y.val + b.height.val && b.y.val < a.y.val + a.height.val

/-- Shrink a rectangle inward by a margin on each side. -/
def rect_inner (r : Rect) (margin : U16) : Rect :=
  let double := margin.val * 2
  if r.width.val <= double || r.height.val <= double then
    { x := r.x, y := r.y, width := 0, height := 0 }
  else
    { x := ⟨r.x.val + margin.val⟩,
      y := ⟨r.y.val + margin.val⟩,
      width := ⟨r.width.val - double⟩,
      height := ⟨r.height.val - double⟩ }

-- ============================================================
-- Layout functions
-- ============================================================

/-- Split a rectangle into `count` equal parts along `dir`.
    The last piece receives the remainder. -/
def split (r : Rect) (dir : SplitDir) (count : Nat) : List Rect :=
  if count = 0 then []
  else
    let total := match dir with
      | SplitDir.Horizontal => r.width.val
      | SplitDir.Vertical => r.height.val
    let piece_size := total / count
    let rec go (i : Nat) (offset : Nat) : List Rect :=
      if i >= count then []
      else
        let size := if i = count - 1 then total - offset else piece_size
        let rect := match dir with
          | SplitDir.Horizontal =>
            { x := ⟨r.x.val + offset⟩,
              y := r.y,
              width := ⟨size⟩,
              height := r.height }
          | SplitDir.Vertical =>
            { x := r.x,
              y := ⟨r.y.val + offset⟩,
              width := r.width,
              height := ⟨size⟩ }
        rect :: go (i + 1) (offset + piece_size)
    termination_by count - i
    go 0 0

/-- Split a rectangle into two pieces at a given offset. -/
def split_at (r : Rect) (dir : SplitDir) (offset : U16) : Rect × Rect :=
  match dir with
  | SplitDir.Horizontal =>
    let clamped := if offset.val > r.width.val then r.width else offset
    let left := { x := r.x, y := r.y, width := clamped, height := r.height }
    let right : Rect :=
      { x := ⟨r.x.val + clamped.val⟩, y := r.y,
        width := ⟨r.width.val - clamped.val⟩, height := r.height }
    (left, right)
  | SplitDir.Vertical =>
    let clamped := if offset.val > r.height.val then r.height else offset
    let top := { x := r.x, y := r.y, width := r.width, height := clamped }
    let bottom : Rect :=
      { x := r.x, y := ⟨r.y.val + clamped.val⟩,
        width := r.width, height := ⟨r.height.val - clamped.val⟩ }
    (top, bottom)

-- ============================================================
-- Scroll clamp
-- ============================================================

/-- Clamp scroll offset to a valid range. -/
def scroll_clamp (offset : Nat) (item_count : Nat) (visible : Nat) : Nat :=
  if item_count <= visible then 0
  else
    let max_offset := item_count - visible
    if offset > max_offset then max_offset else offset

-- ============================================================
-- Event to action mapping
-- ============================================================

/-- Map a raw event to a semantic action. -/
def event_to_action (event : Event) (_model : AppModel) : Action :=
  match event with
  | Event.Key keycode mods =>
    if mods.bits &&& 1 != 0 then  -- CTRL
      match keycode with
      | KeyCode.Char 99 => Action.Quit   -- 'c'
      | KeyCode.Char 113 => Action.Quit  -- 'q'
      | _ => Action.Noop
    else
      match keycode with
      | KeyCode.Tab => Action.FocusNext
      | KeyCode.BackTab => Action.FocusPrev
      | KeyCode.Enter => Action.Submit
      | KeyCode.Char c => Action.InsertChar c
      | KeyCode.Backspace => Action.DeleteChar
      | KeyCode.Delete => Action.DeleteChar
      | KeyCode.Left => Action.MoveCursor (-1)
      | KeyCode.Right => Action.MoveCursor 1
      | KeyCode.Up => Action.ScrollUp
      | KeyCode.Down => Action.ScrollDown
      | KeyCode.Escape => Action.Quit
      | _ => Action.Noop
  | Event.Resize _ _ => Action.Redraw
  | Event.Tick => Action.Noop

-- ============================================================
-- Focus management
-- ============================================================

/-- Check if a widget is focusable. -/
def is_focusable (kind : WidgetKind) : Bool :=
  match kind with
  | WidgetKind.TextBox _ _ => true
  | WidgetKind.ScrollableList _ _ _ => true
  | _ => false

/-- Find the next focusable widget index, wrapping around. -/
def focus_next (widgets : List WidgetKind) (current : Nat) : Nat :=
  let len := widgets.length
  if len = 0 then current
  else
    let rec go (i : Nat) : Nat :=
      if i >= len then current
      else
        let idx := (current + 1 + i) % len
        match widgets[idx]? with
        | some w => if is_focusable w then idx else go (i + 1)
        | none => current
    termination_by len - i
    go 0

/-- Find the previous focusable widget index, wrapping around. -/
def focus_prev (widgets : List WidgetKind) (current : Nat) : Nat :=
  let len := widgets.length
  if len = 0 then current
  else
    let rec go (i : Nat) : Nat :=
      if i >= len then current
      else
        let idx := (current + len - 1 - i) % len
        match widgets[idx]? with
        | some w => if is_focusable w then idx else go (i + 1)
        | none => current
    termination_by len - i
    go 0

end tui_core
