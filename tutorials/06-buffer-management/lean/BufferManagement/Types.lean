-- [buffer_management]: type definitions
-- This file simulates the Aeneas-generated type definitions for the
-- buffer management data structures.
import Aeneas

open Aeneas Primitives

namespace buffer_management

/-- A fixed-capacity ring buffer storing elements in FIFO order.
    `data` is pre-allocated with default values; `head` and `tail` are
    indices into the circular array; `len` tracks occupancy. -/
structure RingBuffer (T : Type) where
  data : Vec T
  capacity : Usize
  head : Usize
  tail : Usize
  len : Usize
deriving Inhabited

/-- A gap buffer for text editing. The logical content is
    `buffer[0..gap_start] ++ buffer[gap_end..buffer.length]`.
    The gap sits at the cursor position. -/
structure GapBuffer where
  buffer : Vec U8
  gap_start : Usize
  gap_end : Usize
deriving Inhabited

/-- A higher-level input buffer wrapping a GapBuffer with
    editor-like operations (word deletion, line-level cursor movement). -/
structure InputBuffer where
  gap : GapBuffer
deriving Inhabited

end buffer_management
