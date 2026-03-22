# Project Status

**Last updated:** 2026-03-23

## Purpose

**Formally verify Rust programs using Lean 4 and Aeneas.**

This means: mathematically prove that Rust code is correct — not just that it compiles or passes tests, but that it satisfies precise specifications for ALL possible inputs. The proof is checked by the Lean 4 type checker, which is a small trusted kernel. If `lake build` succeeds, the code is verified.

## Current State

### What Works

| Component | Status | Details |
|-----------|--------|---------|
| Rust code | **Working** | 11 tutorials, 297 tests passing, all linted (clippy + fmt) |
| Aeneas translation | **Working** | 10/11 tutorials successfully translated via `charon` + `aeneas` (Nix) |
| Generated Lean code | **Working** | Real Aeneas output in `lean/generated/` — typechecks against real Aeneas library |
| CI (Rust) | **Working** | 11 individual jobs: fmt, clippy, build, test |
| CI (Lean) | **Partially working** | Builds against standalone prelude (not real Aeneas library) |
| Formal proofs | **NOT working** | Theorem statements exist as `axiom` (unproven). No actual verification happening. |

### What Does NOT Work

1. **No real proofs exist.** All theorems are `axiom` declarations — they assert properties without proof. Lean accepts them but verifies nothing about correctness.

2. **The standalone Aeneas prelude is a fake.** Our `Aeneas.lean` defines simplified types (`U32` as bare `Nat` wrapper) without the real bounds guarantees. Proofs against this prelude would not constitute real verification.

3. **The tutorial Lean files don't use the real generated code.** The hand-written `Funs.lean` files are approximations of what Aeneas generates. The real output lives in `lean/generated/` but isn't used by the proof files.

4. **CI doesn't verify proofs.** The Lean CI jobs build against the standalone prelude, which has no proof obligations. They pass trivially.

### Tutorial Translation Results

| Tutorial | Charon | Aeneas | Notes |
|----------|--------|--------|-------|
| 01 Setup | OK | OK | Clean translation |
| 02 RPN Calculator | OK | OK | Clean (lib only, binary had extern error) |
| 03 Infix Calculator | OK | OK | Clean |
| 04 State Machines | OK | Partial | `type_var_id` errors in generic `run_machine` |
| 05 Message Protocol | OK | OK | Clean |
| 06 Buffer Management | OK | OK | Clean |
| 07 TUI Core | OK | **Failed** | `Unreachable` in `focus_next`/`focus_prev`/`new` |
| 08 LLM Client Core | OK | Partial | `shallow-init-box` error in `Conversation::new` |
| 09 Agent Reasoning | OK | Partial | `break to outer loop` in `validate_tool_call` |
| 10 Multi-Agent | OK | Partial | Missing `filter`/`collect` iterator support |
| 11 Full Integration | OK | Partial | `nested borrows` in `render_conversation` |

### Infrastructure

- **GitHub:** https://github.com/e6qu/rust-lean-aeneas
- **CI:** GitHub Actions — 11 Rust jobs + 11 Lean jobs + gate
- **Rust:** Edition 2024, rustc 1.94.0
- **Lean:** v4.28.0 via elan
- **Aeneas:** Latest from `github:aeneasverif/aeneas` via Nix
- **Charon:** Latest from `github:aeneasverif/aeneas#charon` via Nix
