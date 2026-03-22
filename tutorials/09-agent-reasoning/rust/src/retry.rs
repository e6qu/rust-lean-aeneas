//! Retry logic with exponential backoff.
//!
//! `RetryState` tracks the current attempt and computed delay.
//! `next_retry` bumps the attempt counter and doubles the delay (capped).

/// Mutable retry state carried across attempts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RetryState {
    pub attempt: u32,
    pub delay_ms: u32,
    pub max_attempts: u32,
    pub base_delay_ms: u32,
    pub max_delay_ms: u32,
}

/// Create the initial retry state (attempt 0, delay = base).
pub fn initial_retry_state(max_attempts: u32, base_delay_ms: u32, max_delay_ms: u32) -> RetryState {
    RetryState {
        attempt: 0,
        delay_ms: base_delay_ms,
        max_attempts,
        base_delay_ms,
        max_delay_ms,
    }
}

/// Advance to the next retry.  Returns `None` if `attempt >= max_attempts`.
///
/// The delay doubles each time, capped at `max_delay_ms`.
pub fn next_retry(state: &RetryState) -> Option<RetryState> {
    if state.attempt >= state.max_attempts {
        return None;
    }
    let new_attempt = state.attempt + 1;
    let doubled = state.delay_ms.saturating_mul(2);
    let new_delay = if doubled > state.max_delay_ms {
        state.max_delay_ms
    } else {
        doubled
    };
    Some(RetryState {
        attempt: new_attempt,
        delay_ms: new_delay,
        max_attempts: state.max_attempts,
        base_delay_ms: state.base_delay_ms,
        max_delay_ms: state.max_delay_ms,
    })
}

/// Returns `true` if another retry is allowed.
pub fn should_retry(state: &RetryState) -> bool {
    state.attempt < state.max_attempts
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let s = initial_retry_state(3, 100, 1000);
        assert_eq!(s.attempt, 0);
        assert_eq!(s.delay_ms, 100);
        assert!(should_retry(&s));
    }

    #[test]
    fn test_next_retry_increases_attempt() {
        let s = initial_retry_state(5, 100, 10000);
        let s2 = next_retry(&s).unwrap();
        assert_eq!(s2.attempt, 1);
    }

    #[test]
    fn test_delay_doubles() {
        let s = initial_retry_state(5, 100, 10000);
        let s2 = next_retry(&s).unwrap();
        assert_eq!(s2.delay_ms, 200);
        let s3 = next_retry(&s2).unwrap();
        assert_eq!(s3.delay_ms, 400);
    }

    #[test]
    fn test_delay_capped() {
        let s = initial_retry_state(10, 500, 1000);
        let s2 = next_retry(&s).unwrap(); // 1000
        assert_eq!(s2.delay_ms, 1000);
        let s3 = next_retry(&s2).unwrap(); // still 1000 (capped)
        assert_eq!(s3.delay_ms, 1000);
    }

    #[test]
    fn test_retry_stops_at_max() {
        let s = initial_retry_state(2, 100, 10000);
        let s2 = next_retry(&s).unwrap(); // attempt 1
        let s3 = next_retry(&s2).unwrap(); // attempt 2
        assert!(!should_retry(&s3));
        assert_eq!(next_retry(&s3), None);
    }

    #[test]
    fn test_zero_max_attempts() {
        let s = initial_retry_state(0, 100, 1000);
        assert!(!should_retry(&s));
        assert_eq!(next_retry(&s), None);
    }
}
