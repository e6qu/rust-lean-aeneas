# Rust + Lean 4 Formal Verification via Aeneas: Master Plan

> This is the master implementation plan. For the project overview, see [README.md](README.md).
> For current status, see [STATUS.md](STATUS.md). For gap analysis, see [GAP_ANALYSIS.md](GAP_ANALYSIS.md).

## Purpose

**Formally verify Rust programs using Lean 4 and Aeneas.**

The verification pipeline:
1. Write Rust code
2. Translate to pure Lean via Aeneas (`charon` + `aeneas`)
3. Write Lean proofs about the generated code
4. `lake build` typechecks proofs → Rust code is mathematically proven correct

## Context

Build a comprehensive, beginner-friendly tutorial series that teaches formal verification of Rust programs using Lean 4 and Aeneas. The series culminates in a verified TUI multi-agent LLM harness. Each tutorial includes actual working Rust code, **real** Aeneas-generated Lean code, and Lean proofs verified by the Lean typechecker.

Target audience: total beginners (basic algorithm knowledge only).
Architecture: "Functional Core, Imperative Shell" throughout.

## Related Documents

- [STATUS.md](STATUS.md) — Current project status
- [WHAT_WE_DID.md](WHAT_WE_DID.md) — History of what was built
- [GAP_ANALYSIS.md](GAP_ANALYSIS.md) — What's missing for real verification
- [DO_NEXT.md](DO_NEXT.md) — Immediate next steps

## Project Structure

```
rust-lean-aeneas/
├── README.md                           # Project overview, how to navigate
├── PLAN.md                             # This plan (root level)
├── LEAN.md                             # Deep reference on Lean 4
├── AENEAS.md                           # Deep reference on Aeneas
└── tutorials/
    ├── README.md                       # Tutorial series overview + roadmap
    ├── PLAN.md                         # Tutorials master sub-plan
    ├── 01-setup-hello-proof/
    │   ├── README.md                   # Full tutorial walkthrough
    │   ├── PLAN.md                     # Sub-plan for this tutorial
    │   ├── rust/  (Cargo project)
    │   ├── lean/  (Lake project + proofs)
    │   └── scripts/translate.sh
    ├── 02-rpn-calculator/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 03-infix-calculator/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 04-state-machines/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 05-message-protocol/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 06-buffer-management/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 07-tui-core/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 08-llm-client-core/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 09-agent-reasoning/
    │   ├── README.md / PLAN.md / rust/ / lean/
    ├── 10-multi-agent-orchestrator/
    │   ├── README.md / PLAN.md / rust/ / lean/
    └── 11-full-integration/
        ├── README.md / PLAN.md
        ├── rust/src/verified_core/    # Verified pure core
        ├── rust/src/shell/            # Unverified I/O shell
        └── lean/                      # Generated + composition proofs
```

---

## Phase 1: Reference Documents

### LEAN.md
Deep reference covering:
- History (Leonardo de Moura, MSR, Lean FRO), current status (v4.28+)
- Core theory: CIC, Curry-Howard, dependent types, universes, `Prop` vs `Type`
- Language features: dual nature (programming + proving), metaprogramming, do-notation, monads
- Type system: Pi/Sigma types, propositional equality, decidable equality, proof irrelevance
- Mathlib (1.9M+ lines of verified math)
- Tooling: elan, Lake, VS Code extension, LSP
- All key tactics: `simp`, `omega`, `decide`, `ring`, `linarith`, `cases`, `induction`, `apply`, `exact`, `intro`, `calc`, `step`, `scalar_tac`
- Practical verification: specs as types, pre/post conditions
- Installation guide, official resources with URLs

### AENEAS.md
Deep reference covering:
- What it is: Rust → pure functional Lean 4 translator (Son Ho, Inria/Microsoft)
- Pipeline: Rust → MIR → Charon → ULLBC → LLBC → Pure Lean/Coq/F*/HOL4
- Key insight: forward functions + backward functions eliminate memory reasoning
- Charon: MIR extraction, CFG → structured AST
- Supported Rust: safe sequential code, traits, closures (Feb 2026), `dyn Trait`, generics, enums, structs, loops
- NOT supported: async, concurrency, unsafe, interior mutability
- Proof workflow: write Rust → run Aeneas → write Lean proofs (extrinsic)
- The `step`/`progress` tactic, `@[step]` annotation, monadic `Result` encoding
- Comparison with Prusti, Creusot, Verus, Kani
- Installation (Nix or manual), GitHub repos, papers (ICFP 2022, 2024, CAV 2025)

