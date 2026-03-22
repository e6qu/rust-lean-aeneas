-- [HAND-WRITTEN PROOFS]
-- This file contains the proofs we write ourselves about the Aeneas-generated code.
-- These proofs demonstrate that our Rust functions are correct — not just that
-- they compile or pass tests, but that they satisfy precise mathematical properties
-- for ALL possible inputs.

import HelloProof.Funs

open Aeneas Aeneas.Std Result
open hello_proof

-- ============================================================================
-- SECTION 1: checked_add proofs
-- ============================================================================

/-- **Theorem: checked_add never panics.**

    No matter what u32 values you pass in, the function always returns `ok _`.
    The outer Aeneas Result is always successful — the function handles all
    edge cases internally (returning None for overflow instead of panicking).

    This is our very first theorem. Let's break down what it says:
    - `∀ (x y : U32)` — "for all unsigned 32-bit integers x and y"
    - `∃ r` — "there exists some result r"
    - `checked_add x y = ok r` — "calling checked_add gives ok (not fail)"

    Proof strategy: Unfold the definition and show both branches return `ok`.
-/
@[step]
theorem checked_add_no_panic (x y : U32) :
    ∃ r, checked_add x y = ok r := by
  unfold checked_add
  split
  · -- Case: y ≤ U32.MAX - x (no overflow)
    -- The subtraction U32.MAX - x succeeds because x ≤ U32.MAX
    -- The addition x + y succeeds because y ≤ U32.MAX - x
    simp only [bind_ok]
    progress as ⟨diff, hdiff⟩   -- U32.MAX - x = ok diff
    simp only [bind_ok]
    split
    · progress as ⟨sum, hsum⟩   -- x + y = ok sum
      exact ⟨some sum, rfl⟩
    · exact ⟨none, rfl⟩
  · -- Case: y > U32.MAX - x (overflow)
    simp only [bind_ok]
    progress as ⟨diff, hdiff⟩
    simp only [bind_ok]
    split
    · progress as ⟨sum, hsum⟩
      exact ⟨some sum, rfl⟩
    · exact ⟨none, rfl⟩

/-- **Theorem: checked_add is correct.**

    The function returns Some(sum) exactly when x + y fits in a u32,
    and None exactly when it overflows.

    Here `↑x` coerces the bounded U32 to a mathematical Nat for reasoning.
    `U32.max` is 2^32 - 1 = 4294967295.

    This is a *specification theorem* — it precisely characterizes the
    function's behavior for all inputs.
-/
@[step]
theorem checked_add_spec (x y : U32) :
    (↑x + ↑y ≤ U32.max →
      ∃ z, checked_add x y = ok (some z) ∧ (↑z : Int) = ↑x + ↑y) ∧
    (↑x + ↑y > U32.max →
      checked_add x y = ok none) := by
  constructor
  · -- Case: no overflow
    intro h_no_overflow
    unfold checked_add
    simp only [bind_ok]
    progress as ⟨diff, hdiff⟩
    simp only [bind_ok]
    split
    · progress as ⟨sum, hsum⟩
      exact ⟨sum, rfl, hsum⟩
    · -- Contradiction: we assumed no overflow but ended up in the else branch
      omega
  · -- Case: overflow
    intro h_overflow
    unfold checked_add
    simp only [bind_ok]
    progress as ⟨diff, hdiff⟩
    simp only [bind_ok]
    split
    · -- Contradiction: we assumed overflow but ended up in the then branch
      omega
    · rfl

-- ============================================================================
-- SECTION 2: safe_divide proofs
-- ============================================================================

/-- **Theorem: safe_divide with non-zero divisor succeeds.**

    If y ≠ 0, the function returns Ok(x / y) — the inner Result is .ok,
    and the division produces the correct mathematical quotient.

    Note the nested structure:
    - Outer `ok`: Aeneas says no panic occurred
    - Inner `.ok r`: Rust's Result says division succeeded
    - `↑r = ↑x / ↑y`: the result equals mathematical division
-/
@[step]
theorem safe_divide_nonzero (x y : I64) (hy : (↑y : Int) ≠ 0) :
    ∃ r, safe_divide x y = ok (.ok r) ∧ (↑r : Int) = ↑x / ↑y := by
  unfold safe_divide
  simp only [bind_ok]
  split
  · -- Case: y = 0 — contradicts our hypothesis hy
    rename_i h_zero
    simp at h_zero
    omega
  · -- Case: y ≠ 0 — division succeeds
    progress as ⟨result, hresult⟩
    exact ⟨result, rfl, hresult⟩

/-- **Theorem: safe_divide by zero returns Err.**

    If y = 0, the function returns Err(()) — correctly catching the error
    instead of panicking or producing undefined behavior.
