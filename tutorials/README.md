[← Back to Project Root](../README.md)

# Tutorials: Verified Rust with Lean 4 and Aeneas

This directory contains 11 tutorials that progressively teach formal verification of Rust programs. Each tutorial builds on the previous ones, introducing new concepts while constructing increasingly sophisticated verified software.

## How to Navigate

**Start with Tutorial 01** and work through them in order. Each tutorial's README.md is self-contained but references concepts from earlier tutorials.

## Overview

### Foundation (Tutorials 01-04)

These tutorials teach the fundamentals of Lean 4, Aeneas, and formal proof.

| # | Tutorial | Description | Key Proof |
|---|---------|-------------|-----------|
| 01 | [Setup and Hello Proof](01-setup-hello-proof/) | Install toolchain, verify simple functions | `checked_add` never panics |
| 02 | [RPN Calculator](02-rpn-calculator/) | Stack machine with functional linked list | Well-formed expressions always evaluate correctly |
| 03 | [Infix Calculator](03-infix-calculator/) | Recursive descent parser and AST | Infix evaluation = RPN evaluation (cross-tutorial!) |
| 04 | [State Machines](04-state-machines/) | Generic StateMachine trait, door lock, traffic light | Generic invariant preservation theorem |

### Building Blocks (Tutorials 05-08)

These tutorials build the verified components that the final application uses.

| # | Tutorial | Description | Key Proof |
|---|---------|-------------|-----------|
| 05 | [Message Protocol](05-message-protocol/) | Binary serialize/deserialize with TLV format | Roundtrip: `deserialize(serialize(msg)) = msg` |
| 06 | [Buffer Management](06-buffer-management/) | Ring buffer, gap buffer, input buffer | FIFO ordering, content preservation |
| 07 | [TUI Core](07-tui-core/) | Pure TUI model: layout, widgets, events | Layout non-overlap, render bounds |
| 08 | [LLM Client Core](08-llm-client-core/) | Pure LLM protocol: requests, conversations, streaming | Context window guarantee, streaming correctness |

### Agent System (Tutorials 09-11)

These tutorials build and verify the multi-agent LLM system.

| # | Tutorial | Description | Key Proof |
|---|---------|-------------|-----------|
| 09 | [Agent Reasoning](09-agent-reasoning/) | Single-agent state machine, tools, guardrails | Agent always terminates within budget |
| 10 | [Multi-Agent Orchestrator](10-multi-agent-orchestrator/) | Message bus, routing, scheduling, protocols | Round-robin fairness, no message loss |
| 11 | [Full Integration](11-full-integration/) | Complete TUI multi-agent LLM application | End-to-end state consistency |

## Concept Introduction Schedule

Each tutorial introduces specific Lean 4 concepts. By the end, you'll know:

| Tutorial | New Lean Concepts |
|----------|-------------------|
| 01 | Curry-Howard correspondence, `Result` monad, `step`/`step*` tactic, `scalar_tac`, `↑` coercion |
| 02 | Inductive types, `cases`, `induction`, `List` reasoning |
| 03 | Recursive data types, structural induction on trees, `calc` blocks, `simp`, `omega` |
| 04 | Traits as Lean structures, generic proofs, invariant proofs, `decide` tactic |
| 05 | Byte sequence reasoning (`List U8`), `omega` for arithmetic bounds |
| 06 | Modular arithmetic, Vec/array proofs, loop invariants for fixpoints |
| 07 | Widget trees as inductives, Rect arithmetic lemmas, composing verified components |
| 08 | Trait specifications, `axiom` keyword, approximation proofs |
| 09 | Reachability analysis, termination via decreasing measures |
| 10 | Fairness proofs, `Finset`, pigeonhole reasoning |
| 11 | Module composition, I/O axiomatization, trust boundaries |

## Common Patterns

All tutorials follow the **Functional Core, Imperative Shell** architecture:
- **Verified core**: Pure Rust code → Aeneas → Lean proofs
- **Imperative shell**: I/O wrappers (terminal, network) — NOT verified

All tutorials use these Aeneas-friendly Rust patterns:
- `Vec<u8>` instead of `String`
- Explicit `while` loops instead of iterator chains
- Enum-dispatch instead of `dyn Trait` where possible
- `(bool, T)` instead of `Option<&T>` for returns

## The Translation Pipeline

Every tutorial uses this same workflow:

```bash
# 1. Build and test Rust code
cd rust/
cargo test

# 2. Extract MIR with Charon
charon cargo --preset=aeneas
# Produces: *.llbc

# 3. Translate to Lean with Aeneas
aeneas -backend lean *.llbc
# Produces: lean/*.lean files

# 4. Build and check Lean proofs
cd ../lean/
lake build
# If this succeeds, all proofs are verified!
```
