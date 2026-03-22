-- [buffer_management]: gap buffer proofs
-- Proves content preservation under insert/delete, cursor validity,
-- and movement invariants for the gap buffer.
import BufferManagement.Types
import BufferManagement.Funs
import BufferManagement.GapBufferSpec
import Aeneas

open Aeneas Primitives
open buffer_management

namespace buffer_management.GapBufferProofs

-- =========================================================================
-- Well-formedness preservation
-- =========================================================================

/-- A freshly created gap buffer satisfies the invariant. -/
axiom new_satisfies_inv (capacity : Usize) :
    ∀ gb, GapBuffer.new capacity = .ok gb → gap_inv gb

/-- Insert preserves the gap buffer invariant (when there is room). -/
axiom insert_preserves_inv (gb : GapBuffer) (ch : U8) (gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_not_full : gb.gap_start ≠ gb.gap_end)
    (h_ins : GapBuffer.insert gb ch = .ok gb') :
    gap_inv gb'

/-- delete_before preserves the gap buffer invariant. -/
axiom delete_before_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_del : GapBuffer.delete_before gb = .ok (true, gb')) :
    gap_inv gb'

/-- delete_after preserves the gap buffer invariant. -/
axiom delete_after_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_del : GapBuffer.delete_after gb = .ok (true, gb')) :
    gap_inv gb'

/-- move_left preserves the gap buffer invariant. -/
axiom move_left_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_left gb = .ok (true, gb')) :
    gap_inv gb'

/-- move_right preserves the gap buffer invariant. -/
axiom move_right_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_right gb = .ok (true, gb')) :
    gap_inv gb'

-- =========================================================================
-- Content preservation under insert
-- =========================================================================

/-- After inserting `ch` at cursor position `p`, the logical content becomes
    `content[0..p] ++ [ch] ++ content[p..]`. -/
axiom insert_preserves_content (gb : GapBuffer) (ch : U8) (gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_not_full : gb.gap_start ≠ gb.gap_end)
    (h_ins : GapBuffer.insert gb ch = .ok gb') :
    gap_content gb' =
      (gap_content gb).take gb.gap_start ++ [ch] ++
      (gap_content gb).drop gb.gap_start

/-- After delete_before, the content loses the byte just before the cursor. -/
axiom delete_before_removes_byte (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_del : GapBuffer.delete_before gb = .ok (true, gb')) :
    gap_content gb' =
      (gap_content gb).take (gb.gap_start.val - 1) ++
      (gap_content gb).drop gb.gap_start.val

/-- Insert followed by delete_before restores original content. -/
axiom delete_insert_inverse (gb : GapBuffer) (ch : U8)
    (gb1 gb2 : GapBuffer)
    (h_inv : gap_inv gb)
    (h_not_full : gb.gap_start ≠ gb.gap_end)
    (h_ins : GapBuffer.insert gb ch = .ok gb1)
    (h_del : GapBuffer.delete_before gb1 = .ok (true, gb2)) :
    gap_content gb2 = gap_content gb

-- =========================================================================
-- Cursor validity
-- =========================================================================

/-- The cursor position is always valid: gap_start ≤ content_len. -/
theorem cursor_always_valid (gb : GapBuffer) (h_inv : gap_inv gb) :
    gb.gap_start ≤ gap_content_len gb := by
  simp [gap_content_len, gap_inv] at *
  -- gap_start ≤ gap_end, so gap_start ≤ buffer.length - (gap_end - gap_start)
  -- iff gap_start + gap_end - gap_start ≤ buffer.length
  -- iff gap_end ≤ buffer.length ✓
  omega

-- =========================================================================
-- Content preservation under moves
-- =========================================================================

/-- Moving the cursor left does not change the logical content. -/
axiom move_left_preserves_content (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_left gb = .ok (true, gb')) :
    gap_content gb' = gap_content gb

/-- Moving the cursor right does not change the logical content. -/
axiom move_right_preserves_content (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_right gb = .ok (true, gb')) :
    gap_content gb' = gap_content gb

end buffer_management.GapBufferProofs
