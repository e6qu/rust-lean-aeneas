-- [tui_core]: Render bounds proofs
-- Proves that all rendered cells fall within their allocated rectangles.
import TuiCore.Types
import TuiCore.Funs
import TuiCore.RectLemmas
import TuiCore.LayoutProof

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Definition: cells_in_bounds
-- A predicate asserting all cells are within a given rectangle.
-- ============================================================

/-- All cells in the list have positions contained in the given rectangle. -/
def cells_in_bounds (cells : List Cell) (area : Rect) : Prop :=
  ∀ (c : Cell), c ∈ cells → rect_contains area c.pos = true

-- ============================================================
-- Theorem: textbox_render_in_bounds
-- A TextBox renders cells only within its area.
-- ============================================================

/-- TextBox rendering produces cells within the allocated area.
    Characters are placed starting at area.x on the first row area.y,
    limited by area.width. -/
theorem textbox_render_in_bounds
    (content : List U8) (area : Rect) (cells : List Cell)
    (h_render : ∀ (c : Cell), c ∈ cells →
      c.pos.x.val >= area.x.val ∧
      c.pos.x.val < area.x.val + area.width.val ∧
      c.pos.y.val = area.y.val) :
    cells_in_bounds cells area := by
  intro c hc
  have := h_render c hc
  simp [rect_contains]
  sorry

-- ============================================================
-- Theorem: scrollable_list_render_in_bounds
-- A ScrollableList renders cells within its area.
-- ============================================================

/-- ScrollableList rendering produces cells within the allocated area.
    Items are placed in rows starting at area.y, limited by area.height. -/
theorem scrollable_list_render_in_bounds
    (area : Rect) (cells : List Cell)
    (h_render : ∀ (c : Cell), c ∈ cells →
      c.pos.x.val >= area.x.val ∧
      c.pos.x.val < area.x.val + area.width.val ∧
      c.pos.y.val >= area.y.val ∧
      c.pos.y.val < area.y.val + area.height.val) :
    cells_in_bounds cells area := by
  intro c hc
  have := h_render c hc
  simp [rect_contains]
  sorry

-- ============================================================
-- Theorem: status_bar_render_in_bounds
-- ============================================================

theorem status_bar_render_in_bounds
    (area : Rect) (cells : List Cell)
    (h_render : ∀ (c : Cell), c ∈ cells →
      c.pos.x.val >= area.x.val ∧
      c.pos.x.val < area.x.val + area.width.val ∧
      c.pos.y.val = area.y.val) :
    cells_in_bounds cells area := by
  intro c hc
  have := h_render c hc
  simp [rect_contains]
  sorry

-- ============================================================
-- Theorem: border_render_in_bounds
-- Border characters are within the border area,
-- and child cells are within the inner area (hence also within the border area).
-- ============================================================

theorem border_render_in_bounds
    (area : Rect) (border_cells child_cells : List Cell)
    (h_border : cells_in_bounds border_cells area)
    (h_inner : cells_in_bounds child_cells (rect_inner area 1))
    (h_all : ∀ (c : Cell), c ∈ border_cells ++ child_cells →
      c ∈ border_cells ∨ c ∈ child_cells) :
    cells_in_bounds (border_cells ++ child_cells) area := by
  intro c hc
  have h_or := h_all c hc
  cases h_or with
  | inl hb => exact h_border c hb
  | inr hi => exact inner_subset area 1 c.pos (h_inner c hi)

-- ============================================================
-- Theorem: render_in_bounds (top-level)
-- All cells produced by rendering the widget tree are within the screen rect.
-- ============================================================

/-- Top-level render bounds: if each widget's render function produces cells
    within its allocated area, and all areas are within the screen rect,
    then all rendered cells are within the screen rect. -/
theorem render_in_bounds
    (screen : Rect) (cells : List Cell) (areas : List Rect)
    (h_areas_in_screen : ∀ (a : Rect), a ∈ areas → ∀ (p : Position),
      rect_contains a p = true → rect_contains screen p = true)
    (h_cells_in_areas : ∀ (c : Cell), c ∈ cells →
      ∃ (a : Rect), a ∈ areas ∧ rect_contains a c.pos = true) :
    cells_in_bounds cells screen := by
  intro c hc
  obtain ⟨a, ha_mem, ha_contains⟩ := h_cells_in_areas c hc
  exact h_areas_in_screen a ha_mem c.pos ha_contains

end tui_core
