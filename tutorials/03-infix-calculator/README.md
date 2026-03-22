[← Previous: Tutorial 02](../02-rpn-calculator/README.md) | [Index](../README.md) | [Next: Tutorial 04 →](../04-state-machines/README.md)

# Tutorial 03: Infix Calculator

In Tutorial 02 we built an RPN calculator with a flat token stream and a stack machine evaluator. That was a natural fit for formal verification: the stack discipline gave us clean inductive proofs about well-formedness and evaluation.

But nobody writes `3 4 + 2 *` in real life. Real programs parse infix expressions like `(3 + 4) * 2`, which means we need a **recursive descent parser** that produces an **abstract syntax tree** (AST). This tutorial builds that parser, an evaluator for the AST, and then proves something remarkable: the infix calculator and the RPN calculator from Tutorial 02 compute the same function.

## What You Will Learn

- How to write a recursive descent parser in Aeneas-friendly Rust
- How `Box<Expr>` in Rust becomes plain `Expr` in Lean (Aeneas erases heap indirection)
- Structural induction on tree-shaped data (not just lists)
- Parser termination proofs via position-advancement arguments
- Evaluator correctness via structural induction
- Cross-tutorial equivalence proofs bridging two different computation strategies

## Prerequisites

- Tutorial 01 (Lean basics, `step`, `simp`, `omega`)
- Tutorial 02 (inductive types, `cases`, `induction`, RPN evaluator)

---

## 1. The Grammar

Our infix calculator handles the four arithmetic operators with standard precedence. The grammar in BNF notation:

```
expr   ::= term (('+' | '-') term)*
term   ::= factor (('*' | '/') factor)*
factor ::= NUMBER | '(' expr ')'
```

This is a classic precedence-climbing grammar:
- `expr` handles the lowest-precedence operators (`+`, `-`)
- `term` handles higher-precedence operators (`*`, `/`)
- `factor` handles atoms: literal numbers and parenthesized sub-expressions

The parenthesized case in `factor` creates mutual recursion: `factor` calls `expr`, which calls `term`, which calls `factor`. This is what makes recursive descent parsing interesting for formal verification.

## 2. Rust: The Lexer

The lexer converts a byte slice into a list of tokens. We use `&[u8]` instead of `&str` because Aeneas handles byte slices cleanly.

```rust
pub fn lex(input: &[u8]) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut i: usize = 0;

    while i < input.len() {
        let ch = input[i];
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
            i += 1;  // skip whitespace
        } else if ch == b'+' {
            tokens.push(Token::Operator(Op::Add));
            i += 1;
        } else if ch >= b'0' && ch <= b'9' {
            // Parse multi-digit number
            let mut value: i64 = 0;
            while i < input.len() && input[i] >= b'0' && input[i] <= b'9' {
                value = value * 10 + (input[i] - b'0') as i64;
                i += 1;
            }
            tokens.push(Token::Num(value));
        }
        // ... other cases
    }
    Ok(tokens)
}
```

Key design choices for Aeneas compatibility:
- **Explicit index variable** (`i: usize`) instead of iterators
- **`while` loop** with manual advancement instead of `for`
- **Byte comparison** (`b'+'`) instead of character matching
- **No closures or iterator adapters** — just straightforward imperative code

## 3. Rust: The AST Type

The abstract syntax tree is a recursive enum:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    BinOp(Op, Box<Expr>, Box<Expr>),
}
```

`Box<Expr>` is required in Rust because enum variants must have known size at compile time. A recursive enum without `Box` would have infinite size.

**The key insight for Aeneas**: `Box<Expr>` in Rust becomes just `Expr` in Lean. Aeneas erases the heap indirection because Lean's inductive types are naturally recursive:

```lean
-- Rust's Box<Expr> disappears entirely!
inductive Expr where
  | Num : I64 → Expr
  | BinOp : Op → Expr → Expr → Expr
