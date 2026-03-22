//! Reasoning chain: the Observe-Think-Decide-Act loop.
//!
//! A *reasoning chain* is a sequence of `Step` values whose *order tags*
//! (Observe=0, Think=1, Decide=2, Act=3) are monotonically non-decreasing.
//! This models the agent progressing through observation, deliberation,
//! decision, and action without going backwards.

/// A decision produced during the Decide phase.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Decision {
    CallTool { tool_name_idx: u32, args_hash: u32 },
    Respond,
    AskClarification,
    GiveUp,
}

/// One step in the chain-of-thought reasoning.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Step {
    /// Observation — ingesting new information.  Payload is an opaque id.
    Observe(u32),
    /// Internal deliberation.  Payload is a thought id.
    Think(u32),
    /// A concrete decision.
    Decide(Decision),
    /// Execution of an action.  Payload is an action id.
    Act(u32),
}

/// Map a step to its phase-order tag: Observe=0, Think=1, Decide=2, Act=3.
pub fn chain_step_order(step: &Step) -> u32 {
    match step {
        Step::Observe(_) => 0,
        Step::Think(_) => 1,
        Step::Decide(_) => 2,
        Step::Act(_) => 3,
    }
}

/// Check that a reasoning chain is *well-formed*: the order tags are
/// monotonically non-decreasing.
pub fn is_chain_well_formed(chain: &[Step]) -> bool {
    if chain.is_empty() {
        return true;
    }
    let mut i: usize = 1;
    while i < chain.len() {
        if chain_step_order(&chain[i]) < chain_step_order(&chain[i - 1]) {
            return false;
        }
        i += 1;
    }
    true
}

/// Append `step` to `chain` only if the result remains well-formed.
///
/// Returns `true` if the step was appended, `false` if it was rejected.
pub fn append_step(chain: &mut Vec<Step>, step: Step) -> bool {
    if let Some(last) = chain.last() {
        if chain_step_order(&step) < chain_step_order(last) {
            return false;
        }
    }
    chain.push(step);
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_step_order() {
        assert_eq!(chain_step_order(&Step::Observe(0)), 0);
        assert_eq!(chain_step_order(&Step::Think(0)), 1);
        assert_eq!(chain_step_order(&Step::Decide(Decision::Respond)), 2);
        assert_eq!(chain_step_order(&Step::Act(0)), 3);
    }

    #[test]
    fn test_empty_chain_well_formed() {
        assert!(is_chain_well_formed(&[]));
    }

    #[test]
    fn test_single_step_well_formed() {
        assert!(is_chain_well_formed(&[Step::Think(1)]));
    }

    #[test]
    fn test_monotonic_chain_well_formed() {
        let chain = vec![
            Step::Observe(1),
            Step::Think(2),
            Step::Think(3),
            Step::Decide(Decision::Respond),
            Step::Act(4),
        ];
        assert!(is_chain_well_formed(&chain));
    }

    #[test]
    fn test_non_monotonic_chain_rejected() {
        let chain = vec![
            Step::Think(1),
            Step::Observe(2),  // goes backwards
        ];
        assert!(!is_chain_well_formed(&chain));
    }

    #[test]
    fn test_append_valid_step() {
        let mut chain = vec![Step::Observe(1)];
        assert!(append_step(&mut chain, Step::Think(2)));
        assert_eq!(chain.len(), 2);
        assert!(is_chain_well_formed(&chain));
    }

    #[test]
    fn test_append_equal_order_step() {
        let mut chain = vec![Step::Think(1)];
        assert!(append_step(&mut chain, Step::Think(2)));
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_append_rejected_step() {
        let mut chain = vec![Step::Act(1)];
        assert!(!append_step(&mut chain, Step::Observe(2)));
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_append_to_empty() {
        let mut chain: Vec<Step> = Vec::new();
        assert!(append_step(&mut chain, Step::Decide(Decision::GiveUp)));
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_append_preserves_well_formedness() {
        let mut chain = vec![Step::Observe(0), Step::Think(1)];
        assert!(is_chain_well_formed(&chain));
        assert!(append_step(&mut chain, Step::Decide(Decision::AskClarification)));
        assert!(is_chain_well_formed(&chain));
        assert!(append_step(&mut chain, Step::Act(5)));
        assert!(is_chain_well_formed(&chain));
    }
}
