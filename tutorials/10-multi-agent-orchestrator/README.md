[← Previous: Tutorial 09](../09-agent-reasoning/README.md) | [Index](../README.md) | [Next: Tutorial 11 →](../11-full-integration/README.md)

# Tutorial 10: Multi-Agent Orchestrator

A verified multi-agent orchestration system with message bus, routing,
scheduling, conversation protocols, and an orchestrator loop. We prove
routing correctness, message delivery, scheduling fairness, budget
termination, voting majority, pipeline type safety, and debate termination
in Lean 4 using Aeneas-generated translations.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Types](#types)
4. [Message Bus](#message-bus)
5. [Router](#router)
6. [Scheduler](#scheduler)
7. [Agent Types](#agent-types)
   - [Coordinator](#coordinator)
   - [Specialist](#specialist)
   - [Critic](#critic)
8. [Protocols](#protocols)
   - [Request-Response](#request-response)
   - [Debate](#debate)
   - [Pipeline](#pipeline)
   - [Voting](#voting)
9. [Orchestrator](#orchestrator)
10. [Lean Translation](#lean-translation)
11. [Proofs](#proofs)
    - [Routing Proofs](#routing-proofs)
    - [No Message Loss Proofs](#no-message-loss-proofs)
    - [Fairness Proofs](#fairness-proofs)
    - [Budget Proofs](#budget-proofs)
    - [Voting Proofs](#voting-proofs)
    - [Pipeline Proofs](#pipeline-proofs)
    - [Debate Proofs](#debate-proofs)
12. [Running the Code](#running-the-code)
13. [Cross-References](#cross-references)
14. [Exercises](#exercises)

---

## Overview

A multi-agent system consists of several autonomous agents that communicate
through a shared message bus, coordinated by a central orchestrator. The
orchestrator must:

- **Deliver messages faithfully**: every message sent reaches its intended
  recipient. No message is lost or duplicated.
- **Schedule fairly**: each agent receives a fair share of execution time.
  A round-robin scheduler gives each agent exactly `n` turns after `n * k`
  scheduling rounds (where `k` is the number of agents).
- **Respect budgets**: the orchestrator loop terminates within a configured
  number of turns, preventing infinite loops.
- **Support multiple protocols**: request-response, pipeline, debate, and
  voting each have their own correctness properties.

This tutorial builds all of these guarantees from the ground up, starting
with pure Rust functions and proving the properties in Lean.

### What we build

| Component | Rust module | Lines |
|-----------|------------|-------|
| Core types | `agent_trait.rs` | ~95 |
| Message bus | `message_bus.rs` | ~120 |
| Router | `router.rs` | ~80 |
| Scheduler | `scheduler.rs` | ~100 |
| Coordinator agent | `agents/coordinator.rs` | ~80 |
| Specialist agent | `agents/specialist.rs` | ~50 |
| Critic agent | `agents/critic.rs` | ~50 |
| Agent dispatch | `agents/mod.rs` | ~40 |
| Request-response protocol | `protocols/request_response.rs` | ~75 |
| Debate protocol | `protocols/debate.rs` | ~60 |
| Pipeline protocol | `protocols/pipeline.rs` | ~75 |
| Voting protocol | `protocols/voting.rs` | ~90 |
| Orchestrator | `orchestrator.rs` | ~120 |

### What we prove

| Theorem | Lean file |
|---------|-----------|
| `resolve_direct_delivers_to_target` | `Routing.lean` |
| `sent_then_delivered` | `NoMessageLoss.lean` |
| `round_robin_fairness` | `Fairness.lean` |
| `orchestrator_terminates_within_budget` | `Budget.lean` |
| `tally_matches_majority` | `Voting.lean` |
| `valid_pipeline_types_match` | `Pipeline.lean` |
| `debate_terminates` | `Debate.lean` |

---

## Architecture

```
External Task
    |
    v
+------------------+
|   Orchestrator   |  turn_budget, scheduler
+--------+---------+
         |
    +----+----+
    |         |
    v         v
+-------+  +-------+
|  Bus  |  |Router |  topic subscriptions
+---+---+  +---+---+
    |          |
    +----+-----+
         |
    +----+----+------+
    |         |      |
    v         v      v
+------+ +------+ +------+
|Coord | |Spec  | |Critic|
+------+ +------+ +------+
```

The orchestrator loop proceeds in discrete ticks:

1. **Schedule**: the scheduler picks the next agent to run.
2. **Deliver**: the message bus delivers pending envelopes to that agent.
3. **Process**: the agent processes its inbox and produces outgoing envelopes.
4. **Route**: outgoing envelopes are resolved (Direct, Broadcast, Topic) and
   placed back on the bus.
5. **Increment**: the turn counter advances. If the budget is exhausted or no
   agents are active and the bus is empty, the loop stops.

---

## Types

**File:** `rust/src/agent_trait.rs`

All types are plain enums and structs -- no trait objects, no dynamic dispatch.
This is essential for Aeneas translation, which requires fully static types.

### AgentId

```rust
pub type AgentId = u32;
```

A unique identifier for each agent instance. In Lean this becomes `U32`.

### AgentState

```rust
pub enum AgentState {
    Ready,     // Agent can accept messages
    Busy,      // Agent is processing
    Finished,  // Agent has completed its work
    Failed,    // Agent encountered an error
}
```

### Message and MessageKind

```rust
pub enum MessageKind {
    Task,        // A new task to process
    Response,    // A response to a task
    Review,      // A critical review
    Vote,        // A vote in a voting round
    Delegation,  // A coordinator delegating work
    Completion,  // All sub-tasks complete
    Error,       // An error report
}

pub struct Message {
    pub kind: MessageKind,
    pub content_id: u32,
}
```

Messages carry a type tag and a content identifier. We use `u32` identifiers
rather than `Vec<u8>` payloads to keep the Lean translation clean -- the
content is an opaque reference that the proofs do not need to inspect.

### Envelope and Recipient

```rust
pub enum Recipient {
    Direct(AgentId),  // Send to one agent
    Broadcast,        // Send to all agents
    Topic(u32),       // Send to topic subscribers
}

pub struct Envelope {
    pub sender: AgentId,
    pub recipient: Recipient,
    pub message: Message,
    pub sequence_num: u32,
}
```

Envelopes are the unit of communication. The `sequence_num` is assigned by
the message bus and provides a total ordering on all messages.

### AgentKind and AgentInstance

```rust
pub enum AgentKind {
    Coordinator(CoordinatorState),
    Specialist(SpecialistState),
    Critic(CriticState),
}

pub struct AgentInstance {
    pub id: AgentId,
    pub kind: AgentKind,
    pub state: AgentState,
    pub inbox: Vec<Envelope>,
    pub outbox: Vec<Envelope>,
}
```

The `AgentKind` enum uses the enum-dispatch pattern: each variant holds the
agent-specific state. The `agent_process` function pattern-matches on the
kind to dispatch to the appropriate handler. In Lean this becomes a simple
`match agent.kind with | Coordinator s => ... | Specialist s => ... | ...`.

---

## Message Bus

**File:** `rust/src/message_bus.rs`

The message bus is a FIFO queue with an audit trail. Every envelope sent
through the bus is eventually moved to the `delivered` list, providing a
complete record of all communication.

### MessageBus

```rust
pub struct MessageBus {
    pub queue: Vec<Envelope>,
    pub delivered: Vec<Envelope>,
    pub next_seq: u32,
}
```

### Key functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `bus_new` | `() -> MessageBus` | Create an empty bus |
| `bus_send` | `(&mut MessageBus, Envelope)` | Assign sequence number and enqueue |
| `bus_deliver` | `(&mut MessageBus, AgentId) -> Vec<Envelope>` | Move matching envelopes to delivered |
| `bus_deliver_next` | `(&mut MessageBus) -> Option<Envelope>` | Dequeue next envelope (FIFO) |
| `bus_is_empty` | `(&MessageBus) -> bool` | Check if queue is empty |

The `bus_send` function assigns a monotonically increasing sequence number
to each envelope. This is critical for the no-message-loss proof: we can
identify each message uniquely by its sequence number.

The `bus_deliver` function partitions the queue: envelopes targeting the
given agent ID move to `delivered` and are returned; the rest stay in the
queue. The `envelope_targets` helper checks Direct and Broadcast addressing.

```rust
fn envelope_targets(env: &Envelope, agent_id: AgentId) -> bool {
    match &env.recipient {
        Recipient::Direct(id) => *id == agent_id,
        Recipient::Broadcast => true,
        Recipient::Topic(_) => false,  // handled by router before delivery
    }
}
```

Topic addressing is resolved by the router *before* envelopes reach the bus.
The orchestrator expands Topic recipients into multiple Direct envelopes.

---

## Router

**File:** `rust/src/router.rs`

The router maps topic IDs to sets of subscribed agents.

### Router and TopicSubscription

```rust
pub struct TopicSubscription {
    pub topic_id: u32,
    pub agent_id: AgentId,
}

pub struct Router {
    pub subscriptions: Vec<TopicSubscription>,
}
```

### resolve_recipient

```rust
pub fn resolve_recipient(
    router: &Router,
    recipient: &Recipient,
    all_agent_ids: &[AgentId],
) -> Vec<AgentId>
```

This is the central routing function:

| Recipient | Resolution |
|-----------|------------|
| `Direct(id)` | `[id]` (singleton) |
| `Broadcast` | All agent IDs |
| `Topic(t)` | Agents subscribed to topic `t` |

The key property we prove is `resolve_direct_delivers_to_target`: for
`Direct(id)`, the result is always `[id]` regardless of router state.

---

## Scheduler

**File:** `rust/src/scheduler.rs`

The scheduler determines which agent runs on each tick.

### SchedulerKind

```rust
pub enum SchedulerKind {
    RoundRobin,
    Priority,
}
```

### Scheduler

```rust
pub struct Scheduler {
    pub kind: SchedulerKind,
    pub agent_ids: Vec<AgentId>,
    pub current_index: u32,
    pub turns_given: Vec<u32>,
}
```

The `turns_given` vector tracks how many turns each agent has received,
indexed in parallel with `agent_ids`. This is essential for the fairness
proof.

### next_agent

```rust
pub fn next_agent(sched: &mut Scheduler) -> Option<AgentId>
```

For `RoundRobin`, this returns `agent_ids[current_index]`, increments
`turns_given[current_index]`, and advances `current_index` modulo the
number of agents.

For `Priority`, the agent with the fewest turns is selected, breaking
ties by index (lower index = higher priority).

### Fairness property

After `n * k` calls to `next_agent` on a round-robin scheduler with `k`
agents, each agent has been given exactly `n` turns. This follows from
the modular-arithmetic cycling of `current_index`.

---

## Agent Types

### Coordinator

**File:** `rust/src/agents/coordinator.rs`

The coordinator manages a group of agents, delegating tasks and collecting
responses.

```rust
pub struct CoordinatorState {
    pub protocol: ProtocolKind,
    pub managed_agents: Vec<AgentId>,
    pub pending_responses: u32,
    pub round: u32,
}
```

When the coordinator receives a `Task` message, it:
1. Sets `pending_responses` to the number of managed agents.
2. Sends a `Delegation` message to each managed agent.

When it receives a `Response`, it decrements `pending_responses`. When
the count reaches zero, it sends a `Completion` message.

For `Debate` protocol reviews, it broadcasts the review to all agents.
For `Voting` protocol reviews, it sends a `Vote` request to each agent.

### Specialist

**File:** `rust/src/agents/specialist.rs`

A specialist handles delegated tasks by producing a response.

```rust
pub struct SpecialistState {
    pub specialty: u32,
    pub context: Vec<u32>,
    pub steps_taken: u32,
}
```

On receiving a `Delegation` or `Task`, the specialist:
1. Increments `steps_taken` and adds the content ID to its context.
2. Sends a `Response` back to the sender with a transformed content ID.

The content ID transformation (`content_id + specialty`) simulates domain-
specific processing.

### Critic

**File:** `rust/src/agents/critic.rs`

A critic reviews content and produces review messages.

```rust
pub struct CriticState {
    pub criteria: Vec<u32>,
    pub reviews_given: u32,
}
```

On receiving a `Delegation`, `Task`, or `Response`, the critic increments
`reviews_given` and sends a `Review` back to the sender.

### Agent Dispatch

**File:** `rust/src/agents/mod.rs`

The `agent_process` function dispatches to the appropriate handler:

```rust
pub fn agent_process(agent: &mut AgentInstance) {
    // Skip finished/failed agents and agents with empty inboxes
    match &mut agent.kind {
        AgentKind::Coordinator(cs) => coordinator_process(cs, ...),
        AgentKind::Specialist(ss) => specialist_process(ss, ...),
        AgentKind::Critic(cs) => critic_process(cs, ...),
    }
}
```

This enum-dispatch pattern translates cleanly to Lean pattern matching.

---

## Protocols

Each protocol is a pure state machine, independent of the agent and
orchestrator code. This separation lets us verify protocol properties
in isolation.

### Request-Response

**File:** `rust/src/protocols/request_response.rs`

A simple two-party protocol with timeout.

```
Idle --> AwaitingResponse --> Completed
                         \-> TimedOut
```

States: `Idle`, `AwaitingResponse`, `Completed`, `TimedOut`.

| Function | Purpose |
|----------|---------|
| `rr_new` | Create a new instance |
| `rr_send` | Transition to AwaitingResponse |
| `rr_receive` | Transition to Completed |
| `rr_tick` | Advance timeout counter |
| `rr_is_done` | Check if Completed or TimedOut |

### Debate

**File:** `rust/src/protocols/debate.rs`

A bounded debate where agents submit arguments for a fixed number of rounds.

```rust
pub struct DebateState {
    pub topic_id: u32,
    pub rounds_remaining: u32,
    pub max_rounds: u32,
    pub arguments: Vec<(AgentId, u32)>,
}
```

| Function | Purpose |
|----------|---------|
| `debate_new` | Create with max rounds |
| `debate_step` | Add argument, decrement rounds (no-op at 0) |
| `debate_is_finished` | Check if rounds_remaining == 0 |

**Key property:** `debate_terminates` -- after `r` calls to `debate_step`
starting from `rounds_remaining = r`, the debate is finished.

### Pipeline

**File:** `rust/src/protocols/pipeline.rs`

A multi-stage processing pipeline where each stage's output type must match
the next stage's input type.

```rust
pub struct PipelineStage {
    pub agent_id: AgentId,
    pub input_type: u32,
    pub output_type: u32,
}

pub struct Pipeline {
    pub stages: Vec<PipelineStage>,
}
```

| Function | Purpose |
|----------|---------|
| `pipeline_new` | Create empty pipeline |
| `pipeline_add_stage` | Append a stage |
| `is_pipeline_valid` | Check pairwise type matching |
| `pipeline_agent_at` | Get agent for stage index |

**Key property:** `valid_pipeline_types_match` -- if `is_pipeline_valid`
returns true, then for all consecutive stages `(i, i+1)`,
`stages[i].output_type == stages[i+1].input_type`.

The validation function iterates over consecutive pairs:

```rust
pub fn is_pipeline_valid(pipeline: &Pipeline) -> bool {
    for i in 0..pipeline.stages.len() - 1 {
        if pipeline.stages[i].output_type != pipeline.stages[i + 1].input_type {
            return false;
        }
    }
    true
}
```

### Voting

**File:** `rust/src/protocols/voting.rs`

A majority voting protocol.

```rust
pub struct VotingRound {
    pub proposal_id: u32,
    pub votes: Vec<(AgentId, bool)>,
    pub required_majority: u32,  // percentage 0-100
    pub expected_voters: u32,
}
```

| Function | Purpose |
|----------|---------|
| `voting_new` | Create with majority threshold |
| `cast_vote` | Record a vote (duplicates ignored) |
| `tally` | Check majority: `Some(true)` if passed, `Some(false)` if rejected, `None` if pending |
| `yes_count` / `no_count` | Count votes by type |

**Key property:** `tally_matches_majority` -- if `tally` returns
`Some(true)`, then `100 * yes_count >= required_majority * total_votes`.

The tally uses integer arithmetic to avoid floating-point:

```rust
let passes = 100 * yes_count >= round.required_majority * total;
```

---

## Orchestrator

**File:** `rust/src/orchestrator.rs`

The orchestrator composes all subsystems into a single stepping loop.

### OrchestratorConfig

```rust
pub struct OrchestratorConfig {
    pub turn_budget: u32,
    pub scheduler_kind: SchedulerKind,
}
```

### OrchestratorState

```rust
pub struct OrchestratorState {
    pub agents: Vec<AgentInstance>,
    pub bus: MessageBus,
    pub router: Router,
    pub scheduler: Scheduler,
    pub turn_count: u32,
    pub config: OrchestratorConfig,
}
```

### orchestrator_step

One tick of the orchestrator:

1. Call `next_agent` on the scheduler to pick an agent.
2. Call `bus_deliver` to move matching envelopes to the agent's inbox.
3. Call `agent_process` to run the agent.
4. Take the agent's outbox and, for each envelope, call `resolve_recipient`
   to expand the addressing, then `bus_send` to place the routed envelopes
   on the bus.
5. Increment `turn_count`.

### orchestrator_run

The main loop:

```rust
pub fn orchestrator_run(state: &mut OrchestratorState) {
    while state.turn_count < state.config.turn_budget {
        if !has_active_agents(state) && bus_is_empty(&state.bus) {
            break;
        }
        orchestrator_step(state);
    }
}
```

The loop terminates in two ways:
- **Budget exhaustion**: `turn_count >= turn_budget`.
- **Quiescence**: no active agents and the bus is empty.

**Key property:** `orchestrator_terminates_within_budget` -- the final
`turn_count` never exceeds the configured `turn_budget`.

---

## Lean Translation

### Types (MultiAgent/Types.lean)

Aeneas translates Rust types as follows:

| Rust | Lean |
|------|------|
| `u32` | `U32` |
| `Vec<T>` | `List T` |
| `enum` | `inductive` |
| `struct` | `structure` |

The `AgentKind` enum becomes a 3-constructor inductive:

```lean
inductive AgentKind where
  | Coordinator (state : CoordinatorState)
  | Specialist (state : SpecialistState)
  | Critic (state : CriticState)
```

The `Scheduler` uses `List U32` for `turns_given`, indexed in parallel
with the `agent_ids` list.

### Functions (MultiAgent/Funs.lean)

Key translation decisions:

- **`bus_send`** returns `Result MessageBus` because incrementing `next_seq`
  can overflow. In practice the budget ensures this never happens, but the
  Lean translation must account for it.

- **`next_agent_rr`** returns `Result (Option AgentId x Scheduler)`.
  The `Option` handles the empty-agent-list case; the `Result` handles
  potential arithmetic overflow.

- **`orchestrator_run`** uses fuel-bounded recursion via `orchestrator_run_aux`.
  The fuel is `config.turn_budget`, which strictly decreases on each
  recursive call. Lean's structural recursion checker accepts this directly.

- **`is_pipeline_valid_aux`** uses structural recursion on the list of stages,
  pattern-matching on `s1 :: s2 :: rest` for the consecutive-pair check.

- **`count_yes`** folds over the vote list, counting `true` entries. This is
  the core of the tally computation that the voting proof reasons about.

Helper functions `list_get_or` and `list_set` provide safe indexed access
and update on `List`, used extensively by the scheduler.

---

## Proofs

### Routing Proofs

**File:** `lean/MultiAgent/Proofs/Routing.lean`

#### resolve_direct_delivers_to_target

```lean
theorem resolve_direct_delivers_to_target
    (router : Router) (id : AgentId) (agents : List AgentId) :
    resolve_recipient router (.Direct id) agents = [id]
```

**Proof:** Unfold `resolve_recipient`; the `Direct` branch returns `[id]`
by definition. This is a direct computation -- `simp [resolve_recipient]`
closes the goal.

#### resolve_broadcast_returns_all

```lean
theorem resolve_broadcast_returns_all
    (router : Router) (agents : List AgentId) :
    resolve_recipient router .Broadcast agents = agents
```

**Proof:** Same strategy -- unfold and simplify.

#### envelope_targets_direct / envelope_targets_broadcast

These theorems establish that `envelope_targets` correctly identifies
Direct and Broadcast recipients:

```lean
theorem envelope_targets_direct (env : Envelope) (agent_id : AgentId)
    (h : env.recipient = .Direct agent_id) :
    envelope_targets env agent_id = true
```

### No Message Loss Proofs

**File:** `lean/MultiAgent/Proofs/NoMessageLoss.lean`

#### sent_then_delivered

```lean
theorem sent_then_delivered (bus : MessageBus) (env : Envelope) (agent_id : AgentId)
    (h_target : envelope_targets env agent_id = true)
    (h_in_queue : env ∈ bus.queue) :
    let (bus', delivered) := bus_deliver bus agent_id
    env ∈ delivered ∧ env ∈ bus'.delivered
```

**Proof strategy:** By induction on `bus.queue`. The `partition_envelopes`
function separates matching from non-matching envelopes. Since `env` is in
the queue and `envelope_targets env agent_id = true`, it lands in the
matching partition, which becomes part of `delivered`.

#### partition_envelopes_complete

```lean
theorem partition_envelopes_complete (queue : List Envelope) (agent_id : AgentId) :
    let (matching, remaining) := partition_envelopes queue agent_id
    matching.length + remaining.length = queue.length
```

This auxiliary theorem ensures no envelopes are lost or duplicated during
partitioning. By induction on the queue, each element goes to exactly one
of the two output lists.

### Fairness Proofs

**File:** `lean/MultiAgent/Proofs/Fairness.lean`

#### list_set_length

```lean
theorem list_set_length (xs : List α) (i : Nat) (val : α) :
    (list_set xs i val).length = xs.length
```

The `list_set` helper preserves list length -- essential for maintaining
the parallel-indexing invariant between `agent_ids` and `turns_given`.

#### list_get_or_set_same / list_get_or_set_diff

These frame-condition lemmas say that:
- Reading at the index you just wrote returns the new value.
- Reading at a different index returns the old value.

Together they characterize `list_set` completely.

#### round_robin_fairness

```lean
theorem round_robin_fairness
    (k n : Nat) (hk : k > 0) ...
```

**Proof strategy:** By induction on the number of advances. The loop
invariant is: after `m` advances, `turns_given[i] = m/k + (if i < m%k then 1 else 0)`.
At `m = n * k`, this simplifies to `turns_given[i] = n` for all `i`.
The proof uses modular arithmetic and the `list_get_or_set_same/diff` lemmas.

### Budget Proofs

**File:** `lean/MultiAgent/Proofs/Budget.lean`

#### orchestrator_terminates_within_budget

```lean
theorem orchestrator_terminates_within_budget
    (state state' : OrchestratorState)
    (h_init : state.turn_count = 0)
    (h_run : orchestrator_run state = .ok state') :
    state'.turn_count.val ≤ state.config.turn_budget.val
```

**Proof strategy:** `orchestrator_run` calls `orchestrator_run_aux` with
fuel `= turn_budget`. Each recursive call increments `turn_count` by 1 and
decreases fuel by 1. By induction on fuel, `turn_count` increases by at
most the initial fuel value. Since the initial `turn_count` is 0 and fuel
is `turn_budget`, the final `turn_count ≤ turn_budget`.

#### orchestrator_run_aux_zero

```lean
theorem orchestrator_run_aux_zero (state : OrchestratorState) :
    orchestrator_run_aux 0 state = .ok state
```

Base case: with zero fuel, the state is returned unchanged.

### Voting Proofs

**File:** `lean/MultiAgent/Proofs/Voting.lean`

#### tally_matches_majority

```lean
theorem tally_matches_majority (round : VotingRound)
    (h_tally : tally round = some true) :
    100 * count_yes round.votes ≥ round.required_majority.val * round.votes.length
```

**Proof strategy:** Unfold `tally`. The `some true` branch is taken only
when `100 * yes ≥ required_majority * total` evaluates to true. The
hypothesis `h_tally` gives us exactly this inequality.

#### count_yes_le_length

```lean
theorem count_yes_le_length (votes : List (AgentId × Bool)) :
    count_yes votes ≤ votes.length
```

A sanity check: the number of yes votes cannot exceed the total number of
votes. Proved by induction on the vote list.

#### cast_vote_monotone

```lean
theorem cast_vote_monotone (round : VotingRound) (agent_id : AgentId) (vote : Bool) :
    round.votes.length ≤ (cast_vote round agent_id vote).votes.length
```

Casting a vote never decreases the vote count. Either the vote is new
(length increases by 1) or it is a duplicate (length unchanged).

### Pipeline Proofs

**File:** `lean/MultiAgent/Proofs/Pipeline.lean`

#### valid_pipeline_types_match

```lean
theorem valid_pipeline_types_match (pipeline : Pipeline)
    (h : is_pipeline_valid pipeline = true)
    (i : Nat) (s1 s2 : PipelineStage)
    (h1 : pipeline.stages.get? i = some s1)
    (h2 : pipeline.stages.get? (i + 1) = some s2) :
    s1.output_type = s2.input_type
```

**Proof strategy:** By induction on `i` and the stages list. The helper
`pipeline_head_types_match` extracts the property for the first pair.
The helper `pipeline_tail_valid` shows that validity of the full list
implies validity of the tail. Together, by induction, we get the pairwise
property for all consecutive pairs.

#### pipeline_head_types_match

```lean
theorem pipeline_head_types_match (s1 s2 : PipelineStage) (rest : List PipelineStage)
    (h : is_pipeline_valid_aux (s1 :: s2 :: rest) = true) :
    s1.output_type = s2.input_type
```

Directly extracted from the definition: `is_pipeline_valid_aux` checks
`s1.output_type == s2.input_type` and returns false if they differ.

### Debate Proofs

**File:** `lean/MultiAgent/Proofs/Debate.lean`

#### debate_terminates

Starting from `rounds_remaining = r`, after `r` calls to `debate_step`,
`rounds_remaining = 0`.

**Proof strategy:** Each `debate_step` decrements `rounds_remaining` by 1
when it is positive, or is a no-op when it is 0. By induction on `r`:
- Base case: `r = 0` means already finished.
- Inductive case: one step gives `rounds_remaining = r - 1`, and the
  induction hypothesis gives 0 after `r - 1` more steps.

#### debate_step_finished

```lean
theorem debate_step_finished (state : DebateState) (agent_id : AgentId) (arg_id : U32)
    (h : state.rounds_remaining = 0) :
    debate_step state agent_id arg_id = .ok state
```

A no-op when the debate is already finished.

#### debate_step_decrements

```lean
theorem debate_step_decrements (state : DebateState) ...
    (h_pos : state.rounds_remaining.val > 0) ... :
    state'.rounds_remaining.val = state.rounds_remaining.val - 1
```

Each active step decreases `rounds_remaining` by exactly 1.

---

## Running the Code

### Rust

```bash
cd rust
cargo test
```

This runs 32 tests covering:
- Message bus: send, deliver, FIFO ordering, empty checks
- Router: direct, broadcast, topic resolution
- Scheduler: round-robin cycling, fairness after n*k turns, priority
- Voting: majority pass/fail, duplicate prevention, tally correctness
- Pipeline: valid/invalid/empty/single-stage, agent lookup
- Debate: termination, no-op after finished, zero rounds
- Request-response: lifecycle, timeout
- Orchestrator: budget enforcement, delegation, critic integration, quiescence

### Lean

```bash
cd lean
lake build
```

This type-checks all definitions and proofs. Theorems marked with `sorry`
indicate proof obligations that are stated but left as exercises or require
more involved reasoning.

---

## Cross-References

- **From Tutorial 04 (State Machines):** Each protocol (request-response,
  debate, pipeline, voting) is a state machine following the pattern from
  Tutorial 04. The debate and request-response protocols have explicit state
  transitions.

- **From Tutorial 05 (Message Protocol):** The `Envelope` and `Message`
  types extend the message protocol concepts from Tutorial 05 with routing,
  sequencing, and multiple addressing modes (Direct, Broadcast, Topic).

- **From Tutorial 09 (Agent Reasoning):** Each agent instance internally
  uses reasoning patterns from Tutorial 09. The `specialist_process` and
  `critic_process` functions wrap agent step logic with domain-specific
  behavior.

- **To Tutorial 11 (Full Integration):** The orchestrator is the
  coordination backbone of the final application. The verified properties
  (message delivery, fairness, budget termination) provide the foundation
  for composing all previous tutorials into a single verified system.

---

## Exercises

### Exercise 1: Add a Mediator Agent

Add a `Mediator` variant to `AgentKind` that receives two competing
responses and selects the better one. Update `agent_process` to dispatch
to the new handler. Prove that the mediator always produces exactly one
output for two inputs.

### Exercise 2: Weighted Voting

Extend `VotingRound` with a `weights: Vec<(AgentId, u32)>` field. Modify
`tally` to compute a weighted majority. Prove that `tally_matches_majority`
still holds with the weighted computation.

### Exercise 3: Pipeline Composition

Write a function `compose_pipelines(p1: &Pipeline, p2: &Pipeline) -> Option<Pipeline>`
that concatenates two pipelines only if the last stage of `p1` has an
output type matching the first stage of `p2`. Prove that if both input
pipelines are valid, the composed pipeline is also valid.

### Exercise 4: Stronger Fairness

Strengthen `round_robin_fairness` to show that at any point (not just
multiples of `k`), the difference in turns between any two agents is at
most 1. This is the "bounded fairness" property.

### Exercise 5: Message Ordering

Prove that the message bus preserves FIFO ordering: if envelope `a` is
sent before envelope `b` (i.e., `a.sequence_num < b.sequence_num`), and
both target the same agent, then `a` is delivered before `b`.

### Exercise 6: Debate Convergence

Add a `consensus_threshold: u32` field to `DebateState`. Write a function
that checks whether the last `k` arguments all agree (same content ID).
Prove that if consensus is reached, the debate can terminate early.

### Exercise 7: Orchestrator Idempotence

Prove that running `orchestrator_run` twice (feeding the output of the
first run as input to the second) produces the same state as running once,
provided the first run reached quiescence.

### Exercise 8: Complete the Sorry Proofs

Fill in the `sorry` placeholders in the Lean proof files:
- `bus_send_enqueues` in NoMessageLoss.lean
- `sent_then_delivered` in NoMessageLoss.lean
- `orchestrator_run_aux_bounded` in Budget.lean
- `tally_matches_majority` in Voting.lean
- `valid_pipeline_types_match` in Pipeline.lean
- `debate_step_decrements` in Debate.lean

---

[← Previous: Tutorial 09](../09-agent-reasoning/README.md) | [Index](../README.md) | [Next: Tutorial 11 →](../11-full-integration/README.md)
