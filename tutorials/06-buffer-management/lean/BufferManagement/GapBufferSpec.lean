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
  gb.gap_start.val ≤ gb.gap_end.val ∧
  gb.gap_end.val ≤ gb.buffer.val.size

-- =========================================================================
-- Abstraction function: gap buffer → logical content
-- =========================================================================

/-- The logical content of a gap buffer: the bytes before the gap
    concatenated with the bytes after the gap.
    `gap_content gb = buffer[0..gap_start] ++ buffer[gap_end..buffer.length]` -/
def gap_content (gb : GapBuffer) : List U8 :=
  (gb.buffer.val.toList.take gb.gap_start.val) ++ (gb.buffer.val.toList.drop gb.gap_end.val)

/-- The content length equals the total buffer size minus the gap size. -/
def gap_content_len (gb : GapBuffer) : Nat :=
  gb.buffer.val.size - (gb.gap_end.val - gb.gap_start.val)

/-- The content length matches the length of the materialized content list. -/
axiom gap_content_len_eq (gb : GapBuffer) (h_inv : gap_inv gb) :
    (gap_content gb).length = gap_content_len gb

end buffer_management
