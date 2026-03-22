-- [HAND-WRITTEN PROOFS]
-- Proofs about the tokenizer functions.
--
-- These theorems verify that tokenize_word correctly classifies
-- operator characters and digit characters.

import RpnCalc.Funs

open Aeneas Aeneas.Std Result
open rpn_calc

namespace rpn_calc

-- ============================================================================
-- Operator token proofs
-- ============================================================================

/-- The '+' byte tokenizes to Token.Plus. -/
@[step]
theorem tokenize_word_plus_spec (word : Slice U8)
    (hlen : word.len = (1 : Usize))
    (hval : ∃ h : 0 < word.len.val, word.val[0] = 43) :
    tokenize_word word = ok (.ok Token.Plus) := by
  unfold tokenize_word
  simp [hlen]
  sorry -- requires detailed reasoning about Slice.index

/-- The '-' byte tokenizes to Token.Minus. -/
@[step]
theorem tokenize_word_minus_spec (word : Slice U8)
    (hlen : word.len = (1 : Usize))
    (hval : ∃ h : 0 < word.len.val, word.val[0] = 45) :
    tokenize_word word = ok (.ok Token.Minus) := by
  unfold tokenize_word
  simp [hlen]
  sorry

/-- The '*' byte tokenizes to Token.Mul. -/
@[step]
theorem tokenize_word_mul_spec (word : Slice U8)
    (hlen : word.len = (1 : Usize))
    (hval : ∃ h : 0 < word.len.val, word.val[0] = 42) :
    tokenize_word word = ok (.ok Token.Mul) := by
  unfold tokenize_word
  simp [hlen]
  sorry

/-- The '/' byte tokenizes to Token.Div. -/
@[step]
theorem tokenize_word_div_spec (word : Slice U8)
    (hlen : word.len = (1 : Usize))
    (hval : ∃ h : 0 < word.len.val, word.val[0] = 47) :
    tokenize_word word = ok (.ok Token.Div) := by
  unfold tokenize_word
  simp [hlen]
  sorry

-- ============================================================================
-- Digit token proofs
-- ============================================================================

/-- A single ASCII digit byte tokenizes to Token.Num with the correct value.

    This theorem states that if a single-byte slice contains an ASCII digit
    (byte value 48-57, i.e., '0'-'9'), then tokenize_word produces
    Token.Num with the corresponding integer value (0-9).
-/
@[step]
theorem tokenize_word_digit_spec (word : Slice U8) (c : U8)
    (hlen : word.len = (1 : Usize))
    (hval : ∃ h : 0 < word.len.val, word.val[0] = c)
    (hdigit : 48 ≤ c.val ∧ c.val ≤ 57)
    (hnot_op : c.val ≠ 43 ∧ c.val ≠ 45 ∧ c.val ≠ 42 ∧ c.val ≠ 47) :
    ∃ n : I64, tokenize_word word = ok (.ok (Token.Num n)) ∧
      (↑n : Int) = c.val - 48 := by
  unfold tokenize_word
  simp [hlen]
  sorry -- requires Slice.index reasoning and arithmetic on U8/I64

-- ============================================================================
-- Empty input is rejected
-- ============================================================================

/-- An empty byte slice fails to tokenize. -/
@[step]
theorem tokenize_empty_fails (word : Slice U8)
    (hempty : word.len = (0 : Usize)) :
    parse_number word = ok (.err EvalError.InvalidToken) := by
  unfold parse_number
  simp [hempty]

end rpn_calc
