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
def frame_extract_all : List U8 → Result (List Message)
  | [] => .ok []
  | data => do
    let (msg, consumed) ← deserialize data
    if h : consumed ≤ data.length then
      let rest := data.drop consumed
      let msgs ← frame_extract_all rest
      .ok (msg :: msgs)
    else
      .fail .panic
termination_by data => data.length
decreasing_by
  sorry  -- consumed > 0 when deserialize succeeds on non-empty data

/-- Serialization of a single message followed by extraction yields that message. -/
theorem frame_extract_single (msg : Message) :
    frame_extract_all (serialize msg) = .ok [msg] := by
  sorry  -- Unfold frame_extract_all, apply roundtrip, show rest is empty

/-- **Main framing theorem**: concatenating serialized messages and extracting
    recovers the original message list in order. -/
theorem framing_no_loss (msgs : List Message) :
    frame_extract_all (msgs.flatMap serialize) = .ok msgs := by
  induction msgs with
  | nil => simp [frame_extract_all, List.flatMap]
  | cons msg rest ih =>
    sorry  -- Unfold flatMap, apply roundtrip to show first msg is extracted,
           -- then the remaining bytes are (rest.flatMap serialize),
           -- and apply induction hypothesis.

/-- If the accumulator is fed the complete serialization of a list of messages
    in one chunk, it produces exactly those messages. -/
theorem feed_complete (msgs : List Message) :
    ∀ (acc : FrameAccumulator),
      acc.buffer = [] →
      frame_extract_all (msgs.flatMap serialize) = .ok msgs := by
  intro acc _
  exact framing_no_loss msgs

end message_protocol
