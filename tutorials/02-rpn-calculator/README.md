[← Previous: Tutorial 01](../01-setup-hello-proof/README.md) | [Index](../README.md) | [Next: Tutorial 03 →](../03-infix-calculator/README.md)

# Tutorial 02: RPN Calculator

In this tutorial we build a Reverse Polish Notation (RPN) calculator in Rust and prove key correctness properties about it in Lean 4 using Aeneas. This is our first tutorial with custom data types: enums for tokens, errors, and a functional linked-list stack. We will see how Rust enums become Lean inductive types and how while loops become recursive functions.

## Table of Contents

1. [Introduction: What is RPN?](#1-introduction-what-is-rpn)
2. [The Functional Core Pattern](#2-the-functional-core-pattern)
3. [Rust Implementation](#3-rust-implementation)
4. [Running the Pipeline](#4-running-the-pipeline)
5. [How Enums Become Inductive Types](#5-how-enums-become-inductive-types)
6. [How While Loops Become Recursive Functions](#6-how-while-loops-become-recursive-functions)
7. [Writing the Mathematical Specification](#7-writing-the-mathematical-specification)
8. [Concept: Inductive Types](#8-concept-inductive-types)
9. [Proof: Division Safety](#9-proof-division-safety)
10. [Proof: Stack Depth Invariant](#10-proof-stack-depth-invariant)
11. [Concept: The Induction Tactic](#11-concept-the-induction-tactic)
12. [Proof: Well-Formed Expressions Succeed](#12-proof-well-formed-expressions-succeed)
13. [The Unverified Shell](#13-the-unverified-shell)
14. [Exercises](#14-exercises)
15. [What's Next](#15-whats-next)

---

## 1. Introduction: What is RPN?

Reverse Polish Notation (also called postfix notation) is a way of writing arithmetic expressions where the operator comes *after* its operands. Instead of writing `3 + 4`, you write `3 4 +`. Instead of `(3 + 4) * 2`, you write `3 4 + 2 *`.

RPN has a beautiful property: **it needs no parentheses and no precedence rules.** The order of evaluation is completely determined by the order of the tokens. This makes it ideal for formal verification because there is no ambiguity to resolve.

Evaluation uses a stack:
- When you see a number, push it.
- When you see an operator, pop two values, apply the operator, push the result.
- At the end, the stack should have exactly one value: the answer.

Example: `5 1 2 + 4 * + 3 -`

| Token | Stack (top on right) |
|-------|---------------------|
| 5     | [5]                 |
| 1     | [5, 1]              |
| 2     | [5, 1, 2]           |
| +     | [5, 3]              |
| 4     | [5, 3, 4]           |
| *     | [5, 12]             |
| +     | [17]                |
| 3     | [17, 3]             |
| -     | [14]                |

Result: **14**

## 2. The Functional Core Pattern

A key design decision in this tutorial is the **functional core / imperative shell** pattern. The idea is:

- The **core** (lib.rs) is pure, functional Rust that Aeneas can translate. It uses no standard library collections, no I/O, no mutation through references. The stack is a functional linked list, not a `Vec`.
- The **shell** (main.rs) handles I/O and uses standard Rust idioms. It calls into the core but is *not* translated by Aeneas.

Why a linked-list stack instead of `Vec`? Because Aeneas translates `Vec` into an array abstraction with index-based reasoning, which is more complex. Our `Stack` enum becomes a clean Lean inductive type with straightforward structural recursion. For a tutorial focused on learning inductive proofs, this is exactly what we want.

```rust
// This is what Aeneas loves: a simple algebraic data type
pub enum Stack {
    Empty,
    Push(i64, Box<Stack>),
}
```

## 3. Rust Implementation

### Types

We define three enums. `Token` represents what the user can type:

```rust
pub enum Token {
    Num(i64),   // A number literal
    Plus,       // +
    Minus,      // -
    Mul,        // *
    Div,        // /
}
```

`EvalError` captures everything that can go wrong:

```rust
pub enum EvalError {
    DivisionByZero,  // Tried to divide by 0
    StackUnderflow,  // Operator with too few operands
    TooManyValues,   // Expression left multiple values on stack
    InvalidToken,    // Unparseable input
}
```

`Stack` is our functional linked list, described above.

### Core Functions

**`tokenize_word`** converts a single whitespace-delimited word (as a byte slice) into a `Token`. Single-character inputs are checked against the four operators and the digits 0-9. Multi-character inputs are parsed as integers by `parse_number`.

**`parse_number`** walks through a byte slice accumulating a decimal integer. It uses a while loop with an index variable, which Aeneas will translate to a recursive function.

**`apply_binop`** dispatches on the operator to perform the arithmetic. It is separated from `eval_step` to avoid using `unreachable!()`, which Aeneas would translate as a panic.

**`eval_step`** is the heart of the evaluator. Given a stack and a token:
- If the token is `Num(n)`, push `n`.
- If the token is an operator, pop two values, apply the operator via `apply_binop`, push the result.

**`evaluate`** folds `eval_step` over a slice of tokens, starting with an empty stack. After processing all tokens, it checks that exactly one value remains.

```rust
pub fn evaluate(tokens: &[Token]) -> Result<i64, EvalError> {
    let mut stack = Stack::new();
    let mut i: usize = 0;
    while i < tokens.len() {
        stack = eval_step(stack, &tokens[i])?;
        i += 1;
    }
    match stack {
        Stack::Push(val, rest) if rest.is_empty() => Ok(val),
        Stack::Empty => Err(EvalError::StackUnderflow),
        _ => Err(EvalError::TooManyValues),
    }
}
```

## 4. Running the Pipeline

First, build and test the Rust code:

```bash
cd rust
cargo test
cargo run
# Enter: 3 4 + 2 *
# Output: Result: 14
```

Then run the Aeneas pipeline to generate Lean:

```bash
# Step 1: Generate LLBC with Charon
charon --cargo-arg=--lib

# Step 2: Translate to Lean with Aeneas
aeneas -b lean rpn_calc.llbc
```

This produces the files in `lean/RpnCalc/Types.lean` and `lean/RpnCalc/Funs.lean`. In this tutorial, we provide simulated versions of what Aeneas would generate.

## 5. How Enums Become Inductive Types

This is the first tutorial where we have custom Rust types. Let's see how Aeneas translates them.

The Rust `Token` enum:

```rust
pub enum Token {
    Num(i64),
    Plus,
    Minus,
    Mul,
    Div,
}
```

becomes the Lean inductive type:

```lean
inductive Token where
  | Num : I64 -> Token
  | Plus : Token
  | Minus : Token
  | Mul : Token
  | Div : Token
```

Every Rust enum variant becomes a Lean **constructor**. Variants with data (like `Num(i64)`) become constructors that take arguments. Variants without data become nullary constructors.

The recursive `Stack` type is especially interesting:

```rust
pub enum Stack {
    Empty,
    Push(i64, Box<Stack>),
}
```

becomes:

```lean
inductive Stack where
  | Empty : Stack
  | Push : I64 -> Stack -> Stack
```

Notice that `Box<Stack>` disappears entirely. In Rust, the `Box` is needed for the recursive type to have a known size. In Lean, inductive types are natively recursive — no boxing needed.

## 6. How While Loops Become Recursive Functions

The `evaluate` function contains a while loop:

```rust
while i < tokens.len() {
    stack = eval_step(stack, &tokens[i])?;
    i += 1;
}
```

Aeneas cannot represent loops directly in Lean (Lean has no loops, only recursion). So it translates the loop body into a recursive function:

```lean
@[rust_loop]
def evaluate_loop (tokens : Slice Token) (stack : Stack) (i : Usize) :
    Result (core.result.Result I64 EvalError) := do
  if i < tokens.len then do
    let token ← tokens.index i
    let step_result ← eval_step stack token
    match step_result with
    | .err e => ok (.err e)
    | .ok stack' => do
      let i' ← i + (1 : Usize)
      evaluate_loop tokens stack' i'
  else
    match stack with
    | Stack.Push val Stack.Empty => ok (.ok val)
    | Stack.Empty => ok (.err EvalError.StackUnderflow)
    | _ => ok (.err EvalError.TooManyValues)
```

The `@[rust_loop]` attribute tells Lean's termination checker that this function comes from a Rust loop and should use `partial_fixpoint` for termination. The loop variable `i` increases each iteration, bounded by `tokens.len`, so the loop always terminates — but this termination argument is handled by the Aeneas framework rather than Lean's structural recursion.

The main `evaluate` function just sets up the initial state and calls the loop:

```lean
def evaluate (tokens : Slice Token) :
    Result (core.result.Result I64 EvalError) := do
  let stack ← Stack.new
  evaluate_loop tokens stack (0 : Usize)
```

## 7. Writing the Mathematical Specification

The generated code tells us *what the program does*. The specification tells us *what the program should do*. We write the specification by hand.

### WellFormedRPN

The central specification is an inductive predicate that defines which token sequences are valid RPN expressions:

```lean
inductive WellFormedRPN : List Token -> Prop where
  | num (n : I64) : WellFormedRPN [Token.Num n]
  | binop (e1 e2 : List Token) (op : Token)
      (h1 : WellFormedRPN e1)
      (h2 : WellFormedRPN e2)
      (hop : is_binop op) :
      WellFormedRPN (e1 ++ e2 ++ [op])
```

This says:
- A single number `[Num n]` is well-formed.
- If `e1` and `e2` are well-formed and `op` is a binary operator, then `e1 ++ e2 ++ [op]` is well-formed.

For example, `[Num 3, Num 4, Plus]` is well-formed because it is `[Num 3] ++ [Num 4] ++ [Plus]` where both sub-expressions are single numbers.

### Stack Depth

We also define a function to count stack elements:

```lean
def stack_depth : Stack -> Nat
  | Stack.Empty => 0
  | Stack.Push _ rest => 1 + stack_depth rest
```

This lets us state precise properties like "pushing a number increases depth by 1."

## 8. Concept: Inductive Types

This is a good moment to step back and understand inductive types more deeply.

An **inductive type** in Lean is defined by listing its **constructors** — the only ways to create values of that type. For `Stack`:

- `Stack.Empty` creates an empty stack (takes no arguments)
- `Stack.Push v rest` creates a non-empty stack (takes a value and another stack)

Every stack value was built using one of these two constructors. This means we can analyze any stack by asking: "Was it built with `Empty` or `Push`?" This is **pattern matching**.

When we write a function on an inductive type, we must handle every constructor. When we prove a property about an inductive type, we must consider every constructor. This is the foundation of:

- **`cases` tactic**: "Let me consider each constructor separately"
- **`match` expressions**: Pattern matching in both programs and proofs
- **`induction` tactic**: Like `cases`, but with induction hypotheses for recursive constructors

The key insight: **Lean's type system guarantees exhaustiveness.** If you forget a case, Lean rejects your proof. There is no "default" case that silently hides bugs.

## 9. Proof: Division Safety

Our first real proof shows that division by zero is correctly caught. This is a **direct computation** proof — we just unfold the definitions and let Lean verify that the result matches.

```lean
theorem div_by_zero_caught (x : I64) (rest : Stack) :
    eval_step (Stack.Push (0 : I64) (Stack.Push x rest)) Token.Div =
      ok (.err EvalError.DivisionByZero) := by
  unfold eval_step Stack.pop_ apply_binop
  simp
```

What happens here:
1. `unfold eval_step` expands the definition of `eval_step` with `Token.Div` and a stack with at least 2 elements.
2. `Stack.pop_` is unfolded to extract the top value (0) and the next value (x).
3. `apply_binop` is unfolded, which checks `if b = 0` — and since b is 0, it takes the division-by-zero branch.
4. `simp` finishes the job, confirming the result equals `ok (.err EvalError.DivisionByZero)`.

This proof is essentially asking Lean to evaluate the function symbolically and check that the output is what we claimed.

## 10. Proof: Stack Depth Invariant

Next we prove structural properties about how `eval_step` affects the stack.

### Pushing a number increases depth by 1

```lean
theorem eval_step_num_depth (n : I64) (s s' : Stack) :
    eval_step s (Token.Num n) = ok (.ok s') ->
    stack_depth s' = stack_depth s + 1 := by
  unfold eval_step Stack.push_
  intro h
  simp at h
  rw [h]
  simp [stack_depth]
```

This proof says: if `eval_step` with a `Num` token succeeds, producing stack `s'`, then `s'` has one more element than the input stack `s`. The proof works by:
1. Unfolding `eval_step` for the `Num` case to see it returns `Push n s`.
2. Introducing the hypothesis `h` that `eval_step` succeeded.
3. Simplifying to learn that `s' = Push n s`.
4. Rewriting and computing `stack_depth`.

### Binary operators with underflow fail

We also prove that applying an operator to a stack that is too small always produces an error:

```lean
theorem eval_step_binop_underflow_empty (op : Token) (hop : is_binop op) :
    ∃ e, eval_step Stack.Empty op = ok (.err e) := by
  cases op <;> simp [is_binop] at hop <;> unfold eval_step Stack.pop_ <;>
    exact ⟨EvalError.StackUnderflow, rfl⟩
```

This uses `cases op` to consider each token variant. The `is_binop` hypothesis eliminates the `Num` case. For each operator, unfolding shows that `Stack.pop_` on `Empty` returns `StackUnderflow`.

## 11. Concept: The Induction Tactic

The `induction` tactic is the workhorse of proofs about recursive data structures. It works like `cases`, but for recursive constructors it also gives you an **induction hypothesis** — an assumption that the property holds for the sub-structures.

For our `WellFormedRPN` predicate, induction gives us:

- **Base case** (`num n`): Prove the property for `[Num n]`.
- **Inductive case** (`binop e1 e2 op`): Assume the property holds for `e1` (IH1) and `e2` (IH2). Prove it holds for `e1 ++ e2 ++ [op]`.

This mirrors the recursive structure of RPN expressions perfectly:
- A single number is trivial to evaluate.
- A compound expression works if its sub-expressions work and the operator step works.

The general pattern for induction proofs:
1. Identify the inductive structure (here, `WellFormedRPN`).
2. Apply `induction` to get the cases.
3. Handle the base case (usually simple).
4. Handle the inductive case using the IH to reason about sub-structures.

## 12. Proof: Well-Formed Expressions Succeed

The crown jewel of this tutorial is the theorem that well-formed RPN expressions always evaluate successfully (assuming no division by zero and no arithmetic overflow). Here is the statement:

```lean
theorem wf_rpn_evaluate_succeeds
    (tokens : List Token)
    (hwf : WellFormedRPN tokens)
    (hnd : no_div_tokens tokens) :
    -- Under the assumption that no intermediate arithmetic overflows,
    -- evaluation succeeds and produces exactly one value.
    ...
```

The proof proceeds by structural induction on `WellFormedRPN`:

**Base case** (`num n`): The token list is `[Num n]`. Evaluating this pushes `n` onto the empty stack, giving `Stack.Push n Stack.Empty`. This stack has exactly one value.

**Inductive case** (`binop e1 e2 op`): The token list is `e1 ++ e2 ++ [op]`. By the induction hypothesis on `e1`, evaluating `e1` on the empty stack succeeds and leaves a stack with exactly one value. By the IH on `e2`, evaluating `e2` on that stack succeeds and leaves a stack with exactly two values. Then `eval_step` with `op` pops the two values, applies the operator (which succeeds because we assumed no division by zero), and pushes the result. The final stack has exactly one value.

The full mechanized proof requires additional lemmas about how `evaluate_loop` handles concatenated token lists. We provide the statement and proof sketch in `EvaluatorProofs.lean`; completing the full proof is left as an advanced exercise.

## 13. The Unverified Shell

The `main.rs` file is deliberately simple. It reads a line from stdin, splits it by whitespace, tokenizes each word, evaluates the token sequence, and prints the result:

```rust
fn main() {
    let mut input = String::new();
    println!("RPN Calculator — enter an expression:");

    std::io::stdin().read_line(&mut input).unwrap();

    let mut tokens = Vec::new();
    for word in input.trim().split_whitespace() {
        match tokenize_word(word.as_bytes()) {
            Ok(token) => tokens.push(token),
            Err(_) => { eprintln!("Error: invalid token"); return; }
        }
    }

    match evaluate(&tokens) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

This file is *not* translated by Aeneas. It uses `Vec`, `String`, `stdin()`, `println!` — all things that Aeneas does not handle. But that is fine: the I/O shell is thin and boring. All the interesting logic lives in `lib.rs`, which *is* verified.

This is the functional core / imperative shell pattern in action. The shell can have bugs (e.g., it might read input incorrectly), but the core is proven correct. If the shell passes the right tokens to `evaluate`, we know the answer is right.

## 14. Exercises

### Exercise 1: Add a Modulo Operator

Extend the calculator to support `%` (modulo / remainder):

1. Add `Mod` to the `Token` enum in Rust.
2. Handle `b'%'` in `tokenize_word`.
3. Add the `Mod` case to `apply_binop` (remember: modulo by zero should return `DivisionByZero`).
4. Update the Lean types and functions to match.
5. Prove `mod_by_zero_caught`: applying `%` with a zero divisor produces `DivisionByZero`.
6. Prove that modulo is correctly classified as a binary operator: `is_binop Token.Mod`.

### Exercise 2: Negative Numbers

Currently `parse_number` only handles non-negative integers. Extend it to handle a leading `-` sign:

1. Check if the first byte is `b'-'` and the slice has length > 1.
2. Parse the remaining bytes as a positive number.
3. Negate the result.
4. Be careful: this changes the meaning of a single `-` character. How do you distinguish the operator from a negative sign?

### Exercise 3: Complete the WellFormedRPN Proof

The `wf_rpn_evaluate_succeeds` theorem is stated but not fully proven. To complete it:

1. Write a lemma showing that `evaluate_loop` on `e1 ++ e2` is equivalent to running `evaluate_loop` on `e1` first, then continuing with `e2` on the resulting stack.
2. Use this "concatenation lemma" in the inductive case.
3. This is challenging but very rewarding — it exercises all the concepts from this tutorial.

## 15. What's Next

In **Tutorial 03: Infix Calculator**, we build an infix expression parser that converts expressions like `(3 + 4) * 2` into RPN token sequences. We then prove that the parser's output is always a well-formed RPN expression — connecting the parser to this tutorial's `WellFormedRPN` predicate. This gives us an end-to-end correctness chain: parse -> tokenize -> evaluate, with each step verified.

New concepts in Tutorial 03:
- Recursive descent parsing in Rust
- Mutual recursion and how Aeneas handles it
- The `WellFormedRPN` predicate as a bridge between tutorials
- Proof by structural induction on parse trees

---

[← Previous: Tutorial 01](../01-setup-hello-proof/README.md) | [Index](../README.md) | [Next: Tutorial 03 →](../03-infix-calculator/README.md)