```

This is one of the beautiful aspects of the Aeneas translation: Rust's memory management concerns (Box, ownership, borrowing) vanish in the logical model, leaving clean mathematical structure.

## 4. Rust: The Parser

The parser uses **index-passing style**: each parsing function takes a position `pos` and returns `(result, new_position)`. This avoids iterators and mutable references, which Aeneas handles less cleanly.

```rust
pub fn parse_expr(tokens: &[Token], pos: usize)
    -> Result<(Expr, usize), ParseError>
{
    let (mut left, mut pos) = parse_term(tokens, pos)?;

    while pos < tokens.len() {
        match &tokens[pos] {
            Token::Operator(Op::Add) => {
                let (right, next) = parse_term(tokens, pos + 1)?;
                left = Expr::BinOp(Op::Add, Box::new(left), Box::new(right));
                pos = next;
            }
            Token::Operator(Op::Sub) => {
                let (right, next) = parse_term(tokens, pos + 1)?;
                left = Expr::BinOp(Op::Sub, Box::new(left), Box::new(right));
                pos = next;
            }
            _ => break,
        }
    }
    Ok((left, pos))
}
```

Notice the pattern:
1. Parse the first sub-expression (higher precedence)
2. Loop: if we see an operator at our precedence level, parse another sub-expression and combine
3. Otherwise, return what we have

Each level of the grammar follows this exact pattern. `parse_term` is identical but matches `*` and `/` and calls `parse_factor`. `parse_factor` handles the base cases:

```rust
pub fn parse_factor(tokens: &[Token], pos: usize)
    -> Result<(Expr, usize), ParseError>
{
    if pos >= tokens.len() {
        return Err(ParseError::UnexpectedEnd);
    }
    match &tokens[pos] {
        Token::Num(n) => Ok((Expr::Num(*n), pos + 1)),
        Token::LParen => {
            let (expr, next) = parse_expr(tokens, pos + 1)?;
            // Check for closing paren...
            Ok((expr, next + 1))
        }
        _ => Err(ParseError::UnexpectedToken),
    }
}
```

## 5. Rust: The Evaluator

The evaluator is structural recursion on the AST — the simplest part of the whole system:

```rust
pub fn eval(expr: &Expr) -> Result<i64, EvalError> {
    match expr {
        Expr::Num(n) => Ok(*n),
        Expr::BinOp(op, left, right) => {
            let l = eval(left)?;
            let r = eval(right)?;
            match op {
                Op::Add => l.checked_add(r).ok_or(EvalError::Overflow),
                Op::Sub => l.checked_sub(r).ok_or(EvalError::Overflow),
                Op::Mul => l.checked_mul(r).ok_or(EvalError::Overflow),
                Op::Div => {
                    if r == 0 {
                        Err(EvalError::DivisionByZero)
                    } else {
                        l.checked_div(r).ok_or(EvalError::Overflow)
                    }
                }
            }
        }
    }
}
```

We use `checked_*` operations to detect overflow instead of panicking. This maps to Aeneas's bounded arithmetic: in the Lean translation, `l + r` on `I64` returns `Result I64`, failing on overflow.

## 6. Translation to Lean

The Aeneas translation of this code introduces several important patterns.

### Parser Loops Become `partial_fixpoint`

The `while` loops in `parse_expr` and `parse_term` become `partial_fixpoint` recursive functions decorated with `@[rust_loop]`:

```lean
@[rust_loop]
partial_fixpoint def parse_expr_loop
    (tokens : Vec Token) (left : Expr) (pos : Usize)
    : Result (core.result.Result (Expr × Usize) ParseError) := do
  if h : pos < tokens.len then do
    let tok ← tokens.index ⟨pos, h⟩
    match tok with
    | Token.Operator Op.Add => do
        let right_result ← parse_term tokens (← pos + (1 : Usize))
        match right_result with
        | .ok (right, pos2) =>
          parse_expr_loop tokens (Expr.BinOp Op.Add left right) pos2
        | .err e => ok (.err e)
    | _ => ok (.ok (left, pos))
  else
    ok (.ok (left, pos))
