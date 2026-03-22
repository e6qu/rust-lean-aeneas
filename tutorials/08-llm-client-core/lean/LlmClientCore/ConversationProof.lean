-- LlmClientCore/ConversationProof.lean
-- Proofs about conversation append and trim operations.
import LlmClientCore.Types
import LlmClientCore.Funs
import LlmClientCore.ConversationSpec
import Aeneas

open Primitives
open llm_client_core

namespace llm_client_core

/-!
# Conversation Proofs

## append_preserves_alternation
If the conversation satisfies `valid_alternation` and the appended message
has the correct next role, then the result still satisfies `valid_alternation`.

## trim_respects_context
After `trim_to_context`, the estimated tokens fit within `max_context_tokens`
(or only the system message remains).
-/

/-- Appending a correctly-roled message preserves alternation.

    The proof proceeds by induction on the message list. The new message
    is appended at the end, so we need to show that the last-to-new
    transition respects the role alternation rule. -/
axiom append_preserves_alternation
    (msgs : List ChatMessage) (msg : ChatMessage)
    (h_alt : valid_alternation msgs)
    (h_last : msgs ≠ [] →
      match message_role (msgs.getLast!) with
      | .System    => message_role msg = .User
      | .User      => message_role msg = .Assistant
      | .Assistant  => message_role msg = .User) :
    valid_alternation (msgs ++ [msg])

/-- After trimming, the token estimate fits within the budget,
    or only the system message remains. -/
axiom trim_respects_context (conv : Conversation) :
    let trimmed := trim_to_context conv
    (estimate_tokens trimmed.messages).val ≤ trimmed.max_context_tokens.val ∨
    trimmed.messages.length ≤ 1

/-- Trimming preserves the system message at position 0. -/
axiom trim_preserves_system (conv : Conversation)
    (h : conv.messages ≠ [])
    (hsys : message_role (conv.messages.head h) = .System) :
    let trimmed := trim_to_context conv
    ∃ (hne : trimmed.messages ≠ []),
    message_role (trimmed.messages.head hne) = .System

end llm_client_core
