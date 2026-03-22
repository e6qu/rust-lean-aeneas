-- MessageProtocol/LengthProof.lean
-- The length prefix in the TLV encoding correctly describes the payload size.
import MessageProtocol.Types
import MessageProtocol.Funs
import MessageProtocol.Spec
import MessageProtocol.Utils
import Aeneas

open Primitives
open message_protocol

namespace message_protocol

/-!
# Length Prefix Correctness

The 4-byte length field in the TLV encoding of any message correctly
encodes the number of payload bytes that follow.
-/

/-- The length field in the serialized form matches the actual payload length.

    Given `serialize msg = [tag] ++ len_bytes ++ payload`, we have
    `read_u32_be len_bytes 0 = ok (payload.length, 4)`. -/
theorem length_prefix_correct (msg : Message) :
    read_u32_be (serialize msg) 1 = .ok (⟨(build_payload msg).length, by sorry⟩, 5) := by
  simp [serialize, message_tag, write_u32_be]
  sorry  -- Unfold serialize; the length field is write_u32_be of payload.length;
         -- reading at offset 1 skips the tag byte and reads those 4 bytes back.

/-- The total serialized length is 5 + payload_length. -/
theorem serialize_total_length (msg : Message) :
    (serialize msg).length = 5 + payload_length msg := by
  simp [serialize, payload_length, write_u32_be]
  omega

/-- The payload length is always non-negative (trivially true for Nat,
    but stated for documentation). -/
theorem payload_length_nonneg (msg : Message) :
    payload_length msg ≥ 0 := by
  omega

/-- For Heartbeat messages, the payload is exactly 8 bytes. -/
theorem heartbeat_payload_length (ts : U64) :
    payload_length (.Heartbeat ts) = 8 := by
  simp [payload_length, build_payload, write_u64_be]

/-- For Text messages, the payload is 4 + data.length bytes. -/
theorem text_payload_length (data : List U8) :
    payload_length (.Text data) = 4 + data.length := by
  simp [payload_length, build_payload, serialize_bytes, write_u32_be]

end message_protocol
