# Tutorial 11: Full Integration

## Goal

Wire all components into a complete TUI multi-agent LLM application with a verified pure core and an unverified I/O shell.

## File Structure

```
11-full-integration/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── verified_core/
│       │   ├── mod.rs
│       │   ├── integration_types.rs  # AppEvent, PaneId
│       │   ├── app_state.rs          # AppState (combines TUI + orchestrator + LLM + buffers)
│       │   ├── app_update.rs         # Pure update: (AppState, AppEvent) -> AppState
│       │   ├── app_view.rs           # Pure view: AppState -> ViewTree (4 panes)
│       │   └── message_bridge.rs     # Type conversions between components
│       ├── shell/
│       │   ├── mod.rs
│       │   ├── main.rs              # Entry point
│       │   ├── terminal_io.rs       # crossterm I/O (NOT verified)
│       │   ├── http_client.rs       # ureq HTTP (NOT verified)
│       │   ├── event_loop.rs        # Main loop: read -> update -> render -> side effects
│       │   └── adapters.rs          # Shell type <-> core type mapping
│       └── deps/
│           └── mod.rs               # Re-exports from tutorials 05-10
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── FullIntegration.lean
    ├── FullIntegration/
    │   ├── Types.lean
    │   ├── Funs.lean
    │   └── Proofs/
    │       ├── TypeSafety.lean      # Bridge function correctness
    │       ├── StateConsistency.lean # state_consistent invariant
    │       ├── EndToEnd.lean        # Full pipeline message flow
    │       ├── IOBoundary.lean      # Axioms for I/O layer
    │       └── Composition.lean     # Composes theorems from tutorials 05-10
```

## Rust Code Outline

The application is split into two layers: a verified pure core and an unverified imperative shell.

### Verified Core (~500 lines)

#### Types

- **`AppEvent`** — Enum: `KeyPress(u8, u8) | Resize(u16, u16) | Tick | LlmChunk(Vec<u8>) | LlmDone(Vec<u8>) | LlmError(Vec<u8>) | AgentMessage(u32, Vec<u8>)`. All external stimuli normalized into a single event type.
- **`PaneId`** — Enum: `Input | Conversation | AgentStatus | SystemLog`. The four UI panes.
- **`AppState`** — Combines all verified component states:
  - `tui: TuiState` (from Tutorial 07) — layout, widgets, focus
  - `orchestrator: OrchestratorState` (from Tutorial 10) — agents, bus, scheduler
  - `llm_contexts: Vec<Conversation>` (from Tutorial 08) — one per agent
  - `input_buffer: InputBuffer` (from Tutorial 06) — gap buffer for user input
  - `conversation_log: Vec<ChatMessage>` (from Tutorial 08) — visible conversation
  - `system_log: Vec<Vec<u8>>` — log entries
  - `pending_side_effects: Vec<SideEffect>` — effects to be executed by the shell
- **`SideEffect`** — Enum: `SendHttpRequest(Vec<u8>) | WriteTerminal(Vec<Cell>) | LogMessage(Vec<u8>)`. Describes an effect without performing it.
- **`ViewTree`** — `panes: Vec<(PaneId, Rect, Vec<Cell>)>`. The complete rendered output as data.

#### Functions

- **`app_state_new(config: AppConfig) -> AppState`** — Initialize all sub-components.
- **`app_update(state: AppState, event: AppEvent) -> AppState`** — Pure Elm-architecture update. Dispatches events to the appropriate sub-component:
  - `KeyPress` with Enter on Input pane: extract text from `input_buffer`, create `ChatMessage`, append to `conversation_log`, build LLM request via Tutorial 08, create `SideEffect::SendHttpRequest`, send to orchestrator bus.
  - `LlmChunk`/`LlmDone`: feed into `StreamAccumulator`, update conversation, trigger agent step.
  - `AgentMessage`: route through orchestrator, update agent states.
  - `Tick`: run one `orchestrator_step`.
  - Other `KeyPress`: delegate to TUI input handling (Tutorial 07).
- **`app_view(state: &AppState) -> ViewTree`** — Pure render. Splits the screen into 4 panes and renders each:
  - Input pane: current input buffer contents with cursor.
  - Conversation pane: scrollable list of messages.
  - Agent status pane: current agent phases and activity.
  - System log pane: scrollable log entries.
- **`bridge_chat_to_envelope(msg: &ChatMessage, from: u32, to: u32) -> Envelope`** — Convert a `ChatMessage` (Tutorial 08) to an `Envelope` (Tutorial 10).
- **`bridge_envelope_to_chat(env: &Envelope) -> Option<ChatMessage>`** — Reverse conversion.
- **`state_consistent(state: &AppState) -> bool`** — Check internal consistency: agent count matches LLM context count, focus index in bounds, orchestrator budget not exceeded, all buffer invariants hold.

### Imperative Shell (~400 lines, NOT verified)

#### Functions

- **`main()`** — Entry point: parse config, initialize terminal, run event loop, restore terminal on exit.
- **`read_terminal_event() -> Option<AppEvent>`** — Read from crossterm, convert to `AppEvent`.
- **`send_http_request(data: &[u8]) -> Result<Vec<u8>, Vec<u8>>`** — Execute HTTP request via ureq.
- **`event_loop(state: AppState)`** — The main loop:
  1. Read terminal event (or tick on timeout).
  2. Call `app_update(state, event)` (pure, verified).
  3. Call `app_view(&state)` (pure, verified).
  4. Execute `pending_side_effects` (I/O, unverified).
  5. Render `ViewTree` to terminal (I/O, unverified).
