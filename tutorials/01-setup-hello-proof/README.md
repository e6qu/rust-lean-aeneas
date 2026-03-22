[Index](../README.md) | [Next: Tutorial 02 →](../02-rpn-calculator/README.md)

# Tutorial 01: Setup and Hello Proof

## Welcome

In this tutorial you will install the Lean 4 theorem prover and the Aeneas
verification toolchain, translate a small Rust library into pure Lean code,
and write your first formal proofs about that code.

By the end you will have proved, mathematically, that a Rust function
handles every possible input correctly -- not just the inputs you thought
to test, but literally all of them.

**What you will learn:**

- How to install Lean 4, Charon, and Aeneas.
- How the Charon/Aeneas pipeline translates Rust into Lean.
- How to read the generated Lean code.
- How to state and prove theorems about Rust functions.
- The core idea behind formal verification: types are propositions,
  programs are proofs.

**Why this matters.** Tests check examples. A test for `checked_add(2, 3)`
tells you the function works for that one pair of inputs. A formal proof of
`checked_add` tells you the function works for every one of the roughly
eighteen quintillion possible pairs of `u32` inputs. That is the difference
between testing and verification.


## Prerequisites

You need:

- Familiarity with basic programming (variables, functions, if-statements).
- A working command line (terminal) on macOS or Linux.
- Rust installed (`rustup` and `cargo`). If you do not have Rust yet,
  visit <https://rustup.rs>.

No prior knowledge of theorem proving, Lean, or Aeneas is required.
This tutorial explains every concept from the ground up.


## Installation

### Step 1: Install Lean 4

Lean is installed through `elan`, its version manager (similar to `rustup`
for Rust).

```bash
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh
```

Follow the prompts, then restart your terminal (or `source ~/.profile`)
so that `lean` is on your PATH. Verify the installation:

```bash
lean --version
```

You should see output like `leanprover/lean4:v4.x.0`.

### Step 2: Install VS Code and the Lean 4 extension

While you can use any editor, VS Code gives you the best experience with
Lean. Install the **lean4** extension from the VS Code marketplace. It
provides:

- Real-time type checking as you type.
- An interactive "Lean Infoview" panel that shows your proof state.
- Go-to-definition, hover information, and error highlighting.

Open VS Code, press Ctrl+Shift+X (Cmd+Shift+X on macOS), search for
"lean4", and install it.

### Step 3: Install Aeneas

Aeneas is the tool that translates Rust into Lean. It comes with a
companion tool called **Charon** that extracts Rust's internal
representation.

**Option A -- Nix (easiest, recommended)**

If you have the Nix package manager installed:

```bash
# Test that Aeneas works
nix run github:aeneasverif/aeneas -L -- --version

# Test that Charon works
nix run github:aeneasverif/aeneas#charon -L -- --version
```

If you do not have Nix, install it from <https://nixos.org/download>.

**Option B -- From source**

```bash
# Install OCaml 5.x via opam
opam switch create 5.1.0

# Clone and build Aeneas (includes Charon)
git clone https://github.com/aeneasverif/aeneas.git
cd aeneas
make
```

After building, make sure `charon` and `aeneas` are on your PATH.

### Step 4: Verify everything works

```bash
lean --version          # Lean 4
charon --version        # Charon
cargo --version         # Rust
```

If all three commands print version information, you are ready to go.


## The Rust Code

Open `rust/src/lib.rs`. It contains six small functions, each chosen to
illustrate a different aspect of formal verification.

### checked_add -- overflow detection

```rust
pub fn checked_add(x: u32, y: u32) -> Option<u32> {
    if y <= u32::MAX - x {
        Some(x + y)
    } else {
        None
    }
}
```

This function adds two unsigned 32-bit integers. If the result fits in a
`u32`, it returns `Some(result)`. If it would overflow, it returns `None`.
This is the "hello world" of verified Rust: simple enough to understand at
a glance, yet it has a real correctness property (it never panics and
correctly detects overflow).

### safe_divide -- error handling

```rust
pub fn safe_divide(x: i64, y: i64) -> Result<i64, ()> {
    if y == 0 { Err(()) } else { Ok(x / y) }
}
```

Returns `Err(())` on division by zero, `Ok(quotient)` otherwise. This
shows how Rust's `Result` type translates into Lean: Aeneas wraps
everything in its own `Result` (for panics), and our `Result<i64, ()>`
becomes an inner `Result` nested inside the outer one.