-/
@[step]
theorem safe_divide_zero (x : I64) :
    safe_divide x (0 : I64) = ok (.err ()) := by
  unfold safe_divide
  simp

-- ============================================================================
-- SECTION 3: safe_abs proofs
-- ============================================================================

/-- **Theorem: safe_abs is correct for non-MIN inputs.**

    For any i64 value that is not i64::MIN, safe_abs returns Ok(|x|).
    The result equals the mathematical absolute value.
-/
@[step]
theorem safe_abs_correct (x : I64) (hx : (↑x : Int) ≠ I64.min) :
    ∃ r, safe_abs x = ok (.ok r) ∧ (↑r : Int) = Int.natAbs ↑x := by
  unfold safe_abs
  simp only [bind_ok]
  split
  · -- Case: x = I64.MIN — contradicts hypothesis
    rename_i h_min
    simp at h_min
    omega
  · split
    · -- Case: x < 0 — negate it
      rename_i h_neg
      progress as ⟨neg_x, hneg⟩
      refine ⟨neg_x, rfl, ?_⟩
      omega
    · -- Case: x ≥ 0 — return as-is
      rename_i h_nonneg
      refine ⟨x, rfl, ?_⟩
      simp at h_nonneg
      omega

/-- **Theorem: safe_abs correctly rejects i64::MIN.**
-/
@[step]
theorem safe_abs_min_rejected :
    safe_abs I64.MIN = ok (.err ()) := by
  unfold safe_abs
  simp

-- ============================================================================
-- SECTION 4: clamp proofs
-- ============================================================================

/-- **Theorem: clamp never fails.**

    The function always returns `ok` — no panics, no overflow.
    This is because clamp only uses comparisons, never arithmetic.
-/
@[step]
theorem clamp_no_fail (x lo hi : I32) :
    ∃ r, clamp x lo hi = ok r := by
  unfold clamp
  split
  · exact ⟨lo, rfl⟩
  · split
    · exact ⟨hi, rfl⟩
    · exact ⟨x, rfl⟩

/-- **Theorem: clamp result is always in [lo, hi].**

    Given the precondition that lo ≤ hi, the result is guaranteed
    to be within bounds. This is the key correctness property.

    Note: without the precondition `lo ≤ hi`, the property doesn't hold
    (e.g., clamp(5, 10, 0) would return 10, which is not ≤ 0).
-/
@[step]
theorem clamp_in_bounds (x lo hi : I32) (h : (↑lo : Int) ≤ ↑hi) :
    ∃ r, clamp x lo hi = ok r ∧ (↑lo : Int) ≤ ↑r ∧ (↑r : Int) ≤ ↑hi := by
  unfold clamp
  split
  · -- x < lo: return lo
    rename_i h_lt
    refine ⟨lo, rfl, le_refl _, ?_⟩
    exact h
  · split
    · -- x > hi: return hi
      rename_i h_not_lt h_gt
      refine ⟨hi, rfl, ?_, le_refl _⟩
      exact h
    · -- lo ≤ x ≤ hi: return x
      rename_i h_not_lt h_not_gt
      simp at h_not_lt h_not_gt
      exact ⟨x, rfl, by omega, by omega⟩

/-- **Theorem: clamp is idempotent.**

    If the value is already in range, clamp returns it unchanged.
-/
@[step]
theorem clamp_idempotent (x lo hi : I32)
    (h_lo : (↑lo : Int) ≤ ↑x) (h_hi : (↑x : Int) ≤ ↑hi) :
    clamp x lo hi = ok x := by
  unfold clamp
  split
  · rename_i h; simp at h; omega
  · split
    · rename_i h; simp at h; omega
    · rfl

-- ============================================================================
-- EXERCISES
-- ============================================================================

-- Exercise 1: Prove that max_of always succeeds (never fails).
-- theorem max_of_no_fail (a b : I32) : ∃ r, max_of a b = ok r := by
--   sorry

-- Exercise 2: Prove that max_of returns a value ≥ both inputs.
-- theorem max_of_spec (a b : I32) :
--     ∃ r, max_of a b = ok r ∧ (↑r : Int) ≥ ↑a ∧ (↑r : Int) ≥ ↑b := by
--   sorry

-- Exercise 3: Prove that min_of returns a value ≤ both inputs.
-- theorem min_of_spec (a b : I32) :
--     ∃ r, min_of a b = ok r ∧ (↑r : Int) ≤ ↑a ∧ (↑r : Int) ≤ ↑b := by
--   sorry

-- Exercise 4: Prove that max_of(a, b) ≥ min_of(a, b).
-- (Hint: use the specs from exercises 2 and 3)
-- theorem max_ge_min (a b : I32) :
--     ∃ mx mn, max_of a b = ok mx ∧ min_of a b = ok mn ∧ (↑mx : Int) ≥ ↑mn := by
--   sorry