- **`execute_side_effect(effect: &SideEffect) -> Option<AppEvent>`** — Perform the I/O and produce a follow-up event if needed (e.g., HTTP response becomes `LlmDone`).
- **`adapt_terminal_event(raw: crossterm::Event) -> Option<AppEvent>`** — Shell-specific type conversion.

### Estimated Lines

~900 lines total Rust (~500 verified core, ~400 shell).

## Generated Lean (Approximate)

Aeneas will produce translations only for the `verified_core` module:

- **`FullIntegration/Types.lean`**: `AppEvent`, `PaneId`, `AppState`, `SideEffect`, `ViewTree`. `AppState` is a large structure referencing types from Tutorials 06-10.
- **`FullIntegration/Funs.lean`**: `app_update`, `app_view`, `bridge_chat_to_envelope`, `bridge_envelope_to_chat`, `state_consistent`. `app_update` is a large match on `AppEvent` that calls into sub-component functions.

Key translation notes:
- The shell code is NOT translated — it uses `crossterm` and `ureq` which are not Aeneas-compatible.
- The verified core imports types and functions from Tutorials 06-10 via Lean `import` statements.
- `AppState` in Lean is a nested structure; proofs about `app_update` must unfold through multiple layers.
- `SideEffect` is purely descriptive — it carries data but performs no effects. In Lean it is just an inductive with data payloads.

## Theorems to Prove

### `submit_produces_valid_envelope`
**Statement:** When `app_update` processes a `KeyPress Enter` on the Input pane, and the input buffer is non-empty, the resulting `pending_side_effects` contains a well-formed HTTP request, and the orchestrator bus contains a valid envelope with the user's message.
**Proof strategy:** Unfold `app_update` for the Enter/Input case. Show that `bridge_chat_to_envelope` produces a valid envelope (well-formed fields), and `build_request` succeeds (via `build_request_well_formed` from Tutorial 08).

### `app_update_preserves_consistency`
**Statement:** If `state_consistent state = true`, then `state_consistent (app_update state event) = true` for all `event`.
**Proof strategy:** Case split on `event`. For each case, show that the sub-component update preserves its own invariants (using theorems from Tutorials 06-10), and that cross-component invariants (matching counts, bounds) are maintained. This is the main composition theorem.

### `submit_reaches_bus`
**Statement:** A user submission (Enter key with non-empty input) results in an envelope on the orchestrator's message bus.
**Proof strategy:** Follow the `app_update` code path: input buffer extraction, ChatMessage construction, bridge to envelope, `bus_send`. Each step is deterministic; compose the intermediate results.

### `submit_then_tick_delivers`
**Statement:** If `submit_reaches_bus` puts an envelope on the bus, then a subsequent `Tick` event causes `orchestrator_step` to deliver it (via `sent_then_delivered` from Tutorial 10, instantiated for one step).
**Proof strategy:** Apply `sent_then_delivered` from Tutorial 10. The tick event triggers `orchestrator_step`, which calls `bus_deliver`. Since the envelope is at the head of the queue, it is delivered in one step.

### `end_to_end_under_io_axiom`
**Statement:** Assuming the I/O axiom (HTTP requests receive valid LLM responses), a user submission eventually produces a visible response in the conversation pane.
**Proof strategy:** Chain the following: (1) `submit_produces_valid_envelope`, (2) `submit_reaches_bus`, (3) `submit_then_tick_delivers`, (4) the I/O axiom provides `LlmDone`, (5) `app_update` for `LlmDone` appends to `conversation_log`, (6) `app_view` renders the conversation pane from `conversation_log`. The I/O axiom is stated in `IOBoundary.lean`.

### `full_app_orchestrator_terminates`
**Statement:** The orchestrator sub-component of `AppState` terminates within its configured budget across any sequence of `Tick` events.
**Proof strategy:** Direct application of `orchestrator_terminates_within_budget` from Tutorial 10, instantiated with the orchestrator config embedded in `AppState`.

### Estimated Lines

~500 lines proofs.

## New Lean Concepts Introduced

- **Composing verified modules**: Importing and combining theorems from Tutorials 05-10. Lean's module system and `import` mechanism allow building on previously proved results.
- **Axiomatizing I/O boundaries**: The shell performs real I/O that cannot be verified. We introduce axioms that describe the expected behavior of I/O operations (e.g., "HTTP responses are well-formed LLM responses"). These axioms form the **trust boundary**.
- **Trust boundaries**: Explicitly identifying which code is verified (the pure core) and which is trusted (the shell). The Lean development makes this boundary visible: only `verified_core` has proofs; the shell is described by axioms.
- **Verification pyramid**: The tutorial presents the architecture as a pyramid: foundational data structures (Tutorials 05-06) at the base, domain logic (Tutorials 07-10) in the middle, and integration (Tutorial 11) at the top. Each layer's proofs build on the layer below.

## Cross-References

- **From Tutorial 05 (Message Protocol):** Serialization types used in `SideEffect::SendHttpRequest` payloads.
- **From Tutorial 06 (Buffer Management):** `InputBuffer` (gap buffer) provides the user input editing in the Input pane.
- **From Tutorial 07 (TUI Core):** `TuiState`, layout splitting, widget rendering, and focus management provide the entire UI layer.
- **From Tutorial 08 (LLM Client Core):** `Conversation`, `ChatMessage`, `Request`, `StreamAccumulator` handle all LLM protocol logic.
- **From Tutorial 09 (Agent Reasoning):** Each agent instance uses the reasoning engine for its internal step logic.
- **From Tutorial 10 (Multi-Agent Orchestrator):** `OrchestratorState`, message bus, router, and scheduler coordinate multiple agents.
- **This is the capstone tutorial** that ties everything together. All prior tutorials are dependencies.