### safe_abs -- subtle edge cases

```rust
pub fn safe_abs(x: i64) -> Result<i64, ()> {
    if x == i64::MIN { Err(()) }
    else if x < 0    { Ok(-x) }
    else              { Ok(x) }
}
```

Why does `i64::MIN` get special treatment? Because the absolute value of
`i64::MIN` (-9223372036854775808) is 9223372036854775808, which is one
larger than `i64::MAX`. Negating `i64::MIN` would overflow. This is
exactly the kind of subtle bug that formal verification catches and that
tests can easily miss if you do not think to test that specific value.

### clamp -- a pure function

```rust
pub fn clamp(x: i32, lo: i32, hi: i32) -> i32 {
    if x < lo      { lo }
    else if x > hi  { hi }
    else            { x }
}
```

Restricts a value to the range [lo, hi]. This is a pure function with no
failure modes -- no arithmetic that can overflow, just comparisons. It
gives us the simplest proofs in this tutorial.

### max_of and min_of -- exercises

```rust
pub fn max_of(a: i32, b: i32) -> i32 { if a >= b { a } else { b } }
pub fn min_of(a: i32, b: i32) -> i32 { if a <= b { a } else { b } }
```

These two are left as exercises for you to prove correct.

### Running the tests

Before we do any verification, make sure the Rust code compiles and the
conventional tests pass:

```bash
cd rust && cargo test
```

You should see all tests pass. Remember: these tests check specific input
values. Our proofs will check all values.


## The Translation Pipeline

Translating Rust to Lean is a two-step process.

### Step 1: Charon extracts Rust MIR

```bash
cd rust
charon cargo --preset=aeneas
```

This runs the Rust compiler up to the point where it produces **MIR**
(Mid-level Intermediate Representation). MIR is Rust's internal
representation after type checking, borrow checking, and monomorphization.
It is a simplified, control-flow-graph form of your code. Charon reads
the MIR and writes an `.llbc` file -- LLBC stands for Low-Level Borrow
Calculus, which is Aeneas's own intermediate representation.

### Step 2: Aeneas translates LLBC to Lean

```bash
aeneas -backend lean hello_proof.llbc
```

Aeneas reads the `.llbc` file and produces pure Lean 4 code: one file for
types (`Types.lean`) and one for function definitions (`Funs.lean`).

### Using the provided script

Instead of running both steps manually, you can use the automation script:

```bash
./scripts/translate.sh
```

This script runs Charon and then Aeneas, printing each step as it goes.
After it finishes, the generated Lean files are in `lean/HelloProof/`.

The generated files (`Types.lean` and `Funs.lean`) should not be edited
by hand. They are overwritten every time you re-run the pipeline. Your
hand-written proofs go in `Proofs.lean`, which the pipeline does not
touch.


## Understanding the Generated Lean Code

This is the most important section of the tutorial. Open
`lean/HelloProof/Funs.lean` and look at the translation of `checked_add`:

```lean
def checked_add (x : U32) (y : U32) : Result (Option U32) := do
  let max_minus_x ← U32.MAX - x
  if y ≤ max_minus_x then do
    let sum ← x + y
    ok (some sum)
  else
    ok none
```

Let us go through every piece of this.

### `U32` -- bounded integer types

`U32` is Aeneas's type for unsigned 32-bit integers. Unlike a
mathematical natural number, which can be arbitrarily large, a `U32` is
bounded: its values range from 0 to 2^32 - 1 (4294967295). Aeneas
provides `U32`, `I32`, `I64`, and other Rust integer types, each with
their precise bounds.

### `Result T` -- the monadic wrapper

Every Aeneas-generated function returns `Result T`. This is Aeneas's way
of tracking that Rust operations can panic or overflow. A value of type
`Result T` is either:

- `ok v` -- the operation succeeded and produced value `v`, or
- `fail e` -- the operation panicked or overflowed.

For `checked_add`, the return type is `Result (Option U32)`. The outer
`Result` is Aeneas's wrapper; the inner `Option U32` is the translation
of Rust's `Option<u32>`.

### `do` notation -- sequencing fallible operations

The `do` keyword begins a block of sequential operations, similar to
`async/await` in other languages but for fallible computations instead
of asynchronous ones. Each step in the `do` block might fail, and if it
does, the entire block immediately returns that failure.

### `let x <- expr` -- bind and propagate

This is the key pattern. It means:

