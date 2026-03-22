-- LlmClientCore/StringUtils.lean
-- Byte-list manipulation lemmas used across proofs.
import Aeneas

open Primitives

namespace llm_client_core

/-! ## List append / split lemmas -/

/-- Taking and dropping recombines to the original list. -/
theorem take_drop_eq {α : Type} (l : List α) (n : Nat) :
    l.take n ++ l.drop n = l := by
  exact List.take_append_drop n l

/-- Length of take is min of n and list length. -/
theorem take_length {α : Type} (l : List α) (n : Nat) :
    (l.take n).length = min n l.length := by
  simp [List.length_take]

/-- Dropping all elements yields empty. -/
theorem drop_length_eq_nil {α : Type} (l : List α) :
    l.drop l.length = [] := by
  simp

/-- Append of empty is identity. -/
theorem append_nil {α : Type} (l : List α) : l ++ [] = l := by
  simp

/-- Append is associative. -/
theorem append_assoc {α : Type} (a b c : List α) : (a ++ b) ++ c = a ++ (b ++ c) := by
  simp [List.append_assoc]

/-- The join of a list split into chunks equals the original. -/
theorem join_chunks {α : Type} (l : List α) (n : Nat) (hn : n > 0) :
    ∀ (chunks : List (List α)),
    (∀ i (h : i < chunks.length), (chunks[i]).length ≤ n) →
    chunks.flatten = l →
    l.length = (chunks.map List.length).foldl (· + ·) 0 := by
  sorry

end llm_client_core
