-- [buffer_management]: input buffer proofs
-- Proves correctness of delete_word, move_to_start, and move_to_end
-- for the InputBuffer abstraction over GapBuffer.
import BufferManagement.Types
import BufferManagement.Funs
import BufferManagement.GapBufferSpec
import BufferManagement.GapBufferProofs
import Aeneas

open Aeneas Primitives
open buffer_management

namespace buffer_management.InputBufferProofs

-- =========================================================================
-- Helper: well-formedness of InputBuffer
-- =========================================================================

/-- An InputBuffer is well-formed when its underlying gap buffer is. -/
def ib_inv (ib : InputBuffer) : Prop :=
  gap_inv ib.gap

-- =========================================================================
-- move_to_start correctness
-- =========================================================================

/-- After move_to_start, the cursor is at position 0. -/
theorem move_to_start_cursor_zero (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_start ib = .ok ib') :
    ib'.gap.gap_start = 0 := by
  simp [InputBuffer.move_to_start] at h_mv
  -- move_to_start_loop repeatedly calls move_left until gap_start = 0
  -- The loop terminates because gap_start is a natural number decreasing each step
  sorry -- requires loop unrolling / induction on gap_start

/-- move_to_start preserves the gap buffer content. -/
theorem move_to_start_preserves_content (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_start ib = .ok ib') :
    gap_content ib'.gap = gap_content ib.gap := by
  -- Each step of move_to_start calls move_left, which preserves content
  sorry -- induction using GapBufferProofs.move_left_preserves_content

-- =========================================================================
-- move_to_end correctness
-- =========================================================================

/-- After move_to_end, the cursor is at the content length. -/
theorem move_to_end_cursor_at_end (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_end ib = .ok ib') :
    ib'.gap.gap_start = gap_content_len ib'.gap := by
  simp [InputBuffer.move_to_end] at h_mv
  -- move_to_end_loop repeatedly calls move_right until gap_start = content_len
  sorry -- requires loop unrolling / induction

/-- move_to_end preserves the gap buffer content. -/
theorem move_to_end_preserves_content (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_end ib = .ok ib') :
    gap_content ib'.gap = gap_content ib.gap := by
  sorry -- induction using GapBufferProofs.move_right_preserves_content

-- =========================================================================
-- delete_word correctness
-- =========================================================================

/-- Helper: characterize "the last word" in a byte list.
    `remove_last_word` strips trailing spaces, then strips trailing non-spaces. -/
def remove_trailing_spaces : List U8 → List U8
  | [] => []
  | xs =>
    if xs.getLast? == some 32 then
      remove_trailing_spaces (xs.dropLast)
    else
      xs

def remove_trailing_word : List U8 → List U8
  | [] => []
  | xs =>
    if xs.getLast? != none && xs.getLast? != some 32 then
      remove_trailing_word (xs.dropLast)
    else
      xs

/-- The specification-level "remove last word" operation. -/
def remove_last_word (content : List U8) (cursor : Nat) : List U8 :=
  let before := content.take cursor
  let after := content.drop cursor
  let before' := remove_trailing_word (remove_trailing_spaces before)
  before' ++ after

/-- delete_word removes exactly the last word before the cursor.
    After delete_word, the content equals the original content with
    the last word (and any trailing spaces) before the cursor removed. -/
theorem delete_word_correct (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_del : InputBuffer.delete_word ib = .ok (true, ib')) :
    gap_content ib'.gap =
      remove_last_word (gap_content ib.gap) ib.gap.gap_start := by
  -- delete_word has two phases:
  -- Phase 1 (ws_loop): skip spaces backward → removes trailing spaces before cursor
  -- Phase 2 (nws_loop): delete non-spaces backward → removes the word
  -- Each delete_before call removes one byte from gap_content at cursor - 1
  sorry -- requires induction on both loops

/-- delete_word preserves the invariant. -/
theorem delete_word_preserves_inv (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_del : InputBuffer.delete_word ib = .ok (true, ib')) :
    ib_inv ib' := by
  simp [ib_inv] at *
  -- Each delete_before in the loop preserves gap_inv
  sorry -- induction using GapBufferProofs.delete_before_preserves_inv

/-- delete_word on an empty buffer returns false. -/
theorem delete_word_empty (ib : InputBuffer)
    (h_inv : ib_inv ib)
    (h_empty : ib.gap.gap_start = 0) :
    ∃ ib', InputBuffer.delete_word ib = .ok (false, ib') := by
  -- When gap_start = 0, both loops immediately return without deleting
  sorry

end buffer_management.InputBufferProofs
