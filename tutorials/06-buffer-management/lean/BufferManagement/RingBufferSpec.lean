-- [buffer_management]: ring buffer specification
-- Hand-written abstraction function and well-formedness invariant
-- for the ring buffer data structure.
import BufferManagement.Types
import Aeneas

open Aeneas Primitives

namespace buffer_management

-- =========================================================================
-- Well-formedness invariant
-- =========================================================================

/-- A ring buffer is well-formed when its indices are within bounds
    and the data vector has been properly allocated. -/
def ring_inv (T : Type) (rb : RingBuffer T) : Prop :=
  rb.capacity.val > 0 ∧
  rb.len.val ≤ rb.capacity.val ∧
  rb.head.val < rb.capacity.val ∧
  rb.tail.val < rb.capacity.val ∧
  rb.data.val.size = rb.capacity.val ∧
  -- The tail is consistent with head + len (mod capacity)
  rb.tail.val = (rb.head.val + rb.len.val) % rb.capacity.val

-- =========================================================================
-- Abstraction function: ring buffer → logical list
-- =========================================================================

/-- Extract the logical list of elements from a well-formed ring buffer.
    Reads `len` elements starting at `head`, wrapping around using modular
    arithmetic. We define this recursively on the count of remaining elements. -/
def ring_to_list_aux (T : Type) (data : Vec T) (capacity head remaining : Nat)
    (hcap : capacity > 0) : List T :=
  match remaining with
  | 0 => []
  | n + 1 =>
    let idx := head % capacity
    -- We trust that the invariant guarantees idx < data.length
    if h : idx < data.val.size then
      data.val[idx] :: ring_to_list_aux T data capacity (head + 1) n hcap
    else
      [] -- unreachable under the invariant

/-- The logical list represented by a ring buffer. -/
def ring_to_list (T : Type) (rb : RingBuffer T) (hcap : rb.capacity > 0) : List T :=
  ring_to_list_aux T rb.data rb.capacity rb.head rb.len hcap

end buffer_management
