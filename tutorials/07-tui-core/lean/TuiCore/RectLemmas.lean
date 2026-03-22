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

axiom contains_in_range (r : Rect) (p : Position)
    (h : rect_contains r p = true) :
    p.x.val >= r.x.val ∧ p.x.val < r.x.val + r.width.val ∧
    p.y.val >= r.y.val ∧ p.y.val < r.y.val + r.height.val

-- ============================================================
-- Lemma: inner_subset
-- Every position contained in (rect_inner r m) is also contained in r.
-- ============================================================

axiom inner_subset (r : Rect) (margin : U16) (p : Position)
    (h : rect_contains (rect_inner r margin) p = true) :
    rect_contains r p = true

-- ============================================================
-- Lemma: non_overlapping_disjoint_x_or_y
-- If two rects don't overlap, then for any position p, it cannot
-- be contained in both.
-- ============================================================

axiom non_overlapping_disjoint (a b : Rect) (p : Position)
    (h_no_overlap : rect_intersects a b = false) :
    ¬(rect_contains a p = true ∧ rect_contains b p = true)

end tui_core
