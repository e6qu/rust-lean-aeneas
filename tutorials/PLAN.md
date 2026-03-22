# Tutorials Master Plan

## Implementation Order

Tutorials must be implemented sequentially (each builds on prior), but within each tutorial the Rust code and Lean proofs can be developed in parallel once the Aeneas translation is done.

1. **Tutorial 01** — Foundation: install tools, first proofs
2. **Tutorial 02** — First real data structure (functional Stack)
3. **Tutorial 03** — First cross-tutorial proof (imports 02)
4. **Tutorial 04** — Key building block: StateMachine trait (reused in 07-11)
5. **Tutorials 05-06** — Can partially parallelize (independent components)
6. **Tutorials 07-08** — Can partially parallelize (TUI and LLM client are independent)
7. **Tutorial 09** — Depends on 04, 08
8. **Tutorial 10** — Depends on 04, 05, 09
9. **Tutorial 11** — Depends on everything

## Shared Conventions

### Aeneas Translation
- Pipeline: `charon cargo --preset=aeneas` → `aeneas -backend lean *.llbc`
- Use `-split-files` flag for tutorials 05+ (separate Types.lean, Funs.lean, template files)

### Generated Lean Patterns
- `import Aeneas` / `open Aeneas Aeneas.Std Result`
- Rust enums → `inductive`, structs → `structure`, traits → `structure` with `Self : Type`
- Loops → `@[rust_loop] partial_fixpoint` recursive functions
- `&mut` → return pair `(value, backward_fn)`
- All functions return `Result T`

### Proof Patterns
- Theorems: `theorem name (params) (preconds) : conclusion := by`
- `step` / `step*` for monadic decomposition; `@[step]` for composable specs
- `scalar_tac` for bounded integer arithmetic
- `simp`, `omega`, `split`, `cases` for most reasoning

### Rust Patterns (Aeneas-friendly)
- `Vec<u8>` not `String`
- `while` loops with explicit index, not iterators
- `(bool, T)` not `Option<&T>`
- Enum-dispatch not `dyn Trait`
- `u32` IDs not strings in verified core

## Cross-Tutorial Dependencies

| Tutorial | Imports From | Exports To |
|----------|-------------|------------|
| 01 | — | concepts only |
| 02 | — | `rpn.evaluate`, `WellFormedRPN`, `Stack` → 03 |
| 03 | 02 (for equivalence proof) | concepts only |
| 04 | — | `StateMachine` pattern, `Reachable`, `IsInvariant` → 07,08,09,10,11 |
| 05 | — | `Message`, `serialize`/`deserialize` → 10,11 |
| 06 | — | `GapBuffer`, `InputBuffer`, `RingBuffer` → 07,11 |
| 07 | 06 (GapBuffer) | `AppModel`, `WidgetKind`, layout → 11 |
| 08 | — | `ChatMessage`, `Conversation`, `LlmTransport` → 09,10,11 |
| 09 | 04 (SM pattern), 08 (msg types) | `AgentSnapshot`, `agent_run` → 10,11 |
| 10 | 04,05,09 | `OrchestratorState`, `MessageBus` → 11 |
| 11 | 05,06,07,08,09,10 | — (final application) |

## Size Estimates

Total: ~24,500 lines of code + ~8,700 lines of README documentation.
- Rust: ~5,100 lines
- Generated Lean: ~4,900 lines
- Hand-written Lean proofs: ~5,800 lines
- READMEs: ~8,700 lines
