use crate::agent_trait::AgentId;

/// State of a bounded debate protocol.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DebateState {
    pub topic_id: u32,
    pub rounds_remaining: u32,
    pub max_rounds: u32,
    pub arguments: Vec<(AgentId, u32)>,
}

/// Create a new debate with a given number of rounds.
pub fn debate_new(topic_id: u32, max_rounds: u32) -> DebateState {
    DebateState {
        topic_id,
        rounds_remaining: max_rounds,
        max_rounds,
        arguments: Vec::new(),
    }
}

/// Add an argument to the debate and decrement the round counter.
/// If rounds_remaining is already 0, this is a no-op.
pub fn debate_step(state: &mut DebateState, agent_id: AgentId, argument_id: u32) {
    if state.rounds_remaining == 0 {
        return;
    }
    state.arguments.push((agent_id, argument_id));
    state.rounds_remaining -= 1;
}

/// Returns true if the debate has ended (no rounds remaining).
pub fn debate_is_finished(state: &DebateState) -> bool {
    state.rounds_remaining == 0
}

/// Returns the number of arguments submitted so far.
pub fn debate_argument_count(state: &DebateState) -> u32 {
    state.arguments.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debate_terminates() {
        let mut debate = debate_new(42, 3);
        assert!(!debate_is_finished(&debate));

        debate_step(&mut debate, 1, 100);
        debate_step(&mut debate, 2, 101);
        debate_step(&mut debate, 1, 102);
        assert!(debate_is_finished(&debate));
        assert_eq!(debate_argument_count(&debate), 3);
    }

    #[test]
    fn test_debate_noop_after_finished() {
        let mut debate = debate_new(42, 1);
        debate_step(&mut debate, 1, 100);
        assert!(debate_is_finished(&debate));

        // Further steps are no-ops
        debate_step(&mut debate, 2, 101);
        assert_eq!(debate_argument_count(&debate), 1);
    }

    #[test]
    fn test_debate_zero_rounds() {
        let debate = debate_new(42, 0);
        assert!(debate_is_finished(&debate));
    }
}