1. Evaluate `expr`, which returns a `Result`.
2. If the result is `fail`, stop here and propagate the failure.
3. If the result is `ok v`, bind `v` to the name `x` and continue.

This is analogous to Rust's `?` operator. In fact, that is exactly what
Aeneas is modeling.

### Bounded arithmetic

`U32.MAX - x` is **not** mathematical subtraction. It is bounded
subtraction on `U32` values: it returns `ok (MAX - x)` when `x <= MAX`
(which is always true for `U32`), or `fail` on underflow. Aeneas is
conservative -- it wraps the operation even when a human can see it cannot
fail.

Similarly, `x + y` is bounded addition. It returns `fail` if the sum
exceeds `U32.MAX`. But notice that we only reach this line when
`y <= max_minus_x`, i.e., when `y <= MAX - x`, so `x + y <= MAX` and
the addition always succeeds. Our proofs will formalize exactly this
reasoning.

### `coercion` -- bridging bounded and mathematical worlds

When you see `↑x` in a proof (the upward arrow), it is a **coercion**
that lifts a bounded integer to a mathematical integer. If `x : U32`,
then `↑x : Int` is an unbounded mathematical integer with the same
numeric value. This lets you reason about values without worrying about
overflow, and then connect back to the bounded world.

### The safe_divide translation

For comparison, here is `safe_divide`:

```lean
def safe_divide (x : I64) (y : I64) : Result (core.result.Result I64 Unit) := do
  if y = (0 : I64) then
    ok (.err ())
  else do
    let result ← x / y
    ok (.ok result)
```

Notice the double-Result pattern:

- **Outer `Result`**: Aeneas's wrapper for panics. If `x / y` overflows
  (which happens when `x = I64.MIN` and `y = -1`), the outer Result
  becomes `fail`.
- **Inner `core.result.Result I64 Unit`**: The translation of Rust's
  `Result<i64, ()>`. `.ok result` means division succeeded; `.err ()`
  means division by zero.

This layered structure is important to understand: Aeneas separates
"the function panicked" (outer fail) from "the function returned an
error value" (inner .err).


## Concept: The Curry-Howard Correspondence

Before we dive into proofs, you need to understand the single most
important idea in formal verification.

**Types are propositions. Programs are proofs.**

This is called the Curry-Howard correspondence, and it is not a metaphor
or analogy. It is literally how Lean works.

- The type `Nat` is a proposition: "natural numbers exist." Any value
  like `42 : Nat` is a proof that a natural number exists.
- The type `a = b` is a proposition: "a equals b." A term of this type
  is a proof of equality.
- A function type `A -> B` is a proposition: "A implies B." A function
  from `A` to `B` is a proof of the implication: given a proof of `A`,
  it produces a proof of `B`.
- `Exists r, f x = ok r` is a proposition: "there exists some `r` such
  that `f x` returns `ok r`." A proof constructs a specific `r` and shows
  it satisfies the equation.
- `forall x, P x` is a proposition: "for all x, P holds." A proof is a
  function that, given any `x`, produces a proof of `P x`.

When you write a theorem in Lean, you are declaring a type. When you write
the proof, you are constructing a value of that type. Lean's type checker
then verifies that your value actually has the type you claimed -- which is
the same as verifying that your proof actually proves the theorem.

This is why formal verification is so trustworthy: the "proof checker" is
just the type checker, the same battle-tested piece of infrastructure that
checks every line of code.


## Your First Theorem

Open `lean/HelloProof/Proofs.lean` and find `checked_add_no_panic`. This
is our first theorem. Let us take it apart piece by piece.

### The statement

```lean
theorem checked_add_no_panic (x y : U32) :
    ∃ r, checked_add x y = ok r := by
```

Reading this aloud:

- `theorem` -- we are declaring a theorem (like `def` but for proofs).
- `checked_add_no_panic` -- the theorem's name.
- `(x y : U32)` -- universally quantified parameters. "For all unsigned
  32-bit integers x and y..."
- `: exists r, checked_add x y = ok r` -- the conclusion. "...there
  exists some result r such that `checked_add x y` equals `ok r`."
- `:= by` -- we will prove this using **tactic mode**.

In other words: no matter what `u32` values you pass in, `checked_add`
never fails. It always returns `ok` with some value inside.

### Tactic mode

The `by` keyword switches Lean into tactic mode. Instead of directly
writing a proof term, you interact with a **proof state** made up of
goals. Each goal is something you still need to prove. Tactics transform
goals into simpler subgoals. When no goals remain, the proof is complete.

