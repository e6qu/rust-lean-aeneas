# Rust + Lean 4 Formal Verification via Aeneas

A comprehensive, beginner-friendly tutorial series that teaches **formal verification of Rust programs** using **Lean 4** and **Aeneas**. The series culminates in a fully verified TUI multi-agent LLM harness.

## What This Project Is

This project demonstrates how to **prove Rust programs correct** — not just that they compile, not just that they pass tests, but that they satisfy precise mathematical specifications for *all possible inputs*.

The key tools:

- **Rust** — the systems programming language whose type system guarantees memory safety
- **Lean 4** — a theorem prover and programming language that can express and verify mathematical proofs
- **Aeneas** — a tool that translates Rust code into pure functional Lean code, enabling formal proofs about Rust programs

## Architecture: Functional Core, Imperative Shell

Every project in this series follows the same pattern:

```
┌─────────────────────────────────────────┐
│         Imperative Shell (I/O)          │  ← NOT verified (terminal, network, files)
│  ┌───────────────────────────────────┐  │
│  │       Verified Pure Core          │  │  ← Translated by Aeneas, proved in Lean
│  │  (state machines, parsers,        │  │
│  │   protocols, business logic)      │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

The **pure core** contains all the logic that matters — state transitions, data transformations, protocol handling. This is what Aeneas translates and what we prove correct. The **imperative shell** handles I/O (reading the terminal, making HTTP calls) and is intentionally left unverified.

## Prerequisites

- Basic knowledge of algorithms (loops, recursion, data structures)
- A computer with macOS or Linux
- Willingness to learn — no prior Lean, formal verification, or advanced Rust knowledge required

## Project Structure

```
rust-lean-aeneas/
├── LEAN.md                    Deep reference on Lean 4
├── AENEAS.md                  Deep reference on Aeneas
├── PLAN.md                    Master implementation plan
│
└── tutorials/
    ├── 01-setup-hello-proof/       Install tools, first proof
    ├── 02-rpn-calculator/          Stack machine + correctness proofs
    ├── 03-infix-calculator/        Parser + equivalence with RPN
    ├── 04-state-machines/          Generic state machines + invariant proofs
    ├── 05-message-protocol/        Binary protocol + roundtrip proofs
    ├── 06-buffer-management/       Ring buffer, gap buffer + data structure proofs
    ├── 07-tui-core/                Pure TUI model + layout/render proofs
    ├── 08-llm-client-core/         LLM protocol logic + well-formedness proofs
    ├── 09-agent-reasoning/         Agent state machine + termination proofs
    ├── 10-multi-agent-orchestrator/ Multi-agent system + fairness/routing proofs
    └── 11-full-integration/        Complete verified TUI LLM agent harness
```

## Tutorial Progression

The tutorials are designed to be followed in order. Each one introduces new Lean concepts and builds on previous work:

| Tutorial | What You Build | What You Prove | New Concepts |
|----------|---------------|----------------|--------------|
| [01](tutorials/01-setup-hello-proof/README.md) | Simple arithmetic functions | No panics, correct results | Curry-Howard, `step` tactic |
| [02](tutorials/02-rpn-calculator/README.md) | RPN calculator | Stack invariants, evaluator correctness | Inductive types, `cases`, `induction` |
| [03](tutorials/03-infix-calculator/README.md) | Infix calculator + parser | Parser termination, RPN equivalence | Recursive types, tree induction, `simp` |
| [04](tutorials/04-state-machines/README.md) | State machine framework | Safety, liveness, mutual exclusion | Traits as structures, `decide` |
| [05](tutorials/05-message-protocol/README.md) | Binary message protocol | Roundtrip, no message loss | Byte reasoning, `omega` |
| [06](tutorials/06-buffer-management/README.md) | Ring buffer, gap buffer | Capacity, FIFO, content preservation | Modular arithmetic, loop invariants |
| [07](tutorials/07-tui-core/README.md) | TUI model (pure) | Layout correctness, render bounds | Widget trees, Rect arithmetic |
| [08](tutorials/08-llm-client-core/README.md) | LLM client (pure) | Request validity, context window | Trait specs, axioms |
| [09](tutorials/09-agent-reasoning/README.md) | Agent reasoning engine | Termination, tool safety, guardrails | Decreasing measures, reachability |
| [10](tutorials/10-multi-agent-orchestrator/README.md) | Multi-agent orchestrator | Routing, fairness, budget, voting | Distributed reasoning, `Finset` |
| [11](tutorials/11-full-integration/README.md) | Full TUI LLM agent | End-to-end composition, state consistency | Module composition, I/O axioms |

## Dependency Graph

```
01 Setup
 ↓
02 RPN Calculator
 ↓
03 Infix Calculator ←── imports 02 for equivalence proof
 ↓
04 State Machines ─── KEY BUILDING BLOCK for 07-11
 ↓
05 Message Protocol ──────────────────────┐
 ↓                                        │
06 Buffer Management ─────────────┐       │
 ↓                                │       │
07 TUI Core ←── uses 06           │       │
 ↓                                │       │
08 LLM Client Core ←── uses 04   │       │
 ↓                                │       │
09 Agent Reasoning ←── uses 04,08 │       │
 ↓                                │       │
10 Multi-Agent ←── uses 04,05,09  │       │
 ↓                                ↓       ↓
11 Full Integration ←── uses 05,06,07,08,09,10
```

## Quick Start

See [Tutorial 01: Setup and Hello Proof](tutorials/01-setup-hello-proof/README.md) to get started.

## Reference Documents

- [LEAN.md](LEAN.md) — Everything you need to know about Lean 4
- [AENEAS.md](AENEAS.md) — Everything you need to know about Aeneas

## How Each Tutorial Works

Every tutorial (02 and above) has the same structure:

```
tutorials/NN-name/
├── README.md          Detailed walkthrough with explanations
├── PLAN.md            Implementation plan and design decisions
├── rust/
│   ├── Cargo.toml     Rust project
│   └── src/           Rust source code (the verified core)
└── lean/
    ├── lakefile.lean   Lean project config
    ├── lean-toolchain  Lean version
    └── ...             Generated Lean + hand-written proofs
```

The workflow for each tutorial:
1. Read the README for context and theory
2. Study the Rust code in `rust/src/`
3. Run Charon + Aeneas to generate Lean code
4. Study the generated Lean in `lean/`
5. Read and understand the hand-written proofs
6. Try the exercises

## Key Design Decisions

Several decisions were made to keep the code Aeneas-friendly:

- **`Vec<u8>` instead of `String`** — Aeneas maps `Vec<u8>` directly to Lean's `List U8`
- **Explicit `while` loops instead of iterators** — translates cleanly to recursive functions
- **Enum-dispatch instead of `dyn Trait`** — enables simpler case-splitting in proofs
- **Functional data structures** (e.g., linked-list Stack) — natural inductive types in Lean
- **`u32` IDs instead of strings** in the verified core — avoids string complexity

See [PLAN.md](PLAN.md) for the full rationale behind each decision.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
