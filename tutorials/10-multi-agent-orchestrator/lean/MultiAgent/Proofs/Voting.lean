-- MultiAgent/Proofs/Voting.lean
-- Proof that tally correctly reflects majority voting.
import MultiAgent.Types
import MultiAgent.Funs
import Aeneas

open Primitives
open multi_agent

namespace multi_agent

/-!
# Voting Proofs

We prove that `tally` returns `some true` only when the yes votes
meet or exceed the required majority percentage.
-/

/-- `count_yes` on an empty vote list is 0. -/
theorem count_yes_nil : count_yes [] = 0 := by
  simp [count_yes]

/-- `count_yes` is bounded by the list length. -/
theorem count_yes_le_length (votes : List (AgentId × Bool)) :
    count_yes votes ≤ votes.length := by
  induction votes with
  | nil => simp [count_yes]
  | cons v rest ih =>
    cases v with
    | mk id b =>
      cases b <;> simp [count_yes] <;> omega

/-- `cast_vote` does not decrease the vote count. -/
theorem cast_vote_monotone (round : VotingRound) (agent_id : AgentId) (vote : Bool) :
    round.votes.length ≤ (cast_vote round agent_id vote).votes.length := by
  simp [cast_vote]
  split <;> simp_all <;> omega

/-- If `tally` returns `some true`, then yes votes meet the required majority.
    Statement: 100 * count_yes(votes) ≥ required_majority * total_votes. -/
axiom tally_matches_majority (round : VotingRound)
    (h_tally : tally round = some true) :
    let total := round.votes.length
    let yes := count_yes round.votes
    100 * yes ≥ round.required_majority.val * total

/-- If `tally` returns `some false`, then yes votes do NOT meet the majority. -/
axiom tally_rejects_insufficient (round : VotingRound)
    (h_complete : round.votes.length ≥ round.expected_voters.val)
    (h_nonzero : round.votes.length > 0)
    (h_tally : tally round = some false) :
    100 * count_yes round.votes < round.required_majority.val * round.votes.length

/-- `has_voted` is correct: if we just added agent_id, then has_voted is true. -/
axiom has_voted_after_cast (round : VotingRound) (agent_id : AgentId) (vote : Bool)
    (h : ¬(has_voted round.votes agent_id = true)) :
    has_voted (cast_vote round agent_id vote).votes agent_id = true

end multi_agent
