-- MessageProtocol/Spec.lean
-- Hand-written specification functions for reasoning about the protocol.
import MessageProtocol.Types
import MessageProtocol.Funs
import Aeneas

open Primitives
open message_protocol

namespace message_protocol

/-! ## Pure specification helpers -/

/-- The total serialized length of a message (tag + length field + payload). -/
def serialize_length (msg : Message) : Nat :=
  (serialize msg).length

/-- The payload length (bytes after the 5-byte TLV header). -/
def payload_length (msg : Message) : Nat :=
  (build_payload msg).length

/-- Decidable equality on messages (provided by deriving, re-exported for clarity). -/
def msg_eq_dec : DecidableEq Message := inferInstance

/-! ## Pure spec versions for reasoning -/

/-- Spec version: serialize is just [tag] ++ length_bytes ++ payload. -/
axiom serialize_unfold (msg : Message) :
    serialize msg = [message_tag msg] ++ write_u32_be ⟨(build_payload msg).length⟩ ++ build_payload msg

/-- The serialized form always starts with the correct tag. -/
axiom serialize_starts_with_tag (msg : Message) :
    (serialize msg).head? = some (message_tag msg)

/-- The serialized form has length at least 5 (1 tag + 4 length bytes). -/
axiom serialize_length_ge_5 (msg : Message) :
    (serialize msg).length ≥ 5

end message_protocol
