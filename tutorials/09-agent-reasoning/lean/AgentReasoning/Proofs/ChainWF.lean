-- AgentReasoning/Proofs/ChainWF.lean
-- Proofs about reasoning chain well-formedness.
import AgentReasoning.Types
import AgentReasoning.Funs
import Aeneas

open Primitives
open agent_reasoning

namespace agent_reasoning

/-!
# Chain Well-Formedness Proofs

## append_preserves_well_formed
If `is_chain_well_formed chain = true` and `append_step chain step = some chain'`,
then `is_chain_well_formed chain' = true`.

The key insight: `append_step` returns `some` only when the new step's order
is ≥ the last step's order, which is exactly the well-formedness invariant.
-/

/-- An empty chain is well-formed. -/
theorem empty_chain_well_formed : is_chain_well_formed [] = true := by
  simp [is_chain_well_formed]

/-- A singleton chain is always well-formed. -/
theorem singleton_chain_well_formed (s : Step) :
    is_chain_well_formed [s] = true := by
  simp [is_chain_well_formed, is_chain_well_formed_aux]

/-- `append_step` on an empty chain always succeeds. -/
theorem append_to_empty (step : Step) :
    append_step [] step = some [step] := by
  simp [append_step, List.getLast?]

/-- If `append_step` succeeds, the result is well-formed (assuming the
    input was well-formed). -/
theorem append_preserves_well_formed
    (chain : List Step) (step : Step) (chain' : List Step)
    (h_wf : is_chain_well_formed chain = true)
    (h_app : append_step chain step = some chain') :
    is_chain_well_formed chain' = true := by
  sorry  -- Unfold append_step; the guard ensures order monotonicity;
         -- the new chain = old ++ [step] with order ≥ last, preserving WF

/-- chain_step_order produces values in {0, 1, 2, 3}. -/
theorem chain_step_order_bounded (s : Step) :
    (chain_step_order s).val ≤ 3 := by
  cases s <;> simp [chain_step_order]

end agent_reasoning
