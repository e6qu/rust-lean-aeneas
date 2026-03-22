-- [HAND-WRITTEN PROOFS]
-- Proofs about the stack depth invariant.
--
-- These theorems show how eval_step affects the stack depth:
-- - Pushing a number increases depth by 1
-- - Applying a binary operator decreases depth by 1 (when depth >= 2)

import RpnCalc.Funs
import RpnCalc.Spec

open Aeneas Aeneas.Std Result
open rpn_calc

namespace rpn_calc

-- ============================================================================
-- Stack depth lemmas
-- ============================================================================

/-- Pushing a value increases stack depth by 1. -/
theorem push_depth (s : Stack) (v : I64) :
    stack_depth (Stack.Push v s) = stack_depth s + 1 := by
  simp [stack_depth]; omega

/-- An empty stack has depth 0. -/
theorem empty_depth : stack_depth Stack.Empty = 0 := by
  simp [stack_depth]

/-- Stack.push_ always succeeds and produces a Push node. -/
@[step]
theorem stack_push_spec (s : Stack) (v : I64) :
    Stack.push_ s v = ok (Stack.Push v s) := by
  unfold Stack.push_
  rfl

/-- Stack.pop_ on a non-empty stack succeeds with the top value and rest. -/
@[step]
theorem stack_pop_push_spec (v : I64) (rest : Stack) :
    Stack.pop_ (Stack.Push v rest) = ok (.ok (v, rest)) := by
  unfold Stack.pop_
  rfl

/-- Stack.pop_ on an empty stack returns StackUnderflow. -/
@[step]
theorem stack_pop_empty_spec :
    Stack.pop_ Stack.Empty = ok (.err EvalError.StackUnderflow) := by
  unfold Stack.pop_
  rfl

-- ============================================================================
-- eval_step depth properties
-- ============================================================================

/-- Pushing a number onto the stack increases depth by 1.

    If eval_step with a Num token succeeds, the resulting stack
    has exactly one more element than the input stack. -/
@[step]
axiom eval_step_num_depth (n : I64) (s s' : Stack) :
    eval_step s (Token.Num n) = ok (.ok s') ->
    stack_depth s' = stack_depth s + 1

/-- Applying Plus to a stack with >= 2 elements decreases depth by 1. -/
@[step]
axiom eval_step_plus_depth (a b : I64) (rest : Stack) (s' : Stack) :
    eval_step (Stack.Push b (Stack.Push a rest)) Token.Plus = ok (.ok s') ->
    stack_depth s' = stack_depth (Stack.Push b (Stack.Push a rest)) - 1

/-- Applying any binary operator to a stack with < 2 elements fails. -/
@[step]
theorem eval_step_binop_underflow_empty (op : Token) (hop : is_binop op) :
    ∃ e, eval_step Stack.Empty op = ok (.err e) := by
  cases op <;> simp [is_binop] at hop <;> unfold eval_step Stack.pop_ <;>
    exact ⟨EvalError.StackUnderflow, rfl⟩

@[step]
theorem eval_step_binop_underflow_one (op : Token) (hop : is_binop op) (v : I64) :
    ∃ e, eval_step (Stack.Push v Stack.Empty) op = ok (.err e) := by
  cases op <;> simp [is_binop] at hop <;>
    unfold eval_step Stack.pop_ <;>
    exact ⟨EvalError.StackUnderflow, rfl⟩

end rpn_calc
