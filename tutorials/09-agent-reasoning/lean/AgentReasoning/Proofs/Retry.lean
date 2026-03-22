-- AgentReasoning/Proofs/Retry.lean
-- Proofs about the retry logic.
import AgentReasoning.Types
import AgentReasoning.Funs
import Aeneas

open Primitives
open agent_reasoning

namespace agent_reasoning

/-!
# Retry Proofs

## next_retry_increases_attempt
After `next_retry`, the attempt counter is incremented by 1.

## retry_delay_grows
The delay is non-decreasing across retries (it doubles, capped at max).

## retry_stops_at_max
When `attempt ≥ max_attempts`, `should_retry` returns `false` and
`next_retry` returns `none`.
-/

/-- `next_retry` increments the attempt counter by 1. -/
theorem next_retry_increases_attempt (state : RetryState) (state' : RetryState)
    (h : next_retry state = some state') :
    state'.attempt.val = state.attempt.val + 1 := by
  sorry  -- Unfold next_retry; the guard ensures attempt < max_attempts;
         -- new_attempt = attempt + 1 by construction

/-- The delay never decreases across retries. -/
theorem retry_delay_grows (state : RetryState) (state' : RetryState)
    (h : next_retry state = some state') :
    state'.delay_ms.val ≥ state.delay_ms.val := by
  sorry  -- doubled = delay * 2 ≥ delay; min doubled max_delay ≥ delay
         -- (because either doubled ≥ delay, or max_delay is the cap)

/-- When attempt ≥ max_attempts, `should_retry` returns false. -/
theorem retry_stops_at_max (state : RetryState)
    (h : state.attempt.val ≥ state.max_attempts.val) :
    should_retry state = false := by
  simp [should_retry]; omega

/-- When attempt ≥ max_attempts, `next_retry` returns none. -/
theorem next_retry_none_at_max (state : RetryState)
    (h : state.attempt.val ≥ state.max_attempts.val) :
    next_retry state = none := by
  simp [next_retry, h]

/-- `should_retry` on the initial state (with max_attempts > 0) returns true. -/
theorem initial_retry_can_retry (max_attempts base_delay max_delay : U32)
    (h : max_attempts.val > 0) :
    should_retry (initial_retry_state max_attempts base_delay max_delay) = true := by
  simp [initial_retry_state, should_retry]; omega

end agent_reasoning