---

## Phase 2: Tutorials

### Aeneas Conventions (shared across all tutorials)

**Generated Lean patterns:**
- `import Aeneas` then `open Aeneas Aeneas.Std Result`
- Rust enums → Lean `inductive`, structs → Lean `structure`, traits → Lean `structure` with `Self : Type`
- Loops → `@[rust_loop] partial_fixpoint` recursive functions
- Mutable refs → `(value, backward_fn)` return pairs
- All fallible ops return `Result T`; Rust `Result<A,B>` nests inside Aeneas `Result`
- Vec ops use `alloc.vec.Vec.*` namespace

**Proof patterns:**
- `theorem func_spec (params) (preconds) : func params = ok result := by`
- `step` tactic decomposes monadic ops; `step*` for repeated; specs annotated `@[step]`
- `scalar_tac` for bounded integer arithmetic
- `simp`, `simp_all`, `omega`, `split` for simplification and case analysis

**Pipeline:** `charon cargo --preset=aeneas` → `.llbc` → `aeneas -backend lean file.llbc` → `.lean`

---

### Tutorial 01: Setup and Hello Proof

**Goal:** Install toolchain, translate 4 simple Rust functions, write first proofs.

**Rust code** (`lib.rs`, ~50 lines):
- `checked_add(x: u32, y: u32) -> Option<u32>` — overflow-safe addition
- `safe_divide(x: i64, y: i64) -> Result<i64, ()>` — division-by-zero safe
- `safe_abs(x: i64) -> Result<i64, ()>` — handles `i64::MIN`
- `clamp(x: i32, lo: i32, hi: i32) -> i32` — bounded clamping

**Proofs** (~80 lines):
1. `checked_add_no_panic` — never panics (outer `Result` always `ok`)
2. `checked_add_spec` — full correctness: returns `Some` iff no overflow
3. `safe_divide_spec` — returns result when `y ≠ 0`, result = `x / y`
4. `clamp_spec` — result always in `[lo, hi]`

**New concepts:** Curry-Howard, types-as-propositions, monadic translation, `step` tactic, `scalar_tac`, `Result` monad, `↑` coercion.

**Cross-refs:** → Tutorial 02 (first real data structure)

---

### Tutorial 02: RPN Calculator

**Goal:** Stack machine with deep correctness proofs.

**Rust code** (~180 lines):
- `Token` enum: `Num(i64)`, `Plus`, `Minus`, `Mul`, `Div`
- `Stack` as functional linked list (not `Vec` — Aeneas-friendly): `Empty | Push(i64, Box<Stack>)`
- `eval_step(stack, token)` — one evaluation step
- `evaluate(tokens)` — full evaluation loop

**Proofs** (~250 lines):
1. `tokenize_word_digit_spec` — valid digit bytes → `Num` token
2. `eval_step_num_depth` / `eval_step_binop_depth` — stack depth changes ±1
3. `wf_rpn_evaluate_succeeds` — well-formed expressions always evaluate (structural induction on `WellFormedRPN`)
4. `div_by_zero_caught` — division by zero detected

**Specification:** Inductive `WellFormedRPN` predicate + `rpn_semantics` function.

**New concepts:** Inductive types in Lean, pattern matching in proofs, `cases`, `induction`, List reasoning.

**Cross-refs:** ← Tutorial 01 (`step`, `scalar_tac`); → Tutorial 03 (equivalence proof imports these)

---

### Tutorial 03: Infix Calculator

**Goal:** Recursive descent parser + AST evaluator + equivalence with RPN.

**Rust code** (~225 lines):
- `Op`, `Token` (with `LParen`, `RParen`), `Expr` (recursive with `Box`)
- Lexer: `lex(input: &[u8]) -> Result<Vec<Token>, ParseError>`
- Parser: `parse_expr`, `parse_term`, `parse_factor` (index-passing style, no iterators)
- Evaluator: `eval(expr: &Expr) -> Result<i64, EvalError>`

**Proofs** (~350 lines):
1. `parse_factor_advances` — parser always advances position (termination)
2. `eval_correct` — evaluator matches mathematical `expr_semantics`
3. `infix_rpn_equivalence` — **cross-tutorial**: `eval(expr) = rpn.evaluate(expr_to_rpn(expr))`
4. `parse_terminates` — well-founded induction on token consumption

**New concepts:** Recursive data types, structural induction on trees, `calc` blocks, `simp`, `omega`.

