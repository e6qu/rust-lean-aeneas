-- [buffer_management]: gap buffer specification
-- Hand-written abstraction function and well-formedness invariant
-- for the gap buffer data structure.
import BufferManagement.Types
import Aeneas

open Aeneas Primitives

namespace buffer_management

-- =========================================================================
-- Well-formedness invariant
-- =========================================================================

/-- A gap buffer is well-formed when `gap_start ≤ gap_end ≤ buffer.length`. -/
def gap_inv (gb : GapBuffer) : Prop :=
  gb.gap_start ≤ gb.gap_end ∧
  gb.gap_end ≤ gb.buffer.length

-- =========================================================================
-- Abstraction function: gap buffer → logical content
-- =========================================================================

/-- The logical content of a gap buffer: the bytes before the gap
    concatenated with the bytes after the gap.
    `gap_content gb = buffer[0..gap_start] ++ buffer[gap_end..buffer.length]` -/
def gap_content (gb : GapBuffer) : List U8 :=
  (gb.buffer.val.take gb.gap_start) ++ (gb.buffer.val.drop gb.gap_end)

/-- The content length equals the total buffer size minus the gap size. -/
def gap_content_len (gb : GapBuffer) : Nat :=
  gb.buffer.length - (gb.gap_end - gb.gap_start)

/-- The content length matches the length of the materialized content list. -/
theorem gap_content_len_eq (gb : GapBuffer) (h_inv : gap_inv gb) :
    (gap_content gb).length = gap_content_len gb := by
  simp [gap_content, gap_content_len, gap_inv] at *
  simp [List.length_append, List.length_take, List.length_drop]
  omega

end buffer_management
