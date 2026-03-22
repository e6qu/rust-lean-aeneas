[← Previous: Tutorial 08](../08-llm-client-core/README.md) | [Index](../README.md) | [Next: Tutorial 10 →](../10-multi-agent-orchestrator/README.md)

# Tutorial 09: Agent Reasoning

A verified single-agent reasoning engine with state machine, chain-of-thought,
tool registry, retry logic, and guardrails. We prove termination, safety, and
invariant preservation in Lean 4 using Aeneas-generated translations.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Agent State Machine](#agent-state-machine)
4. [Reasoning Chains](#reasoning-chains)
5. [Tool Registry](#tool-registry)
6. [Decision Logic](#decision-logic)
7. [Retry with Exponential Backoff](#retry-with-exponential-backoff)
8. [Guardrails](#guardrails)
9. [Agent Orchestration](#agent-orchestration)
10. [Lean Translation](#lean-translation)
11. [Proofs](#proofs)
    - [State Machine Proofs](#state-machine-proofs)
    - [Termination Proofs](#termination-proofs)
    - [Tool Safety Proofs](#tool-safety-proofs)
    - [Chain Well-Formedness Proofs](#chain-well-formedness-proofs)
    - [Retry Proofs](#retry-proofs)
    - [Guardrails Proofs](#guardrails-proofs)
12. [Running the Code](#running-the-code)
13. [Cross-References](#cross-references)
14. [Exercises](#exercises)

---

## Overview

An AI agent is a program that receives user input, reasons about it (possibly
calling external tools), and produces a response. The reasoning process must:

- **Terminate**: the agent cannot loop forever.
- **Respect safety limits**: message sizes, recursion depth, and step counts
  are bounded.
- **Use only registered tools**: tool calls must reference tools that exist in
  the registry and satisfy parameter constraints.
- **Maintain chain-of-thought invariants**: the reasoning chain follows a
  monotonic Observe-Think-Decide-Act progression.

This tutorial builds all of these guarantees from the ground up, starting with
pure Rust functions and proving the properties in Lean.

### What we build

| Component | Rust module | Lines |
|-----------|------------|-------|
| State machine | `agent_state.rs` | ~140 |
| Reasoning chain | `reasoning.rs` | ~120 |
| Tool registry | `tool_registry.rs` | ~170 |
| Decision logic | `decision.rs` | ~100 |
| Retry logic | `retry.rs` | ~100 |
| Guardrails | `guardrails.rs` | ~90 |
| Agent loop | `agent.rs` | ~180 |

### What we prove

| Theorem | Lean file |
|---------|-----------|
| Terminal states reject all events | `StateMachine.lean` |
| `agent_run` always terminates | `Termination.lean` |
| Tool calls reference registered tools | `ToolSafety.lean` |
| `append_step` preserves well-formedness | `ChainWF.lean` |
| Retry attempt increases and delay grows | `Retry.lean` |
| All guards pass implies within limits | `Guardrails.lean` |

---

## Architecture

```
User input
    │
    ▼
┌──────────┐   AgentEvent    ┌────────────────┐
│   Idle   │ ──────────────► │   Thinking     │
└──────────┘  UserMessage    └───────┬────────┘
                                     │
                        ┌────────────┼────────────┐
                        ▼            ▼            ▼
                  ToolCallNeeded  LlmResponse  ThinkingDone
                        │            │            │
                        ▼            ▼            ▼
                  ┌──────────┐  ┌──────────┐  (Composing)
                  │CallingTool│  │Composing │
                  └────┬─────┘  └────┬─────┘
                       │             │
                  ToolResult    ComposeDone
                       │             │
                       ▼             ▼
                  ┌──────────┐  ┌──────────┐
                  │Awaiting  │  │   Done   │
                  │ToolResult│  └──────────┘
                  └────┬─────┘
                       │
                  ToolResult
                       │
                       ▼
                  (back to Thinking)
```

Any non-terminal state can transition to `Error` on `Cancel` or `Timeout`.

---

## Agent State Machine

**File:** `rust/src/agent_state.rs`

The agent lifecycle is modelled as a 7-state deterministic state machine with
three types:

### AgentPhase

```rust
pub enum AgentPhase {
    Idle,              // Waiting for user input
    Thinking,          // LLM is processing
    CallingTool,       // A tool call has been dispatched
    AwaitingToolResult,// Waiting for tool output
    Composing,         // Assembling the final response
    Done,              // Terminal: response emitted
    Error,             // Terminal: something went wrong
}
```

### AgentEvent

Eight possible stimuli:

| Event | Meaning |
|-------|---------|
| `UserMessage` | New user input arrived |
| `LlmResponse` | The LLM returned a text response |
| `ToolCallNeeded` | The LLM wants to call a tool |
| `ToolResult` | A tool has returned its output |
| `Timeout` | A deadline was exceeded |
| `Cancel` | The user or system cancelled the run |
| `ComposeDone` | The response composition is complete |
| `ThinkingDone` | Thinking phase completed without tool use |

### AgentAction

Five possible outputs: `SendToLlm`, `ExecuteTool`, `EmitResponse`, `LogEntry`, `Noop`.

### Transition function

```rust
pub fn agent_transition(phase: AgentPhase, event: &AgentEvent)
    -> Option<(AgentPhase, AgentAction)>
```

This is a pure function with a large `match` on `(phase, event)` pairs. The key
design decisions:

1. **Terminal states reject everything.** `Done` and `Error` return `None` for
   all events. This is the foundation of our termination proof.

2. **Cancel and Timeout are universal.** From any non-terminal state, these
   events transition to `Error` with a `LogEntry` action.

3. **Invalid transitions return `None`.** The caller (the agent loop) treats
   `None` as "stop processing."

### is_terminal

```rust
pub fn is_terminal(phase: AgentPhase) -> bool
```

Returns `true` for `Done` and `Error`. Used by the agent loop to decide when to
stop.

---

## Reasoning Chains

**File:** `rust/src/reasoning.rs`

A *reasoning chain* models the agent's internal thought process as a sequence
of typed steps:

```rust
pub enum Step {
    Observe(u32),        // Ingest information (order 0)
    Think(u32),          // Deliberate (order 1)
    Decide(Decision),    // Choose an action (order 2)
    Act(u32),            // Execute (order 3)
}
```

Each step has an *order tag* (0-3). A chain is **well-formed** if the order tags
are monotonically non-decreasing. This prevents the agent from going backwards
(e.g., observing after it has already decided).

### chain_step_order

Maps each step variant to its order tag: `Observe=0, Think=1, Decide=2, Act=3`.

### is_chain_well_formed

Iterates through the chain checking that each step's order is >= the previous
step's order. An empty chain is trivially well-formed.

### append_step

Attempts to append a new step to the chain. It only succeeds if the new step's
order is >= the last step's order. This is the guard that maintains the
well-formedness invariant incrementally.

### Decision

```rust
pub enum Decision {
    CallTool { tool_name_idx: u32, args_hash: u32 },
    Respond,
    AskClarification,
    GiveUp,
}
```

The four possible decisions the agent can make during the Decide phase.

---

## Tool Registry

**File:** `rust/src/tool_registry.rs`

Tools are external capabilities the agent can invoke (web search, code execution,
file read, etc.). Each tool is described by a `ToolSpec`:

```rust
pub struct ToolSpec {
    pub name_id: u32,         // Opaque identifier
    pub description_id: u32,  // Opaque description ref
    pub params: Vec<ToolParam>,
}
```

Parameters have a name, a kind (`StringParam`, `IntParam`, `BoolParam`), and
a `required` flag.

### find_tool

Linear search through the registry by `name_id`. Returns the index if found.
We use explicit `while` loops (no iterators) for Aeneas compatibility.

### validate_tool_call

Two-way validation:
1. Every **required** parameter in the spec must be present in the args with
   the correct kind.
2. Every argument provided must correspond to a parameter in the spec with
   the correct kind.

This prevents both missing-parameter errors and injection of unknown parameters.

---

## Decision Logic

**File:** `rust/src/decision.rs`

The `decide_next_action` function is a deterministic priority-based decision:

```
1. step_count >= max_steps          → GiveUp
2. pending_tool_results > 0         → AskClarification
3. conversation_len == 0            → AskClarification
4. tools available + signal == 1    → CallTool (first tool)
5. otherwise                        → Respond
```

The `DecisionContext` struct gathers all the information needed for the decision
without requiring access to the full agent state.

---

## Retry with Exponential Backoff

**File:** `rust/src/retry.rs`

When a tool call or LLM request fails, the agent can retry with exponential
backoff:

```rust
pub struct RetryState {
    pub attempt: u32,
    pub delay_ms: u32,
    pub max_attempts: u32,
    pub base_delay_ms: u32,
    pub max_delay_ms: u32,
}
```

### Key functions

- **`initial_retry_state`**: Creates a fresh state with `attempt = 0` and
  `delay_ms = base_delay_ms`.

- **`next_retry`**: If `attempt < max_attempts`, increments `attempt` by 1 and
  doubles `delay_ms` (capped at `max_delay_ms`). Returns `None` if exhausted.

- **`should_retry`**: Simple predicate: `attempt < max_attempts`.

### Properties we prove

1. `next_retry` always increments `attempt` by exactly 1.
2. The delay is non-decreasing across retries.
3. When `attempt >= max_attempts`, `should_retry` returns `false` and
   `next_retry` returns `None`.

---

## Guardrails

**File:** `rust/src/guardrails.rs`

Guardrails are safety limits that gate every agent step:

```rust
pub struct GuardrailConfig {
    pub max_message_len: u32,
    pub max_recursion_depth: u32,
    pub max_reasoning_steps: u32,
}
```

Four check functions:

| Function | Condition |
|----------|-----------|
| `check_message_length` | `msg_len <= max_message_len` |
| `check_recursion_depth` | `depth <= max_recursion_depth` |
| `check_reasoning_steps` | `step_count < max_reasoning_steps` |
| `all_guards_pass` | Conjunction of all three |

The `all_guards_pass` function is the single entry point used by the agent loop.

---

## Agent Orchestration

**File:** `rust/src/agent.rs`

### AgentConfig

Static configuration for an agent run:

```rust
pub struct AgentConfig {
    pub max_steps: u32,
    pub guardrails: GuardrailConfig,
    pub retry_config: RetryState,
}
```

### AgentSnapshot

Complete agent state at one point in time:

```rust
pub struct AgentSnapshot {
    pub phase: AgentPhase,
    pub reasoning_chain: Vec<Step>,
    pub step_count: u32,
    pub retry_state: RetryState,
    pub last_action: AgentAction,
}
```

### agent_step

Processes one event:

1. Check that `step_count < max_steps` (guard).
2. Call `agent_transition` to get `(next_phase, action)`.
3. If the transition is invalid, return `None`.
4. Otherwise, return a new snapshot with updated phase, action, and
   incremented step count.

### agent_run

The main loop. Iterates over a finite list of events:

```rust
pub fn agent_run(config: &AgentConfig, initial: AgentSnapshot,
                 events: &[AgentEvent]) -> AgentSnapshot
```

Stops when:
- The agent reaches a terminal phase (`Done` or `Error`).
- All events have been consumed.
- A step fails (invalid transition or guard violation).

Because the event list is finite and each iteration consumes exactly one event,
**`agent_run` always terminates**. This is the key property we prove in Lean.

---

## Lean Translation

### Types (`AgentReasoning/Types.lean`)

Aeneas translates the Rust types into Lean inductives and structures:

- `AgentPhase` becomes a 7-constructor inductive.
- `Step` and `Decision` are nested inductives.
- `AgentSnapshot` is a structure with `List Step` for the reasoning chain.
- All `u32` counters become `U32` (Aeneas primitive with overflow checks).

### Functions (`AgentReasoning/Funs.lean`)

Key translation decisions:

- **`agent_transition`** becomes a large `match` on `(AgentPhase, AgentEvent)`
  pairs. Lean's exhaustiveness checker ensures all 56 combinations are handled.

- **`agent_run`** becomes a structurally recursive function on the event list.
  Aeneas translates the Rust `while` loop into recursion on the list tail.
  Lean accepts this without a `decreasing_by` annotation because the list
  argument strictly decreases.

- **`validate_tool_call`** uses `List.all` and `List.any` for parameter
  iteration, corresponding to the Rust `while` loops.

- **`is_chain_well_formed`** uses an auxiliary function with an accumulator
  (`prev` order value), mirroring the Rust index-based loop.

---

## Proofs

### State Machine Proofs

**File:** `lean/AgentReasoning/Proofs/StateMachine.lean`

#### `agent_transition_from_terminal_is_none`

**Statement:** If `is_terminal phase = true`, then `agent_transition phase event = none`
for all `event`.

**Proof strategy:** Case split on `phase`. The `is_terminal` hypothesis
eliminates all cases except `Done` and `Error`. For each of these, case split
on `event` and observe that every branch of the `match` in `agent_transition`
returns `none`.

```lean
theorem agent_transition_from_terminal_is_none
    (phase : AgentPhase) (event : AgentEvent)
    (h : is_terminal phase = true) :
    agent_transition phase event = none := by
  cases phase <;> simp [is_terminal] at h <;> cases event <;> simp [agent_transition]
```

This is a fully automated proof — `simp` with the definition of
`agent_transition` handles each case.

#### `cancel_goes_to_error` and `timeout_goes_to_error`

**Statement:** From any non-terminal state, `Cancel` (resp. `Timeout`) always
transitions to `Error` with `LogEntry`.

**Proof strategy:** Same case-split approach. The `is_terminal` hypothesis
being `false` eliminates `Done` and `Error`, and the remaining 5 cases all
match the `Cancel`/`Timeout` branch.

### Termination Proofs

**File:** `lean/AgentReasoning/Proofs/Termination.lean`

#### `agent_run_terminates`

**Statement:** `agent_run` always terminates — there exists a result for any
inputs.

**Proof:** This is witnessed by the function definition itself. Because
`agent_run` is structurally recursive on the event list (each recursive call
passes the tail), Lean's termination checker accepts it without annotation.
The theorem is trivially `⟨agent_run config snapshot events, rfl⟩`.

#### `agent_run_terminal`

**Statement:** If the snapshot is already in a terminal phase, `agent_run`
returns it unchanged regardless of the event list.

**Proof:** The first `if` in `agent_run` checks `is_terminal` and returns
immediately.

#### `agent_run_step_count_bounded`

**Statement:** The output step count is at most the initial step count plus
the length of the event list.

**Proof sketch:** By induction on the event list. Each step increments
`step_count` by at most 1.

### Tool Safety Proofs

**File:** `lean/AgentReasoning/Proofs/ToolSafety.lean`

#### `tool_call_uses_registered_tool`

**Statement:** If `find_tool registry name_id = some idx`, then the tool at
index `idx` in the registry has `name_id` as its identifier.

**Proof strategy:** Induction on the registry list through the auxiliary
`find_tool_aux`. At each step, if the current tool matches, the index is
returned directly. Otherwise, we recurse with an incremented base index.

#### `validate_ensures_param_match`

**Statement:** If `validate_tool_call spec args = true`, then:
1. Every required parameter in `spec` is present in `args` with matching kind.
2. Every argument in `args` corresponds to a parameter in `spec` with matching
   kind.

**Proof strategy:** Unfold `validate_tool_call` as a conjunction of two
`List.all` predicates. Use `List.all_iff` to convert to universal
quantification over list elements.

### Chain Well-Formedness Proofs

**File:** `lean/AgentReasoning/Proofs/ChainWF.lean`

#### `append_preserves_well_formed`

**Statement:** If `is_chain_well_formed chain = true` and
`append_step chain step = some chain'`, then
`is_chain_well_formed chain' = true`.

**Proof strategy:** `append_step` returns `some` only when the new step's
order is >= the last step's order. This is exactly the condition needed to
extend a well-formed chain while maintaining the monotonicity invariant.
Unfold both definitions and show the extended chain satisfies each
pairwise comparison.

#### `chain_step_order_bounded`

**Statement:** `chain_step_order s` is always <= 3.

**Proof:** Case split on `s`; each variant produces a value in {0, 1, 2, 3}.

### Retry Proofs

**File:** `lean/AgentReasoning/Proofs/Retry.lean`

#### `next_retry_increases_attempt`

**Statement:** If `next_retry state = some state'`, then
`state'.attempt = state.attempt + 1`.

**Proof:** Unfold `next_retry`; the new attempt is constructed as
`state.attempt + 1` directly.

#### `retry_delay_grows`

**Statement:** If `next_retry state = some state'`, then
`state'.delay_ms >= state.delay_ms`.

**Proof:** The new delay is `min(delay * 2, max_delay)`. Since `delay * 2 >= delay`
and `min` returns one of its arguments, the result is >= the original delay.

#### `retry_stops_at_max`

**Statement:** If `attempt >= max_attempts`, then `should_retry` returns `false`.

**Proof:** Unfold `should_retry`; the condition `attempt < max_attempts` fails
when `attempt >= max_attempts`, so the result is `false`. This is direct by
`omega`.

### Guardrails Proofs

**File:** `lean/AgentReasoning/Proofs/Guardrails.lean`

#### `all_guards_pass_implies_within_limits`

**Statement:** If `all_guards_pass msg_len depth steps config = true`, then:
- `msg_len <= config.max_message_len`
- `depth <= config.max_recursion_depth`
- `steps < config.max_reasoning_steps`

**Proof:** Unfold `all_guards_pass` as a conjunction of three Boolean checks.
Each conjunct, when `true`, directly gives the corresponding inequality.

```lean
theorem all_guards_pass_implies_within_limits ... := by
  simp [all_guards_pass, check_message_length, check_recursion_depth,
        check_reasoning_steps] at h
  exact h
```

---

## Running the Code

### Rust

```bash
cd rust
cargo test
```

All 42 tests should pass, covering:
- State machine transitions (valid, invalid, terminal, cancel/timeout)
- Reasoning chain well-formedness and append
- Tool registry lookup and validation
- Decision logic priority rules
- Retry state progression and bounds
- Guardrail checks individually and in conjunction
- Agent step and run (happy path, tool path, cancel, bounds, empty events)

### Lean

```bash
cd lean
lake build
```

This type-checks all definitions and proofs. Theorems marked with `sorry` are
proof obligations that document the intended proof strategy.

---

## Cross-References

### From earlier tutorials

- **Tutorial 04 (State Machines):** `agent_transition` follows exactly the
  state machine pattern from Tutorial 04, now with 7 states and richer
  events/actions. The `run_machine` pattern maps to `agent_run`.

- **Tutorial 08 (LLM Client Core):** The `ChatMessage` and request types from
  Tutorial 08 inform the `AgentEvent::LlmResponse` and
  `AgentAction::SendToLlm` payloads. In a full system, the agent would
  construct `Request` values to send to the LLM.

### To later tutorials

- **Tutorial 10 (Multi-Agent Orchestrator):** Each agent in the orchestrator
  runs an instance of this reasoning engine internally. The `AgentSnapshot`
  is the state that the orchestrator tracks per agent.

- **Tutorial 11 (Full Integration):** Agent reasoning is a core component
  of the final application, connecting the LLM client, tool registry, and
  user interface.

---

## Exercises

### Exercise 1: Add a new phase

Add a `Planning` phase between `Idle` and `Thinking`. The agent enters
`Planning` on `UserMessage` and transitions to `Thinking` on a new
`PlanReady` event.

1. Add `Planning` to `AgentPhase` and `PlanReady` to `AgentEvent`.
2. Update `agent_transition` with the new transitions.
3. Add tests for the new transitions.
4. Update the Lean types and functions.
5. Prove that `Planning` is not terminal.

### Exercise 2: Richer chain validation

Extend `is_chain_well_formed` to also require that the chain starts with an
`Observe` step (if non-empty) and ends with an `Act` step (if it contains
a `Decide` step).

1. Add the new checks to `is_chain_well_formed`.
2. Update `append_step` to maintain the stronger invariant.
3. Prove the new version of `append_preserves_well_formed`.

### Exercise 3: Tool call rate limiting

Add a `max_tool_calls_per_run: u32` field to `GuardrailConfig` and a
`tool_call_count: u32` field to `AgentSnapshot`. Update `agent_step` to
increment the counter on `ExecuteTool` actions and reject steps that would
exceed the limit.

1. Add the new fields.
2. Update `agent_step` to check and increment.
3. Add a new guardrail check and include it in `all_guards_pass`.
4. Prove that `tool_call_count` never exceeds `max_tool_calls_per_run`.

### Exercise 4: Retry budget proof

Prove that the total delay across all retries is bounded:

```
total_delay <= base_delay * (2^max_attempts - 1)
```

when `max_delay` is large enough not to cap any individual delay.

Hint: Use induction on the number of retries and the geometric series formula.

### Exercise 5: Reachability analysis

Prove that `Done` is reachable from `Idle`: there exists a sequence of events
that takes the agent from `Idle` to `Done`.

```lean
theorem done_reachable_from_idle :
    ∃ events : List AgentEvent,
      (agent_run config idle_snapshot events).phase = .Done
```

Construct the witness: `[UserMessage, LlmResponse, ComposeDone]`.

### Exercise 6: Error recovery

Add an `ErrorRecovery` phase and a `Retry` event. When the agent is in
`Error` and receives `Retry` (if retries remain), it transitions back to the
phase it was in before the error. This requires storing the previous phase in
the snapshot.

1. Add `previous_phase: Option<AgentPhase>` to `AgentSnapshot`.
2. Update `agent_transition` to handle `(Error, Retry)`.
3. Prove that error recovery respects the retry limit.

---

[← Previous: Tutorial 08](../08-llm-client-core/README.md) | [Index](../README.md) | [Next: Tutorial 10 →](../10-multi-agent-orchestrator/README.md)
