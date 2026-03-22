-- AgentReasoning/Proofs/Guardrails.lean
-- Proofs about the guardrail checks.
import AgentReasoning.Types
import AgentReasoning.Funs
import Aeneas

open Primitives
open agent_reasoning

namespace agent_reasoning

/-!
# Guardrails Proofs

## all_guards_pass_implies_within_limits
If `all_guards_pass` returns `true`, then each individual limit is satisfied.
This is the conjunction-elimination direction of the guardrails.
-/

/-- If all guards pass, then message length is within limits. -/
theorem all_guards_pass_implies_message_len
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    check_message_length msg_len config = true := by
  simp [all_guards_pass] at h; exact h.1

/-- If all guards pass, then recursion depth is within limits. -/
theorem all_guards_pass_implies_depth
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    check_recursion_depth depth config = true := by
  simp [all_guards_pass] at h; exact h.2.1

/-- If all guards pass, then reasoning steps are within limits. -/
theorem all_guards_pass_implies_steps
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    check_reasoning_steps steps config = true := by
  simp [all_guards_pass] at h; exact h.2.2

/-- Full conjunction elimination: all guards pass implies every limit is met. -/
theorem all_guards_pass_implies_within_limits
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    msg_len.val ≤ config.max_message_len.val ∧
    depth.val ≤ config.max_recursion_depth.val ∧
    steps.val < config.max_reasoning_steps.val := by
  simp [all_guards_pass, check_message_length, check_recursion_depth,
        check_reasoning_steps] at h
  exact h

end agent_reasoning
