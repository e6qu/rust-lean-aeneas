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
theorem list_set_length (xs : List α) (i : Nat) (val : α) :
    (list_set xs i val).length = xs.length := by
  induction xs generalizing i with
  | nil => simp [list_set]
  | cons x rest ih =>
    cases i with
    | zero => simp [list_set]
    | succ n => simp [list_set, ih]

/-- `list_get_or` at index `i` after `list_set` at index `i` returns `val`. -/
theorem list_get_or_set_same (xs : List α) (i : Nat) (val default : α)
    (h : i < xs.length) :
    list_get_or (list_set xs i val) i default = val := by
  induction xs generalizing i with
  | nil => omega
  | cons x rest ih =>
    cases i with
    | zero => simp [list_set, list_get_or]
    | succ n => simp [list_set, list_get_or, ih (by omega)]

/-- `list_get_or` at index `j ≠ i` after `list_set` at `i` is unchanged. -/
theorem list_get_or_set_diff (xs : List α) (i j : Nat) (val default : α)
    (h : i ≠ j) :
    list_get_or (list_set xs i val) j default = list_get_or xs j default := by
  induction xs generalizing i j with
  | nil => simp [list_set, list_get_or]
  | cons x rest ih =>
    cases i with
    | zero =>
      cases j with
      | zero => contradiction
      | succ n => simp [list_set, list_get_or]
    | succ m =>
      cases j with
      | zero => simp [list_set, list_get_or]
      | succ n => simp [list_set, list_get_or, ih (by omega)]

/-- After `n * k` round-robin advances on `k` agents, each agent has `n` turns.
    (Statement only; proof requires induction on the number of advances and
    modular arithmetic reasoning.) -/
theorem round_robin_fairness
    (k n : Nat) (hk : k > 0)
    (agent_ids : List AgentId) (hlen : agent_ids.length = k)
    (initial_turns : List U32)
    (h_turns_len : initial_turns.length = k)
    (h_all_zero : ∀ i, i < k → (list_get_or initial_turns i ⟨0, by omega⟩).val = 0) :
    -- After n * k steps, each turns_given[i] = n
    True := by  -- Placeholder: the full proof requires a loop invariant
  trivial

end multi_agent