**Cross-refs:** ← Tutorial 02 (imports `rpn.evaluate` and `WellFormedRPN` for equivalence proof)

---

### Tutorial 04: State Machines

**Goal:** Generic `StateMachine` trait + two concrete machines + generic invariant proof.

**Rust code** (~290 lines):
- `trait StateMachine { type State; type Event; type Action; fn transition(...); }`
- `run_machine<M: StateMachine>(...)` — generic runner
- `DoorLock`: states `Locked/Unlocked/Alarmed`, events `EnterCode(u32)/TurnHandle/Reset`, tracks `wrong_attempts`
- `TrafficLight`: 4-phase cycle, NS/EW light colors

**Proofs** (~300 lines):
1. `alarmed_needs_three_wrong` — Alarmed only reachable after 3 wrong codes
2. `reset_goes_to_locked` — Reset always → Locked (liveness)
3. `traffic_never_both_green` — mutual exclusion invariant
4. `invariant_induction` — **generic**: if `P(init)` and `∀ s e, P(s) → P(trans(s,e))`, then `P` is invariant

**Specification:** `Reachable` inductive predicate, `IsInvariant` definition, `multi_step`.

**New concepts:** Traits as Lean structures, generic proofs, state space reasoning, invariant proofs, `decide` tactic.

**Cross-refs:** ← Tutorials 01-03; → Tutorials 07-11 (StateMachine pattern reused everywhere). **KEY BUILDING BLOCK.**

---

### Tutorial 05: Message Protocol

**Goal:** Binary serialize/deserialize with roundtrip proof.

**Rust code** (~400 lines):
- `Message` enum: `Text(Vec<u8>)`, `Command(CmdType, Vec<Vec<u8>>)`, `Error(ErrorCode, Vec<u8>)`, `Heartbeat(u64)`
- TLV format: `[tag: u8][length: u32 BE][payload]`
- `serialize(&Message) -> Vec<u8>`, `deserialize(&[u8]) -> Result<(Message, usize), ParseError>`
- `FrameAccumulator`: feed bytes, extract complete messages

**Design note:** Use `Vec<u8>` throughout (not `String`) — Aeneas handles `Vec<u8>` as `List U8` natively.

**Proofs** (~500 lines):
1. `roundtrip` — `deserialize(serialize(msg)) = ok (msg, serialize_length(msg))` for all messages
2. `length_prefix_correct` — encoded length matches payload
3. `framing_no_loss` — complete byte stream → all messages extracted in order
4. `invalid_tag_rejected` — malformed input always caught

**New concepts:** Byte sequence reasoning (`List U8`), `omega` for arithmetic bounds, serialization invariants.

**Cross-refs:** → Tutorials 10-11 (wire format for inter-agent communication)

---

### Tutorial 06: Buffer Management

**Goal:** Verified data structures for the TUI.

**Rust code** (~450 lines):
- `RingBuffer<T>`: fixed-capacity circular buffer (pre-allocated `Vec`, modular indexing). Uses `(bool, T)` instead of `Option<T>` for Aeneas compatibility.
- `GapBuffer`: text editing buffer (`Vec<u8>`, gap_start/gap_end). Content = `buffer[..gap_start] ++ buffer[gap_end..]`.
- `InputBuffer`: wraps GapBuffer with `delete_word` (loop: skip whitespace backward, skip non-whitespace backward).

**Proofs** (~700 lines):
1. `capacity_invariant` — `rb.len <= rb.capacity` always
2. `fifo_order` — `ring_to_list(push(rb, item)) = ring_to_list(rb) ++ [item]`
3. `push_pop_roundtrip` — push then pop returns pushed element
4. `insert_preserves_content` — gap buffer content after insert = `pre ++ [ch] ++ post`
5. `cursor_always_valid` — `gap_start <= content_len`
6. `delete_word_correct` — removes exactly the word before cursor

**Abstraction functions:** `ring_to_list(rb)`, `gap_content(gb)`.

**New concepts:** Modular arithmetic reasoning, Vec/array proofs, gap buffer theory, loop invariants for fixpoints.

**Cross-refs:** ← Tutorial 04 (invariant pattern); → Tutorial 07 (`GapBuffer`/`InputBuffer` imported for TextBox widget)

---

### Tutorial 07: TUI Core

**Goal:** Pure TUI model — layout, widgets, events, focus — with zero actual terminal I/O.

