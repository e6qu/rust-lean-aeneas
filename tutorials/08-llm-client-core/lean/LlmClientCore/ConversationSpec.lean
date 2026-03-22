-- LlmClientCore/ConversationSpec.lean
-- Specification predicates for conversation invariants.
import LlmClientCore.Types
import LlmClientCore.Funs
import Aeneas

open Primitives
open llm_client_core

namespace llm_client_core

/-! ## Alternation predicate -/

/-- Valid alternation for a message list:
    - The first message (if any) must be System.
    - After System, the next must be User.
    - After User, the next must be Assistant.
    - After Assistant, the next must be User. -/
def valid_alternation : List ChatMessage → Prop
  | [] => True
  | [msg] => message_role msg = .System
  | msg₁ :: msg₂ :: rest =>
    let r₁ := message_role msg₁
    let r₂ := message_role msg₂
    (match r₁ with
     | .System    => r₂ = .User
     | .User      => r₂ = .Assistant
     | .Assistant  => r₂ = .User) ∧
    valid_alternation (msg₂ :: rest)

/-- The conversation invariant: messages satisfy alternation and
    the estimated tokens are within the context budget. -/
def conv_inv (conv : Conversation) : Prop :=
  valid_alternation conv.messages ∧
  (estimate_tokens conv.messages).val ≤ conv.max_context_tokens.val

/-! ## Helper lemmas -/

/-- A singleton system message has valid alternation. -/
theorem valid_alternation_singleton_system (content : List U8) :
    valid_alternation [.RoleMessage .System content] := by
  simp [valid_alternation, message_role]

/-- conversation_new produces a conversation with valid alternation. -/
theorem conversation_new_valid (sys : List U8) (max_ctx : U32) :
    valid_alternation (conversation_new sys max_ctx).messages := by
  simp [conversation_new]
  exact valid_alternation_singleton_system sys

end llm_client_core