```

The loop state (accumulated `left` expression and current `pos`) becomes explicit parameters to the recursive function.

### Box Disappears

The most striking translation is `Expr` itself. Rust's:
```rust
BinOp(Op, Box<Expr>, Box<Expr>)
```
becomes Lean's:
```lean
| BinOp : Op → Expr → Expr → Expr
```

No `Box`, no pointers, no heap. Just a recursive inductive type. This is possible because Lean's type theory natively supports recursive types — the very thing that Rust's type system needs `Box` to encode.

### Evaluator Is Clean Structural Recursion

The evaluator translates almost verbatim, because it is already structural recursion on `Expr`:

```lean
def eval (expr : Expr) : Result (core.result.Result I64 EvalError) := do
  match expr with
  | Expr.Num n => ok (.ok n)
  | Expr.BinOp op left right => do
    let lr ← eval left
    match lr with
    | .ok l => do
      let rr ← eval right
      match rr with
      | .ok r =>
        match op with
        | Op.Add => do
            let result ← l + r  -- bounded arithmetic, can fail
            ok (.ok result)
        -- ... other ops
```

## 7. Concept: Recursive Data Types

In Tutorial 02, we worked with `Stack` — a linked list. Lists are the simplest recursive data type: each node has one recursive child.

`Expr` is a **tree**: each `BinOp` node has *two* recursive children (left and right). This is our first encounter with tree-shaped data in the tutorial series.

```
    BinOp(Mul)
    /         \
BinOp(Add)   Num(4)
  /    \
Num(2) Num(3)
```

In Lean, this tree is an inductive type:

```lean
inductive Expr where
  | Num : I64 → Expr          -- leaf
  | BinOp : Op → Expr → Expr → Expr  -- internal node with two children
```

The `Num` constructor is a **leaf** (no recursive children). The `BinOp` constructor is an **internal node** with two recursive positions.

## 8. Concept: Structural Induction on Trees

To prove properties about all expressions, we use **structural induction** on `Expr`. This gives us:

- **Base case**: Prove the property for `Expr.Num n` (for all `n`)
- **Inductive case**: Prove the property for `Expr.BinOp op left right`, assuming the property holds for `left` and `right` (the **induction hypotheses**)

This is strictly more powerful than induction on lists (which gives one IH for the tail). With trees, we get an IH for *each* subtree.

In Lean:

```lean
theorem some_property (e : Expr) : P e := by
  induction e with
  | Num n =>
    -- Base case: prove P (Num n)
    sorry
  | BinOp op left right ih_left ih_right =>
    -- Inductive case: prove P (BinOp op left right)
    -- Available: ih_left : P left
    --            ih_right : P right
    sorry
```

## 9. Mathematical Specification

Before we can prove anything, we need a **specification** — the mathematical truth that our code should implement. For the evaluator, this is `expr_semantics`:

```lean
def expr_semantics : Expr → Option Int
  | Expr.Num n => some ↑n
  | Expr.BinOp op left right =>
    match expr_semantics left, expr_semantics right with
    | some l, some r =>
      match op with
      | Op.Add => some (l + r)
      | Op.Sub => some (l - r)
      | Op.Mul => some (l * r)
      | Op.Div => if r = 0 then none else some (l / r)
    | _, _ => none
