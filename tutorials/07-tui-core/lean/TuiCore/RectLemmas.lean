-- [tui_core]: Rectangle arithmetic lemmas
-- Proofs about containment, inner subsets, and non-overlapping rectangles.
import TuiCore.Types
import TuiCore.Funs

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Lemma: contains_in_range
-- If rect_contains r p = true, then p.x and p.y are within r's bounds.
-- ============================================================

theorem contains_in_range (r : Rect) (p : Position)
    (h : rect_contains r p = true) :
    p.x.val >= r.x.val ∧ p.x.val < r.x.val + r.width.val ∧
    p.y.val >= r.y.val ∧ p.y.val < r.y.val + r.height.val := by
  simp [rect_contains] at h
  exact h

-- ============================================================
-- Lemma: inner_subset
-- Every position contained in (rect_inner r m) is also contained in r.
-- ============================================================

theorem inner_subset (r : Rect) (margin : U16) (p : Position)
    (h : rect_contains (rect_inner r margin) p = true) :
    rect_contains r p = true := by
  simp [rect_inner] at h
  split at h
  · -- width <= double or height <= double: inner has width=0 or height=0
    simp [rect_contains] at h
  · -- Normal case: inner is strictly inside r
    simp [rect_contains] at h ⊢
    omega

-- ============================================================
-- Lemma: non_overlapping_disjoint_x_or_y
-- If two rects don't overlap, then for any position p, it cannot
-- be contained in both.
-- ============================================================

theorem non_overlapping_disjoint (a b : Rect) (p : Position)
    (h_no_overlap : rect_intersects a b = false) :
    ¬(rect_contains a p = true ∧ rect_contains b p = true) := by
  simp [rect_intersects] at h_no_overlap
  simp [rect_contains]
  omega

end tui_core