If you are using VS Code with the Lean 4 extension, place your cursor
after `by` and look at the Lean Infoview panel. You will see the current
goal displayed for you. As you add each tactic, the goal updates in
real time. This interactive feedback loop is one of the best features of
working with Lean.

### Walking through the proof

```lean
  unfold checked_add
```

**`unfold`** replaces the function name with its definition. After this
tactic, the goal no longer mentions `checked_add` -- instead it shows the
full if-then-else expression from the function body. This lets the
subsequent tactics see the structure of the code.

```lean
  split
```

**`split`** case-splits on the `if` expression. It creates two subgoals:
one where the condition is true (no overflow), and one where the condition
is false (overflow). You prove each branch separately.

```lean
  · simp only [bind_ok]
    progress as ⟨diff, hdiff⟩
```

The dot `·` focuses on one subgoal. **`simp only [bind_ok]`** simplifies
the monadic bind operations. **`progress`** is an Aeneas-specific tactic
that advances past a monadic `let x <- expr` step. It automatically
proves that the operation succeeds (using arithmetic reasoning about
bounded integers) and names the result. Here `diff` is the value of
`U32.MAX - x` and `hdiff` is a proof of its arithmetic property.

```lean
      exact ⟨some sum, rfl⟩
```

**`exact`** provides the exact proof term. The angle brackets `⟨..⟩` build
an existential witness: `some sum` is the value we claim exists, and `rfl`
(reflexivity) proves the equality. "The result is `some sum`, and the
equation holds by definition."

```lean
    · exact ⟨none, rfl⟩
```

In the else branch (overflow), the result is `none`, and again the
equation holds by definition.

The key takeaway: the proof mirrors the structure of the code. The `if`
creates two cases, and we prove each case returns `ok`. The `progress`
tactic handles the arithmetic details automatically.


## Deeper Proofs

### checked_add_spec -- full correctness

The no-panic theorem tells us the function always returns `ok`, but it
does not say anything about the value inside. The specification theorem
says exactly what value is returned:

```lean
theorem checked_add_spec (x y : U32) :
    (↑x + ↑y ≤ U32.max →
      ∃ z, checked_add x y = ok (some z) ∧ (↑z : Int) = ↑x + ↑y) ∧
    (↑x + ↑y > U32.max →
      checked_add x y = ok none) := by
```

This says two things:

1. **No overflow case**: If the mathematical sum of x and y fits in a
   `u32` (is at most `U32.max`), the function returns `Some(z)` where
   `z` equals `x + y`.
2. **Overflow case**: If the mathematical sum exceeds `U32.max`, the
   function returns `None`.

Notice the coercion `↑x`: this lifts the bounded `U32` to a mathematical
`Int` so we can talk about the "true" sum without worrying about overflow.

The proof uses `constructor` to split the conjunction into two subgoals,
then proves each side with the same unfold/split/progress pattern. When
a case leads to a contradiction (e.g., assuming no overflow but landing
in the else branch), the `omega` tactic resolves it automatically.
`omega` is a decision procedure for linear arithmetic -- it can solve
goals involving addition, subtraction, and comparison of integers.

### clamp_in_bounds -- result is always in range

```lean
theorem clamp_in_bounds (x lo hi : I32) (h : (↑lo : Int) ≤ ↑hi) :
    ∃ r, clamp x lo hi = ok r ∧ (↑lo : Int) ≤ ↑r ∧ (↑r : Int) ≤ ↑hi := by
```

Given that `lo <= hi`, the clamped result is always between `lo` and `hi`
inclusive. This is the key correctness property of clamp. The proof is
short -- just unfold, split on the two if-conditions, and each case is
solved by simple arithmetic. The case where `x < lo` returns `lo`, which
satisfies `lo <= lo` trivially and `lo <= hi` by hypothesis. The case
where `x > hi` returns `hi`, which satisfies `lo <= hi` by hypothesis
and `hi <= hi` trivially. The middle case returns `x`, which is in range
by the negation of both conditions.

### clamp_idempotent -- stability

```lean
theorem clamp_idempotent (x lo hi : I32)
    (h_lo : (↑lo : Int) ≤ ↑x) (h_hi : (↑x : Int) ≤ ↑hi) :
    clamp x lo hi = ok x := by
```

If `x` is already in range, clamp returns it unchanged. This is called
idempotency: applying the operation a second time has no effect.

