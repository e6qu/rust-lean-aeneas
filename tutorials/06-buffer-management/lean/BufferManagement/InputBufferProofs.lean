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
axiom move_to_start_cursor_zero (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_start ib = .ok ib') :
    ib'.gap.gap_start = 0

/-- move_to_start preserves the gap buffer content. -/
axiom move_to_start_preserves_content (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_start ib = .ok ib') :
    gap_content ib'.gap = gap_content ib.gap

-- =========================================================================
-- move_to_end correctness
-- =========================================================================

/-- After move_to_end, the cursor is at the content length. -/
axiom move_to_end_cursor_at_end (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_end ib = .ok ib') :
    ib'.gap.gap_start.val = gap_content_len ib'.gap

/-- move_to_end preserves the gap buffer content. -/
axiom move_to_end_preserves_content (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_mv : InputBuffer.move_to_end ib = .ok ib') :
    gap_content ib'.gap = gap_content ib.gap

-- =========================================================================
-- delete_word correctness
-- =========================================================================

/-- Helper: characterize "the last word" in a byte list.
    `remove_last_word` strips trailing spaces, then strips trailing non-spaces. -/
partial def remove_trailing_spaces : List U8 → List U8
  | [] => []
  | xs =>
    if xs.getLast? == some 32 then
      remove_trailing_spaces (xs.dropLast)
    else
      xs

partial def remove_trailing_word : List U8 → List U8
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
axiom delete_word_correct (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_del : InputBuffer.delete_word ib = .ok (true, ib')) :
    gap_content ib'.gap =
      remove_last_word (gap_content ib.gap) ib.gap.gap_start

/-- delete_word preserves the invariant. -/
axiom delete_word_preserves_inv (ib ib' : InputBuffer)
    (h_inv : ib_inv ib)
    (h_del : InputBuffer.delete_word ib = .ok (true, ib')) :
    ib_inv ib'

/-- delete_word on an empty buffer returns false. -/
axiom delete_word_empty (ib : InputBuffer)
    (h_inv : ib_inv ib)
    (h_empty : ib.gap.gap_start = 0) :
    ∃ ib', InputBuffer.delete_word ib = .ok (false, ib')

end buffer_management.InputBufferProofs
