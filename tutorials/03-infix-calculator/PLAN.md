# Tutorial 03: Infix Calculator

## Goal

Implement a recursive descent parser and AST evaluator for infix arithmetic expressions; prove parser termination, evaluator correctness, and equivalence with the RPN calculator from Tutorial 02.

## File Structure

```
03-infix-calculator/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Op, Token, Expr, lex, parse_expr/term/factor, eval
│       └── main.rs             # CLI wrapper
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── InfixCalc/Types.lean    # generated
    ├── InfixCalc/Funs.lean     # generated
    ├── InfixCalc/Spec.lean     # hand-written: expr_semantics
    ├── InfixCalc/ParserProofs.lean
    ├── InfixCalc/EvaluatorProofs.lean
    └── InfixCalc/Equivalence.lean    # cross-tutorial proof (imports Tutorial 02)
```

## Rust Code Outline (~225 lines)

### Key Types

| Type | Definition | Description |
|------|-----------|-------------|
| `Op` | `enum { Add, Sub, Mul, Div }` | Binary operators with standard precedence |
| `Token` | `enum { Num(i64), Op(Op), LParen, RParen }` | Lexer tokens including parentheses |
| `Expr` | `enum { Num(i64), BinOp(Op, Box<Expr>, Box<Expr>) }` | Abstract syntax tree for expressions |
| `ParseError` | `enum { UnexpectedToken, UnexpectedEnd, ... }` | Parser error variants |

### Key Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `lex` | `(&[u8]) -> Result<Vec<Token>, ParseError>` | Tokenize a byte slice into a token list |
| `parse_expr` | `(&[Token], usize) -> Result<(Expr, usize), ParseError>` | Parse additive expressions (`+`, `-`); returns (AST, next index) |
| `parse_term` | `(&[Token], usize) -> Result<(Expr, usize), ParseError>` | Parse multiplicative expressions (`*`, `/`) |
| `parse_factor` | `(&[Token], usize) -> Result<(Expr, usize), ParseError>` | Parse atoms: numbers and parenthesized sub-expressions |
| `eval` | `(&Expr) -> Result<i64, ()>` | Evaluate an AST; fails only on division by zero |
| `expr_to_rpn` | `(&Expr) -> Vec<Token>` | Convert AST to RPN token list (for equivalence proof) |

Design note: The parser uses **index-passing style** (`pos: usize` parameter and return) rather than iterators or mutable references, because Aeneas handles index arithmetic cleanly but cannot translate iterator adapters.

## Generated Lean (approximate)

- **Types.lean**: Inductive types for `Op`, `Token`, `Expr`, `ParseError`.

```lean
-- approximate generated types
inductive Expr where
  | Num : I64 → Expr
  | BinOp : Op → Expr → Expr → Expr
```

Note: `Box<Expr>` in Rust becomes just `Expr` in Lean (Aeneas erases the indirection).

- **Funs.lean**: The parser functions will be mutually recursive in the generated output. Aeneas handles this via `mutual ... end` blocks or fuel-based translation.

## Theorems to Prove (~350 lines)

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `parse_factor_advances` | `parse_factor tokens pos = ok (e, pos') → pos' > pos` | Case analysis on what `parse_factor` matched (number or `(`); in the parenthesized case, use IH from `parse_expr` |
| `parse_terminates` | The parser always terminates on any finite input | Follows from `parse_factor_advances`: each recursive call strictly advances the position; position is bounded by `tokens.length` |
| `eval_correct` | `eval expr = ok v → v.val = expr_semantics expr` | Structural induction on `Expr`; the `Num` case is immediate; `BinOp` uses IH on both children and correctness of the arithmetic operation |
| `infix_rpn_equivalence` | `eval expr = ok v → rpn.evaluate (expr_to_rpn expr) = ok v` | **Cross-tutorial proof.** Induction on `Expr`; uses `wf_rpn_evaluate_succeeds` from Tutorial 02 to show the generated RPN is well-formed, then shows evaluation agrees |

### Spec Definitions (hand-written)

- `expr_semantics : Expr → Int` — the intended mathematical value of an expression (pure function, no errors).
- `expr_to_rpn_wf : ∀ e, WellFormedRPN (expr_to_rpn e)` — lemma showing the conversion produces valid RPN.

## New Lean Concepts Introduced

- **Recursive data types**: Working with tree-structured `Expr` — how `Box<Expr>` in Rust becomes direct recursion in Lean.
- **Structural induction on trees**: Induction on `Expr` gives two cases (leaf/node) with IH for both subtrees.
- **`calc` blocks**: Step-by-step equational reasoning chains, useful for showing `eval` agrees with `expr_semantics`.
- **`simp`**: The simplification tactic — how to use it effectively with `[simp]`-tagged lemmas and `simp only [...]`.
- **`omega`**: Linear arithmetic decision procedure — closes goals involving `<`, `≤`, `+`, `-` on `Nat`/`Int`.

## Cross-References

- **Prerequisites**: Tutorial 02 — imports `RpnCalc.Types` (for RPN `Token`), `RpnCalc.Funs` (for `evaluate`), and `RpnCalc.Spec` (for `WellFormedRPN`) to state and prove `infix_rpn_equivalence`.
- **Forward**: Tutorial 04 (State Machines) — shifts from data structure proofs to behavioral/state-based reasoning.

## Estimated Lines of Code

| Component | Lines |
|-----------|-------|
| Rust source | ~225 |
| Generated Lean (Types + Funs) | ~160 |
| Hand-written specs | ~50 |
| Hand-written proofs | ~350 |
| README | ~350 |
| **Total** | **~1135** |
