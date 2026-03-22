# Tutorial 10: Multi-Agent Orchestrator

## Goal

Implement multi-agent orchestration with message bus, routing, scheduling, and conversation protocols; prove routing, fairness, budget, voting, pipeline, and debate correctness.

## File Structure

```
10-multi-agent-orchestrator/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── agent_trait.rs      # AgentId, AgentState, Message, Envelope, Recipient, AgentKind, AgentInstance
│       ├── message_bus.rs      # MessageBus: bus_new, bus_send, bus_deliver
│       ├── router.rs           # Router, TopicSubscription, resolve_recipient
│       ├── scheduler.rs        # Scheduler (RoundRobin/Priority), next_agent
│       ├── agents/
│       │   ├── mod.rs
│       │   ├── coordinator.rs  # CoordinatorState, ProtocolKind, coordinator_process
│       │   ├── specialist.rs   # SpecialistState, specialist_process
│       │   └── critic.rs       # CriticState, critic_process
│       ├── protocols/
│       │   ├── mod.rs
│       │   ├── request_response.rs
│       │   ├── debate.rs       # DebateState, debate_step
│       │   ├── pipeline.rs     # PipelineStage, Pipeline, is_pipeline_valid
│       │   └── voting.rs       # VotingRound, cast_vote, tally
│       └── orchestrator.rs     # OrchestratorConfig, OrchestratorState, orchestrator_step, orchestrator_run
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── MultiAgent.lean
    ├── MultiAgent/
    │   ├── Types.lean
    │   ├── Funs.lean
    │   └── Proofs/
    │       ├── Routing.lean
    │       ├── NoMessageLoss.lean
    │       ├── Fairness.lean
    │       ├── Budget.lean
    │       ├── Voting.lean
    │       ├── Pipeline.lean
    │       └── Debate.lean
```

## Rust Code Outline

### Types

- **`AgentId`** — `u32`. Unique identifier for each agent instance.
- **`Message`** — `content: Vec<u8>, msg_type: u8`. Payload with a type tag (0=request, 1=response, 2=broadcast, 3=vote, 4=critique).
- **`Envelope`** — `seq: u32, from: AgentId, to: Recipient, message: Message, timestamp: u32`. Sequenced wrapper for routing.
- **`Recipient`** — Enum: `Direct(AgentId) | Topic(u8) | Broadcast`. Addressing modes.
- **`AgentKind`** — Enum (dispatch): `Coordinator(CoordinatorState) | Specialist(SpecialistState) | Critic(CriticState)`. Each variant holds the agent-specific state.
- **`AgentInstance`** — `id: AgentId, kind: AgentKind`. A running agent.
- **`CoordinatorState`** — `protocol: ProtocolKind, agents: Vec<AgentId>, pending_responses: u32, round: u32`.
- **`ProtocolKind`** — Enum: `RequestResponse | Pipeline | Debate | Voting`.
- **`SpecialistState`** — `specialty: u8, context: Vec<u8>, steps_taken: u32`.
- **`CriticState`** — `criteria: Vec<u8>, reviews_given: u32`.
- **`MessageBus`** — `queue: Vec<Envelope>, delivered: Vec<Envelope>, next_seq: u32`. The central message transport with an audit trail.
- **`TopicSubscription`** — `agent_id: AgentId, topic: u8`.
- **`Router`** — `subscriptions: Vec<TopicSubscription>`. Maps topics to subscribing agents.
- **`Scheduler`** — Enum: `RoundRobin { agents: Vec<AgentId>, current: u32, turns_given: Vec<u32> } | Priority { agents: Vec<AgentId>, priorities: Vec<u32>, turns_given: Vec<u32> }`. Tracks execution fairness.
- **`VotingRound`** — `proposal: Vec<u8>, votes: Vec<(AgentId, bool)>, required_majority: u32 (percent, 0-100)`.
- **`PipelineStage`** — `agent_id: AgentId, input_type: u8, output_type: u8`.
- **`Pipeline`** — `stages: Vec<PipelineStage>`.
- **`DebateState`** — `topic: Vec<u8>, rounds_remaining: u32, max_rounds: u32, arguments: Vec<(AgentId, Vec<u8>)>`.
- **`OrchestratorConfig`** — `max_ticks: u32, max_messages: u32`. Budget limits.
- **`OrchestratorState`** — `agents: Vec<AgentInstance>, bus: MessageBus, router: Router, scheduler: Scheduler, tick: u32, config: OrchestratorConfig`.