**Rust code** (~600 lines):
- `Rect`, `Position`, `Cell` (u16 coords, u8 char, u8 style)
- `Event` enum: `Key(KeyCode, Modifiers)`, `Resize(u16, u16)`, `Tick`
- Layout: `split(area, dir, count) -> Vec<Rect>`, `split_at(area, dir, offset) -> (Rect, Rect)`
- `WidgetKind` enum (arena pattern, index-based children — not recursive Box):
  `TextBox(InputBuffer)`, `ScrollableList{items, selected, scroll_offset}`, `StatusBar`, `Border{child_index}`, `Container{dir, children}`
- `AppModel`: widgets `Vec<WidgetKind>`, areas `Vec<Rect>`, focus_index, screen dimensions
- `update(model, event) -> Action`, `render(model) -> Vec<Cell>`

**Design note:** Arena pattern (flat `Vec<WidgetKind>` with index refs) avoids recursive `Box` trees — cleaner Aeneas translation and simpler proofs.

**Proofs** (~900 lines):
1. `split_no_overlap` — sub-rects don't intersect
2. `split_covers` — sub-rects tile parent exactly
3. `render_in_bounds` — all cells within allocated Rect
4. `single_focus` — `focus_index < widgets.len` always
5. `key_events_to_focus` — non-Tab keys only modify focused widget
6. `scroll_in_bounds` — scroll position always valid

**New concepts:** Widget trees as inductives, structural proofs on trees, Rect arithmetic lemmas, composing verified components.

**Cross-refs:** ← Tutorial 06 (GapBuffer in TextBox); ← Tutorial 04 (update = state machine pattern); → Tutorial 11

---

### Tutorial 08: LLM Client Core

**Goal:** Pure LLM protocol logic — no HTTP, no JSON.

**Rust code** (~550 lines):
- `ChatMessage` enum: `RoleMessage(Role, Vec<u8>)`, `ToolCall(ToolCallInfo)`, `ToolResult(ToolResultInfo)`
- `Request`: model, messages, temperature (u32 fixed-point), max_tokens, tools
- `build_request(...)` — validates and constructs (returns `Result<Request, RequestError>`)
- `Response` parsing from simplified length-prefixed format (not real JSON)
- Token estimation: `estimate_tokens(msgs) ≈ total_bytes / 4 + overhead`
- `Conversation`: append (validates alternation), `trim_to_context` (removes oldest non-system messages)
- `StreamAccumulator`: accumulate response chunks
- `trait LlmTransport { fn send_request(...) }` — boundary trait

**Design notes:** `Vec<u8>` not `String`; `u32` fixed-point not `f32` for temperature; simplified response format (real JSON parsing too complex for Aeneas).

**Proofs** (~750 lines):
1. `build_request_well_formed` — validated requests satisfy all API constraints
2. `append_preserves_alternation` — conversation role pattern maintained
3. `trim_respects_context` — after trim, tokens <= max OR messages at minimum (2)
4. `chunks_concat_eq_original` — streaming chunks reconstruct full response
5. `estimate_within_factor_2` — token estimate within 2x of actual (uses `axiom token_bound`)

**New concepts:** Modeling external APIs as traits with specs, axioms for external properties, approximation proofs.

**Cross-refs:** ← Tutorial 04 (trait-as-structure); ← Tutorial 05 (serialization ideas); → Tutorials 09-11

---

### Tutorial 09: Agent Reasoning

**Goal:** Verified single-agent reasoning engine.

**Rust code** (~550 lines):
- `AgentPhase`: `Idle`, `Thinking`, `CallingTool`, `AwaitingToolResult`, `Composing`, `Done`, `Error`
- `agent_transition(phase, event) -> Option<(AgentPhase, AgentAction)>` — pure match on pairs
- `Step` enum: `Observe(u32)`, `Think(u32)`, `Decide(Decision)`, `Act(u32)` — chain-of-thought as data
- `is_chain_well_formed(chain)` — monotonicity of step order
- `ToolSpec`, `find_tool(registry, name_id)`, `validate_tool_call(spec, args)`
- `RetryState`: exponential backoff as pure state transformation
- `GuardrailConfig`: max_message_len, max_recursion_depth, max_reasoning_steps
- `agent_run(config, initial, events) -> AgentSnapshot` — bounded main loop

**Design notes:** `u32` IDs instead of `String` (verified core avoids strings); `Vec<ToolSpec>` not `HashMap`; enum-dispatch not `dyn Trait`.

