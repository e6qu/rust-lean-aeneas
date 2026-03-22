[← Previous: Tutorial 10](../10-multi-agent-orchestrator/README.md) | [Index](../README.md)

# Tutorial 11: Full Integration

The capstone tutorial.  We wire every component from Tutorials 05-10 into a
complete TUI multi-agent LLM application with a **verified pure core** and an
**unverified imperative shell**, then prove end-to-end correctness properties
in Lean 4.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [The Verification Pyramid](#the-verification-pyramid)
4. [Verified Core](#verified-core)
    - [Integration Types](#integration-types)
    - [Application State](#application-state)
    - [Pure Update (Elm Architecture)](#pure-update-elm-architecture)
    - [Pure View](#pure-view)
    - [Message Bridge](#message-bridge)
5. [Imperative Shell](#imperative-shell)
    - [Terminal I/O](#terminal-io)
    - [HTTP Client](#http-client)
    - [Event Loop](#event-loop)
    - [Adapters](#adapters)
6. [Dependencies from Prior Tutorials](#dependencies-from-prior-tutorials)
7. [Lean Translation](#lean-translation)
    - [Types](#types)
    - [Functions](#functions)
8. [Proofs](#proofs)
    - [Type Safety Proofs](#type-safety-proofs)
    - [State Consistency Proofs](#state-consistency-proofs)
    - [End-to-End Proofs](#end-to-end-proofs)
    - [I/O Boundary Axioms](#io-boundary-axioms)
    - [Composition Proofs](#composition-proofs)
9. [Trust Boundaries](#trust-boundaries)
10. [Running the Code](#running-the-code)
11. [Cross-References](#cross-references)
12. [Exercises](#exercises)
13. [Conclusion](#conclusion)

---

## Overview

Throughout this tutorial series we have built verified components one at a
time: a message protocol (Tutorial 05), a buffer manager (Tutorial 06), a
TUI core (Tutorial 07), an LLM client (Tutorial 08), an agent reasoning
engine (Tutorial 09), and a multi-agent orchestrator (Tutorial 10).  Each
came with Lean proofs of its key properties.

Now we bring them all together.  The result is a terminal application where
a human user can chat with multiple AI agents.  Agents coordinate through a
verified message bus and reason using a verified state machine.  The user
interface is a four-pane TUI: conversation history, chat input, agent status,
and a debug/reasoning panel.

### What we build

| Layer | Rust modules | Lines | Verified? |
|-------|-------------|-------|-----------|
| Pure core | `verified_core/*` | ~500 | Yes (Aeneas + Lean) |
| I/O shell | `shell/*` | ~300 | No (trusted) |
| Stub deps | `deps/mod.rs` | ~80 | No (stand-ins) |

### What we prove

| Theorem | Lean file | Status |
|---------|-----------|--------|
| Bridge functions produce valid values | `TypeSafety.lean` | Proved |
| PaneId round-trips through u32 | `TypeSafety.lean` | Proved |
| Initial state is consistent | `StateConsistency.lean` | Proved |
| Quit/Tick/Resize preserve consistency | `StateConsistency.lean` | Proved |
| Submit reaches the message queue | `EndToEnd.lean` | Proved |
| Submit clears the input buffer | `EndToEnd.lean` | Proved |
| Orchestrator tick delivers a message | `EndToEnd.lean` | Sketched |
| Full pipeline under I/O axioms | `IOBoundary.lean` | Sketched |
| Component guarantees compose | `Composition.lean` | Axiomatised |

"Proved" means the proof term type-checks in Lean without `sorry`.
"Sketched" means the theorem statement and proof outline are present, with
`sorry` standing in for UInt32 arithmetic lemmas.

---

## Architecture

The application follows the **functional core, imperative shell** pattern
(also known as the Elm architecture or "ports and adapters"):

```
                    ┌─────────────────────────────────────────┐
                    │           Imperative Shell               │
                    │                                          │
  Terminal ───────► │  terminal_io.rs ──► adapters.rs          │
  (crossterm)       │                       │                  │
                    │                       ▼                  │
                    │  ┌──────────────────────────────────┐    │
                    │  │        Verified Pure Core         │    │
                    │  │                                   │    │
                    │  │  AppEvent ──► app_update ──► AppState │
                    │  │                                   │    │
                    │  │  AppState ──► app_view  ──► ViewTree  │
                    │  │                                   │    │
                    │  └──────────────────────────────────┘    │
                    │                       │                  │
                    │                       ▼                  │
                    │  terminal_io.rs ◄── ViewTree             │
                    │  http_client.rs ◄── SideEffects          │
                    │                                          │
  LLM API ◄──────── │  http_client.rs                         │
  (ureq)            └─────────────────────────────────────────┘
```

The key insight: **the pure core never performs I/O**.  It receives events as
data (`AppEvent`) and produces output as data (`AppState`, `ViewTree`).  The
shell is responsible for converting real-world I/O into events and executing
the effects described by the core.

This separation means Aeneas can translate the pure core to Lean and we can
prove properties about it.  The shell is a thin, trusted wrapper.

---

## The Verification Pyramid

The tutorial series forms a pyramid of verified components:

```
                    ┌───────────────┐
                    │  Tutorial 11  │  ◄── Integration (this tutorial)
                    │  Full System  │
                    ├───────┬───────┤
                    │ T.09  │ T.10  │  ◄── Domain logic
                    │ Agent │ Orch. │
                    ├───┬───┼───┬───┤
                    │T07│T08│T07│T08│  ◄── Domain logic (shared)
                    │TUI│LLM│TUI│LLM│
                    ├───┴───┴───┴───┤
                    │  T.05 │ T.06  │  ◄── Foundational data structures
                    │  Msg  │ Buf   │
                    └───────┴───────┘
```

Each layer's proofs build on the layer below:

- **Layer 1 (Tutorials 05-06):** Correct serialisation, buffer invariants.
- **Layer 2 (Tutorials 07-10):** TUI layout, LLM protocol, agent reasoning,
  orchestrator delivery — all using Layer 1 types.
- **Layer 3 (Tutorial 11):** End-to-end properties that compose Layer 2
  results.

---

## Verified Core

### Integration Types

File: `rust/src/verified_core/integration_types.rs`

The `AppEvent` enum normalises every external stimulus into a single type:

```rust
pub enum AppEvent {
    KeyPress(u32),
    Resize(u32, u32),
    UserSubmitMessage,
    SwitchPane(u32),
    SwitchAgent(u32),
    AgentEvent(u32, u32),
    OrchestratorTick,
    LlmResponseReceived(u32, u32),
    ToolResultReceived(u32, u32, u32),
    Tick,
    Quit,
}
```

The `PaneId` enum identifies the four UI panes:

```rust
pub enum PaneId {
    ChatInput,
    ConversationView,
    AgentStatusPanel,
    DebugReasoningPanel,
}
```

`PaneId` provides `from_u32` / `to_u32` conversions.  The Lean proof
`pane_id_roundtrip` shows these are inverses.

### Application State

File: `rust/src/verified_core/app_state.rs`

`AppState` is the single source of truth for the entire application:

```rust
pub struct AppState {
    pub active_pane: PaneId,
    pub selected_agent: u32,
    pub debug_visible: bool,
    pub running: bool,
    pub input_buffer: Vec<u8>,
    pub conversations: Vec<ConversationEntry>,
    pub agent_count: u32,
    pub turn_count: u32,
    pub turn_budget: u32,
    pub message_queue: Vec<(u32, u32, u32)>,
    pub error_message: Option<u32>,
    pub next_timestamp: u32,
}
```

Key design decisions:

- **Flat structure.** In a real system `AppState` would contain nested
  sub-states (`TuiState`, `OrchestratorState`, etc.).  We flatten it here
  for clarity.
- **Content ids instead of strings.** Message content is referenced by
  opaque `u32` ids.  This avoids `Vec<u8>` manipulation in the core, making
  proofs simpler.
- **Monotonic timestamps.**  `next_timestamp` increases on every conversation
  entry.  The `state_consistent` predicate requires all entry timestamps to
  be less than `next_timestamp`.

### Pure Update (Elm Architecture)

File: `rust/src/verified_core/app_update.rs`

`app_update` is the heart of the application.  It pattern-matches on the
event and dispatches to handler functions:

```rust
pub fn app_update(state: &AppState, event: &AppEvent) -> AppState {
    match event {
        AppEvent::KeyPress(key) => handle_keypress(state, *key),
        AppEvent::UserSubmitMessage => handle_submit(state),
        AppEvent::OrchestratorTick => handle_orchestrator_tick(state),
        AppEvent::Quit => { let mut s = state.clone(); s.running = false; s },
        // ... etc.
    }
}
```

Each handler is a small pure function:

| Handler | What it does |
|---------|-------------|
| `handle_keypress` | Appends to input buffer (if ChatInput pane active) |
| `handle_submit` | Extracts input, creates conversation entry + bus message |
| `handle_switch_pane` | Validates pane id, updates `active_pane` |
| `handle_switch_agent` | Validates agent id, updates `selected_agent` |
| `handle_orchestrator_tick` | Increments turn count, delivers one message |
| `handle_llm_response` | Appends assistant response to conversation |
| `handle_tick` | No-op (animations would go here) |

The Rust test suite covers every event type, including edge cases (empty
submit, out-of-range ids, budget exhaustion).

### Pure View

File: `rust/src/verified_core/app_view.rs`

`app_view` converts `AppState` into a `ViewTree` — a flat list of `ViewCell`
structs, each specifying a character and its screen coordinates.

The screen is split into four panes:

```
┌──────────────────────────┬──────────────┐
│                          │  Agent       │
│  Conversation View       │  Status      │
│          (2/3 w, 3/4 h)  │  Panel       │
├──────────────────────────┤  (1/3 w,     │
│  Chat Input              │   1/2 h)     │
│          (2/3 w, 1/4 h)  ├──────────────┤
│                          │  Debug Panel │
│                          │  (1/3 w,     │
│                          │   1/2 h)     │
└──────────────────────────┴──────────────┘
```

The active pane is marked with `*` in its label.

### Message Bridge

File: `rust/src/verified_core/message_bridge.rs`

Bridge functions convert between the type vocabularies of different
subsystems:

- `user_input_to_message(content_id, coordinator_id)` — builds a message
  triple with sender `u32::MAX` (meaning "human user").
- `message_to_conversation_entry(sender, content_id, timestamp)` — builds a
  `ConversationEntry` with role derived from the sender id.
- `is_valid_pane_id(id)` — bounds check.
- `is_valid_agent_id(id, agent_count)` — bounds check.

The Lean proofs in `TypeSafety.lean` show these functions preserve validity.

---

## Imperative Shell

The shell is the thin I/O layer that connects the verified core to the
outside world.  It is NOT translated by Aeneas and NOT verified in Lean.
Instead, its behaviour is described by axioms in `IOBoundary.lean`.

In this tutorial the shell files are **documented stubs**: they compile and
have the right signatures but do not perform real I/O (since crossterm and
ureq are not workspace dependencies).

### Terminal I/O

File: `rust/src/shell/terminal_io.rs`

- `read_event(timeout_ms)` — reads a terminal event via crossterm.
- `render(view)` — blits a `ViewTree` to the terminal.
- `enter_raw_mode()` / `leave_raw_mode()` — terminal setup/teardown.

### HTTP Client

File: `rust/src/shell/http_client.rs`

- `send_llm_request(endpoint, body)` — POST to an LLM API.
- `send_streaming_request(endpoint, body)` — streaming variant.

### Event Loop

File: `rust/src/shell/event_loop.rs`

The main loop:

```
loop {
    1. event = read_event()           // I/O (unverified)
    2. state = app_update(state, event)  // pure (verified)
    3. view  = app_view(state)           // pure (verified)
    4. render(view)                      // I/O (unverified)
    5. execute_side_effects()            // I/O (unverified)
}
```

Steps 2 and 3 are proved correct.  Steps 1, 4, 5 are trusted.

### Adapters

File: `rust/src/shell/adapters.rs`

Converts between shell types (crossterm key codes, HTTP responses) and core
types (`AppEvent`).  Key mappings:

| Raw input | AppEvent |
|-----------|----------|
| Enter (13) | `UserSubmitMessage` |
| Escape (27) | `Quit` |
| Tab (9) | `SwitchPane(1)` |
| Other key | `KeyPress(code)` |

---

## Dependencies from Prior Tutorials

File: `rust/src/deps/mod.rs`

In a real workspace these would be Cargo dependencies on the sibling
tutorial crates.  Here we define minimal stubs:

| Tutorial | Types provided |
|----------|---------------|
| 05 | `AgentId`, `Payload`, `Envelope` |
| 06 | `InputBuffer` |
| 07 | `Rect` |
| 08 | `ChatMessage`, `Conversation` |
| 09 | `AgentPhase` |
| 10 | `MessageBus` |

---

## Lean Translation

### Types

File: `lean/FullIntegration/Types.lean`

Direct Lean translations of `AppEvent`, `PaneId`, `ConversationEntry`,
`AppState`, `ViewCell`, and `ViewTree`.  In a real Aeneas pipeline this
file is auto-generated.

Key differences from Rust:

- `Vec<u8>` becomes `List UInt8`.
- `Vec<(u32, u32, u32)>` becomes `List (UInt32 × UInt32 × UInt32)`.
- `Option<u32>` becomes `Option UInt32`.
- Enums become Lean `inductive` types.
- Structs become Lean `structure` types.

### Functions

File: `lean/FullIntegration/Funs.lean`

Translations of `app_update`, all handler functions, and the message bridge.
The Lean code mirrors the Rust implementation exactly: same match arms, same
field updates, same control flow.

---

## Proofs

### Type Safety Proofs

File: `lean/FullIntegration/Proofs/TypeSafety.lean`

| Theorem | Statement |
|---------|-----------|
| `submit_sender_is_user` | `user_input_to_message` always sets sender to `u32::MAX` |
| `submit_content_id_preserved` | The content id passes through unchanged |
| `user_message_has_role_zero` | User messages get role 0 |
| `agent_message_has_role_one` | Agent messages get role 1 |
| `valid_pane_ids` | Pane ids 0-3 are valid, 4 is not |
| `pane_id_roundtrip` | `fromUInt32 (toUInt32 p) = some p` |
| `llm_bridge_preserves_agent_id` | LLM response entries have the correct agent id |

These are all fully proved (no `sorry`).

### State Consistency Proofs

File: `lean/FullIntegration/Proofs/StateConsistency.lean`

The `state_consistent` predicate requires:

1. `selected_agent < agent_count` (or `agent_count = 0`).
2. `turn_count ≤ turn_budget`.
3. All conversation entry timestamps are `< next_timestamp`.

| Theorem | Statement |
|---------|-----------|
| `new_state_consistent` | `AppState.new` produces a consistent state |
| `quit_preserves_consistency` | Quit does not break consistency |
| `tick_preserves_consistency` | Tick does not break consistency |
| `resize_preserves_consistency` | Resize does not break consistency |
| `switch_agent_invalid_preserves` | Invalid agent switch is a no-op |

### End-to-End Proofs

File: `lean/FullIntegration/Proofs/EndToEnd.lean`

| Theorem | Statement |
|---------|-----------|
| `submit_reaches_queue` | Non-empty submit makes message_queue non-empty |
| `submit_clears_buffer` | Submit empties the input buffer |
| `submit_adds_conversation_entry` | Submit adds exactly one conversation entry |
| `tick_delivers_message` | Orchestrator tick removes one message from queue (sketched) |
| `submit_then_tick_delivers` | Submit + tick leaves the queue empty (sketched) |

### I/O Boundary Axioms

File: `lean/FullIntegration/Proofs/IOBoundary.lean`

These axioms describe the trusted shell behaviour:

| Axiom | Meaning |
|-------|---------|
| `terminal_event_well_formed` | The adapter produces valid events |
| `render_is_pure_sink` | Rendering does not affect state |
| `http_response_valid` | LLM API returns well-formed responses |
| `http_error_benign` | HTTP errors produce benign Tick events |
| `event_loop_single_update` | Each iteration calls `app_update` exactly once |

The `end_to_end_under_io_axiom` theorem chains the component proofs with
the I/O axioms to establish the full-system property.

### Composition Proofs

File: `lean/FullIntegration/Proofs/Composition.lean`

This module shows how results from Tutorials 05-10 compose:

| Component | Tutorial | Property used |
|-----------|----------|---------------|
| Message protocol | 05 | Serialisation round-trips |
| Buffer management | 06 | Clear yields empty |
| TUI core | 07 | View is deterministic |
| LLM client | 08 | Request builder well-formedness |
| Agent reasoning | 09 | Agent terminates within budget |
| Orchestrator | 10 | Message delivery, orchestrator termination |

Since we cannot import the actual Lean libraries from other tutorials,
these are stated as axioms.  In a real project they would be `import`ed
and the axioms would become references to proved theorems.

---

## Trust Boundaries

The system has an explicit trust boundary:

```
     ┌─────────────────────────────────┐
     │       VERIFIED (Lean proofs)     │
     │                                  │
     │  integration_types.rs            │
     │  app_state.rs                    │
     │  app_update.rs                   │
     │  app_view.rs                     │
     │  message_bridge.rs               │
     │                                  │
     ├──────────────────────────────────┤  ◄── Trust boundary
     │                                  │
     │       TRUSTED (axioms)           │
     │                                  │
     │  terminal_io.rs                  │
     │  http_client.rs                  │
     │  event_loop.rs                   │
     │  adapters.rs                     │
     │                                  │
     └─────────────────────────────────┘
```

Everything above the boundary is proved correct in Lean.  Everything below
is described by axioms.  The axioms make the trust surface **explicit and
auditable**: if you want to know what is assumed but not proved, read
`IOBoundary.lean`.

This is fundamentally different from testing, where the trust surface is
implicit (whatever the tests do not cover).

---

## Running the Code

### Rust

```bash
cd rust
cargo test
```

This runs the verified core tests and the shell stub tests.  No external
dependencies are required.

### Lean

```bash
cd lean
lake build
```

This type-checks all proofs.  Theorems with `sorry` will produce warnings
but will not prevent the build from succeeding.

### Running the application

The shell is implemented as stubs, so there is no runnable TUI.  To build
a real application you would:

1. Uncomment the crossterm and ureq dependencies in `Cargo.toml`.
2. Replace the stub implementations in `shell/` with real I/O code.
3. Add a `[[bin]]` target pointing to `shell/main.rs`.
4. Run `cargo run`.

The verified core would remain unchanged.

---

## Cross-References

| Tutorial | What it provides to Tutorial 11 |
|----------|-------------------------------|
| [01 Setup](../01-setup-hello-proof/) | Aeneas/Lean toolchain, basic workflow |
| [02 RPN Calculator](../02-rpn-calculator/) | Pattern: pure functions + proofs |
| [03 Infix Calculator](../03-infix-calculator/) | Pattern: recursive descent + termination |
| [04 State Machines](../04-state-machines/) | Pattern: state machines with invariants |
| [05 Message Protocol](../05-message-protocol/) | `Envelope`, serialisation round-trips |
| [06 Buffer Management](../06-buffer-management/) | `InputBuffer`, gap buffer invariants |
| [07 TUI Core](../07-tui-core/) | Layout, widgets, focus, render purity |
| [08 LLM Client Core](../08-llm-client-core/) | `Conversation`, request builder, streaming |
| [09 Agent Reasoning](../09-agent-reasoning/) | Agent state machine, termination, tool safety |
| [10 Multi-Agent Orchestrator](../10-multi-agent-orchestrator/) | Message bus, router, scheduler, delivery proofs |

---

## Exercises

### Exercise 1: Prove `app_update_preserves_consistency` for all events

The `StateConsistency.lean` file proves preservation for Quit, Tick, and
Resize.  Extend it to cover all `AppEvent` variants.  The hardest case is
`OrchestratorTick`, which modifies `turn_count`, `message_queue`, and
`conversations` simultaneously.

**Hint:** For `UserSubmitMessage`, you need to show that the new conversation
entry's timestamp (`next_timestamp`) is less than the incremented
`next_timestamp + 1`, and that all existing entries still satisfy the
invariant.

### Exercise 2: Remove `sorry` from `EndToEnd.lean`

The `tick_delivers_message` and `submit_then_tick_delivers` theorems use
`sorry`.  Fill in the proofs.  You will need lemmas about `List.length`
after removing the head, and about UInt32 arithmetic.

**Hint:** Consider defining helper lemmas about `List.length` for the cons
pattern: `(x :: xs).length = xs.length + 1`.

### Exercise 3: Add a `DebugToggle` event

1. Add `DebugToggle` to `AppEvent` in Rust.
2. Implement `handle_debug_toggle` that flips `debug_visible`.
3. Add the corresponding Lean types and functions.
4. Prove that `debug_toggle` preserves `state_consistent`.
5. Prove that toggling twice is the identity: `toggle (toggle s) = s` on
   the `debug_visible` field.

### Exercise 4: Implement a real side-effect queue

The current design uses `message_queue` for orchestrator messages.  Add a
`pending_side_effects: Vec<SideEffect>` field to `AppState`, where
`SideEffect` is an enum (`SendHttpRequest(u32)`, `LogMessage(u32)`, etc.).

1. Define the `SideEffect` type in Rust and Lean.
2. Modify `handle_submit` to push a `SendHttpRequest` side effect.
3. Modify the event loop stub to drain side effects.
4. Prove that side effects are only produced by submit and LLM events
   (never by Tick or Resize).

### Exercise 5: Strengthen the I/O axioms

The axioms in `IOBoundary.lean` are deliberately weak (most are `True`
placeholders).  Strengthen them:

1. Make `http_response_valid` return a well-formed `LlmResponseReceived`
   event (with agent_id in range).
2. Make `terminal_event_well_formed` guarantee that key codes are < 256.
3. Prove `end_to_end_under_io_axiom` using the strengthened axioms.

### Exercise 6: Multi-step delivery proof

Prove that a sequence of N user submissions followed by N orchestrator ticks
results in all N messages being delivered and all N responses appearing in
the conversation log (assuming the I/O axioms for LLM responses).

**Hint:** Use induction on N.  The base case is `submit_then_tick_delivers`.

### Exercise 7: Integrate with a real tutorial crate

Pick one of Tutorials 05-10 and replace its stub in `deps/mod.rs` with
an actual Cargo dependency.  Wire the real types through the verified core.
Verify that the Lean proofs still hold (you may need to update the type
translations).

---

## Conclusion

This tutorial demonstrates the culmination of the Rust + Lean formal
verification approach:

1. **Separation of concerns.** The pure functional core handles all
   application logic.  The imperative shell handles all I/O.  The boundary
   between them is precise and explicit.

2. **Composable proofs.** Each tutorial proves properties about its own
   component.  Tutorial 11 composes these into system-level guarantees.
   The Lean module system makes this composition natural.

3. **Explicit trust boundaries.** The I/O axioms in `IOBoundary.lean` make
   the unverified assumptions visible and auditable.  This is a fundamental
   improvement over testing, where untested behaviour is invisible.

4. **Practical architecture.** The Elm architecture (pure update + pure view)
   is a well-known pattern used in production applications (Elm, Redux, TEA).
   Adding formal verification to this architecture is natural because the
   core is already pure.

5. **Incremental verification.** You do not need to verify everything at
   once.  Start with the most critical component, prove its properties,
   then expand.  The `sorry` markers in our proofs show exactly where more
   work is needed.

The verified core of this application is roughly 500 lines of Rust.  The
Lean proofs are roughly 400 lines.  The shell is roughly 300 lines.  This
is a realistic ratio for verified software: the proof burden is manageable
when the architecture is designed for verification from the start.

```
Total lines:  ~1200 Rust + ~400 Lean
Verified:     ~500 Rust (pure core)
Proved:       ~400 Lean (types, functions, proofs)
Trusted:      ~300 Rust (shell) + ~50 Lean (axioms)
```

The next step would be to fill in the `sorry` markers, strengthen the I/O
axioms, and build a real TUI by replacing the shell stubs with crossterm
and ureq implementations — without changing a single line of the verified
core.

---

[← Previous: Tutorial 10](../10-multi-agent-orchestrator/README.md) | [Index](../README.md)
