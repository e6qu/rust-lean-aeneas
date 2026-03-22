use crate::agent_trait::AgentId;

/// A voting round with majority threshold.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VotingRound {
    pub proposal_id: u32,
    pub votes: Vec<(AgentId, bool)>,
    pub required_majority: u32, // percentage 0-100
    pub expected_voters: u32,
}

/// Create a new voting round.
pub fn voting_new(proposal_id: u32, required_majority: u32, expected_voters: u32) -> VotingRound {
    VotingRound {
        proposal_id,
        votes: Vec::new(),
        required_majority,
        expected_voters,
    }
}

/// Cast a vote. If the agent has already voted, the vote is ignored.
pub fn cast_vote(round: &mut VotingRound, agent_id: AgentId, vote: bool) {
    for &(id, _) in &round.votes {
        if id == agent_id {
            return; // already voted
        }
    }
    round.votes.push((agent_id, vote));
}

/// Tally the votes.
///
/// Returns `Some(true)` if the yes votes meet or exceed the required majority,
/// `Some(false)` if enough votes are in that the majority cannot be reached,
/// or `None` if voting is still in progress.
pub fn tally(round: &VotingRound) -> Option<bool> {
    let total = round.votes.len() as u32;
    if total < round.expected_voters {
        return None; // voting still in progress
    }
    let yes_count = round.votes.iter().filter(|(_, v)| *v).count() as u32;
    // Check: 100 * yes_count >= required_majority * total
    // This avoids floating-point: yes_count / total >= required_majority / 100
    if total == 0 {
        return Some(false);
    }
    let passes = 100 * yes_count >= round.required_majority * total;
    Some(passes)
}

/// Returns the number of yes votes cast so far.
pub fn yes_count(round: &VotingRound) -> u32 {
    round.votes.iter().filter(|(_, v)| *v).count() as u32
}

/// Returns the number of no votes cast so far.
pub fn no_count(round: &VotingRound) -> u32 {
    round.votes.iter().filter(|(_, v)| !*v).count() as u32
}

/// Returns true if all expected voters have voted.
pub fn voting_is_complete(round: &VotingRound) -> bool {
    round.votes.len() as u32 >= round.expected_voters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voting_majority_pass() {
        let mut round = voting_new(1, 50, 3);
        cast_vote(&mut round, 1, true);
        cast_vote(&mut round, 2, true);
        assert_eq!(tally(&round), None); // still waiting
        cast_vote(&mut round, 3, false);
        assert_eq!(tally(&round), Some(true)); // 2/3 >= 50%
    }

    #[test]
    fn test_voting_majority_fail() {
        let mut round = voting_new(1, 75, 4);
        cast_vote(&mut round, 1, true);
        cast_vote(&mut round, 2, false);
        cast_vote(&mut round, 3, false);
        cast_vote(&mut round, 4, false);
        assert_eq!(tally(&round), Some(false)); // 1/4 = 25% < 75%
    }

    #[test]
    fn test_no_duplicate_votes() {
        let mut round = voting_new(1, 50, 2);
        cast_vote(&mut round, 1, true);
        cast_vote(&mut round, 1, false); // duplicate, should be ignored
        assert_eq!(round.votes.len(), 1);
    }

    #[test]
    fn test_unanimous() {
        let mut round = voting_new(1, 100, 3);
        cast_vote(&mut round, 1, true);
        cast_vote(&mut round, 2, true);
        cast_vote(&mut round, 3, true);
        assert_eq!(tally(&round), Some(true));
    }

    #[test]
    fn test_tally_matches_majority() {
        // If tally returns Some(true), then yes_count * 100 >= required_majority * total
        let mut round = voting_new(1, 60, 5);
        cast_vote(&mut round, 1, true);
        cast_vote(&mut round, 2, true);
        cast_vote(&mut round, 3, true);
        cast_vote(&mut round, 4, false);
        cast_vote(&mut round, 5, false);
        let result = tally(&round);
        assert_eq!(result, Some(true));
        let yc = yes_count(&round);
        let total = round.votes.len() as u32;
        assert!(100 * yc >= round.required_majority * total);
    }
}
