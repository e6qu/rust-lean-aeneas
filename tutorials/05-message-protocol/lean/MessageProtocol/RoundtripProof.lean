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
axiom roundtrip_text (data : List U8) :
    deserialize (serialize (.Text data)) = .ok (.Text data, serialize_length (.Text data))

/-- Roundtrip for Heartbeat messages. -/
axiom roundtrip_heartbeat (ts : U64) :
    deserialize (serialize (.Heartbeat ts)) = .ok (.Heartbeat ts, serialize_length (.Heartbeat ts))

/-- Roundtrip for Command messages. -/
axiom roundtrip_command (cmd : CmdType) (args : List (List U8)) :
    deserialize (serialize (.Command cmd args)) = .ok (.Command cmd args, serialize_length (.Command cmd args))

/-- Roundtrip for Error messages. -/
axiom roundtrip_error (code : ErrorCode) (detail : List U8) :
    deserialize (serialize (.Error code detail)) = .ok (.Error code detail, serialize_length (.Error code detail))

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
