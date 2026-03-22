-- [HAND-WRITTEN SPECIFICATION]
-- Mathematical specifications for the infix calculator.
-- These define the "ground truth" that our proofs verify against.

import InfixCalc.Types

open infix_calc

namespace infix_calc.Spec

/-- Mathematical semantics of an expression.
    This is the "intended meaning" — a total function from Expr to Option Int.
    Returns none on division by zero (the only mathematical error;
    mathematical integers cannot overflow). -/
def expr_semantics : Expr → Option Int
  | Expr.Num n => some ↑n
  | Expr.BinOp op left right =>
    match expr_semantics left, expr_semantics right with
    | some l, some r =>
      match op with
      | Op.Add => some (l + r)
      | Op.Sub => some (l - r)
      | Op.Mul => some (l * r)
      | Op.Div =>
        if r = 0 then none
        else some (l / r)
    | _, _ => none

/-- Depth of an expression tree.
    Useful for induction measures and complexity reasoning. -/
def expr_depth : Expr → Nat
  | Expr.Num _ => 0
  | Expr.BinOp _ left right => 1 + max (expr_depth left) (expr_depth right)

/-- All intermediate results of evaluating the expression fit in i64 range.
    This is the precondition needed to ensure the Rust evaluator does not overflow. -/
def expr_in_i64_range : Expr → Prop
  | Expr.Num _ => True  -- Num values are already I64, so always in range
  | Expr.BinOp op left right =>
    expr_in_i64_range left ∧
    expr_in_i64_range right ∧
    match expr_semantics (Expr.BinOp op left right) with
    | some v => I64.min ≤ v ∧ v ≤ I64.max
    | none => True  -- division by zero is a separate concern

/-- No division by zero anywhere in the expression tree.
    This ensures expr_semantics returns some value. -/
def expr_no_div_zero : Expr → Prop
  | Expr.Num _ => True
  | Expr.BinOp op left right =>
    expr_no_div_zero left ∧
    expr_no_div_zero right ∧
    match op with
    | Op.Div =>
      match expr_semantics right with
      | some v => v ≠ 0
      | none => False  -- right side already has div-by-zero
    | _ => True

/-- If there are no divisions by zero, expr_semantics returns a value. -/
theorem expr_semantics_defined (e : Expr) (h : expr_no_div_zero e) :
    ∃ v, expr_semantics e = some v := by
  induction e with
  | Num n => exact ⟨↑n, rfl⟩
  | BinOp op left right ih_left ih_right =>
    simp [expr_no_div_zero] at h
    obtain ⟨h_left, h_right, h_op⟩ := h
    obtain ⟨vl, hvl⟩ := ih_left h_left
    obtain ⟨vr, hvr⟩ := ih_right h_right
    simp [expr_semantics, hvl, hvr]
    cases op <;> simp [h_op, hvr]
    · -- Div case
      simp [expr_semantics, hvr] at h_op
      exact ⟨vl / vr, by simp [h_op]⟩

end infix_calc.Spec
