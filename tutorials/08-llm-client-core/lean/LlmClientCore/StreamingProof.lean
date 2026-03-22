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
theorem fold_push_accumulated (chunks : List (List U8)) (acc : StreamAccumulator) :
    (fold_push chunks acc).accumulated = acc.accumulated ++ chunks.join := by
  induction chunks generalizing acc with
  | nil => simp [fold_push, List.join]
  | cons chunk rest ih =>
    simp [fold_push, stream_push, ih]
    simp [List.join, List.append_assoc]

/-- Chunking then reassembling via the accumulator is the identity.

    `stream_finish (fold_push (chunk_response data n) empty) = data`
    when `n > 0`. -/
theorem chunks_concat_eq_original (data : List U8) (n : Nat) (hn : n > 0) :
    stream_finish (fold_push (chunk_response data n) stream_accumulator_new) = data := by
  simp [stream_finish, stream_accumulator_new]
  rw [fold_push_accumulated]
  simp
  sorry  -- Show that (chunk_response data n).join = data using take_drop_eq

/-- The accumulator correctly tracks the number of chunks. -/
theorem accumulator_chunk_count (chunks : List (List U8)) :
    (fold_push chunks stream_accumulator_new).chunks.length = chunks.length := by
  sorry  -- Induction on chunks; each stream_push appends exactly one chunk

end llm_client_core
