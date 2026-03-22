# Tutorial 02: RPN Calculator

## Goal

Implement a stack-based Reverse Polish Notation calculator in Rust; prove stack invariants and evaluator correctness using inductive reasoning on algebraic data types.

## File Structure

```
02-rpn-calculator/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Token, Stack, eval_step, evaluate, tokenize_word, parse_number
│       └── main.rs             # CLI wrapper (unverified shell)
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── RpnCalc/Types.lean      # generated
    ├── RpnCalc/Funs.lean       # generated
    ├── RpnCalc/Spec.lean       # hand-written: WellFormedRPN, rpn_semantics
    ├── RpnCalc/TokenizerProofs.lean
    ├── RpnCalc/EvaluatorProofs.lean
    └── RpnCalc/StackInvariant.lean
```

## Rust Code Outline (~180 lines)

### Key Types

| Type | Definition | Description |
|------|-----------|-------------|
| `Token` | `enum { Num(i64), Plus, Minus, Mul, Div }` | RPN token — either a number or an operator |
| `EvalError` | `enum { StackUnderflow, DivisionByZero, TooManyValues }` | Errors that can occur during evaluation |
| `Stack` | `enum { Empty, Push(i64, Box<Stack>) }` | Functional linked list for the operand stack |

### Key Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `stack_depth` | `(&Stack) -> u32` | Returns the number of elements on the stack |
| `eval_step` | `(Token, Stack) -> Result<Stack, EvalError>` | Process one token: push number or apply binary op |
| `evaluate` | `(&[Token]) -> Result<i64, EvalError>` | Fold `eval_step` over token slice; check exactly one value remains |
| `tokenize_word` | `(&str) -> Result<Token, ()>` | Convert a single whitespace-delimited word to a Token |
| `parse_number` | `(&str) -> Result<i64, ()>` | Parse a decimal string to i64 with bounds checking |

Design note: `Stack` is a functional linked list (not `Vec`) so Aeneas produces clean inductive types in Lean.

## Generated Lean (approximate)

- **Types.lean**: Inductive definitions for `Token`, `EvalError`, and `Stack`. These map directly from the Rust enums.

```lean
-- approximate generated types
inductive Token where
  | Num : I64 → Token
  | Plus : Token
  | Minus : Token
  | Mul : Token
  | Div : Token

inductive Stack where
  | Empty : Stack
  | Push : I64 → Stack → Stack
```

- **Funs.lean**: Monadic translations of each function. `eval_step` will pattern-match on both the token and the stack.

## Theorems to Prove (~250 lines)

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `tokenize_word_digit_spec` | A string of ASCII digits tokenizes to `Token.Num` with the correct value | Unfold `tokenize_word` and `parse_number`; use properties of digit-to-int conversion |
| `eval_step_num_depth` | `eval_step (Num n) s = ok s' → stack_depth s' = stack_depth s + 1` | Pattern match on `Token.Num` case; the result is `Push n s`, depth increases by 1 |
| `eval_step_binop_depth` | `eval_step op s = ok s' → is_binop op → stack_depth s' = stack_depth s - 1` | Case analysis on operator; requires stack has ≥ 2 elements; pops 2, pushes 1 |
| `wf_rpn_evaluate_succeeds` | A well-formed RPN expression (defined inductively) always evaluates without error | Induction on `WellFormedRPN`; the `Num` case is trivial; the `BinOp` case uses the IH to show the stack has enough elements |
| `div_by_zero_caught` | `eval_step (Div) (Push 0 (Push x s)) = EvalError.DivisionByZero` | Direct computation; unfold `eval_step`, the zero check triggers |

### Spec Definitions (hand-written)

- `WellFormedRPN : List Token → Prop` — inductively defines valid RPN expressions (a number is WF; if `e1` and `e2` are WF and `op` is a binop, then `e1 ++ e2 ++ [op]` is WF).
- `rpn_semantics : List Token → Int` — the "intended" mathematical meaning of a WF RPN expression.

## New Lean Concepts Introduced

- **Inductive types**: How Rust enums become Lean `inductive` definitions; constructors, eliminators, and structural recursion.
- **Pattern matching in proofs**: Using `cases` and `match` inside tactic proofs to handle enum variants.
- **`cases` tactic**: Structural case analysis — splits a goal based on which constructor was used.
- **`induction` tactic**: Structural induction on inductive types; generates induction hypotheses automatically.
- **List reasoning**: Working with `List` (append, length, membership) as the mathematical model for sequences.

## Cross-References

- **Prerequisites**: Tutorial 01 — uses `step`, `scalar_tac`, and the monadic `Result` proof pattern.
- **Forward**: Tutorial 03 (Infix Calculator) imports `evaluate` and `WellFormedRPN` for the equivalence proof.

## Estimated Lines of Code

| Component | Lines |
|-----------|-------|
| Rust source | ~180 |
| Generated Lean (Types + Funs) | ~120 |
| Hand-written specs | ~60 |
| Hand-written proofs | ~250 |
| README | ~300 |
| **Total** | **~910** |
