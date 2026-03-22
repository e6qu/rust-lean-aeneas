-- [tui_core]: Scroll bounds proofs
-- Proves that scroll_clamp always produces a valid offset.
import TuiCore.Types
import TuiCore.Funs

open Aeneas Primitives

namespace tui_core

-- ============================================================
-- Theorem: scroll_in_bounds
-- scroll_clamp always returns a value <= item_count - visible
-- (when item_count >= visible), or 0 otherwise.
-- ============================================================

/-- When there are fewer items than visible rows, scroll offset is 0. -/
theorem scroll_clamp_few_items (offset item_count visible : Nat)
    (h : item_count <= visible) :
    scroll_clamp offset item_count visible = 0 := by
  simp [scroll_clamp, h]

/-- When there are enough items, scroll_clamp returns at most item_count - visible. -/
theorem scroll_clamp_upper_bound (offset item_count visible : Nat)
    (h : item_count > visible) :
    scroll_clamp offset item_count visible <= item_count - visible := by
  simp [scroll_clamp]
  split
  · omega
  · split
    · omega
    · omega

/-- scroll_clamp is idempotent: clamping an already-clamped value is a no-op. -/
theorem scroll_clamp_idempotent (offset item_count visible : Nat) :
    scroll_clamp (scroll_clamp offset item_count visible) item_count visible
      = scroll_clamp offset item_count visible := by
  simp [scroll_clamp]
  split <;> split <;> split <;> omega

end tui_core
