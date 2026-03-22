//! Guardrails: safety limits that the agent must respect.
//!
//! Each check is a simple comparison.  `all_guards_pass` is the conjunction
//! of every individual check; it gates every agent step.

/// Configuration of safety limits.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GuardrailConfig {
    pub max_message_len: u32,
    pub max_recursion_depth: u32,
    pub max_reasoning_steps: u32,
}

/// Is the message length within limits?
pub fn check_message_length(message_len: u32, config: &GuardrailConfig) -> bool {
    message_len <= config.max_message_len
}

/// Is the recursion depth within limits?
pub fn check_recursion_depth(depth: u32, config: &GuardrailConfig) -> bool {
    depth <= config.max_recursion_depth
}

/// Is the number of reasoning steps within limits?
pub fn check_reasoning_steps(step_count: u32, config: &GuardrailConfig) -> bool {
    step_count < config.max_reasoning_steps
}

/// Do all guards pass?
pub fn all_guards_pass(
    message_len: u32,
    recursion_depth: u32,
    step_count: u32,
    config: &GuardrailConfig,
) -> bool {
    check_message_length(message_len, config)
        && check_recursion_depth(recursion_depth, config)
        && check_reasoning_steps(step_count, config)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> GuardrailConfig {
        GuardrailConfig {
            max_message_len: 4096,
            max_recursion_depth: 10,
            max_reasoning_steps: 50,
        }
    }

    #[test]
    fn test_message_length_ok() {
        assert!(check_message_length(100, &sample_config()));
    }

    #[test]
    fn test_message_length_exceeded() {
        assert!(!check_message_length(5000, &sample_config()));
    }

    #[test]
    fn test_recursion_depth_ok() {
        assert!(check_recursion_depth(5, &sample_config()));
    }

    #[test]
    fn test_recursion_depth_exceeded() {
        assert!(!check_recursion_depth(11, &sample_config()));
    }

    #[test]
    fn test_reasoning_steps_ok() {
        assert!(check_reasoning_steps(49, &sample_config()));
    }

    #[test]
    fn test_reasoning_steps_exceeded() {
        assert!(!check_reasoning_steps(50, &sample_config()));
    }

    #[test]
    fn test_all_guards_pass_ok() {
        assert!(all_guards_pass(100, 5, 10, &sample_config()));
    }

    #[test]
    fn test_all_guards_fail_on_message_len() {
        assert!(!all_guards_pass(5000, 5, 10, &sample_config()));
    }

    #[test]
    fn test_all_guards_fail_on_depth() {
        assert!(!all_guards_pass(100, 20, 10, &sample_config()));
    }

    #[test]
    fn test_all_guards_fail_on_steps() {
        assert!(!all_guards_pass(100, 5, 50, &sample_config()));
    }
}
