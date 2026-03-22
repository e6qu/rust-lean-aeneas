-- MessageProtocol/RoundtripProof.lean
-- Main roundtrip correctness theorem for the message protocol.
import MessageProtocol.Types
import MessageProtocol.Funs
import MessageProtocol.Spec
import MessageProtocol.Utils
import Aeneas

open Primitives
open message_protocol

namespace message_protocol

/-!
# Roundtrip Correctness

The central theorem: serializing a message and then deserializing the result
recovers the original message, along with the correct number of bytes consumed.

The proof proceeds by case analysis on the message variant, unfolding
`serialize` and `deserialize`, and appealing to the primitive roundtrip
lemmas for `u32_be` and `u64_be` from `Utils.lean`.
-/

/-- Roundtrip for Text messages. -/
theorem roundtrip_text (data : List U8) :
    deserialize (serialize (.Text data)) = .ok (.Text data, serialize_length (.Text data)) := by
  simp [serialize, deserialize, build_payload, message_tag, serialize_length]
  sorry  -- Unfold and apply read_u32_be_append, read_bytes roundtrip

/-- Roundtrip for Heartbeat messages. -/
theorem roundtrip_heartbeat (ts : U64) :
    deserialize (serialize (.Heartbeat ts)) = .ok (.Heartbeat ts, serialize_length (.Heartbeat ts)) := by
  simp [serialize, deserialize, build_payload, message_tag, serialize_length]
  sorry  -- Unfold and apply read_u32_be_append, read_u64_be_append

/-- Roundtrip for Command messages. -/
theorem roundtrip_command (cmd : CmdType) (args : List (List U8)) :
    deserialize (serialize (.Command cmd args)) = .ok (.Command cmd args, serialize_length (.Command cmd args)) := by
  simp [serialize, deserialize, build_payload, message_tag, serialize_length]
  sorry  -- Case split on cmd, unfold cmd_type_tag/cmd_type_from_tag, apply list lemmas

/-- Roundtrip for Error messages. -/
theorem roundtrip_error (code : ErrorCode) (detail : List U8) :
    deserialize (serialize (.Error code detail)) = .ok (.Error code detail, serialize_length (.Error code detail)) := by
  simp [serialize, deserialize, build_payload, message_tag, serialize_length]
  sorry  -- Case split on code, unfold error_code_tag/error_code_from_tag, apply list lemmas

/-- **Main theorem**: Roundtrip correctness for all message variants.

    For any message `msg`, deserializing its serialization recovers the
    original message and reports the correct number of consumed bytes. -/
theorem roundtrip (msg : Message) :
    deserialize (serialize msg) = .ok (msg, serialize_length msg) := by
  match msg with
  | .Text data         => exact roundtrip_text data
  | .Command cmd args  => exact roundtrip_command cmd args
  | .Error code detail => exact roundtrip_error code detail
  | .Heartbeat ts      => exact roundtrip_heartbeat ts

end message_protocol
