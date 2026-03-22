# Tutorial 01: Setup & Hello Proof

## Goal

Install Lean 4, Aeneas, and Charon; translate 4 simple Rust functions through the toolchain; write first proofs demonstrating the Curry-Howard correspondence and monadic translation.

## File Structure

```
01-setup-hello-proof/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/lib.rs              # checked_add, safe_divide, safe_abs, clamp
├── lean/
│   ├── lakefile.lean
│   ├── lean-toolchain
│   ├── HelloProof/Types.lean   # generated
│   ├── HelloProof/Funs.lean    # generated
│   └── HelloProof/Proofs.lean  # hand-written
└── scripts/translate.sh
```

## Rust Code Outline (~50 lines)

| Function | Signature | Description |
|----------|-----------|-------------|
| `checked_add` | `(u32, u32) -> Option<u32>` | Returns `None` on overflow instead of panicking |
| `safe_divide` | `(i64, i64) -> Result<i64, ()>` | Returns `Err(())` on division by zero |
| `safe_abs` | `(i64) -> Result<i64, ()>` | Returns `Err(())` for `i64::MIN` (no positive representation) |
| `clamp` | `(i32, i32, i32) -> i32` | Clamps value to `[lo, hi]` range; assumes `lo <= hi` |

All functions are total (no panics on valid input) and use checked arithmetic or explicit error handling.

## Generated Lean (approximate)

Aeneas will produce:

- **Types.lean**: Mostly empty (no custom structs), but will import `Aeneas.Primitives` which provides `U32`, `I64`, `I32`, `Result`, `Scalar` types.
- **Funs.lean**: Monadic translations of each function. For example:

```lean
-- approximate generated code
def checked_add (x : U32) (y : U32) : Result (Option U32) :=
  if x.val + y.val ≤ U32.max then
    do let z ← U32.add x y
       Result.ok (some z)
  else
    Result.ok none

def safe_divide (x : I64) (y : I64) : Result (Result I64 Unit) :=
  if y.val = 0 then Result.ok (Err ())
  else do let z ← I64.div x y
          Result.ok (Ok z)
```

Key observation: Aeneas wraps everything in `Result` (the Aeneas error monad), and arithmetic operations like `U32.add` return `Result U32` because they can fail.

## Theorems to Prove (~80 lines)

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `checked_add_no_panic` | `∀ x y : U32, ∃ r, checked_add x y = Result.ok r` | Case split on overflow condition; `scalar_tac` handles bounds |
| `checked_add_spec` | If result is `some z`, then `z.val = x.val + y.val` | Unfold definition, `simp`, extract from monadic bind |
| `safe_divide_spec` | If `y ≠ 0`, result is `Ok (x / y)` | Case split on `y.val = 0`; use `I64.div` spec |
| `clamp_spec` | Result is always in `[lo, hi]` when `lo ≤ hi` | Unfold, case split on comparisons; `omega` closes arithmetic goals |

## New Lean Concepts Introduced

- **Curry-Howard correspondence**: Types are propositions, terms are proofs; `P → Q` is both a function type and an implication.
- **Types-as-propositions**: A theorem `∀ x, P x` is a dependent function type; proving it means constructing a term of that type.
- **Monadic translation**: How Aeneas wraps Rust's control flow (early return, panic, overflow) into a `Result` monad.
- **`step`**: Tactic that unfolds one layer of monadic computation (resolves `do`-notation binds).
- **`scalar_tac`**: Aeneas-provided tactic for discharging scalar bounds obligations (e.g., `x.val + y.val ≤ U32.max`).
- **`Result` monad**: Aeneas's error monad — `Result.ok` for success, `Result.fail` for panic/overflow.
- **`↑` coercion**: Lifts `U32`/`I64` etc. to `Int` (or `Nat`) for reasoning about mathematical values.

## Cross-References

- **Next**: Tutorial 02 (RPN Calculator) builds on `step`, `scalar_tac`, and monadic proof patterns introduced here.

## Estimated Lines of Code

| Component | Lines |
|-----------|-------|
| Rust source | ~50 |
| Generated Lean (Types + Funs) | ~60 |
| Hand-written proofs | ~80 |
| Scripts / config | ~30 |
| README | ~200 |
| **Total** | **~420** |
