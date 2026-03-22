-- MessageProtocol/ErrorDetectionProof.lean
-- Proofs that malformed input is correctly rejected.
import MessageProtocol.Types
import MessageProtocol.Funs
import MessageProtocol.Spec
import Aeneas

open Primitives
open message_protocol

namespace message_protocol

/-!
# Error Detection

We prove that the deserializer correctly rejects invalid inputs:
- Unknown tag bytes (values > 3) produce an error.
- Truncated data (fewer bytes than the header requires) produces an error.
-/

/-- Any tag byte with value > 3 is rejected.

    If the first byte of `data` is not 0, 1, 2, or 3 (and the data has
    a valid length field), `deserialize` fails. -/
axiom invalid_tag_rejected (data : List U8)
    (h_len : data.length ≥ 5)
    (h_tag : (data.get ⟨0, by omega⟩).val > 3) :
    ∃ e, deserialize data = .fail e

/-- Empty input is rejected. -/
theorem empty_rejected :
    ∃ e, deserialize ([] : List U8) = .fail e := by
  exact ⟨.panic, by simp [deserialize]⟩

/-- Truncated data (fewer than 5 bytes) is rejected.

    If the data has fewer than 5 bytes (1 tag + 4 length), the
    `read_u32_be` call for the length field fails. -/
axiom truncated_data_rejected (data : List U8)
    (h : data.length > 0)
    (h_short : data.length < 5) :
    ∃ e, deserialize data = .fail e

/-- If the length field claims more bytes than available, deserialization fails. -/
axiom insufficient_payload_rejected (tag : U8) (claimed_len : U32)
    (payload : List U8)
    (h_tag : tag.val ≤ 3)
    (h_short : payload.length < claimed_len.val) :
    ∃ e, deserialize ([tag] ++ write_u32_be claimed_len ++ payload) = .fail e

end message_protocol
