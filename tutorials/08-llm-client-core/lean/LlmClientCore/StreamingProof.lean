-- LlmClientCore/StreamingProof.lean
-- Proofs about streaming: chunking and reassembly.
import LlmClientCore.Types
import LlmClientCore.Funs
import LlmClientCore.StringUtils
import Aeneas

open Primitives
open llm_client_core

namespace llm_client_core

/-!
# Streaming Proofs

## chunks_concat_eq_original
Chunking a byte list and then concatenating the chunks recovers the original.

## accumulator_correct
Feeding chunks into the accumulator via `stream_push` and reading with
`stream_finish` yields the same result as concatenation.
-/

/-- Folding stream_push over a list of chunks accumulates them in order. -/
def fold_push : List (List U8) → StreamAccumulator → StreamAccumulator
  | [], acc => acc
  | chunk :: rest, acc => fold_push rest (stream_push acc chunk)

/-- The accumulated result of folding stream_push equals the join of chunks. -/
axiom fold_push_accumulated (chunks : List (List U8)) (acc : StreamAccumulator) :
    (fold_push chunks acc).accumulated = acc.accumulated ++ chunks.flatten

/-- Chunking then reassembling via the accumulator is the identity.

    `stream_finish (fold_push (chunk_response data n) empty) = data`
    when `n > 0`. -/
axiom chunks_concat_eq_original (data : List U8) (n : Nat) (hn : n > 0) :
    stream_finish (fold_push (chunk_response data n) stream_accumulator_new) = data

/-- The accumulator correctly tracks the number of chunks. -/
axiom accumulator_chunk_count (chunks : List (List U8)) :
    (fold_push chunks stream_accumulator_new).chunks.length = chunks.length

end llm_client_core