**Proofs** (~600 lines):
1. `agent_transition_from_terminal_is_none` — terminal states reject all events
2. `agent_run_terminates` — always reaches `Done` or `Error` within max_steps
3. `tool_call_uses_registered_tool` — only registered tools called; params match schema
4. `append_preserves_well_formed` — reasoning chain remains Observe≤Think≤Decide≤Act
5. `next_retry_increases_attempt` / `retry_stops_at_max` — backoff correctness
6. `all_guards_pass_implies_within_limits` — guardrail enforcement

**New concepts:** Reachability analysis, termination via decreasing measures, schema validation proofs.

**Cross-refs:** ← Tutorial 04 (StateMachine); ← Tutorial 08 (message types); → Tutorials 10-11

---

### Tutorial 10: Multi-Agent Orchestrator

**Goal:** Multiple agents communicating via verified message bus with scheduling and protocols.

**Rust code** (~900 lines):
- `AgentKind` enum-dispatch: `Coordinator(CoordinatorState)`, `Specialist(SpecialistState)`, `Critic(CriticState)`
- `AgentInstance`: id, kind, state, inbox, outbox
- `MessageBus`: queue, delivered (audit trail), next_seq. `bus_send`, `bus_deliver`
- `Router`: `resolve_recipient(recipient, all_ids)` — Direct/Broadcast/Topic
- `Scheduler`: RoundRobin/Priority, `turns_given` counter
- Protocols: `VotingRound` (cast_vote, tally), `Pipeline` (stages with input/output kinds), `DebateState` (bounded rounds)
- `orchestrator_step` / `orchestrator_run` — budget-bounded main loop

**Proofs** (~900 lines):
1. `resolve_direct_delivers_to_target` — Direct routing correctness
2. `sent_then_delivered` — no message loss (send → deliver composition)
3. `round_robin_fairness` — after `n * |agents|` turns, each agent has `≥ n` turns
4. `orchestrator_terminates_within_budget` — turn_count ≤ turn_budget always
5. `tally_matches_majority` — voting result = majority of cast votes
6. `valid_pipeline_types_match` — output kind of stage N = input kind of stage N+1
7. `debate_terminates` — decided=true OR rounds≥max_rounds

**New concepts:** Sequential simulation of distributed systems, fairness proofs, `Finset`, pigeonhole, composition of verified components.

**Cross-refs:** ← Tutorial 04 (each protocol is a state machine); ← Tutorial 05 (message types); ← Tutorial 09 (agent internals); → Tutorial 11

---

### Tutorial 11: Full Integration

**Goal:** Wire everything into a complete TUI multi-agent LLM application.

**Rust code — verified core** (~500 lines):
- `AppEvent` enum: unifies TUI, agent, LLM, and system events
- `AppState`: combines `TuiState` (07), `OrchestratorState` (10), LLM contexts (08), `InputBuffer` (06)
- `app_update(state, event) -> AppState` — pure Elm-architecture update
- `app_view(state) -> ViewTree` — pure view (4 panes: chat, agent status, input, debug/reasoning)
- `message_bridge.rs` — converts between component message types

**Rust code — imperative shell** (~400 lines, NOT verified):
- `terminal_io.rs` — crossterm terminal I/O
- `http_client.rs` — ureq HTTP for LLM API
- `event_loop.rs` — read events → `app_update` → render → handle side effects
- `adapters.rs` — maps shell types ↔ core types

**Proofs** (~500 lines):
1. `submit_produces_valid_envelope` — bridge functions produce valid component inputs
2. `app_update_preserves_consistency` — `state_consistent` invariant maintained across all events
3. `submit_reaches_bus` + `submit_then_tick_delivers` — end-to-end message flow
4. `shell_translate_faithful` (axiom) + `end_to_end_under_io_axiom` — I/O boundary
5. `full_app_orchestrator_terminates` — composes Tutorial 10's budget proof

**New concepts:** Composing verified modules, axiomatizing I/O boundaries, trust boundaries, the verification pyramid.

**Cross-refs:** ← Everything from tutorials 05-10 as dependencies

---

## Dependency Graph

```
01 Setup
 ↓
02 RPN Calculator
 ↓ ↘
03 Infix Calculator (imports 02 for equivalence)
 ↓
04 State Machines ──────────────────────────────────────┐
 ↓                                                      │
05 Message Protocol ──────────────────────────────┐     │
 ↓                                                │     │
06 Buffer Management ─────────────────────┐       │     │
 ↓                                        │       │     │
07 TUI Core ←(06 GapBuffer)              │       │     │
 ↓                                        │       │     │
08 LLM Client Core ←(04 traits)          │       │     │
 ↓                                        │       │     │
09 Agent Reasoning ←(04 SM, 08 msgs)     │       │     │
 ↓                                        │       │     │
10 Multi-Agent ←(04,05,09)               │       │     │
 ↓                                        ↓       ↓     ↓
11 Full Integration ←(05,06,07,08,09,10)
```

