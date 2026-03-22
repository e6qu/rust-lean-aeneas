-- LlmClientCore/RequestSpec.lean
-- Specification predicates for well-formed requests.
import LlmClientCore.Types
import LlmClientCore.Funs
import Aeneas

open Primitives
open llm_client_core

namespace llm_client_core

/-! ## Well-formedness predicate for Request -/

/-- A request is well-formed if:
    1. Its messages list is non-empty.
    2. The first message has role System.
    3. The temperature is at most 200 (2.0 fixed-point).
    4. max_tokens > 0. -/
def well_formed (req : Request) : Prop :=
  req.messages ≠ [] ∧
  (match req.messages with
   | [] => False
   | msg :: _ => message_role msg = .System) ∧
  req.temperature.val ≤ 200 ∧
  req.max_tokens.val > 0

/-- Non-empty messages implies the list has a head. -/
theorem well_formed_has_head (req : Request) (h : well_formed req) :
    ∃ msg rest, req.messages = msg :: rest := by
  obtain ⟨hne, _, _, _⟩ := h
  match hm : req.messages with
  | [] => exact absurd hm hne
  | msg :: rest => exact ⟨msg, rest, rfl⟩

/-- Well-formed requests have a system message first. -/
theorem well_formed_system_first (req : Request) (h : well_formed req) :
    ∃ msg rest, req.messages = msg :: rest ∧ message_role msg = .System := by
  obtain ⟨hne, hsys, _, _⟩ := h
  match hm : req.messages with
  | [] => exact absurd hm hne
  | msg :: rest =>
    simp [hm] at hsys
    exact ⟨msg, rest, rfl, hsys⟩

end llm_client_core
