-- [HAND-WRITTEN SPECIFICATION]
-- Mathematical specification for the RPN calculator.
-- These definitions capture what it *means* for an RPN expression to be
-- well-formed and what its mathematical semantics are.

import RpnCalc.Types
import RpnCalc.Funs

open Aeneas Aeneas.Std
open rpn_calc

namespace rpn_calc

-- ============================================================================
-- Stack depth
-- ============================================================================

/-- The number of elements on the stack. -/
def stack_depth : Stack -> Nat
  | Stack.Empty => 0
  | Stack.Push _ rest => 1 + stack_depth rest

-- ============================================================================
-- Operator predicates
-- ============================================================================

/-- A token is a binary operator (not a number). -/
def is_binop : Token -> Prop
  | Token.Plus => True
  | Token.Minus => True
  | Token.Mul => True
  | Token.Div => True
  | Token.Num _ => False

instance : Decidable (is_binop t) := by
  cases t <;> simp [is_binop] <;> infer_instance

-- ============================================================================
-- Well-formed RPN expressions
-- ============================================================================

/-- An inductive definition of well-formed RPN token sequences.

    A well-formed RPN expression is either:
    - A single number: `[Num n]`
    - Two well-formed sub-expressions followed by a binary operator:
      `e1 ++ e2 ++ [op]` where `is_binop op`.

    This definition captures the structural invariant that guarantees
    evaluation will succeed (modulo division by zero). -/
inductive WellFormedRPN : List Token -> Prop where
  | num (n : I64) : WellFormedRPN [Token.Num n]
  | binop (e1 e2 : List Token) (op : Token)
      (h1 : WellFormedRPN e1)
      (h2 : WellFormedRPN e2)
      (hop : is_binop op) :
      WellFormedRPN (e1 ++ e2 ++ [op])

-- ============================================================================
-- Division-by-zero predicate
-- ============================================================================

/-- An expression has no division by zero if every Div operation's second
    operand (the sub-expression on top of the stack) is non-zero.
    For simplicity we define the "easy" version: no Div tokens at all. -/
def no_div_tokens : List Token -> Prop
  | [] => True
  | (Token.Div :: _) => False
  | (_ :: rest) => no_div_tokens rest

instance : Decidable (no_div_tokens ts) := by
  induction ts with
  | nil => simp [no_div_tokens]; infer_instance
  | cons t rest ih =>
    cases t <;> simp [no_div_tokens] <;> exact ih

end rpn_calc
