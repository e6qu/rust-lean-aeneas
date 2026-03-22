-- [HAND-WRITTEN PROOFS]
-- Proofs about parser termination: each parsing function advances the position.
-- This is the key property needed to show the parser always terminates on
-- finite input — the position strictly increases, and it is bounded by
-- tokens.length, giving a well-founded recursion argument.

import InfixCalc.Funs

open Aeneas Aeneas.Std Result
open infix_calc

namespace infix_calc.ParserProofs

-- ============================================================================
-- parse_factor always advances the position
-- ============================================================================

/-- parse_factor, when successful, always advances the position.
    This is the base case for our termination argument.

    Case analysis:
    - Token.Num n: returns pos + 1, which is > pos.
    - Token.LParen: parses a sub-expression and consumes ')',
      so final position is at least pos + 2.

    This property is what makes the parser well-founded: every call
    to parse_factor consumes at least one token. -/
@[step]
theorem parse_factor_advances
    (tokens : Vec Token) (pos : Usize) (expr : Expr) (pos' : Usize)
    (h_ok : parse_factor tokens pos = ok (.ok (expr, pos')))
    : (↑pos' : Int) > ↑pos := by
  -- This theorem captures the essential termination argument.
  -- In a full verification, we would unfold parse_factor and case-split
  -- on the token at position pos.
  -- For the tutorial, we state the theorem and sketch the proof structure.
  sorry

-- ============================================================================
-- parse_term always advances the position
-- ============================================================================

/-- parse_term, when successful, always advances the position.
    This follows from parse_factor_advances, since parse_term calls
    parse_factor first, then only extends the result. -/
@[step]
theorem parse_term_advances
    (tokens : Vec Token) (pos : Usize) (expr : Expr) (pos' : Usize)
    (h_ok : parse_term tokens pos = ok (.ok (expr, pos')))
    : (↑pos' : Int) > ↑pos := by
  -- parse_term calls parse_factor first, which advances by ≥ 1.
  -- The loop (parse_term_loop) only moves forward (each iteration
  -- calls parse_factor again), so the final position is ≥ the
  -- position after the first parse_factor call.
  sorry

-- ============================================================================
-- parse_expr always advances the position
-- ============================================================================

/-- parse_expr, when successful, always advances the position.
    Follows the same structure as parse_term_advances. -/
@[step]
theorem parse_expr_advances
    (tokens : Vec Token) (pos : Usize) (expr : Expr) (pos' : Usize)
    (h_ok : parse_expr tokens pos = ok (.ok (expr, pos')))
    : (↑pos' : Int) > ↑pos := by
  -- parse_expr calls parse_term first (which calls parse_factor),
  -- guaranteeing advancement by ≥ 1. The loop (parse_expr_loop)
  -- only extends further.
  sorry

-- ============================================================================
-- Position bounds
-- ============================================================================

/-- The parser never advances past the end of the token list. -/
theorem parse_factor_bounded
    (tokens : Vec Token) (pos : Usize) (expr : Expr) (pos' : Usize)
    (h_ok : parse_factor tokens pos = ok (.ok (expr, pos')))
    : (↑pos' : Int) ≤ ↑tokens.len := by
  sorry

/-- Combining advancement and boundedness gives well-founded termination:
    each recursive call to parse_expr (via parse_factor) strictly decreases
    the quantity (tokens.length - pos), which is a natural number. -/
theorem parse_termination_measure
    (tokens : Vec Token) (pos : Usize) (expr : Expr) (pos' : Usize)
    (h_ok : parse_factor tokens pos = ok (.ok (expr, pos')))
    (h_bound : (↑pos : Int) < ↑tokens.len)
    : (↑tokens.len : Int) - ↑pos' < ↑tokens.len - ↑pos := by
  have h_adv := parse_factor_advances tokens pos expr pos' h_ok
  omega

end infix_calc.ParserProofs
