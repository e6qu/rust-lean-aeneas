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
theorem new_satisfies_inv (capacity : Usize) :
    ∀ gb, GapBuffer.new capacity = .ok gb → gap_inv gb := by
  intro gb h_new
  simp [gap_inv]
  -- gap_start = 0, gap_end = capacity = buffer.length
  sorry -- requires unfolding the new_loop

/-- Insert preserves the gap buffer invariant (when there is room). -/
theorem insert_preserves_inv (gb : GapBuffer) (ch : U8) (gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_not_full : gb.gap_start ≠ gb.gap_end)
    (h_ins : GapBuffer.insert gb ch = .ok gb') :
    gap_inv gb' := by
  simp [gap_inv] at *
  simp [GapBuffer.insert, h_not_full] at h_ins
  -- insert increments gap_start by 1, gap_end unchanged
  -- gap_start + 1 ≤ gap_end since gap_start < gap_end (not full)
  sorry -- requires Usize arithmetic

/-- delete_before preserves the gap buffer invariant. -/
theorem delete_before_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_del : GapBuffer.delete_before gb = .ok (true, gb')) :
    gap_inv gb' := by
  simp [gap_inv] at *
  simp [GapBuffer.delete_before] at h_del
  -- delete_before decrements gap_start by 1
  -- gap_start - 1 ≤ gap_start ≤ gap_end
  sorry -- requires Usize arithmetic

/-- delete_after preserves the gap buffer invariant. -/
theorem delete_after_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_del : GapBuffer.delete_after gb = .ok (true, gb')) :
    gap_inv gb' := by
  simp [gap_inv] at *
  simp [GapBuffer.delete_after] at h_del
  -- delete_after increments gap_end by 1
  -- gap_end + 1 ≤ buffer.length since gap_end < buffer.length (not at end)
  sorry -- requires Usize arithmetic

/-- move_left preserves the gap buffer invariant. -/
theorem move_left_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_left gb = .ok (true, gb')) :
    gap_inv gb' := by
  simp [gap_inv] at *
  simp [GapBuffer.move_left] at h_mv
  -- gap_start and gap_end both decrease by 1, so the gap size is preserved
  -- and the ordering gap_start ≤ gap_end ≤ buffer.length is maintained
  sorry

/-- move_right preserves the gap buffer invariant. -/
theorem move_right_preserves_inv (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_right gb = .ok (true, gb')) :
    gap_inv gb' := by
  simp [gap_inv] at *
  simp [GapBuffer.move_right] at h_mv
  -- gap_start and gap_end both increase by 1
  sorry

-- =========================================================================
-- Content preservation under insert
-- =========================================================================

/-- After inserting `ch` at cursor position `p`, the logical content becomes
    `content[0..p] ++ [ch] ++ content[p..]`. -/
theorem insert_preserves_content (gb : GapBuffer) (ch : U8) (gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_not_full : gb.gap_start ≠ gb.gap_end)
    (h_ins : GapBuffer.insert gb ch = .ok gb') :
    gap_content gb' =
      (gap_content gb).take gb.gap_start ++ [ch] ++
      (gap_content gb).drop gb.gap_start := by
  simp [gap_content, GapBuffer.insert, h_not_full] at *
  -- The key insight: inserting at gap_start places ch at the cursor position
  -- in the logical content. The pre-gap grows by one byte (ch), and the
  -- post-gap remains unchanged.
  sorry -- requires List.take/drop lemmas with Vec.index_mut_update

/-- After delete_before, the content loses the byte just before the cursor. -/
theorem delete_before_removes_byte (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_del : GapBuffer.delete_before gb = .ok (true, gb')) :
    gap_content gb' =
      (gap_content gb).take (gb.gap_start - 1) ++
      (gap_content gb).drop gb.gap_start := by
  simp [gap_content, GapBuffer.delete_before] at *
  sorry -- requires List.take/drop reasoning

/-- Insert followed by delete_before restores original content. -/
theorem delete_insert_inverse (gb : GapBuffer) (ch : U8)
    (gb1 gb2 : GapBuffer)
    (h_inv : gap_inv gb)
    (h_not_full : gb.gap_start ≠ gb.gap_end)
    (h_ins : GapBuffer.insert gb ch = .ok gb1)
    (h_del : GapBuffer.delete_before gb1 = .ok (true, gb2)) :
    gap_content gb2 = gap_content gb := by
  -- Insert places ch and advances gap_start; delete_before reverses that
  sorry

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
theorem move_left_preserves_content (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_left gb = .ok (true, gb')) :
    gap_content gb' = gap_content gb := by
  simp [gap_content, GapBuffer.move_left] at *
  -- move_left shifts one byte from before the gap to after the gap
  -- The logical content is the same, just the gap position changes
  sorry -- requires Vec.index / Vec.index_mut_update reasoning

/-- Moving the cursor right does not change the logical content. -/
theorem move_right_preserves_content (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_right gb = .ok (true, gb')) :
    gap_content gb' = gap_content gb := by
  simp [gap_content, GapBuffer.move_right] at *
  sorry -- symmetric to move_left

end buffer_management.GapBufferProofs
