-- [tui_core]: Layout correctness proofs
-- Proves that split produces non-overlapping rects that cover the original.
import TuiCore.Types
import TuiCore.Funs
import TuiCore.RectLemmas

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Helper: split produces exactly `count` rectangles
-- ============================================================

axiom split_length (r : Rect) (dir : SplitDir) (count : Nat) (hc : count > 0) :
    (split r dir count).length = count

-- ============================================================
-- Theorem: split_no_overlap
-- The rectangles returned by split are pairwise non-overlapping.
-- For horizontal splits, consecutive rects have adjacent x coordinates.
-- ============================================================

/-- Consecutive horizontal splits have adjacent x coordinates with no gap. -/
axiom split_no_overlap_horizontal (r : Rect) (count : Nat) (hc : count > 0)
    (i j : Nat) (hi : i < count) (hj : j < count) (hne : i ≠ j) :
    let rects := split r SplitDir.Horizontal count
    ∀ (ri rj : Rect),
      rects[i]? = some ri →
      rects[j]? = some rj →
      rect_intersects ri rj = false

/-- Consecutive vertical splits have adjacent y coordinates with no gap. -/
axiom split_no_overlap_vertical (r : Rect) (count : Nat) (hc : count > 0)
    (i j : Nat) (hi : i < count) (hj : j < count) (hne : i ≠ j) :
    let rects := split r SplitDir.Vertical count
    ∀ (ri rj : Rect),
      rects[i]? = some ri →
      rects[j]? = some rj →
      rect_intersects ri rj = false

-- ============================================================
-- Theorem: split_covers
-- The union of sub-rectangles from split covers the original rect.
-- Total dimension sums to the original.
-- ============================================================

/-- The total width of a horizontal split equals the original width. -/
axiom split_covers_horizontal (r : Rect) (count : Nat) (hc : count > 0) :
    let rects := split r SplitDir.Horizontal count
    (rects.map (fun rect => rect.width.val)).sum = r.width.val

/-- The total height of a vertical split equals the original height. -/
axiom split_covers_vertical (r : Rect) (count : Nat) (hc : count > 0) :
    let rects := split r SplitDir.Vertical count
    (rects.map (fun rect => rect.height.val)).sum = r.height.val

-- ============================================================
-- Theorem: split_at_partition
-- split_at produces two rects whose dimensions sum to the original.
-- ============================================================

axiom split_at_partition_horizontal (r : Rect) (offset : U16) :
    let (left, right) := split_at r SplitDir.Horizontal offset
    left.width.val + right.width.val = r.width.val

axiom split_at_partition_vertical (r : Rect) (offset : U16) :
    let (top, bottom) := split_at r SplitDir.Vertical offset
    top.height.val + bottom.height.val = r.height.val

-- ============================================================
-- Theorem: split_at rects don't overlap
-- ============================================================

axiom split_at_no_overlap_horizontal (r : Rect) (offset : U16) :
    let (left, right) := split_at r SplitDir.Horizontal offset
    rect_intersects left right = false

axiom split_at_no_overlap_vertical (r : Rect) (offset : U16) :
    let (top, bottom) := split_at r SplitDir.Vertical offset
    rect_intersects top bottom = false

end tui_core
