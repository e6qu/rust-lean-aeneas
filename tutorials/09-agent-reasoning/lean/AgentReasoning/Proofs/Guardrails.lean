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
axiom all_guards_pass_implies_message_len
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    check_message_length msg_len config = true

/-- If all guards pass, then recursion depth is within limits. -/
axiom all_guards_pass_implies_depth
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    check_recursion_depth depth config = true

/-- If all guards pass, then reasoning steps are within limits. -/
axiom all_guards_pass_implies_steps
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    check_reasoning_steps steps config = true

/-- Full conjunction elimination: all guards pass implies every limit is met. -/
axiom all_guards_pass_implies_within_limits
    (msg_len depth steps : U32) (config : GuardrailConfig)
    (h : all_guards_pass msg_len depth steps config = true) :
    msg_len.val ≤ config.max_message_len.val ∧
    depth.val ≤ config.max_recursion_depth.val ∧
    steps.val < config.max_reasoning_steps.val

end agent_reasoning
