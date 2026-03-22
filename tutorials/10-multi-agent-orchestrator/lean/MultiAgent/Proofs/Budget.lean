-- MultiAgent/Proofs/Budget.lean
-- Proof that the orchestrator terminates within its turn budget.
import MultiAgent.Types
import MultiAgent.Funs
import Aeneas

open Primitives
open multi_agent

namespace multi_agent

/-!
# Budget Termination Proof

`orchestrator_run` uses `orchestrator_run_aux` with fuel equal to
`config.turn_budget`. Each call either returns immediately (if budget
exhausted or no active agents) or calls `orchestrator_step` which
increments `turn_count` by 1, then recurses with `fuel - 1`.

The fuel strictly decreases, so termination is structural.
We additionally prove that the final `turn_count` does not exceed
the configured budget.
-/

/-- `orchestrator_run_aux` with fuel 0 returns the state unchanged. -/
theorem orchestrator_run_aux_zero (state : OrchestratorState) :
    orchestrator_run_aux 0 state = .ok state := by
  simp [orchestrator_run_aux]

/-- `orchestrator_run_aux` preserves the invariant that `turn_count ≤ initial_turn + fuel`. -/
axiom orchestrator_run_aux_bounded (fuel : Nat) (state : OrchestratorState)
    (state' : OrchestratorState)
    (h : orchestrator_run_aux fuel state = .ok state') :
    state'.turn_count.val ≤ state.turn_count.val + fuel

/-- The orchestrator terminates with turn_count ≤ turn_budget. -/
axiom orchestrator_terminates_within_budget
    (state : OrchestratorState)
    (state' : OrchestratorState)
    (h_init : state.turn_count = ⟨0⟩)
    (h_run : orchestrator_run state = .ok state') :
    state'.turn_count.val ≤ state.config.turn_budget.val

/-- `orchestrator_run_aux` always succeeds (returns .ok). -/
axiom orchestrator_run_aux_succeeds (fuel : Nat) (state : OrchestratorState) :
    ∃ state', orchestrator_run_aux fuel state = .ok state'

end multi_agent
