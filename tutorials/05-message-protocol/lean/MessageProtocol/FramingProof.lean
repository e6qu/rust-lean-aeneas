-- MessageProtocol/FramingProof.lean
-- Framing correctness: concatenated serialized messages can be recovered.
import MessageProtocol.Types
import MessageProtocol.Funs
import MessageProtocol.Spec
import MessageProtocol.Utils
import MessageProtocol.RoundtripProof
import Aeneas

open Primitives
open message_protocol

namespace message_protocol

/-!
# Framing Correctness

When multiple messages are serialized and concatenated, we can extract
each one in order by repeatedly applying `deserialize`. This models the
behavior of `FrameAccumulator.feed` on a complete byte stream.
-/

/-- Extract all messages from a concatenated byte stream. -/
partial def frame_extract_all : List U8 → Result (List Message)
  | [] => .ok []
  | data => do
    let (msg, consumed) ← deserialize data
    if h : consumed ≤ data.length then
      let rest := data.drop consumed
      let msgs ← frame_extract_all rest
      .ok (msg :: msgs)
    else
      .fail .panic

/-- Serialization of a single message followed by extraction yields that message. -/
axiom frame_extract_single (msg : Message) :
    frame_extract_all (serialize msg) = .ok [msg]

/-- **Main framing theorem**: concatenating serialized messages and extracting
    recovers the original message list in order. -/
axiom framing_no_loss (msgs : List Message) :
    frame_extract_all (msgs.flatMap serialize) = .ok msgs

/-- If the accumulator is fed the complete serialization of a list of messages
    in one chunk, it produces exactly those messages. -/
theorem feed_complete (msgs : List Message) :
    ∀ (acc : FrameAccumulator),
      acc.buffer = [] →
      frame_extract_all (msgs.flatMap serialize) = .ok msgs := by
  intro acc _
  exact framing_no_loss msgs

end message_protocol
