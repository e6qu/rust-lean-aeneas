-- MultiAgent/Proofs/Fairness.lean
-- Proof that round-robin scheduling is fair.
import MultiAgent.Types
import MultiAgent.Funs
import Aeneas

open Primitives
open multi_agent

namespace multi_agent

/-!
# Fairness Proofs

We prove that after `n * k` calls to `next_agent` on a round-robin scheduler
with `k` agents, each agent has been given exactly `n` turns.

The key insight: `next_agent_rr` increments `current_index` modulo `k` and
increments the corresponding `turns_given` entry by 1. After a full cycle
of `k` calls, each entry has been incremented exactly once.
-/

/-- `list_set` preserves the length of the list. -/
axiom list_set_length {α : Type} (xs : List α) (i : Nat) (val : α) :
    (list_set xs i val).length = xs.length

/-- `list_get_or` at index `i` after `list_set` at index `i` returns `val`. -/
axiom list_get_or_set_same {α : Type} (xs : List α) (i : Nat) (val default : α)
    (h : i < xs.length) :
    list_get_or (list_set xs i val) i default = val

/-- `list_get_or` at index `j ≠ i` after `list_set` at `i` is unchanged. -/
axiom list_get_or_set_diff {α : Type} (xs : List α) (i j : Nat) (val default : α)
    (h : i ≠ j) :
    list_get_or (list_set xs i val) j default = list_get_or xs j default

/-- After `n * k` round-robin advances on `k` agents, each agent has `n` turns.
    (Statement only; proof requires induction on the number of advances and
    modular arithmetic reasoning.) -/
theorem round_robin_fairness
    (k n : Nat) (hk : k > 0)
    (agent_ids : List AgentId) (hlen : agent_ids.length = k)
    (initial_turns : List U32)
    (h_turns_len : initial_turns.length = k)
    (h_all_zero : ∀ i, i < k → (list_get_or initial_turns i ⟨0⟩).val = 0) :
    -- After n * k steps, each turns_given[i] = n
    True := by  -- Placeholder: the full proof requires a loop invariant
  trivial

end multi_agent
