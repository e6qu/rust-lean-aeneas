-- MultiAgent/Funs.lean
-- Simulated Aeneas output: function translations for the multi-agent orchestrator.
import MultiAgent.Types
import Aeneas

open Primitives
open multi_agent

namespace multi_agent

/-! ## Message Bus -/

/-- Create an empty message bus. -/
def bus_new : MessageBus :=
  { queue := [], delivered := [], next_seq := ⟨0⟩ }

/-- Enqueue an envelope, assigning it the next sequence number. -/
def bus_send (bus : MessageBus) (env : Envelope) : Result MessageBus := do
  let env' := { env with sequence_num := bus.next_seq }
  let next ← bus.next_seq + (1 : U32)
  .ok { bus with queue := bus.queue ++ [env'], next_seq := next }

/-- Check whether an envelope targets a given agent (Direct addressing only). -/
def envelope_targets (env : Envelope) (agent_id : AgentId) : Bool :=
  match env.recipient with
  | .Direct id => id == agent_id
  | .Broadcast => true
  | .Topic _ => false

/-- Partition a list of envelopes by whether they target a given agent. -/
def partition_envelopes (queue : List Envelope) (agent_id : AgentId)
    : List Envelope × List Envelope :=
  match queue with
  | [] => ([], [])
  | env :: rest =>
    let (matching, remaining) := partition_envelopes rest agent_id
    if envelope_targets env agent_id then
      (env :: matching, remaining)
    else
      (matching, env :: remaining)

/-- Deliver all envelopes targeting agent_id from the bus queue. -/
def bus_deliver (bus : MessageBus) (agent_id : AgentId) : MessageBus × List Envelope :=
  let (matching, remaining) := partition_envelopes bus.queue agent_id
  ({ bus with queue := remaining, delivered := bus.delivered ++ matching }, matching)

/-- Returns true if the bus queue is empty. -/
def bus_is_empty (bus : MessageBus) : Bool :=
  bus.queue.isEmpty

/-! ## Router -/

/-- Create an empty router. -/
def router_new : Router :=
  { subscriptions := [] }

/-- Collect agent IDs subscribed to a given topic. -/
def topic_subscribers (subs : List TopicSubscription) (topic_id : U32) : List AgentId :=
  match subs with
  | [] => []
  | s :: rest =>
    if s.topic_id == topic_id then
      s.agent_id :: topic_subscribers rest topic_id
    else
      topic_subscribers rest topic_id

/-- Resolve a recipient to a list of concrete agent IDs. -/
def resolve_recipient (router : Router) (recipient : Recipient) (all_agents : List AgentId)
    : List AgentId :=
  match recipient with
  | .Direct id => [id]
  | .Broadcast => all_agents
  | .Topic tid => topic_subscribers router.subscriptions tid

/-! ## Scheduler -/

/-- Get the element at index `i` in a list, returning a default if out of bounds. -/
def list_get_or {α : Type} (xs : List α) (i : Nat) (default : α) : α :=
  match xs, i with
  | [], _ => default
  | x :: _, 0 => x
  | _ :: rest, n + 1 => list_get_or rest n default

/-- Set the element at index `i` in a list. -/
def list_set {α : Type} (xs : List α) (i : Nat) (val : α) : List α :=
  match xs, i with
  | [], _ => []
  | _ :: rest, 0 => val :: rest
  | x :: rest, n + 1 => x :: list_set rest n val

/-- Return the next agent for a round-robin scheduler and advance. -/
def next_agent_rr (sched : Scheduler) : Result (Option AgentId × Scheduler) := do
  if sched.agent_ids.isEmpty then
    .ok (none, sched)
  else
    let idx := sched.current_index.val
    let agent_id := list_get_or sched.agent_ids idx ⟨0⟩
    let old_turns := list_get_or sched.turns_given idx ⟨0⟩
    let new_turns ← old_turns + (1 : U32)
    let turns' := list_set sched.turns_given idx new_turns
    let next_idx := (idx + 1) % sched.agent_ids.length
    .ok (some agent_id,
         { sched with current_index := ⟨next_idx⟩, turns_given := turns' })

/-- Return the next agent according to the scheduling policy. -/
def next_agent (sched : Scheduler) : Result (Option AgentId × Scheduler) :=
  match sched.kind with
  | .RoundRobin => next_agent_rr sched
  | .Priority => next_agent_rr sched  -- simplified: priority uses same logic

/-! ## Voting -/

/-- Check if an agent has already voted. -/
def has_voted (votes : List (AgentId × Bool)) (agent_id : AgentId) : Bool :=
  match votes with
  | [] => false
  | (id, _) :: rest => if id == agent_id then true else has_voted rest agent_id

/-- Cast a vote. If the agent already voted, the vote is ignored. -/
def cast_vote (round : VotingRound) (agent_id : AgentId) (vote : Bool) : VotingRound :=
  if has_voted round.votes agent_id then round
  else { round with votes := round.votes ++ [(agent_id, vote)] }

/-- Count yes votes. -/
def count_yes : List (AgentId × Bool) → Nat
  | [] => 0
  | (_, true) :: rest => 1 + count_yes rest
  | (_, false) :: rest => count_yes rest

/-- Tally votes. Returns some true if majority reached, some false if rejected,
    none if voting still in progress. -/
def tally (round : VotingRound) : Option Bool :=
  let total := round.votes.length
  if total < round.expected_voters.val then
    none
  else if total == 0 then
    some false
  else
    let yes := count_yes round.votes
    -- Check: 100 * yes >= required_majority * total
    some (100 * yes ≥ round.required_majority.val * total)

/-! ## Pipeline -/

/-- Check that consecutive pipeline stages have matching types. -/
def is_pipeline_valid_aux : List PipelineStage → Bool
  | [] => true
  | [_] => true
  | s1 :: s2 :: rest =>
    if s1.output_type == s2.input_type then
      is_pipeline_valid_aux (s2 :: rest)
    else
      false

/-- Validate that the pipeline is well-formed. -/
def is_pipeline_valid (pipeline : Pipeline) : Bool :=
  is_pipeline_valid_aux pipeline.stages

/-! ## Debate -/

/-- Add an argument to the debate and decrement the round counter. -/
def debate_step (state : DebateState) (agent_id : AgentId) (argument_id : U32)
    : Result DebateState :=
  if state.rounds_remaining == (0 : U32) then
    .ok state
  else do
    let r ← state.rounds_remaining - (1 : U32)
    .ok { state with
      arguments := state.arguments ++ [(agent_id, argument_id)]
      rounds_remaining := r }

/-- Returns true if the debate has ended. -/
def debate_is_finished (state : DebateState) : Bool :=
  state.rounds_remaining == (0 : U32)

/-! ## Agent processing (enum dispatch) -/

/-- Check if an agent is active. -/
def is_agent_active (agent : AgentInstance) : Bool :=
  match agent.state with
  | .Finished => false
  | .Failed => false
  | _ => true

/-! ## Orchestrator -/

/-- Collect all agent IDs. -/
def all_agent_ids : List AgentInstance → List AgentId
  | [] => []
  | a :: rest => a.id :: all_agent_ids rest

/-- Find an agent by ID. -/
def find_agent : List AgentInstance → AgentId → Option AgentInstance
  | [], _ => none
  | a :: rest, id => if a.id == id then some a else find_agent rest id

/-- Returns true if any agent is active. -/
def has_active_agents : List AgentInstance → Bool
  | [] => false
  | a :: rest => if is_agent_active a then true else has_active_agents rest

/-- One orchestrator step (simplified: delivers messages and increments turn). -/
def orchestrator_step (state : OrchestratorState) : Result OrchestratorState := do
  let (maybe_agent, sched') ← next_agent state.scheduler
  match maybe_agent with
  | none =>
    let tc ← state.turn_count + (1 : U32)
    .ok { state with scheduler := sched', turn_count := tc }
  | some _agent_id =>
    let tc ← state.turn_count + (1 : U32)
    .ok { state with scheduler := sched', turn_count := tc }

/-- Run the orchestrator with fuel (tick budget). -/
def orchestrator_run_aux (fuel : Nat) (state : OrchestratorState) : Result OrchestratorState :=
  match fuel with
  | 0 => .ok state
  | n + 1 =>
    if state.turn_count.val ≥ state.config.turn_budget.val then
      .ok state
    else if ¬(has_active_agents state.agents) && bus_is_empty state.bus then
      .ok state
    else do
      let state' ← orchestrator_step state
      orchestrator_run_aux n state'

/-- Run the orchestrator until budget or quiescence. Uses turn_budget as fuel. -/
def orchestrator_run (state : OrchestratorState) : Result OrchestratorState :=
  orchestrator_run_aux state.config.turn_budget.val state

end multi_agent
