// Tutorial 01: Setup and Hello Proof
//
// Four simple functions that demonstrate the Aeneas translation pipeline.
// Each function is designed to produce interesting proofs in Lean 4.

/// Adds two u32 values with checked arithmetic.
/// Returns None on overflow, Some(result) otherwise.
///
/// This is our "hello world" of verified Rust: a function simple enough
/// to understand at a glance, but with a real correctness property
/// (never panics, correctly detects overflow).
pub fn checked_add(x: u32, y: u32) -> Option<u32> {
    if y <= u32::MAX - x {
        Some(x + y)
    } else {
        None
    }
}

/// Safe division: returns Err(()) on division by zero,
/// Ok(result) otherwise.
///
/// Demonstrates how Rust's Result type maps to Lean's nested Result:
/// Aeneas wraps everything in its own Result (for panics), and our
/// Result<i64, ()> becomes an inner Result inside that.
pub fn safe_divide(x: i64, y: i64) -> Result<i64, ()> {
    if y == 0 {
        Err(())
    } else {
        Ok(x / y)
    }
}

/// Absolute value that correctly handles the i64::MIN edge case.
/// Returns Err(()) for i64::MIN (whose absolute value cannot be
/// represented as i64), Ok(|x|) otherwise.
///
/// This teaches an important lesson: even "simple" functions can have
/// subtle edge cases that formal verification catches.
pub fn safe_abs(x: i64) -> Result<i64, ()> {
    if x == i64::MIN {
        Err(()) // -i64::MIN overflows i64
    } else if x < 0 {
        Ok(-x)
    } else {
        Ok(x)
    }
}

/// Clamps a value to the range [lo, hi].
/// Precondition: lo <= hi (not enforced at runtime, but required for the proof).
///
/// A pure function with no failure modes — the Aeneas Result will always
/// be Ok. This demonstrates that not every function needs error handling.
pub fn clamp(x: i32, lo: i32, hi: i32) -> i32 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

/// Returns the maximum of two i32 values.
/// Exercise function for the reader to prove correct.
pub fn max_of(a: i32, b: i32) -> i32 {
    if a >= b {
        a
    } else {
        b
    }
}

/// Returns the minimum of two i32 values.
/// Exercise function for the reader to prove correct.
pub fn min_of(a: i32, b: i32) -> i32 {
    if a <= b {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked_add_normal() {
        assert_eq!(checked_add(2, 3), Some(5));
        assert_eq!(checked_add(0, 0), Some(0));
        assert_eq!(checked_add(100, 200), Some(300));
    }

    #[test]
    fn test_checked_add_overflow() {
        assert_eq!(checked_add(u32::MAX, 1), None);
        assert_eq!(checked_add(u32::MAX, u32::MAX), None);
    }

    #[test]
    fn test_checked_add_boundary() {
        assert_eq!(checked_add(u32::MAX - 1, 1), Some(u32::MAX));
        assert_eq!(checked_add(u32::MAX, 0), Some(u32::MAX));
    }

    #[test]
    fn test_safe_divide_normal() {
        assert_eq!(safe_divide(10, 2), Ok(5));
        assert_eq!(safe_divide(-10, 2), Ok(-5));
        assert_eq!(safe_divide(0, 5), Ok(0));
    }

    #[test]
    fn test_safe_divide_by_zero() {
        assert_eq!(safe_divide(10, 0), Err(()));
        assert_eq!(safe_divide(0, 0), Err(()));
    }

    #[test]
    fn test_safe_abs_normal() {
        assert_eq!(safe_abs(5), Ok(5));
        assert_eq!(safe_abs(-5), Ok(5));
        assert_eq!(safe_abs(0), Ok(0));
    }

    #[test]
    fn test_safe_abs_min() {
        assert_eq!(safe_abs(i64::MIN), Err(()));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);   // in range
        assert_eq!(clamp(-5, 0, 10), 0);  // below
        assert_eq!(clamp(15, 0, 10), 10); // above
        assert_eq!(clamp(0, 0, 10), 0);   // at lower bound
        assert_eq!(clamp(10, 0, 10), 10); // at upper bound
    }

    #[test]
    fn test_max_of() {
        assert_eq!(max_of(3, 5), 5);
        assert_eq!(max_of(5, 3), 5);
        assert_eq!(max_of(4, 4), 4);
    }

    #[test]
    fn test_min_of() {
        assert_eq!(min_of(3, 5), 3);
        assert_eq!(min_of(5, 3), 3);
        assert_eq!(min_of(4, 4), 4);
    }
}
