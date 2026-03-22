-- [buffer_management]: modular arithmetic utility lemmas
-- Reusable lemmas for reasoning about modular arithmetic, particularly
-- useful for ring buffer index calculations.
import Aeneas

open Aeneas Primitives

namespace buffer_management.ArithUtils

-- =========================================================================
-- Basic modular arithmetic properties
-- =========================================================================

/-- The result of `a % n` is strictly less than `n` when `n > 0`. -/
theorem mod_lt (a n : Nat) (hn : n > 0) : a % n < n := by
  exact Nat.mod_lt a hn

/-- `a % n` is at most `a` (the modulus never increases the value). -/
theorem mod_le (a n : Nat) (hn : n > 0) : a % n ≤ a := by
  exact Nat.mod_le a n

/-- When `a < n`, the modulus is the identity: `a % n = a`. -/
theorem mod_of_lt (a n : Nat) (h : a < n) : a % n = a := by
  exact Nat.mod_eq_of_lt h

-- =========================================================================
-- Addition with modular wrap-around
-- =========================================================================

/-- `(a + 1) % n < n` when `n > 0` — the key ring buffer index step. -/
theorem add_one_mod_lt (a n : Nat) (hn : n > 0) : (a + 1) % n < n := by
  exact Nat.mod_lt (a + 1) hn

/-- If `a < n` then `(a + 1) % n` is either `a + 1` (if `a + 1 < n`)
    or `0` (if `a + 1 = n`). -/
theorem add_one_mod_cases (a n : Nat) (hn : n > 0) (ha : a < n) :
    (a + 1) % n = if a + 1 < n then a + 1 else 0 := by
  split
  · case isTrue h => exact Nat.mod_eq_of_lt h
  · case isFalse h =>
    push_neg at h
    have hab : a + 1 = n := by omega
    rw [hab]
    exact Nat.mod_self n

/-- Modular addition distributes: `(a % n + b % n) % n = (a + b) % n`. -/
theorem mod_add_mod (a b n : Nat) (hn : n > 0) :
    (a % n + b % n) % n = (a + b) % n := by
  exact (Nat.add_mod a b n).symm

-- =========================================================================
-- Subtraction with modular wrap-around
-- =========================================================================

/-- If `a > 0` and `a ≤ n`, then `(a - 1) % n < n`. -/
theorem sub_one_mod_lt (a n : Nat) (hn : n > 0) (ha : a > 0) :
    (a - 1) % n < n := by
  exact Nat.mod_lt (a - 1) hn

-- =========================================================================
-- Useful combinations for ring buffer reasoning
-- =========================================================================

/-- After `capacity` steps of `(· + 1) % capacity`, we return to start. -/
theorem wrap_full_cycle (start capacity : Nat) (hc : capacity > 0)
    (hs : start < capacity) :
    (start + capacity) % capacity = start := by
  rw [Nat.add_mod, Nat.mod_self, Nat.add_zero]
  exact Nat.mod_eq_of_lt hs

/-- Two consecutive mod steps: `((a + 1) % n + 1) % n = (a + 2) % n`. -/
theorem mod_step_twice (a n : Nat) (hn : n > 0) :
    ((a + 1) % n + 1) % n = (a + 2) % n := by
  have h := mod_add_mod (a + 1) 1 n hn
  omega

end buffer_management.ArithUtils
