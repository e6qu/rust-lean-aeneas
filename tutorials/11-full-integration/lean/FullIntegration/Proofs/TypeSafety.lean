-- FullIntegration/Proofs/TypeSafety.lean
-- Bridge function correctness proofs.
--
-- We show that the message-bridge functions produce well-formed values:
-- valid pane ids, valid message triples, correct role assignments.

import FullIntegration.Funs

namespace FullIntegration.Proofs.TypeSafety

open FullIntegration

/-- Submitting via user_input_to_message always produces sender = u32::MAX. -/
theorem submit_sender_is_user (content_id coordinator_id : UInt32) :
    (user_input_to_message content_id coordinator_id).1 =
      UInt32.ofNat 4294967295 := by
  simp [user_input_to_message]

/-- The content_id passes through user_input_to_message unchanged. -/
theorem submit_content_id_preserved (content_id coordinator_id : UInt32) :
    (user_input_to_message content_id coordinator_id).2.2 = content_id := by
  simp [user_input_to_message]

/-- message_to_conversation_entry assigns role 0 when sender is u32::MAX. -/
theorem user_message_has_role_zero (content_id timestamp : UInt32) :
    (message_to_conversation_entry (UInt32.ofNat 4294967295) content_id timestamp).role = 0 := by
  simp [message_to_conversation_entry]

/-- message_to_conversation_entry assigns role 1 for non-user senders. -/
theorem agent_message_has_role_one (sender content_id timestamp : UInt32)
    (h : sender ≠ UInt32.ofNat 4294967295) :
    (message_to_conversation_entry sender content_id timestamp).role = 1 := by
  simp [message_to_conversation_entry]
  intro heq
  exact absurd heq h

/-- is_valid_pane_id accepts exactly ids 0..3. -/
theorem valid_pane_ids :
    is_valid_pane_id 0 = true ∧
    is_valid_pane_id 1 = true ∧
    is_valid_pane_id 2 = true ∧
    is_valid_pane_id 3 = true ∧
    is_valid_pane_id 4 = false := by
  simp [is_valid_pane_id, PANE_COUNT]
  constructor <;> native_decide

/-- PaneId round-trips through toUInt32/fromUInt32. -/
theorem pane_id_roundtrip (p : PaneId) :
    PaneId.fromUInt32 (PaneId.toUInt32 p) = some p := by
  cases p <;> simp [PaneId.toUInt32, PaneId.fromUInt32] <;> native_decide

/-- The bridge from LLM response to conversation entry preserves the agent_id. -/
theorem llm_bridge_preserves_agent_id (state : AppState) (aid cid : UInt32) :
    ∀ e ∈ (handle_llm_response state aid cid).conversations,
      e ∈ state.conversations ∨ e.agent_id = aid := by
  intro e he
  simp [handle_llm_response] at he
  cases he with
  | inl hmem => exact Or.inl hmem
  | inr heq  => right; rw [heq]; rfl

end FullIntegration.Proofs.TypeSafety
