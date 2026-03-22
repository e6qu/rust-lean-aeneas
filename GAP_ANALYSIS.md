# Gap Analysis: What's Missing for Real Formal Verification

## The Goal

**Prove Rust programs correct** by:
1. Translating Rust → pure Lean via Aeneas
2. Writing Lean proofs about the generated code
3. Lean typechecker verifies the proofs → Rust code is proven correct

## Gap 1: No Real Proofs

**Current:** All theorems are `axiom` — unproven assertions. Lean accepts them but verifies nothing.

**Needed:** Real proofs using Aeneas tactics (`step`/`progress`, `scalar_tac`, `simp_all`).

**How to close:** Write proofs following the ICFP tutorial patterns:
```lean
-- Current (BROKEN — proves nothing):
axiom clamp_in_bounds (x lo hi : Std.I32) (h : lo ≤ hi) :
    ∃ r, clamp x lo hi = ok r ∧ lo ≤ r ∧ r ≤ hi

-- Needed (REAL — Lean verifies this):
@[pspec]
theorem clamp_in_bounds (x lo hi : Std.I32) (h : lo ≤ hi) :
    ∃ r, clamp x lo hi = ok r ∧ lo ≤ r ∧ r ≤ hi := by
  rw [clamp]
  split <;> split <;> simp_all <;> constructor <;> scalar_tac
```

**Effort:** Medium per theorem. Start with tutorials 01-03 (simplest), then 04-06.

## Gap 2: Proof Files Use Fake Prelude Instead of Real Aeneas Library

**Current:** Each tutorial has a standalone `Aeneas.lean` with simplified types (`U32` as bare `Nat` wrapper). Proof files import this fake prelude.

**Needed:** Lakefiles that `require aeneas from git` and proof files that import the real generated code.

**How to close:**
1. Change each tutorial's `lakefile.lean` to depend on the real Aeneas library
2. Replace hand-written `Funs.lean` with the real generated code from `lean/generated/`
3. Rewrite proof files to work against real Aeneas types

**Effort:** Small per tutorial (lakefile change + file reorganization). The proofs themselves are Gap 1.

## Gap 3: Proof Files Don't Reference the Real Generated Code

**Current:** Hand-written `Funs.lean` files approximate what Aeneas generates. The real output is in `lean/generated/` but unused.

**Needed:** Proof files that `import` the real generated code and prove properties about the real generated functions.

**How to close:** For each tutorial:
```
lean/
├── lakefile.lean              # requires aeneas from git
├── lean-toolchain             # matches Aeneas's toolchain
├── HelloProof.lean            # REAL Aeneas output (was in generated/)
└── HelloProof/
    └── Proofs.lean            # Hand-written proofs importing HelloProof
```

## Gap 4: CI Doesn't Verify Proofs Against Real Aeneas

**Current:** Lean CI builds against standalone prelude. Axioms always pass.

**Needed:** Lean CI builds against real Aeneas library. Real proofs must typecheck.

**How to close:**
1. Change lakefiles to use real Aeneas dependency
2. CI installs elan + runs `lake build` (Aeneas library is cached)
3. Build time: ~5 min first run, ~30s cached

**Effort:** Small (lakefile changes + CI cache configuration).

## Gap 5: Some Rust Code Can't Be Translated by Aeneas

**Current:** Tutorials 04, 07, 08, 09, 10, 11 have Aeneas translation errors.

| Tutorial | Error | Root Cause |
|----------|-------|------------|
| 04 | `type_var_id` | Generic `run_machine<M: StateMachine>` |
| 07 | `Unreachable` | `vec![]` macro in `AppModel::new`, modular focus cycling |
| 08 | `shallow-init-box` | `Conversation::new` with `vec![msg]` |
| 09 | `break to outer loop` | Nested loop in `validate_tool_call` |
| 10 | Missing `filter`/`collect` | Iterator combinators |
| 11 | `nested borrows` | Borrow in `render_conversation` |

**Needed:** Refactor Rust code to avoid unsupported patterns.

**How to close:**
- Replace `vec![x]` with `Vec::new()` + `push(x)` (Aeneas doesn't support `vec![]` macro)
- Replace `break` in nested loops with flag variable
- Replace iterator chains with explicit `while` loops
- Simplify generic trait usage
- Avoid nested borrows

**Effort:** Medium. Each fix is small but needs retesting with Aeneas.

## Gap 6: Tutorial 07 Completely Fails Translation

**Current:** Tutorial 07 (TUI Core) fails Aeneas translation entirely.

**Needed:** Rewrite to avoid `Unreachable` patterns.

**How to close:** Rewrite `AppModel::new` to build widget list incrementally instead of using `vec![]`. Rewrite `focus_next`/`focus_prev` to avoid patterns that cause `Unreachable`.

**Effort:** Medium.

## Priority Order

1. **Gap 2 + Gap 3** — Switch to real Aeneas library and real generated code (structural change)
2. **Gap 1** — Write real proofs for tutorial 01 first (easiest, proof of concept)
3. **Gap 5** — Fix Rust code for tutorials with translation errors
4. **Gap 4** — Update CI to build against real Aeneas
5. **Gap 1 continued** — Write real proofs for remaining tutorials
6. **Gap 6** — Fix tutorial 07