### The `@[step]` annotation

You will notice that every theorem is marked with `@[step]`. This
annotation registers the theorem with the `step` tactic (also known as
`progress`). When a later proof encounters a function call like
`checked_add x y`, the `progress` tactic will automatically search for
`@[step]`-annotated theorems about `checked_add` and apply them.

This is how proofs compose. You prove small specifications for individual
functions, annotate them with `@[step]`, and then when you prove
properties of functions that call those functions, the step tactic
automatically uses your earlier proofs. In Tutorial 02, you will see this
composition in action when an evaluator calls arithmetic functions and the
proofs chain together.


## Types as Propositions -- Practical Patterns

Here are the common patterns you will see when stating properties about
Aeneas-translated functions:

| English statement                          | Lean type                                                    |
|--------------------------------------------|--------------------------------------------------------------|
| "f never panics"                           | `forall x, exists r, f x = ok r`                              |
| "f returns a value satisfying P"           | `forall x, exists r, f x = ok r /\ P r`                       |
| "f only fails when Q holds"               | `forall x, not (Q x) -> exists r, f x = ok r`                  |
| "f and g produce the same result"          | `forall x, f x = g x`                                        |
| "f preserves an invariant"                 | `forall x, Inv x -> exists r, f x = ok r /\ Inv r`            |

These patterns recur throughout every tutorial. Once you recognize them,
reading and writing theorem statements becomes much easier.


## Exercises

Open `lean/HelloProof/Proofs.lean` and scroll to the bottom. You will find
four exercises, each commented out. Uncomment them one at a time and
replace `sorry` with a real proof.

### Exercise 1: max_of_no_fail

```lean
theorem max_of_no_fail (a b : I32) : ∃ r, max_of a b = ok r := by
  sorry
```

Prove that `max_of` never fails. **Hint:** This is structurally identical
to `clamp_no_fail`. Unfold the definition, split on the if-condition, and
provide the witness in each branch.

### Exercise 2: max_of_spec

```lean
theorem max_of_spec (a b : I32) :
    ∃ r, max_of a b = ok r ∧ (↑r : Int) ≥ ↑a ∧ (↑r : Int) ≥ ↑b := by
  sorry
```

Prove that the result of `max_of` is greater than or equal to both inputs.
**Hint:** After unfolding and splitting, you will need `omega` to handle
the arithmetic inequalities. In the `a >= b` branch, the result is `a`,
and you need to show `a >= a` (trivial) and `a >= b` (from the condition).

### Exercise 3: min_of_spec

```lean
theorem min_of_spec (a b : I32) :
    ∃ r, min_of a b = ok r ∧ (↑r : Int) ≤ ↑a ∧ (↑r : Int) ≤ ↑b := by
  sorry
```

Prove that the result of `min_of` is less than or equal to both inputs.
**Hint:** Same structure as Exercise 2, with the inequalities reversed.

### Exercise 4: max_ge_min

```lean
theorem max_ge_min (a b : I32) :
    ∃ mx mn, max_of a b = ok mx ∧ min_of a b = ok mn ∧ (↑mx : Int) ≥ ↑mn := by
  sorry
```

Prove that `max_of(a, b) >= min_of(a, b)` for all inputs. **Hint:** Unfold
both `max_of` and `min_of`, then split on all the conditions. In each case,
provide the two witnesses and use `omega` for the arithmetic.


## Building and Checking Proofs

To verify that all proofs (including your exercise solutions) type-check:

```bash
cd lean && lake build
```

If Lake reports no errors, every proof in the project is correct. If you
see an error, the Lean Infoview in VS Code will show you exactly which
goal could not be proved.


## What's Next

In **Tutorial 02: RPN Calculator**, we move beyond simple arithmetic
functions. You will:

- Define a custom **Stack** data type in Rust and see it translated into
  a Lean `inductive` type.
- Build an RPN (Reverse Polish Notation) expression evaluator.
- Prove the evaluator correct for **all** well-formed expressions using
  structural induction -- a technique for proving properties about
  recursive data structures.
- Learn how `@[step]`-annotated specs compose: the evaluator proof will
  automatically use the arithmetic specs you proved here.

The jump from "prove a single function correct" to "prove a system of
functions correct" is where formal verification starts to feel powerful.
See you in Tutorial 02.

---

[Index](../README.md) | [Next: Tutorial 02 →](../02-rpn-calculator/README.md)