### Functions

- **`bus_new() -> MessageBus`** — Create empty bus.
- **`bus_send(bus: MessageBus, from: AgentId, to: Recipient, message: Message, timestamp: u32) -> MessageBus`** — Enqueue an envelope with the next sequence number.
- **`bus_deliver(bus: MessageBus) -> (MessageBus, Option<Envelope>)`** — Dequeue the next envelope, moving it to `delivered`.
- **`resolve_recipient(router: &Router, recipient: &Recipient, all_agents: &[AgentId]) -> Vec<AgentId>`** — Resolve `Direct` to singleton, `Topic` to subscribers, `Broadcast` to all.
- **`next_agent(scheduler: &Scheduler) -> Option<AgentId>`** — Return the next agent to run according to the scheduling policy.
- **`advance_scheduler(scheduler: Scheduler) -> Scheduler`** — Move to the next agent; increment `turns_given`.
- **`coordinator_process(state: CoordinatorState, envelope: &Envelope) -> (CoordinatorState, Vec<Envelope>)`** — Coordinator handles incoming messages, produces outgoing envelopes based on protocol.
- **`specialist_process(state: SpecialistState, envelope: &Envelope) -> (SpecialistState, Vec<Envelope>)`** — Specialist processes a request and produces a response.
- **`critic_process(state: CriticState, envelope: &Envelope) -> (CriticState, Vec<Envelope>)`** — Critic reviews content and produces a critique.
- **`cast_vote(round: VotingRound, agent: AgentId, vote: bool) -> VotingRound`** — Record a vote.
- **`tally(round: &VotingRound) -> Option<bool>`** — Returns `Some(true)` if majority reached, `Some(false)` if rejected, `None` if votes still pending.
- **`is_pipeline_valid(pipeline: &Pipeline) -> bool`** — Check that each stage's `output_type` matches the next stage's `input_type`.
- **`pipeline_step(pipeline: &Pipeline, stage_index: u32, input: &Message) -> Option<Envelope>`** — Route input to the correct stage agent.
- **`debate_step(state: DebateState, argument: (AgentId, Vec<u8>)) -> DebateState`** — Add an argument and decrement rounds.
- **`orchestrator_step(state: OrchestratorState) -> OrchestratorState`** — One tick: deliver a message, run the scheduled agent, collect outgoing messages, increment tick.
- **`orchestrator_run(state: OrchestratorState) -> OrchestratorState`** — Bounded loop: run steps until `tick >= config.max_ticks` or bus is empty and no agent is active.

### Estimated Lines

~900 lines Rust.

## Generated Lean (Approximate)

Aeneas will produce:

- **`MultiAgent/Types.lean`**: Inductive types for all enums and structures. `AgentKind` is a 3-constructor inductive holding nested state structures. `Scheduler` is a 2-constructor inductive. `MessageBus` is a structure with `List Envelope` fields.
- **`MultiAgent/Funs.lean`**: All functions. `orchestrator_run` becomes fuel-bounded recursion. `resolve_recipient` pattern-matches on `Recipient` and filters the subscription list. `tally` folds over the vote list counting true/false.

Key translation notes:
- All `Vec` become `List`; `u32` becomes `U32`.
- Enum dispatch on `AgentKind` translates to pattern matching: `match agent.kind with | Coordinator s => coordinator_process s env | ...`.
- `orchestrator_run` uses fuel (tick budget) as the decreasing measure.
- `turns_given: Vec<u32>` becomes `List U32` indexed in parallel with the agent list — proofs must show indices stay in sync.

