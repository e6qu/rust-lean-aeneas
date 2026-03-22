-- MultiAgent/Types.lean
-- Simulated Aeneas output: inductive types for the multi-agent orchestrator.
import Aeneas

open Primitives

namespace multi_agent

/-- Unique identifier for an agent. -/
abbrev AgentId := U32

/-- Lifecycle state of an agent. -/
inductive AgentState where
  | Ready
  | Busy
  | Finished
  | Failed
deriving DecidableEq, Repr

/-- The kind (type tag) of a message payload. -/
inductive MessageKind where
  | Task
  | Response
  | Review
  | Vote
  | Delegation
  | Completion
  | Error
deriving DecidableEq, Repr

/-- A message payload with a kind tag and content identifier. -/
structure Message where
  kind : MessageKind
  content_id : U32
deriving DecidableEq, Repr

/-- Addressing modes for message delivery. -/
inductive Recipient where
  | Direct (id : AgentId)
  | Broadcast
  | Topic (id : U32)
deriving DecidableEq, Repr

/-- A sequenced envelope wrapping a message for routing. -/
structure Envelope where
  sender : AgentId
  recipient : Recipient
  message : Message
  sequence_num : U32
deriving DecidableEq, Repr

/-- Protocol kinds supported by the coordinator. -/
inductive ProtocolKind where
  | RequestResponse
  | Pipeline
  | Debate
  | Voting
deriving DecidableEq, Repr

/-- State held by a coordinator agent. -/
structure CoordinatorState where
  protocol : ProtocolKind
  managed_agents : List AgentId
  pending_responses : U32
  round : U32
deriving DecidableEq, Repr

/-- State held by a specialist agent. -/
structure SpecialistState where
  specialty : U32
  context : List U32
  steps_taken : U32
deriving DecidableEq, Repr

/-- State held by a critic agent. -/
structure CriticState where
  criteria : List U32
  reviews_given : U32
deriving DecidableEq, Repr

/-- Enum-dispatch agent kind. -/
inductive AgentKind where
  | Coordinator (state : CoordinatorState)
  | Specialist (state : SpecialistState)
  | Critic (state : CriticState)
deriving DecidableEq, Repr

/-- A running agent instance. -/
structure AgentInstance where
  id : AgentId
  kind : AgentKind
  state : AgentState
  inbox : List Envelope
  outbox : List Envelope
deriving DecidableEq, Repr

/-- Central message transport with an audit trail. -/
structure MessageBus where
  queue : List Envelope
  delivered : List Envelope
  next_seq : U32
deriving DecidableEq, Repr

/-- A subscription linking an agent to a topic. -/
structure TopicSubscription where
  topic_id : U32
  agent_id : AgentId
deriving DecidableEq, Repr

/-- Router that maps topics to subscribing agents. -/
structure Router where
  subscriptions : List TopicSubscription
deriving DecidableEq, Repr

/-- The scheduling policy. -/
inductive SchedulerKind where
  | RoundRobin
  | Priority
deriving DecidableEq, Repr

/-- Scheduler state tracking execution fairness. -/
structure Scheduler where
  kind : SchedulerKind
  agent_ids : List AgentId
  current_index : U32
  turns_given : List U32
deriving DecidableEq, Repr

/-- A voting round with majority threshold. -/
structure VotingRound where
  proposal_id : U32
  votes : List (AgentId × Bool)
  required_majority : U32
  expected_voters : U32
deriving DecidableEq, Repr

/-- A single stage in a processing pipeline. -/
structure PipelineStage where
  agent_id : AgentId
  input_type : U32
  output_type : U32
deriving DecidableEq, Repr

/-- A multi-stage processing pipeline. -/
structure Pipeline where
  stages : List PipelineStage
deriving DecidableEq, Repr

/-- State of a bounded debate protocol. -/
structure DebateState where
  topic_id : U32
  rounds_remaining : U32
  max_rounds : U32
  arguments : List (AgentId × U32)
deriving DecidableEq, Repr

/-- Configuration limits for the orchestrator. -/
structure OrchestratorConfig where
  turn_budget : U32
  scheduler_kind : SchedulerKind
deriving DecidableEq, Repr

/-- The full orchestrator state, composing all subsystems. -/
structure OrchestratorState where
  agents : List AgentInstance
  bus : MessageBus
  router : Router
  scheduler : Scheduler
  turn_count : U32
  config : OrchestratorConfig
deriving DecidableEq, Repr

end multi_agent
