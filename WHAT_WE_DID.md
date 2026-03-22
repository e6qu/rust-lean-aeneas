# What We Did

## Phase 1: Research

1. **Deep research on Lean 4** — history, type theory (CIC), tactics, Mathlib, tooling. Produced `LEAN.md` (3,764 lines).
2. **Deep research on Aeneas** — pipeline (Rust→MIR→Charon→LLBC→Lean), forward/backward functions, supported Rust subset, proof workflow. Produced `AENEAS.md` (2,758 lines).
3. **Research on concurrency limitations** — confirmed ALL concurrency is unsupported by Aeneas (not just async). Architecture solution: "Functional Core, Imperative Shell."

## Phase 2: Planning

4. **Designed 11-tutorial progression** — from simple arithmetic proofs to verified multi-agent LLM harness. Each tutorial with Rust code, Lean proofs, and walkthrough README.
5. **Created master plan** (`PLAN.md`) with dependency graph, concept schedule, size estimates, design decisions.
6. **Created per-tutorial plans** (`PLAN.md` in each tutorial directory) with file structures, code outlines, theorem statements, proof strategies.

## Phase 3: Implementation

7. **Built 11 Rust tutorials** — 297 tests passing across all tutorials:
   - 01: checked_add, safe_divide, safe_abs, clamp (10 tests)
   - 02: RPN calculator with functional stack (10 tests)
   - 03: Infix calculator with recursive descent parser (12 tests)
   - 04: Generic state machines — door lock, traffic light (15 tests)
   - 05: TLV message protocol with roundtrip (16 tests)
   - 06: Ring buffer, gap buffer, input buffer (36 tests)
   - 07: Pure TUI model — layout, widgets, focus (41 tests)
   - 08: LLM client — requests, conversations, streaming (30 tests)
   - 09: Agent reasoning — state machine, tools, retry, guardrails (61 tests)
   - 10: Multi-agent orchestrator — bus, routing, scheduling, protocols (32 tests)
   - 11: Full integration — verified core + imperative shell (34 tests)

8. **Wrote simulated Lean files** — hand-written Types.lean, Funs.lean, and proof files for each tutorial. These were approximations of what Aeneas would generate.

9. **Created standalone Aeneas prelude** (`tutorials/shared/Aeneas.lean`) — self-contained type definitions so Lean files compile without the real Aeneas library.

## Phase 4: Tooling & CI

10. **Added linting** — cargo fmt, cargo clippy with `-D warnings`, intentional Aeneas-compatible allows documented.
11. **Added Makefiles** — root + per-tutorial, targets: check, test, lint, format, build, clean.
12. **Set up GitHub Actions CI** — 11 Rust jobs (fmt+clippy+build+test) + 11 Lean jobs (lake build) + gate.
13. **Created GitHub repo** at `e6qu/rust-lean-aeneas` with SSH key configuration.

## Phase 5: Real Aeneas Translation

14. **Installed Aeneas via Nix** — `nix run github:aeneasverif/aeneas#charon` and `nix run github:aeneasverif/aeneas`.
15. **Ran Charon + Aeneas on all 11 tutorials** — produced real Lean output in `lean/generated/`. 10/11 succeeded (tutorial 07 failed).
16. **Verified generated code compiles** — tested `lake build` with real Aeneas library dependency. The generated Lean typechecks.

## Phase 6: Sorry/Axiom Cleanup

17. **Converted `sorry` to `axiom`** — honest about having no proofs rather than broken stubs.
18. **Researched real Aeneas proof patterns** — studied ICFP tutorial solutions, `step`/`progress` tactic, `scalar_tac`. Understood how real proofs work.

## Key Discovery

The hand-written Lean files (with standalone prelude and axiom declarations) do NOT constitute formal verification. To actually verify the Rust code, we need:
- The **real** Aeneas-generated Lean code (already have it in `lean/generated/`)
- The **real** Aeneas Lean library as a dependency
- **Real proofs** written using `step`/`progress`/`scalar_tac` against the generated code
- `lake build` succeeding = Rust code is formally verified