## Theorems to Prove

### `resolve_direct_delivers_to_target`
**Statement:** `resolve_recipient router (Direct id) agents = [id]` when `id` is in `agents`.
**Proof strategy:** Unfold `resolve_recipient`; the `Direct` branch returns a singleton list. Show `id` membership is preserved.

### `sent_then_delivered`
**Statement:** For every envelope placed on the bus via `bus_send`, there exists a future state where that envelope appears in `delivered` (assuming sufficient ticks).
**Proof strategy:** `bus_deliver` dequeues from `queue` and appends to `delivered`. Show that the bus is FIFO: each `bus_deliver` call moves the head of `queue` to `delivered`. By induction on queue position, the envelope is eventually delivered.

### `round_robin_fairness`
**Statement:** After `n * k` calls to `advance_scheduler` on a `RoundRobin` scheduler with `k` agents, each agent has been given exactly `n` turns.
**Proof strategy:** `advance_scheduler` increments `current` modulo `k` and increments the corresponding `turns_given` entry. By induction on the number of advances, `turns_given[i] = (advances + k - 1 - i) / k + (if i <= advances % k then 1 else 0)`. At multiples of `k`, all counts are equal.

### `orchestrator_terminates_within_budget`
**Statement:** `orchestrator_run` returns with `state.tick <= state.config.max_ticks`.
**Proof strategy:** Each `orchestrator_step` increments `tick` by 1. The loop guard checks `tick < max_ticks`. The fuel is `max_ticks - tick`, which strictly decreases.

### `tally_matches_majority`
**Statement:** If `tally round = some true`, then the number of `true` votes exceeds `round.required_majority` percent of total votes.
**Proof strategy:** Unfold `tally`; it computes `100 * yes_count / total_votes` and compares to `required_majority`. The `some true` branch is taken only when this comparison succeeds.

### `valid_pipeline_types_match`
**Statement:** If `is_pipeline_valid pipeline = true`, then for all consecutive stages `(i, i+1)`, `stages[i].output_type = stages[i+1].input_type`.
**Proof strategy:** `is_pipeline_valid` iterates over consecutive pairs and checks type equality. Unfold and use list indexing lemmas to extract the pairwise property.

### `debate_terminates`
**Statement:** Starting from `DebateState` with `rounds_remaining = r`, after `r` calls to `debate_step`, `rounds_remaining = 0`.
**Proof strategy:** Each `debate_step` decrements `rounds_remaining` by 1 (clamped at 0). By induction on `r`, after `r` steps the count is 0.

### Estimated Lines

~900 lines proofs.

## New Lean Concepts Introduced

- **Sequential simulation of distributed systems**: Modeling concurrent agents as a sequential step function over shared state. The `orchestrator_step` function simulates one "tick" of the system.
- **Fairness proofs**: Proving that a round-robin scheduler gives each agent equal turns. Uses modular arithmetic and induction over the number of scheduling rounds.
- **`Finset` and counting**: Using Lean's `Finset` or list-based counting to reason about votes, majorities, and set membership in the agent pool.
- **Pigeonhole-style reasoning**: If there are `k` agents and `n * k` turns, each agent gets exactly `n` turns — a direct application of pigeonhole/division reasoning.
- **Composition of verified components**: The orchestrator composes the message bus, router, scheduler, and individual agent processors — each verified independently, then combined.

## Cross-References

- **From Tutorial 04 (State Machines):** Each protocol (request-response, debate, pipeline, voting) is a state machine following the pattern from Tutorial 04.
- **From Tutorial 05 (Message Protocol):** The `Envelope` and `Message` types extend the message protocol concepts from Tutorial 05 with routing and sequencing.
- **From Tutorial 09 (Agent Reasoning):** Each agent instance internally uses the reasoning engine from Tutorial 09. The `specialist_process` and `critic_process` functions wrap the agent step logic.
- **To Tutorial 11 (Full Integration):** The orchestrator is the coordination backbone of the final application.