## Concept Introduction Schedule

| Tutorial | New Lean Concepts |
|----------|-------------------|
| 01 | Curry-Howard, Result monad, `step`/`step*`, `scalar_tac`, `↑` coercion |
| 02 | Inductive types, `cases`, `induction`, List reasoning |
| 03 | Recursive data types, structural induction on trees, `calc`, `simp`, `omega` |
| 04 | Traits as structures, generic proofs, invariant proofs, `decide` |
| 05 | Byte sequence reasoning (`List U8`), `omega` for bounds, serialization invariants |
| 06 | Modular arithmetic, Vec/array proofs, loop invariants for fixpoints |
| 07 | Widget trees, structural proofs on trees, Rect arithmetic, component composition |
| 08 | Trait specs, `axiom` keyword, approximation proofs |
| 09 | Reachability, termination via decreasing measures, schema validation |
| 10 | Fairness proofs, `Finset`, pigeonhole, distributed systems reasoning |
| 11 | Module composition, I/O axiomatization, trust boundaries |

## Estimated Sizes

| Tutorial | Rust LoC | Gen Lean | Proof Lean | README | Total |
|----------|----------|----------|------------|--------|-------|
| 01 | 50 | 60 | 80 | 500 | ~690 |
| 02 | 180 | 120 | 250 | 450 | ~1000 |
| 03 | 225 | 200 | 350 | 500 | ~1275 |
| 04 | 290 | 180 | 300 | 550 | ~1320 |
| 05 | 400 | 350 | 500 | 800 | ~2050 |
| 06 | 450 | 400 | 700 | 900 | ~2450 |
| 07 | 600 | 550 | 900 | 1000 | ~3050 |
| 08 | 550 | 500 | 750 | 1100 | ~2900 |
| 09 | 550 | 700 | 600 | 900 | ~2750 |
| 10 | 900 | 1200 | 900 | 1000 | ~4000 |
| 11 | 900 | 650 | 500 | 1000 | ~3050 |
| **Total** | **~5095** | **~4910** | **~5830** | **~8700** | **~24535** |

Plus LEAN.md (~2000 lines) and AENEAS.md (~1500 lines).

## Key Design Decisions

1. **`Vec<u8>` not `String`** — Aeneas handles `Vec<u8>` as `List U8` natively
2. **`u32` IDs not strings in core** — verified core avoids string ops; shell maintains string tables
3. **Functional `Stack` not `Vec`** (tutorial 02) — clean inductive type for proofs
4. **Arena pattern for widgets** (tutorial 07) — flat `Vec<WidgetKind>` with index refs, not recursive Box trees
5. **Enum-dispatch not `dyn Trait`** (tutorial 10) — statically known agent types → simpler case-splitting in proofs
6. **Explicit `while` loops not iterators** — cleaner Aeneas translation to `partial_fixpoint`
7. **`(bool, T)` not `Option<T>` for returns** — avoids Aeneas edge cases with borrows in `Option`
8. **No real JSON parsing** (tutorial 08) — simplified length-prefixed format; real JSON is a project in itself

## Implementation Order

1. **LEAN.md** + **AENEAS.md** (can be done in parallel)
2. **Root README.md** + **Root PLAN.md**
3. **tutorials/README.md** + **tutorials/PLAN.md**
4. **Tutorials 01-04** (foundations — sequential, each builds on prior)
5. **Tutorials 05-06** (can partially parallelize — 05 is message protocol, 06 is buffers)
6. **Tutorials 07-08** (can partially parallelize — 07 is TUI, 08 is LLM client)
7. **Tutorial 09** (agent reasoning — depends on 04, 08)
8. **Tutorial 10** (multi-agent — depends on 04, 05, 09)
9. **Tutorial 11** (integration — depends on everything)

## Verification

For each tutorial:
1. `cargo test` passes in `rust/` directory
2. `charon cargo --preset=aeneas` produces `.llbc` without errors
3. `aeneas -backend lean *.llbc` produces Lean files without errors
4. `lake build` in `lean/` directory succeeds (proofs check)
5. README walkthrough is self-consistent with actual code
