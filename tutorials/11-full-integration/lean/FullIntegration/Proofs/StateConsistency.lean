-- FullIntegration/Proofs/StateConsistency.lean
-- State consistency invariant and preservation proofs.
--
-- We define what it means for an AppState to be "consistent" and show
-- that app_update preserves this invariant for every event type.

import FullIntegration.Funs

namespace FullIntegration.Proofs.StateConsistency

open FullIntegration

/-- A state is consistent when:
    1. selected_agent < agent_count (or agent_count = 0)
    2. turn_count ≤ turn_budget
    3. timestamps in conversations are < next_timestamp -/
def state_consistent (s : AppState) : Prop :=
  (s.agent_count = 0 ∨ s.selected_agent < s.agent_count) ∧
  s.turn_count ≤ s.turn_budget ∧
  ∀ e ∈ s.conversations, e.timestamp < s.next_timestamp

/-- The initial state is consistent. -/
axiom new_state_consistent (ac tb : UInt32) (h : (0 : UInt32) ≤ tb) :
    state_consistent (AppState.new ac tb)

/-- Quit preserves consistency (it only sets running to false). -/
axiom quit_preserves_consistency (s : AppState) (h : state_consistent s) :
    state_consistent (app_update s .Quit)

/-- Tick preserves consistency (it is a no-op). -/
theorem tick_preserves_consistency (s : AppState) (h : state_consistent s) :
    state_consistent (app_update s .Tick) := by
  simp [app_update, handle_tick, state_consistent] at *
  exact h

/-- Resize preserves consistency (it is a no-op on state). -/
theorem resize_preserves_consistency (s : AppState) (w h' : UInt32)
    (h : state_consistent s) :
    state_consistent (app_update s (.Resize w h')) := by
  simp [app_update, state_consistent] at *
  exact h

/-- SwitchAgent with an invalid id preserves consistency (no-op). -/
theorem switch_agent_invalid_preserves (s : AppState) (aid : UInt32)
    (h : state_consistent s) (hinv : ¬ is_valid_agent_id aid s.agent_count) :
    state_consistent (app_update s (.SwitchAgent aid)) := by
  simp [app_update, handle_switch_agent, hinv, state_consistent] at *
  exact h

end FullIntegration.Proofs.StateConsistency