```

This is a **total function** on mathematical integers (`Int`), not bounded `I64`. It cannot overflow — the only failure mode is division by zero. This clean mathematical definition is our ground truth.

The correctness theorem says: when `eval` succeeds, it agrees with `expr_semantics`. The Rust code is a faithful implementation of the math, within its domain of success.

## 10. Proof: Parser Termination

The parser has mutual recursion: `parse_factor` calls `parse_expr` (for parenthesized expressions), which calls `parse_term`, which calls `parse_factor`. How do we know this terminates?

The key insight: **every successful parse advances the position**.

```lean
theorem parse_factor_advances
    (tokens : Vec Token) (pos : Usize) (expr : Expr) (pos' : Usize)
    (h_ok : parse_factor tokens pos = ok (.ok (expr, pos')))
    : (↑pos' : Int) > ↑pos
```

This is proved by case analysis on what `parse_factor` matched:
- `Token.Num n`: returns `pos + 1 > pos`
- `Token.LParen`: calls `parse_expr` (which advances), then consumes `)`, so `pos' >= pos + 2`

Since the position is bounded by `tokens.len` (a natural number), and each recursive call strictly increases it, the recursion is **well-founded**: it must terminate after at most `tokens.len` steps.

```lean
theorem parse_termination_measure
    (tokens : Vec Token) (pos : Usize) (expr : Expr) (pos' : Usize)
    (h_ok : parse_factor tokens pos = ok (.ok (expr, pos')))
    (h_bound : (↑pos : Int) < ↑tokens.len)
    : (↑tokens.len : Int) - ↑pos' < ↑tokens.len - ↑pos
```

The measure `tokens.len - pos` is a natural number that strictly decreases on each call. This is a standard **well-founded induction** argument.

## 11. Proof: Evaluator Correctness

The evaluator correctness proof is our first substantial use of structural induction on trees.

### Base Case: Num

```lean
theorem eval_num_correct (n : I64) :
    eval (Expr.Num n) = ok (.ok n) := by
  simp [eval]
```

Both `eval` and `expr_semantics` just return the number. Trivial.

### Inductive Case: BinOp

This is where the induction hypotheses earn their keep:

```lean
-- Given: ih_left : eval left agrees with expr_semantics left
--        ih_right : eval right agrees with expr_semantics right
-- Prove: eval (BinOp op left right) agrees with expr_semantics (BinOp op left right)
```

The proof proceeds:
1. `eval` evaluates `left` (succeeds by hypothesis), getting `vl`
2. `eval` evaluates `right` (succeeds by hypothesis), getting `vr`
3. `eval` applies `op` to `vl` and `vr`
4. By IH, `vl` agrees with `expr_semantics left` and `vr` agrees with `expr_semantics right`
5. Since the bounded arithmetic succeeded (no overflow), the result agrees with the mathematical operation

### The Main Theorem

```lean
theorem eval_correct (e : Expr) (v : I64)
    (h : eval e = ok (.ok v))
    : expr_semantics e = some ↑v
```

This says: whenever the Rust evaluator succeeds, its result matches the mathematical specification. Note the asymmetry — `eval` can *fail* when `expr_semantics` succeeds (due to overflow), but when `eval` succeeds, they always agree.

## 12. Concept: `calc` Blocks

Lean's `calc` blocks let you write step-by-step equational reasoning. They are particularly useful for the evaluator correctness proof:

```lean
calc expr_semantics (BinOp Op.Add left right)
    = some (↑vl + ↑vr)  := by simp [expr_semantics, ih_left, ih_right]
  _ = some ↑result       := by omega  -- because eval succeeded without overflow
```

Each line transforms the expression, with a justification after `:= by`. The `_` on continuation lines means "the expression from the previous line." This creates a readable chain of equalities.

## 13. Concept: `simp` and `omega`

Two of the most powerful automation tactics in Lean:

**`simp`** (simplification): Rewrites the goal using a database of lemmas tagged with `@[simp]`. You can also provide specific lemmas:

```lean
simp [eval, expr_semantics]  -- unfold these definitions and simplify
simp only [List.append_assoc]  -- use only this specific lemma
```

`simp` is powerful but can be unpredictable. Use `simp only [...]` when you want precise control.

**`omega`** (linear arithmetic): Decides goals involving `<`, `<=`, `+`, `-` on `Nat` and `Int`. It handles all the position-arithmetic in our parser proofs:

```lean
-- omega can prove things like:
-- pos + 1 > pos
-- tokens.len - (pos + 1) < tokens.len - pos
-- if a > b and b >= 0 then a >= 1
omega
```

`omega` is a decision procedure — if the goal is a true statement of linear arithmetic, `omega` will close it. If it cannot, the goal is either false or involves non-linear arithmetic.

## 14. The Equivalence Proof

The crown jewel of this tutorial is the equivalence theorem between infix and RPN evaluation.

### The Bridge Function

We define a conversion from `Expr` to RPN token list — a post-order tree traversal:

```lean
def expr_to_rpn : Expr → List rpn.Token
  | Expr.Num n => [rpn.Token.Num n]
  | Expr.BinOp op left right =>
    expr_to_rpn left ++ expr_to_rpn right ++ [op_to_rpn_token op]
```

For example, `(2 + 3) * 4` becomes `[2, 3, +, 4, *]` — valid RPN.

### Well-Formedness

First we prove that `expr_to_rpn` always produces well-formed RPN (using Tutorial 02's definition):

```lean
theorem expr_to_rpn_well_formed (e : Expr) :
    rpn.WellFormedRPN (expr_to_rpn e)
```

By structural induction:
- `Num n` produces `[Num n]`, which is well-formed by the `num` constructor
- `BinOp op l r` produces `expr_to_rpn l ++ expr_to_rpn r ++ [op]`, which is well-formed by the `binop` constructor (using IH on `l` and `r`)

### The Key Theorem

```lean
theorem infix_rpn_equivalence (e : Expr) (v : I64)
    (h_eval : eval e = ok (.ok v))
    : ∃ v', rpn.evaluate (expr_to_rpn e) = ok (.ok v') ∧ (↑v' : Int) = ↑v
```

This says: if the infix evaluator produces `v`, then the RPN evaluator on the converted token list produces the same value.

The proof combines results from both tutorials:
1. `expr_to_rpn_well_formed` ensures the RPN is valid
2. `wf_rpn_evaluate_succeeds` (from Tutorial 02) ensures the RPN evaluator succeeds
3. `eval_correct` ensures the infix evaluator matches `expr_semantics`
4. Both evaluators implement the same mathematical semantics, so they agree

This is a genuine cross-tutorial proof: it depends on definitions and theorems from Tutorial 02, demonstrating how formal verification composes across module boundaries.

## 15. Running the Code

Build and test the Rust code:

```bash
cd rust
cargo test
```

Run the calculator interactively:

```bash
cargo run
# Type expressions like: (2 + 3) * 4 - 10 / 2
```

## 16. Exercises

1. **Add unary minus**: Extend the grammar with a unary minus operator (e.g., `-(3 + 4)`). Add a new `Expr` variant `Neg(Box<Expr>)`, update the parser, and extend `eval` and `expr_semantics`. Prove that `eval_correct` still holds.

2. **Add modulo**: Add a `%` (modulo) operator at the same precedence level as `/`. Update the lexer, parser, evaluator, and proofs. What new error case do you need to handle?

3. **Prove eval_no_panic**: Complete the proof that `eval` always returns `ok` (the outer Aeneas Result is always successful, even when the inner result is an error).

4. **Prove eval_div_zero_sound**: Show that if `eval` returns `DivisionByZero`, then the expression actually contains a division where the right operand evaluates to zero.

5. **Strengthen the equivalence**: Remove the axioms in `Equivalence.lean` by connecting Tutorial 02's actual Lean project as a Lake dependency, and complete the `infix_rpn_equivalence` proof without `sorry`.

## 17. What's Next

In Tutorial 04 (State Machines), we shift from data-structure proofs to behavioral reasoning. Instead of proving properties about trees and lists, we will prove properties about *systems that evolve over time*: state machines with transitions, invariants, and reachability. The `StateMachine` pattern introduced there becomes a building block for the rest of the tutorial series.

Key concepts coming in Tutorial 04:
- State machines as a design pattern
- Invariant preservation across transitions
- Reachability proofs
- The connection between Rust's type-state pattern and Lean's inductive propositions

---

[← Previous: Tutorial 02](../02-rpn-calculator/README.md) | [Index](../README.md) | [Next: Tutorial 04 →](../04-state-machines/README.md)
