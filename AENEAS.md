[← Back to README](README.md) | [← Lean Reference](LEAN.md)

# Aeneas: A Comprehensive Reference Guide

## Rust-to-Lean Verification by Functional Translation

**Audience**: Developers who know basic algorithms and some Rust, but have no
background in formal verification.

**Last updated**: March 2026

---

## Table of Contents

1.  [What is Aeneas?](#1-what-is-aeneas)
2.  [The Problem Aeneas Solves](#2-the-problem-aeneas-solves)
3.  [The Translation Pipeline](#3-the-translation-pipeline)
4.  [The Key Insight: Forward and Backward Functions](#4-the-key-insight-forward-and-backward-functions)
5.  [What Rust Features Are Supported](#5-what-rust-features-are-supported)
6.  [Writing Aeneas-Friendly Rust](#6-writing-aeneas-friendly-rust)
7.  [The Proof Workflow](#7-the-proof-workflow)
8.  [The step/progress Tactic](#8-the-stepprogress-tactic)
9.  [Charon: The MIR Extraction Framework](#9-charon-the-mir-extraction-framework)
10. [Comparison with Other Rust Verification Tools](#10-comparison-with-other-rust-verification-tools)
11. [Real-World Usage](#11-real-world-usage)
12. [Installation](#12-installation)
13. [Research Papers](#13-research-papers)
14. [Resources](#14-resources)

---

## 1. What is Aeneas?

### The Elevator Pitch

Aeneas is a verification toolchain that translates safe Rust programs into
pure functional models in Lean 4. You write ordinary Rust code, run it through
Aeneas, and get Lean 4 code that you can then write mathematical proofs about.
No annotations in your Rust. No special syntax. Just Rust.

It also supports Coq, F*, and HOL4 as proof-assistant backends, though Lean 4
is the primary and most actively developed target.

### Who Built It?

Aeneas was created by **Son Ho** and **Jonathan Protzenko**.

- **Son Ho** completed his PhD at Inria Paris (the French national research
  institute for computer science) and is now a researcher at Microsoft Azure
  Research in Cambridge, UK. He is the primary architect of both the Aeneas
  translator and the Charon extraction framework.

- **Jonathan Protzenko** is a researcher now at Google. He has a long
  background in verified systems software, including significant contributions
  to Project Everest (verified HTTPS) and the F* proof assistant ecosystem.

The project lives under the **AeneasVerif** GitHub organization, which hosts
the main Aeneas repository, Charon, and related tooling.

### The Core Insight

Here is the idea that makes Aeneas work, stated as simply as possible:

> Rust's borrow checker already proves that your program has no aliasing
> violations, no data races, and no use-after-free bugs. Aeneas leverages
> this guarantee to eliminate memory reasoning entirely from the
> verification problem. The result is pure functional code where your
> proofs focus exclusively on functional correctness.

What does "pure functional code" mean here? It means code with no mutable
state, no pointers, no heap. Just values flowing through functions. If you
have ever written Haskell or used `map`/`filter`/`fold` in any language,
you have written pure functional code. Pure functional code is dramatically
easier to reason about mathematically than imperative code with mutable
state.

### Why "Aeneas"?

The name comes from Aeneas, the Trojan hero from Virgil's epic poem the
*Aeneid*. In the story, Aeneas escapes the destruction of Troy and founds
what will become Rome. The metaphor: Aeneas (the tool) takes Rust programs
out of the world of imperative systems code and brings them to the promised
land of pure functional verification.

### What Does "Verification" Mean?

If you are new to formal verification, here is the basic idea:

- **Testing** checks that your program works correctly on specific inputs.
  You run `sort([3,1,2])` and check that you get `[1,2,3]`.

- **Formal verification** proves that your program works correctly on ALL
  possible inputs. You prove mathematically that for EVERY possible list,
  your sort function returns a list that is (a) sorted and (b) a
  permutation of the input.

The difference is absolute. Testing can find bugs. Verification proves
their absence.

Formal verification is done using a **proof assistant** (also called an
**interactive theorem prover**). This is a program that checks mathematical
proofs, with the proofs written in a specialized language. The proof
assistant checks every logical step, so if your proof is accepted, you
can trust the result. Lean 4 is one such proof assistant; Coq, F*, and
HOL4 are others.

### The Aeneas Ecosystem at a Glance

```
+------------------+     +------------------+     +------------------+
|                  |     |                  |     |                  |
|   Your Rust Code |---->|     Charon       |---->|     Aeneas       |
|   (safe, no      |     | (MIR extraction) |     | (translation to  |
|    annotations)  |     |                  |     |  pure Lean 4)    |
|                  |     |                  |     |                  |
+------------------+     +------------------+     +------------------+
                                                          |
                                                          v
                                                  +------------------+
                                                  |                  |
                                                  |  Generated Lean  |
                                                  |  (pure functions)|
                                                  |       +         |
                                                  |  Your Proofs    |
                                                  |  (hand-written) |
                                                  |                  |
                                                  +------------------+
```

---

## 2. The Problem Aeneas Solves

### What Rust Guarantees

Rust's type system and borrow checker give you strong safety guarantees at
compile time:

- **Memory safety**: No use-after-free, no double-free, no dangling
  pointers.
- **No data races**: At most one mutable reference OR any number of
  shared references at a time.
- **No null pointer dereference**: `Option<T>` instead of nullable
  pointers.
- **No buffer overflows**: Bounds checking on array/slice access.

These are enormous guarantees. They eliminate entire classes of bugs that
plague C and C++ programs. This is why Rust is increasingly adopted for
security-critical systems code.

### What Rust Does NOT Guarantee

Rust does NOT guarantee that your program does the right thing. Here are
things Rust will happily let you do:

```rust
// Compiles fine. Rust doesn't care that this is wrong.
fn sort(v: &mut Vec<i32>) {
    // "Sort" by just reversing. This is not sorting.
    v.reverse();
}

// Compiles fine. Rust doesn't care about the math.
fn average(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    // Oops: should divide by values.len(), not values.len() + 1
    sum / (values.len() + 1) as f64
}

// Compiles fine. Rust doesn't check your crypto.
fn encrypt(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    // "Encryption" by XOR with key, repeating.
    // This is trivially breakable. Rust doesn't know that.
    plaintext.iter()
        .zip(key.iter().cycle())
        .map(|(p, k)| p ^ k)
        .collect()
}
```

Every one of these functions compiles, runs without crashing, and is
completely wrong. The sort does not sort. The average is off by one. The
encryption is insecure.

### The Gap

```
     What Rust Guarantees           What You Actually Need
    +---------------------+       +---------------------+
    |                     |       |                     |
    |  Memory safety      |       |  Memory safety      |
    |  No data races      |       |  No data races      |
    |  No null pointers   |       |  No null pointers   |
    |  Bounds checking    |       |  Bounds checking    |
    |                     |       |                     |
    +---------------------+       |  FUNCTIONAL         |
                                  |  CORRECTNESS:       |
                                  |                     |
              THE GAP --------->  |  "sort" sorts       |
                                  |  "encrypt" encrypts |
                                  |  "average" averages |
                                  |  etc.               |
                                  |                     |
                                  +---------------------+
```

### How Aeneas Bridges the Gap

Aeneas bridges this gap by translating your Rust code into a form where
you can write mathematical proofs about it.

The workflow:

1. You write Rust code. Ordinary safe Rust. No annotations, no special
   macros, no verification-specific syntax.

2. Aeneas translates this code into pure Lean 4 functions.

3. You write proofs in Lean 4 about these generated functions.

4. Lean 4 checks your proofs mechanically. If Lean accepts the proof,
   the property is guaranteed to hold for all inputs.

The critical point: the Lean code is a **faithful model** of your Rust
code. It has the same behavior. So proving something about the Lean code
is equivalent to proving it about the Rust code (modulo the correctness
of the translation, which is itself being studied and formalized).

### A Concrete Example of the Gap

Consider this Rust function:

```rust
fn max(a: u32, b: u32) -> u32 {
    if a >= b { a } else { b }
}
```

Rust guarantees: this function will not crash, will not corrupt memory,
will not have undefined behavior. It will always return a `u32`.

But does it actually compute the maximum? That is a mathematical question.
Rust has no opinion on it. Maybe someone accidentally wrote `a <= b`
instead of `a >= b` and the function returns the minimum.

Aeneas translates this to Lean 4, and then you can write a proof:

```lean
-- Prove that max returns a value >= both inputs
theorem max_ge_left (a b : U32) (h : max a b = ok result) :
    a <= result := by
  simp [max] at h
  split at h <;> simp_all

-- Prove that max returns one of the two inputs
theorem max_is_input (a b : U32) (h : max a b = ok result) :
    result = a \/ result = b := by
  simp [max] at h
  split at h <;> simp_all
```

If these proofs go through (and they do), then you have a mathematical
guarantee: `max` really does compute the maximum. Not just for the test
cases you thought of, but for ALL possible `u32` pairs.

---

## 3. The Translation Pipeline

### The Big Picture

```
                         THE AENEAS PIPELINE

  +--------+     +------+     +--------+     +--------+     +--------+
  |        |     |      |     |        |     |        |     |        |
  |  Rust  |---->| rustc|---->|  MIR   |---->| Charon |---->|  ULLBC |
  | Source |     |      |     |        |     |        |     |        |
  |        |     |      |     |        |     |        |     |        |
  +--------+     +------+     +--------+     +--------+     +---+----+
                                                                |
                                                                v
  +--------+     +--------+                              +--------+
  |        |     |        |                              |        |
  |  Pure  |<----| Aeneas |<-----------------------------|  LLBC  |
  | Lean 4 |     | (OCaml)|                              |        |
  |        |     |        |                              |        |
  +--------+     +--------+                              +--------+
```

Let us walk through each stage.

### Stage 1: Rust Source Code

This is your ordinary Rust code. It must be safe Rust (no `unsafe` blocks).
You do not add any annotations, macros, or verification hints. You write
idiomatic Rust, subject to the feature-support constraints described later
in this document.

Example:

```rust
fn increment(x: &mut u32) {
    *x += 1;
}
```

### Stage 2: rustc and MIR

**rustc** is the standard Rust compiler. When you compile Rust normally,
rustc goes through several intermediate representations:

```
Rust Source --> HIR --> THIR --> MIR --> LLVM IR --> Machine Code
```

**MIR** stands for **Mid-level Intermediate Representation**. It is a
simplified, desugared version of your program that the Rust compiler uses
internally. MIR has several important properties:

- All control flow is explicit (no `for` loops -- they are desugared
  into loop + iterator calls).
- All borrow operations are explicit.
- All moves and copies are explicit.
- Type checking and borrow checking have already been performed.

MIR is organized as a **control-flow graph (CFG)**: a collection of
**basic blocks**, each ending with a terminator (branch, return, call,
etc.), with edges between them.

Here is roughly what the MIR for our `increment` function looks like:

```
fn increment(_1: &mut u32) -> () {
    bb0: {
        _2 = CheckedAdd((*_1), const 1_u32);
        assert(!move (_2.1), "attempt to add with overflow") -> bb1;
    }
    bb1: {
        (*_1) = move (_2.0);
        return;
    }
}
```

Note: `bb0` and `bb1` are basic blocks. This is a CFG, not structured
code.

### Stage 3: Charon and ULLBC

**Charon** is a framework developed by the Aeneas team (primarily Son Ho)
that hooks into the Rust compiler to extract MIR. It replaces `rustc` in
the compilation process (via environment variables that Cargo respects).

Charon takes the MIR and produces **ULLBC**: the **Unstructured Low-Level
Borrow Calculus**.

"Unstructured" means it keeps the CFG structure from MIR. The program
is still a graph of basic blocks with goto-style control flow. "Low-Level
Borrow Calculus" is the formal calculus that the Aeneas team developed to
model Rust's borrow semantics.

ULLBC looks conceptually like this for our `increment` function:

```
fn increment(x: &mut u32) -> () {
    block0: {
        x0 = copy *x
        x1 = x0 + 1          // may panic (overflow)
        *x = move x1
        return
    }
}
```

(This is simplified; the real representation includes more detail about
borrow operations.)

ULLBC is close to MIR but cleaned up:

- Certain MIR-specific artifacts are removed.
- Types are simplified.
- The borrow structure is made more explicit.
- All the information Aeneas needs is preserved, and everything it
  does not need is discarded.

### Stage 4: LLBC (Restructured)

Charon then converts ULLBC into **LLBC**: the **(Structured) Low-Level
Borrow Calculus**.

The key transformation here is **control-flow restructuring**. ULLBC is a
control-flow graph (blocks and gotos). LLBC is a structured AST (if/else,
loops, sequences). This transformation is performed using a variant of the
**Relooper** algorithm, originally developed for Emscripten (compiling
LLVM to JavaScript, which also needs structured control flow).

Why does this matter? Because proof assistants work much better with
structured code. You can do induction over loop iterations, case analysis
over branches, etc. Working with a raw CFG in a proof assistant is
extremely painful.

LLBC for our function:

```
fn increment(x: &mut u32) -> () {
    x0 = copy *x
    x1 = x0 + 1              // may panic (overflow)
    *x = move x1
    return
}
```

For this simple function, ULLBC and LLBC look the same because there is
no complex control flow. For functions with loops and branches, the
difference is significant.

Consider a function with a loop:

```rust
fn sum_up_to(n: u32) -> u32 {
    let mut i = 0u32;
    let mut s = 0u32;
    while i < n {
        s += i;
        i += 1;
    }
    s
}
```

ULLBC (CFG form):
```
block0:
    i = 0
    s = 0
    goto block1

block1:                        // loop header
    if i < n goto block2
    else goto block3

block2:                        // loop body
    s = s + i                  // may panic
    i = i + 1                  // may panic
    goto block1

block3:                        // after loop
    return s
```

LLBC (structured form):
```
i = 0
s = 0
loop {
    if i < n {
        s = s + i              // may panic
        i = i + 1              // may panic
        continue
    } else {
        break
    }
}
return s
```

The LLBC version has an explicit `loop` construct instead of goto-based
cycles. This is what the Relooper algorithm produces.

### Stage 5: Aeneas Translation to Pure Lean 4

**Aeneas** itself is an OCaml program that reads LLBC and produces pure
functional code in the target proof assistant (Lean 4, Coq, F*, or HOL4).

This is the most intellectually interesting step. Aeneas must eliminate
all mutable state and produce purely functional code. It does this using
the **forward/backward function decomposition**, which is the subject of
the next section.

For our `increment` function, the output in Lean 4 is:

```lean
def increment (x : U32) : Result U32 :=
  x + 1#u32
```

Note what happened:

- The `&mut u32` became a plain `U32` value (no references).
- The mutation `*x += 1` became a pure expression `x + 1`.
- The function returns `Result U32` because addition on `U32` can
  overflow, which would cause a panic in Rust.
- There is no heap, no pointers, no mutable state. Just a value in,
  a value out.

### The .llbc File Format

The intermediate format between Charon and Aeneas is serialized as JSON
in a `.llbc` file. This file contains:

- All type definitions (structs, enums, type aliases)
- All function signatures and bodies
- All trait declarations and implementations
- All global/const definitions
- Source location information (for error messages)
- Dependency information between items

You generally do not need to inspect `.llbc` files by hand, but knowing
they are JSON is useful for debugging.

### Why This Architecture?

Separating Charon and Aeneas is a deliberate design decision:

1. **Charon is reusable**: Other tools can use Charon to extract Rust
   programs for analysis, not just Aeneas. It is designed as a general
   Rust analysis framework.

2. **Backend flexibility**: Aeneas can target multiple proof assistants
   from the same LLBC input.

3. **Separation of concerns**: Charon deals with the complexity of
   rustc internals. Aeneas deals with the complexity of producing
   correct pure-functional translations. Neither has to deal with both.

---

## 4. The Key Insight: Forward and Backward Functions

This is the most important section of this document. The forward/backward
decomposition is the core intellectual contribution of Aeneas.

### The Problem: Mutable References

Consider this Rust function:

```rust
fn set_to_zero(x: &mut u32) {
    *x = 0;
}
```

In Rust, `x` is a mutable reference. The function modifies the value that
`x` points to. After the function returns, the caller sees the modified
value.

Now, how do you represent this in a pure functional language? Pure
functional languages have no mutable state. There are no pointers. There
is no heap. A function takes values and returns values, period.

### The Solution: Forward and Backward Functions

Aeneas decomposes a Rust function that takes mutable references into
two (or more) functions:

1. **The forward function**: Computes the return value of the original
   Rust function, treating mutable references as plain values.

2. **The backward function(s)**: Computes the updated values that are
   "sent back" through mutable references when the function returns.
   There is one backward function per borrow region (lifetime) that
   has mutable borrows.

Let us see this concretely.

### Example 1: A Simple Setter

Rust:
```rust
fn set_to_zero(x: &mut u32) {
    *x = 0;
}
```

This function takes a mutable reference, sets it to zero, and returns
nothing (`()`).

Aeneas generates a single function (the forward and backward are merged
when the return type is `()`):

```lean
def set_to_zero (x : U32) : Result U32 :=
  ok 0#u32
```

Wait, what happened to the mutable reference? It became a plain `U32`
value input, and the function returns the new value. The caller will
use the returned value wherever it previously used the mutated reference.

In the pure functional world, instead of "modify the value in place,"
we say "return the new value."

### Example 2: A Function That Both Returns and Mutates

Rust:
```rust
fn replace(x: &mut u32, new_val: u32) -> u32 {
    let old = *x;
    *x = new_val;
    old
}
```

This function returns the old value and sets a new one through the
mutable reference. Aeneas produces:

```lean
def replace (x : U32) (new_val : U32) : Result (U32 x U32) :=
  ok (x, new_val)
```

The return type is a pair `(U32 x U32)`: the first element is the
original return value (the old value), and the second element is the
updated value of `x`.

Historically, Aeneas produced separate forward and backward functions.
In the current version of Aeneas, when practical, these are merged into
a single function that returns a tuple containing both the return value
and the updated mutable borrows.

### Example 3: Indexing Into a Vector

Here is a more interesting example. Consider a function that reads from
a vector using a mutable reference:

```rust
fn get_first(v: &mut Vec<u32>) -> &mut u32 {
    &mut v[0]
}
```

This returns a mutable reference to the first element of the vector.
In the pure functional world, this becomes two operations conceptually:

1. **Forward**: Extract the first element (the return value).
2. **Backward**: Given a (possibly modified) first element, reconstruct
   the vector with that element replaced.

The generated Lean (simplified) would look something like:

```lean
-- Returns the first element and a "backward function" (continuation)
def get_first (v : Vec U32) : Result (U32 x (U32 -> Result (Vec U32))) :=
  do
    let elem <- Vec.index_mut_fwd v 0#usize
    ok (elem, fun new_elem => Vec.index_mut_back v 0#usize new_elem)
```

The backward function is a continuation: "given the new value for the
element, produce the updated vector." This is how Aeneas handles nested
mutable borrows.

### Example 4: A Realistic Function

Let us trace through a more complete example:

```rust
fn swap(x: &mut u32, y: &mut u32) {
    let temp = *x;
    *x = *y;
    *y = temp;
}
```

Aeneas generates (simplified):

```lean
def swap (x : U32) (y : U32) : Result (U32 x U32) :=
  ok (y, x)
```

This is beautifully simple. The "forward" computation returns `()` (which
is optimized away), and the "backward" for `x` is `y`, and the backward
for `y` is `x`. Merged together, the function returns the pair `(y, x)`:
the new values for the two mutable references.

### The Monadic Encoding

You have noticed that every function returns `Result T` rather than just
`T`. This is the **monadic encoding** that Aeneas uses.

Why? Because Rust functions can fail:

- Integer overflow causes a panic.
- Vector indexing out of bounds causes a panic.
- Division by zero causes a panic.

In Rust, these are runtime panics. In the pure functional model, they are
represented as error values.

Here are the relevant types:

```lean
-- The error type
inductive Error where
  | panic
  | outOfFuel
  deriving Repr, BEq

-- The result type (like Rust's Result, but for Aeneas)
def Result (T : Type) := Except Error T

-- Convenience aliases
def ok (x : T) : Result T := Except.ok x
def fail : Result T := Except.error Error.panic
```

Every arithmetic operation returns `Result`:

```lean
-- U32 addition: returns error on overflow
def U32.add (a b : U32) : Result U32 :=
  if a.val + b.val > U32.max then fail
  else ok (U32.mk (a.val + b.val))
```

The `do` notation in Lean (similar to Haskell's `do` notation or Rust's
`?` operator) makes this ergonomic:

```lean
def add_three_numbers (a b c : U32) : Result U32 := do
  let ab <- a + b          -- may fail (overflow)
  let abc <- ab + c        -- may fail (overflow)
  ok abc
```

If any intermediate step fails, the whole computation short-circuits and
returns the error. This is exactly how Rust's `?` operator works.

### The `outOfFuel` Error

You might have noticed `Error.outOfFuel` in the error type. This is used
for handling potentially non-terminating computations.

Lean 4 requires all functions to be total (terminating). But Rust allows
loops that might not terminate. Aeneas handles this by adding a "fuel"
parameter to recursive/looping functions:

```lean
-- A loop that sums values from 0 to n-1
def sum_loop (n : U32) (i : U32) (s : U32) : Result U32 :=
  if i < n then do
    let s1 <- s + i
    let i1 <- i + 1#u32
    sum_loop n i1 s1          -- recursive call (the loop)
  else
    ok s
```

In practice, the `outOfFuel` mechanism is used less frequently now.
Aeneas has become better at producing structurally terminating
translations, and you can also provide termination measures directly.

### Why This Works: The Borrow Checker Connection

The forward/backward decomposition works because of Rust's borrow rules.

In Rust, when you have `&mut T`, you have EXCLUSIVE access. No other
reference to that data exists. This means:

1. Reading through `&mut T` is equivalent to reading a value (forward).
2. Writing through `&mut T` is equivalent to producing a new value
   (backward).
3. There is no aliasing to worry about. No one else can observe the
   mutation "as it happens." They only see the state before or after.

This is the key insight of the Aeneas ICFP 2022 paper: Rust's ownership
discipline means that mutable references can be modeled as a pure
functional "borrow and return" pattern. You borrow a value, do
something with it, and return the (possibly modified) value.

Without the borrow checker's guarantees, this transformation would be
unsound. If two references could alias, you could not model mutation as
"return a new value" because the other reference would need to see the
change too. Rust's type system prevents this.

### Understanding Lifetimes in the Translation

When a function has multiple lifetime parameters, Aeneas may generate
multiple backward functions, one per lifetime with mutable borrows.

```rust
fn pick<'a, 'b>(condition: bool, x: &'a mut u32, y: &'b mut u32)
    -> &'a mut u32
{
    if condition { x } else { y }
}
```

This function borrows from two distinct lifetimes and returns a borrow
tied to `'a`. The translation needs backward functions for both `'a`
and `'b`, because depending on the branch:

- If `condition` is true: `x` may be modified through the returned
  reference, `y` is returned unchanged.
- If `condition` is false: The returned reference actually points into
  `y`, but is tied to `'a`. (Actually, this is a simplification; real
  lifetime handling in Aeneas is more nuanced.)

In practice, Aeneas handles this by threading the right backward
functions through the control flow.

---

## 5. What Rust Features Are Supported

Aeneas is under active development. The supported feature set has grown
significantly since the initial ICFP 2022 paper. Here is a detailed
compatibility matrix as of early 2026.

### Fully Supported

These features work well and are regularly used in verified Aeneas
projects:

| Feature                     | Notes                                        |
|-----------------------------|----------------------------------------------|
| Safe sequential code        | The core use case                            |
| Structs                     | Named fields, tuple structs, unit structs    |
| Enums                       | With data, including generic enums            |
| Pattern matching            | `match`, `if let`, nested patterns           |
| Generics                    | Type parameters, with trait bounds           |
| Traits                      | Declarations, implementations, trait objects |
| Associated types            | In trait definitions and impls               |
| Supertraits                 | `trait Foo: Bar`                             |
| Default trait methods       | With the ability to override                 |
| `impl Trait` in args        | Desugared to generics by rustc               |
| `dyn Trait`                 | Dynamic dispatch, trait objects               |
| Function pointers           | `fn(T) -> U` types                           |
| `Box<T>`                    | Treated as a unique pointer                  |
| `Vec<T>`                    | With index, push, len, etc.                  |
| Arrays `[T; N]`             | Fixed-size arrays                            |
| Slices `&[T]`, `&mut [T]`  | With indexing                                |
| Tuples                      | Up to reasonable sizes                       |
| Integer types               | `u8` through `u128`, `i8` through `i128`, `usize`, `isize` |
| Boolean operations          | All standard boolean ops                     |
| Arithmetic                  | `+`, `-`, `*`, `/`, `%` (with overflow check)|
| Bitwise operations          | `&`, `|`, `^`, `<<`, `>>`                    |
| Comparisons                 | `==`, `!=`, `<`, `>`, `<=`, `>=`             |
| Type aliases                | `type Foo = Bar`                             |
| Constants and statics       | `const` items, `static` (immutable)          |
| Closures (basic)            | Closures that capture by shared reference    |
| Nested functions            | Functions defined inside other functions      |
| `if`/`else`                 | Standard conditionals                        |
| `while` loops               | With `break` and `continue`                  |
| `loop`                      | Infinite loops with `break`                  |
| `for` over ranges           | `for i in 0..n` (desugared to while)         |
| References                  | `&T`, `&mut T`, nested borrows               |
| Reborrows                   | Creating sub-borrows from existing borrows   |
| `let` bindings              | Including destructuring                      |
| Method calls                | `self`, `&self`, `&mut self`                 |
| String literals             | As byte slices                               |
| `panic!`                    | Translated as `Error.panic`                  |
| `assert!`                   | Translated as conditional panic              |

### Not Supported

These features are fundamentally incompatible with Aeneas's approach or
have not been implemented:

| Feature                     | Why                                          |
|-----------------------------|----------------------------------------------|
| `unsafe` code               | The entire approach relies on borrow-checker  |
|                             | guarantees; `unsafe` can violate them         |
| Interior mutability         | `Cell`, `RefCell`, `Mutex`, `RwLock`,        |
| (`Cell`, `RefCell`, etc.)   | `UnsafeCell` all bypass borrow checking      |
| Raw pointers                | `*const T`, `*mut T` bypass borrow checking  |
| Async/await                 | `async fn`, `.await`, `Future` not supported |
| Threads                     | `std::thread::spawn` and related             |
| Channels                    | `std::sync::mpsc` and related                |
| Atomics                     | `std::sync::atomic::*`                       |
| Any concurrency             | Aeneas models sequential execution only      |
| GATs                        | Generic Associated Types not yet supported   |
| Trait aliases               | `trait Foo = Bar + Baz;` not supported       |
| Inline assembly             | `asm!` macro                                 |
| `union` types               | Require unsafe to access                     |
| Dynamic `dyn` upcasting     | Coercing `dyn SubTrait` to `dyn SuperTrait`  |
| Complex `return` in loops   | `return` from within nested loop structures  |
| `break`/`continue` to       | `break 'outer` or `continue 'outer` from     |
|   outer loops               |   within inner loops (Relooper limitation)   |
| External C FFI              | `extern "C"` functions                       |
| Procedural macros           | They expand before Charon sees the code, so  |
|                             | simple ones work; complex ones may not       |

### Partially Supported

These features work in some cases but not all:

| Feature                     | Status                                       |
|-----------------------------|----------------------------------------------|
| Closures (mutable capture)  | Basic cases work; edge cases with complex    |
|                             | mutable captures may fail                    |
| Iterator combinators        | `map`, `filter`, etc. usually do not work    |
|                             | because they involve closures + traits in    |
|                             | complex ways. Use explicit `while` loops.    |
| `String`                    | Limited support; use `Vec<u8>` instead       |
| `HashMap`/`BTreeMap`        | Not directly supported; use `Vec` of pairs   |
|                             | or bring your own verified implementation    |
| `Option` combinators        | `map`, `and_then`, etc. may not work; use    |
|                             | explicit `match` instead                     |
| `Result` combinators        | Same as `Option` -- use explicit `match`     |
| Trait objects with           | Complex lifetime bounds on `dyn` traits may  |
|   complex lifetimes         | cause issues                                 |
| Recursive types             | Supported via `Box`; direct recursion without|
|                             | indirection is rejected by Rust already      |
| Const generics              | Basic `[T; N]` works; complex const generics |
|                             | may not                                      |

### Feature Support Philosophy

The Aeneas team's philosophy is to handle unsupported features
**gracefully**. When Charon encounters a feature it does not support, it
does not crash. Instead, it replaces the unsupported item with an opaque
definition and emits a warning. This means:

- Your project can contain unsupported features as long as the functions
  you want to verify do not use them.
- You can incrementally verify parts of a codebase.
- You get clear error messages about what is not supported rather than
  mysterious crashes.

---

## 6. Writing Aeneas-Friendly Rust

If you want to verify your Rust code with Aeneas, here are practical
guidelines for writing code that translates well.

### Guideline 1: Use `Vec<u8>` Instead of `String`

Rust's `String` type has complex internal machinery (UTF-8 validation,
the `Deref` to `str`, etc.) that Aeneas does not fully model. Instead,
represent text as `Vec<u8>`:

```rust
// AVOID: String has complex internals
fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

// PREFER: Vec<u8> is well-supported
fn greet(name: &Vec<u8>) -> Vec<u8> {
    let mut result = b"Hello, ".to_vec();
    result.extend_from_slice(name.as_slice());
    result.push(b'!');
    result
}
```

If you need string-like operations, implement them as functions over
`Vec<u8>` and verify those functions directly.

### Guideline 2: Use `while` Loops with Explicit Indices

Iterator chains (`map`, `filter`, `fold`, etc.) involve closures and
trait machinery that Aeneas often cannot handle. Use explicit `while`
loops instead:

```rust
// AVOID: iterator chains
fn sum_even(v: &Vec<i32>) -> i32 {
    v.iter().filter(|x| **x % 2 == 0).sum()
}

// PREFER: explicit while loop
fn sum_even(v: &Vec<i32>) -> i32 {
    let mut i: usize = 0;
    let mut sum: i32 = 0;
    while i < v.len() {
        if v[i] % 2 == 0 {
            sum += v[i];
        }
        i += 1;
    }
    sum
}
```

The explicit loop version translates cleanly to a recursive function in
Lean 4, while the iterator version would require modeling the `Iterator`
trait, `Filter` adapter, `Sum` trait, and closures -- none of which
Aeneas handles well.

### Guideline 3: Avoid `unwrap()` and `expect()`

These cause panics, which become `Error.panic` in the translation. While
this is semantically correct, it makes proofs harder because you need to
prove that the `None`/`Err` case never happens.

```rust
// AVOID: unwrap forces you to prove it never panics
fn first_element(v: &Vec<u32>) -> u32 {
    *v.first().unwrap()
}

// PREFER: propagate the possibility of failure
fn first_element(v: &Vec<u32>) -> Option<u32> {
    if v.len() > 0 {
        Some(v[0])
    } else {
        None
    }
}
```

Better yet, restructure your code so the empty case is handled by the
caller, or use a type that encodes non-emptiness.

### Guideline 4: Use Explicit Pattern Matching

Use explicit `match` and `if let` instead of combinator methods:

```rust
// AVOID: Option combinators
fn double_if_some(x: Option<u32>) -> Option<u32> {
    x.map(|v| v * 2)
}

// PREFER: explicit match
fn double_if_some(x: Option<u32>) -> Option<u32> {
    match x {
        Some(v) => Some(v * 2),
        None => None,
    }
}
```

### Guideline 5: Use Enums for Dispatch

When you have a fixed set of behaviors, use an enum with a `match`
instead of dynamic dispatch with `dyn Trait`:

```rust
// WORKS but harder to verify:
fn apply(op: &dyn Operation, x: u32) -> u32 {
    op.apply(x)
}

// PREFER: enum dispatch
enum Operation {
    Double,
    AddOne,
    Square,
}

fn apply(op: &Operation, x: u32) -> u32 {
    match op {
        Operation::Double => x * 2,
        Operation::AddOne => x + 1,
        Operation::Square => x * x,
    }
}
```

Both can work with Aeneas, but the enum version produces simpler Lean
code that is easier to write proofs about.

### Guideline 6: Pre-allocate Vectors When Possible

Dynamic vector growth involves reallocation, which adds complexity. When
you know the size upfront, pre-allocate:

```rust
// FINE: but each push requires proving capacity is not exceeded
fn make_zeros(n: usize) -> Vec<u32> {
    let mut v = Vec::new();
    let mut i = 0;
    while i < n {
        v.push(0);
        i += 1;
    }
    v
}

// SOMETIMES BETTER: if you can use a fixed-size array
fn make_zeros_array() -> [u32; 10] {
    [0; 10]
}
```

### Guideline 7: Consider Functional Data Structures

For verified code, sometimes a linked list is actually preferable to a
`Vec`:

```rust
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}
```

This translates to a clean inductive type in Lean 4:

```lean
inductive List (T : Type) where
  | Nil : List T
  | Cons : T -> List T -> List T
```

Lean has excellent support for reasoning about inductive types, so proofs
about linked lists are often easier than proofs about indexed vectors.

### Guideline 8: Keep Functions Small and Composable

Smaller functions produce smaller Lean translations, which are easier to
reason about. Break complex algorithms into helper functions:

```rust
// Instead of one big function:
fn complex_operation(data: &mut Vec<u32>) {
    // 50 lines of code
}

// Break it into pieces:
fn step1(data: &mut Vec<u32>) { /* ... */ }
fn step2(data: &mut Vec<u32>) { /* ... */ }
fn step3(data: &mut Vec<u32>) { /* ... */ }

fn complex_operation(data: &mut Vec<u32>) {
    step1(data);
    step2(data);
    step3(data);
}
```

Each small function gets its own Lean definition, and you can prove
properties about each step independently.

### Guideline 9: Use Newtypes for Clarity

Wrap primitive types in newtypes to make the Lean translation more
readable:

```rust
struct Temperature(i32);
struct Pressure(u32);

fn is_boiling(t: &Temperature) -> bool {
    t.0 >= 100
}
```

This generates:

```lean
structure Temperature where
  val : I32

def is_boiling (t : Temperature) : Result Bool :=
  ok (t.val >= 100#i32)
```

The Lean code is self-documenting, which helps when writing proofs.

### Guideline 10: Avoid `break` and `continue` to Outer Loops

The Relooper algorithm used by Charon cannot always handle `break` or
`continue` that target outer loops:

```rust
// AVOID: break to outer loop label
'outer: for i in 0..n {
    for j in 0..m {
        if condition(i, j) {
            break 'outer;  // This may not translate correctly
        }
    }
}

// PREFER: use a flag variable
let mut done = false;
let mut i: usize = 0;
while i < n && !done {
    let mut j: usize = 0;
    while j < m && !done {
        if condition(i, j) {
            done = true;
        }
        j += 1;
    }
    i += 1;
}
```

---

## 7. The Proof Workflow

This section walks through the complete workflow from writing Rust code
to completing a verified proof.

### Step 1: Write Your Rust Code

Create a standard Rust project:

```bash
cargo init my-verified-project
cd my-verified-project
```

Write your Rust code. No annotations needed. Let us use a simple example:

```rust
// src/lib.rs

/// Adds two u32 values. We want to prove this is commutative.
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// Computes the absolute difference between two u32 values.
pub fn abs_diff(a: u32, b: u32) -> u32 {
    if a >= b {
        a - b
    } else {
        b - a
    }
}
```

### Step 2: Add Charon Configuration

Create a `Charon.toml` file in your project root:

```toml
[charon]
# Use the aeneas preset for standard settings
```

Alternatively, you can pass flags directly on the command line.

### Step 3: Run Charon to Extract LLBC

```bash
charon cargo --preset=aeneas
```

This does the following:
1. Invokes Cargo, but with Charon replacing `rustc`.
2. Compiles your project, extracting MIR.
3. Produces ULLBC, restructures to LLBC.
4. Writes a `.llbc` file (JSON format).

The output file will be at something like:
```
my-verified-project.llbc
```

If Charon encounters unsupported features, it will print warnings but
continue. The unsupported items become opaque in the output.

### Step 4: Run Aeneas to Generate Lean 4

```bash
aeneas -backend lean my-verified-project.llbc
```

This produces a directory of Lean 4 files. The structure typically looks
like:

```
lean/MyVerifiedProject/
    Types.lean          -- Type definitions
    Funs.lean           -- Function definitions
    Clauses/            -- Termination clauses (if needed)
```

For our example, the generated `Funs.lean` would contain something like:

```lean
import Aeneas
import MyVerifiedProject.Types
open Aeneas Primitives

namespace my_verified_project

/-- Adds two u32 values. -/
def add (a : U32) (b : U32) : Result U32 :=
  a + b

/-- Computes the absolute difference. -/
def abs_diff (a : U32) (b : U32) : Result U32 :=
  if a >= b then
    a - b
  else
    b - a

end my_verified_project
```

Note the key transformations:

- `u32` became `U32` (Aeneas's bounded integer type in Lean).
- Both functions return `Result U32` because arithmetic can overflow
  or underflow.
- `a + b` in Lean is actually `U32.add a b`, which returns `Result U32`.
- The code is pure functional. No mutable state anywhere.

### Step 5: Set Up the Lean 4 Project

Create a Lean 4 project that depends on the Aeneas Lean library:

```lean
-- lakefile.lean
import Lake
open Lake DSL

require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "main" / "backends/lean"

package myVerifiedProject where
  leanOptions := #[
    -- Add any needed options
  ]

@[default_target]
lean_lib MyVerifiedProject
```

The Aeneas Lean library provides:

- The `U8`, `U16`, `U32`, `U64`, `U128`, `Usize` types (and signed
  variants).
- The `Result` and `Error` types.
- The `Vec` type and its operations.
- Arithmetic operations with overflow checking.
- The `step` tactic and supporting infrastructure.
- Various utility lemmas.

### Step 6: Write Your Proofs

Now create a proof file, for example `MyVerifiedProject/Proofs.lean`:

```lean
import Aeneas
import MyVerifiedProject.Funs
open Aeneas Primitives
open my_verified_project

-- Prove that add is commutative (when it does not overflow).
-- "add a b = ok result" means the addition succeeded (no overflow).
-- We prove that if both add(a,b) and add(b,a) succeed, they give
-- the same result.
theorem add_comm (a b : U32)
    (h1 : add a b = ok result1)
    (h2 : add b a = ok result2) :
    result1 = result2 := by
  -- Unfold the definition of add
  simp [add] at *
  -- Now h1 and h2 are about U32.add
  -- Use the fact that mathematical addition is commutative
  omega

-- Prove that abs_diff is zero iff the inputs are equal.
theorem abs_diff_zero (a b : U32)
    (h : abs_diff a b = ok result)
    (heq : result = 0#u32) :
    a = b := by
  simp [abs_diff] at h
  split at h <;> simp_all <;> omega

-- Prove that abs_diff a a = 0 (reflexivity).
theorem abs_diff_self (a : U32)
    (h : abs_diff a a = ok result) :
    result = 0#u32 := by
  simp [abs_diff] at h
  omega
```

### Step 7: Check the Proofs

```bash
lake build
```

If all proofs are correct, the build succeeds silently. If a proof is
wrong, Lean gives you an error message indicating which goal could not
be closed.

### A Complete Worked Example: Binary Search

Let us work through a more substantial example. Here is a binary search
implementation in Rust:

```rust
// A binary search that returns the index of the target, or None.
pub fn binary_search(v: &Vec<u32>, target: u32) -> Option<usize> {
    let mut lo: usize = 0;
    let mut hi: usize = v.len();

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let mid_val = v[mid];

        if mid_val == target {
            return Some(mid);
        } else if mid_val < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    None
}
```

After running Charon and Aeneas, the generated Lean code would look
approximately like this (simplified for clarity):

```lean
-- The loop is extracted as a recursive function
def binary_search_loop
    (v : Vec U32) (target : U32) (lo : Usize) (hi : Usize)
    : Result (Option Usize) :=
  if lo < hi then do
    let mid_offset <- hi - lo
    let half <- mid_offset / 2#usize
    let mid <- lo + half
    let mid_val <- Vec.index v mid
    if mid_val = target then
      ok (some mid)
    else if mid_val < target then do
      let lo1 <- mid + 1#usize
      binary_search_loop v target lo1 hi
    else
      binary_search_loop v target lo mid
  else
    ok none

def binary_search (v : Vec U32) (target : U32) : Result (Option Usize) :=
  binary_search_loop v target 0#usize (Vec.len v)
```

Now you could prove properties like:

1. If `binary_search` returns `Some(i)`, then `v[i] == target`.
2. If `binary_search` returns `None` and `v` is sorted, then `target`
   is not in `v`.

These proofs would require lemmas about the loop invariant (that `lo`
and `hi` bracket the search range), but the key point is that all the
reasoning is about pure functions and values. There are no pointers,
no heap, no memory model to deal with.

### The Proof Strategy

When writing proofs about Aeneas-generated code, the general strategy is:

1. **Unfold definitions**: Use `simp [function_name]` to expand the
   generated function definitions.

2. **Step through monadic operations**: Use the `step` tactic (described
   in the next section) to decompose `do`-notation bindings.

3. **Case split**: Use `split` to handle `if`/`then`/`else` and `match`
   expressions.

4. **Reason about arithmetic**: Use `omega` for linear arithmetic goals
   about bounded integers.

5. **Apply induction**: For loops (which become recursive functions),
   use induction on the "fuel" or a decreasing measure.

6. **Use the Aeneas library**: The Aeneas Lean library provides lemmas
   about `Vec.index`, `Vec.len`, arithmetic operations, etc. These are
   essential building blocks for proofs.

---

## 8. The `step`/`progress` Tactic

### What It Does

The `step` tactic (previously called `progress`, and still sometimes
referred to by that name) is the workhorse tactic for proofs about
Aeneas-generated code.

When your proof goal contains a monadic bind like:

```lean
-- Goal:
-- ⊢ (do let x <- someOperation a b; restOfCode x) = ok result
```

The `step` tactic:

1. Identifies the first monadic operation (`someOperation a b`).
2. Looks for a theorem about `someOperation` (annotated with `@[step]`).
3. Applies that theorem to decompose the operation.
4. Introduces the result and any preconditions.

### Basic Usage

```lean
-- Given this goal:
-- ⊢ (do let x <- U32.add a b; ok x) = ok result

-- Use step to decompose the addition:
step

-- After step, you get:
-- x : U32
-- h : a.val + b.val <= U32.max
-- ⊢ ok x = ok result
```

### Naming Results

You can name the variables and hypotheses introduced by `step`:

```lean
-- Name the result and its hypothesis
step as ⟨sum, h_sum⟩

-- Now you have:
-- sum : U32
-- h_sum : sum.val = a.val + b.val
```

The angle brackets `⟨...⟩` are Lean's anonymous constructor syntax,
used here for pattern matching the result.

### Repeated Stepping

Use `step*` to automatically apply `step` repeatedly until no more
monadic operations can be decomposed:

```lean
theorem my_proof (a b c : U32)
    (h : add_three a b c = ok result) : ... := by
  simp [add_three] at h
  step*
  -- All intermediate monadic operations have been decomposed
  -- You now have hypotheses about each intermediate result
  sorry
```

This is convenient but can make proof scripts harder to maintain, since
the number of steps might change if the generated code changes. For
important proofs, consider naming each step explicitly.

### The `@[step]` Annotation

The `step` tactic works by looking for theorems tagged with `@[step]`.
The Aeneas Lean library provides `@[step]` theorems for all primitive
operations:

```lean
@[step]
theorem U32.add_spec (a b : U32) (h : a.val + b.val <= U32.max) :
    U32.add a b = ok (U32.mk (a.val + b.val)) := by
  ...

@[step]
theorem Vec.index_spec {T : Type} (v : Vec T) (i : Usize)
    (h : i.val < v.val.length) :
    Vec.index v i = ok (v.val[i.val]) := by
  ...
```

You can also annotate your own theorems with `@[step]` to make them
available to the tactic:

```lean
-- Prove a spec for your function
@[step]
theorem my_function_spec (x : U32) (h : x.val < 100) :
    my_function x = ok (U32.mk (x.val * 2)) := by
  simp [my_function]
  omega

-- Now step can automatically decompose calls to my_function
theorem uses_my_function (x : U32) (h : x.val < 50) :
    combined_function x = ok result := by
  simp [combined_function]
  step   -- This will find and apply my_function_spec
  ...
```

### Worked Example with `step`

Here is a detailed example showing how `step` works in practice.
Consider this Rust code:

```rust
pub fn add_and_double(a: u32, b: u32) -> u32 {
    let sum = a + b;
    sum + sum
}
```

Generated Lean:

```lean
def add_and_double (a : U32) (b : U32) : Result U32 := do
  let sum <- a + b
  sum + sum
```

Proof that the result equals `2 * (a + b)` (when no overflow occurs):

```lean
theorem add_and_double_spec (a b : U32)
    (h_no_overflow1 : a.val + b.val <= U32.max)
    (h_no_overflow2 : 2 * (a.val + b.val) <= U32.max)
    (h : add_and_double a b = ok result) :
    result.val = 2 * (a.val + b.val) := by
  -- Unfold the definition
  simp [add_and_double] at h
  -- Step through the first bind: let sum <- a + b
  step as ⟨sum, h_sum⟩ at h
  -- Now we know sum.val = a.val + b.val
  -- Step through the second operation: sum + sum
  step as ⟨doubled, h_doubled⟩ at h
  -- Now we know doubled.val = sum.val + sum.val
  -- The goal reduces to arithmetic
  simp_all
  omega
```

### Evolution from `progress` to `step`

The tactic was originally called `progress` in earlier versions of
Aeneas. The name was changed to `step` to be more intuitive -- it
"steps" through one monadic operation at a time.

If you see `progress` in older documentation or tutorials, know that it
is functionally identical to `step`. Some older codebases may still use
`progress` for backward compatibility.

### Tips for Using `step`

1. **Always `simp [function_name]` first**: The `step` tactic operates on
   the unfolded definition. If the function call has not been unfolded,
   `step` will not find the monadic bind to decompose.

2. **Name everything in important proofs**: `step as ⟨name, h_name⟩`
   makes proofs more readable and more robust to changes.

3. **Use `step*` for exploration**: When you are developing a proof and
   want to see what the state looks like after all operations are
   decomposed, `step*` is useful. Replace with explicit steps once the
   proof is stable.

4. **Check the @[step] database**: If `step` fails, it might be because
   there is no `@[step]` theorem for the operation you are trying to
   decompose. You may need to prove one and register it.

5. **Preconditions matter**: `@[step]` theorems often have preconditions
   (like `i.val < v.val.length` for vector indexing). If `step` gets
   stuck, you might need to prove a precondition first.

---

## 9. Charon: The MIR Extraction Framework

### Overview

Charon is a framework for extracting and analyzing Rust programs. While
it was developed primarily to support Aeneas, it is designed as a
general-purpose Rust analysis framework that other tools can also use.

- **Repository**: https://github.com/AeneasVerif/charon
- **Stars**: ~309 (as of early 2026)
- **Language**: Primarily Rust (with some OCaml for parts of the pipeline)
- **Paper**: "Charon: An Analysis Framework for Rust" (CAV 2025)
- **Maturity**: The authors describe it as "alpha software"

### How Charon Integrates with Cargo

Charon works by replacing `rustc` in the Cargo compilation process. Here
is the mechanism:

1. Cargo normally invokes `rustc` to compile each crate. Cargo respects
   the `RUSTC_WRAPPER` environment variable, which lets you specify a
   program that wraps `rustc`.

2. When you run `charon cargo`, Charon sets itself up as the compiler
   wrapper. Cargo invokes Charon instead of `rustc` directly.

3. Charon then invokes the real `rustc` as a library (linking against
   the rustc driver), hooking into the compilation pipeline after MIR
   generation.

4. Charon extracts the MIR, processes it into ULLBC and then LLBC, and
   writes the result to a `.llbc` file.

5. Dependencies of your crate are compiled normally (Charon only extracts
   the target crate, not all dependencies, unless configured otherwise).

The command is:

```bash
charon cargo --preset=aeneas
```

The `--preset=aeneas` flag configures Charon with settings optimized for
Aeneas translation. Without it, you can configure individual options via
command-line flags or `Charon.toml`.

### What Charon Extracts

Charon extracts a comprehensive representation of your Rust program:

**Types:**
- Struct definitions (including generic parameters)
- Enum definitions (with all variants and their fields)
- Type aliases
- Trait-associated types

**Functions:**
- Function signatures (parameters, return types, generic parameters)
- Function bodies (as LLBC: structured control flow)
- Generic functions (with type parameter constraints)
- Methods (desugared to functions with an explicit `self` parameter)

**Traits:**
- Trait declarations (methods, associated types, supertraits, default
  methods)
- Trait implementations (which types implement which traits, with the
  method bodies)

**Globals and Constants:**
- `const` items
- `static` items (immutable only)

**Metadata:**
- Source code locations (file, line, column) for all items
- Dependency relationships between items
- Crate structure (modules, visibility)

### The .llbc JSON Format

The `.llbc` file is a JSON document with a well-defined schema. Here is
a sketch of its structure:

```json
{
  "charon_version": "0.x.y",
  "translated": {
    "type_decls": [
      {
        "def_id": 0,
        "item_meta": {
          "name": ["my_crate", "MyStruct"],
          "span": { "file": "src/lib.rs", "beg": [5, 0], "end": [8, 1] }
        },
        "kind": {
          "Struct": [
            { "name": "field1", "ty": { "Integer": "U32" } },
            { "name": "field2", "ty": { "Bool": null } }
          ]
        },
        "generics": { ... }
      }
    ],
    "fun_decls": [
      {
        "def_id": 0,
        "item_meta": {
          "name": ["my_crate", "my_function"],
          "span": { ... }
        },
        "signature": {
          "inputs": [ ... ],
          "output": { ... },
          "generics": { ... }
        },
        "body": {
          "locals": [ ... ],
          "body": { ... }
        }
      }
    ],
    "global_decls": [ ... ],
    "trait_decls": [ ... ],
    "trait_impls": [ ... ]
  }
}
```

You rarely need to look at this file directly, but it is useful for
debugging when Aeneas produces unexpected output. You can pipe it through
`python -m json.tool` or `jq` for readable formatting.

### Graceful Handling of Unsupported Features

One of Charon's most important design decisions is graceful degradation.
When Charon encounters a Rust feature it does not support:

1. It emits a **warning** identifying the unsupported feature and its
   location in the source code.

2. It replaces the unsupported item with an **opaque declaration** --
   a type or function signature without a body.

3. It continues processing the rest of the crate.

This means:

- A crate with some unsupported features does not fail entirely. Only
  the specific items using unsupported features become opaque.

- You can verify the parts of your code that are supported, even if other
  parts use features like async or unsafe.

- The error messages tell you exactly what is not supported and where,
  helping you decide whether to refactor or accept the opacity.

Example output:

```
warning: Unsupported feature: async function
  --> src/network.rs:42:1
   |
42 | pub async fn fetch_data(url: &str) -> Result<Vec<u8>, Error> {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: This function will be extracted as an opaque declaration

warning: Unsupported feature: unsafe block
  --> src/ffi.rs:15:5
   |
15 |     unsafe { libc::memcpy(...) }
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: This function will be extracted as an opaque declaration
```

### Charon Configuration

Charon can be configured via `Charon.toml` in your project root:

```toml
[charon]
# Extract specific crates (by default, extracts the top-level crate)
# extract_crates = ["my_crate", "my_dependency"]

# Opacity settings: make specific items opaque
# opaque_modules = ["my_crate::internal"]
```

Or via command-line flags:

```bash
# Basic extraction with aeneas preset
charon cargo --preset=aeneas

# Specify output file
charon cargo --preset=aeneas -o my_output.llbc

# Increase verbosity for debugging
charon cargo --preset=aeneas --print-llbc
```

### Charon as a General Framework

While Aeneas is the primary consumer of Charon's output, the framework
is designed to be useful for other purposes:

- **Static analysis tools** can use Charon to get a clean representation
  of Rust programs for analysis.
- **Other verification tools** can build on Charon instead of writing
  their own MIR extraction.
- **Program transformation tools** can consume and produce LLBC.

The Charon repository includes Rust crates (`charon-lib`) that other
tools can depend on to read and manipulate the LLBC format
programmatically.

---

## 10. Comparison with Other Rust Verification Tools

Aeneas is not the only tool for verifying Rust programs. Here is how it
compares with the other major options.

### The Landscape

| Feature              | Aeneas           | Prusti           | Creusot          | Verus            | Kani             |
|----------------------|------------------|------------------|------------------|------------------|------------------|
| **Approach**         | Functional       | Viper-based      | Why3-based       | SMT-based        | Model checking   |
|                      | translation      | verification     | deductive        | verification     | (CBMC)           |
|                      |                  |                  | verification     |                  |                  |
| **Backend**          | Lean 4, Coq,    | Viper (Silicon/  | Why3             | Z3, built into   | CBMC (C Bounded  |
|                      | F*, HOL4        | Carbon)          |                  | Rust             | Model Checker)   |
| **Proof style**      | Extrinsic       | Annotations in   | Annotations in   | Annotations in   | Harnesses in     |
|                      | (separate Lean)  | Rust code        | Rust code        | Rust code        | Rust code        |
| **Annotation burden**| None in Rust,   | Medium to high   | Medium           | Medium to high   | Low              |
|                      | proofs in Lean   | (pre/post/inv)   | (pre/post/inv)   | (pre/post/inv)   | (just harnesses) |
| **Unsafe support**   | No               | Limited          | No               | Limited          | Yes              |
| **Loop handling**    | Automatic        | Requires loop    | Requires loop    | Requires loop    | Bounded          |
|                      | (recursion)      | invariants       | invariants       | invariants       | unrolling        |
| **Automation**       | Low (manual      | High (SMT)       | Medium           | High (SMT)       | High (automatic) |
|                      | proofs)          |                  |                  |                  |                  |
| **Concurrency**      | No               | No               | No               | No               | Limited          |
| **Maturity**         | Research         | Research/        | Research         | Research/        | Production-ish   |
|                      |                  | maturing         |                  | maturing         | (Amazon)         |
| **Primary org**      | Inria/Microsoft  | ETH Zurich       | Univ. Paris-     | Microsoft/       | Amazon           |
|                      |                  |                  | Saclay           | CMU/VMware       |                  |

### Detailed Comparison

#### Aeneas vs. Prusti

**Prusti** uses pre/postconditions and loop invariants written as Rust
attributes (`#[requires(...)]`, `#[ensures(...)]`, `#[invariant(...)]`).
It leverages the Viper verification infrastructure and SMT solvers.

- **Prusti advantage**: Higher automation. Many properties can be verified
  automatically by the SMT solver without manual proofs.
- **Prusti advantage**: Annotations stay in the Rust source, keeping
  code and specs together.
- **Aeneas advantage**: No loop invariants. Loops become recursion, and
  you reason about them by induction instead of finding invariants.
  Finding correct loop invariants is one of the hardest parts of
  verification.
- **Aeneas advantage**: Full proof-assistant power. You can prove anything
  expressible in Lean 4, not just what an SMT solver can handle.
- **Aeneas advantage**: Multiple backends (Lean, Coq, F*, HOL4).

#### Aeneas vs. Creusot

**Creusot** is the most similar tool to Aeneas in philosophy. It also
targets a proof assistant (Why3) and handles safe Rust. Created by
Xavier Denis at Universite Paris-Saclay.

- **Creusot advantage**: Tighter integration with Rust (annotations in
  Rust source using Pearlite specification language).
- **Creusot advantage**: More mature support for closures and some
  higher-order patterns.
- **Aeneas advantage**: The functional translation approach produces
  cleaner, more idiomatic proof-assistant code.
- **Aeneas advantage**: Lean 4 (used by Aeneas) has a more active
  community and better tooling than Why3 (used by Creusot).
- **Aeneas advantage**: Extrinsic proofs mean your Rust code stays
  completely clean.

#### Aeneas vs. Verus

**Verus** builds verification directly into a Rust-like language. You
write specs using Rust-like syntax, and Z3 (an SMT solver) checks them.

- **Verus advantage**: Very high automation for properties that Z3 can
  handle.
- **Verus advantage**: Tight integration -- specs and code in the same
  language.
- **Verus advantage**: Can handle some unsafe code.
- **Aeneas advantage**: When SMT solvers time out or fail, you have no
  recourse in Verus. In Aeneas, you can always construct the proof
  manually.
- **Aeneas advantage**: Aeneas generates standard Lean 4 code, so you
  can use the entire Lean 4 ecosystem (Mathlib, etc.).

#### Aeneas vs. Kani

**Kani** is a model checker from Amazon. It uses bounded model checking
(CBMC) to verify Rust properties.

- **Kani advantage**: Very easy to use -- just write test-like harnesses.
- **Kani advantage**: Handles unsafe code.
- **Kani advantage**: High automation.
- **Kani disadvantage**: Bounded. It checks up to some bound on inputs,
  loops, etc. It does not prove properties for ALL inputs.
- **Aeneas advantage**: Full mathematical proofs, not bounded checking.

### When to Use Aeneas

Choose Aeneas when:

- You need **absolute guarantees** (mathematical proof, not just
  bounded checking).
- Your code is **safe sequential Rust**.
- You are comfortable working in a **proof assistant** (or willing to
  learn).
- You want **extrinsic proofs** (clean Rust code, proofs separate).
- You want to target **multiple proof assistants**.
- You hate writing **loop invariants** (Aeneas handles loops via
  recursion and induction).
- Your problem domain involves **complex mathematical reasoning** that
  SMT solvers struggle with (e.g., cryptographic proofs).

Choose something else when:

- You need to verify **unsafe** code (use Kani or Verus).
- You want **maximum automation** and your properties are SMT-friendly
  (use Prusti or Verus).
- You want a **quick check** that is not a full proof (use Kani).
- You are verifying **concurrent** code (none of these tools handle
  it well; this is an open research problem).

---

## 11. Real-World Usage

### Case Study: Microsoft SymCrypt

**SymCrypt** is Microsoft's core cryptographic library, used across
Windows, Azure, and other Microsoft products. It was originally written
in C.

Microsoft has been working on porting SymCrypt to Rust and verifying
the Rust implementation using the Aeneas/Charon/Eurydice toolchain.
The project involves:

- Translating cryptographic algorithms from C to safe Rust.
- Using Aeneas to generate Lean 4 models.
- Proving correctness properties (that the implementation matches the
  cryptographic specification).
- Using **Eurydice** (a related tool by the same team) to compile
  verified low-level code back to C for deployment where C is needed.

This is one of the most ambitious verification efforts using Aeneas.
Cryptographic code is a particularly good fit because:

- It is sequential (no concurrency).
- Correctness is critical (a bug can compromise security).
- The mathematical specifications are well-defined.
- The code is mostly pure computation on arrays and integers.

### Case Study: ML-KEM (Kyber)

**ML-KEM** (previously known as Kyber) is a post-quantum key
encapsulation mechanism -- one of the algorithms selected by NIST for
standardization as a replacement for classical key exchange algorithms
that would be broken by quantum computers.

The Aeneas team (in collaboration with others) has been involved in
verifying ML-KEM implementations. The approach:

1. Write the ML-KEM algorithm in a subset of Rust.
2. Use Aeneas to translate to Lean 4/F*.
3. Prove that the implementation matches the mathematical specification
   from the NIST standard.
4. Use Eurydice to compile the verified code to C for deployment.

This end-to-end pipeline (Rust -> verification -> C deployment) is a
flagship example of how the Aeneas ecosystem can produce verified,
high-performance code. The verified implementation has been used in
real-world deployments (including in Mozilla Firefox's NSS library and
other contexts).

### Case Study: ICFP Tutorial

At ICFP 2024 (the International Conference on Functional Programming),
the Aeneas team led a hands-on tutorial. This is available as an
open-source repository and is an excellent way to learn Aeneas:

**Repository**: https://github.com/AeneasVerif/icfp-tutorial

The tutorial walks you through:

1. Setting up the Aeneas toolchain.
2. Translating simple Rust programs.
3. Writing your first proofs in Lean 4.
4. Progressively more complex examples.
5. Tips and tricks for real-world verification.

If you are reading this document and want to actually try Aeneas, the
ICFP tutorial is the best starting point after finishing this guide.

### Other Notable Uses

- **Teaching**: Several universities use Aeneas in graduate courses on
  formal verification and programming languages.

- **Research**: The Aeneas project has spawned research into borrow
  checking semantics, Rust formalization, and verified compilation.
  Son Ho's PhD thesis (2024) provides a thorough theoretical foundation.

- **Industrial exploration**: Beyond Microsoft, other companies have
  explored Aeneas for verifying safety-critical Rust code. The exact
  details are often not public, but the Zulip chat occasionally has
  discussions from industrial users.

---

## 12. Installation

There are two main ways to install Aeneas: using Nix (recommended for
most users) or building from source manually.

### Method 1: Nix (Easiest)

Nix is a package manager that provides reproducible builds. If you do
not have Nix, install it first:

```bash
# Install Nix (multi-user installation)
sh <(curl -L https://nixos.org/nix/install) --daemon
```

Once Nix is installed, you can run Aeneas directly without installing
it permanently:

```bash
# Run Aeneas directly from GitHub
nix run github:aeneasverif/aeneas -L -- -backend lean my_file.llbc
```

The `-L` flag shows build logs (useful the first time, when Nix is
downloading and building dependencies). The `--` separates nix flags
from aeneas flags.

For Charon:

```bash
# Run Charon via Nix
nix run github:aeneasverif/charon -L -- cargo --preset=aeneas
```

If you want to install them permanently into your Nix profile:

```bash
nix profile install github:aeneasverif/aeneas
nix profile install github:aeneasverif/charon
```

**Advantages of the Nix approach:**
- Reproducible: everyone gets the same versions.
- No dependency management: Nix handles OCaml, Rust, etc.
- Easy updates: just point to a newer commit.

**Disadvantages:**
- Nix can be slow on first build (it compiles everything).
- Nix has a learning curve if you are not already using it.
- Disk space usage can be high.

### Method 2: Manual Installation (From Source)

This method requires you to install dependencies yourself.

#### Prerequisites

1. **Rust (via rustup)**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   Charon requires a specific nightly Rust toolchain. The Charon
   repository's `rust-toolchain.toml` file specifies exactly which
   version. Rustup will install it automatically.

2. **OCaml 5 (via OPAM)**:
   ```bash
   # Install OPAM (OCaml package manager)
   # On macOS:
   brew install opam
   # On Ubuntu/Debian:
   sudo apt-get install opam

   # Initialize OPAM
   opam init

   # Create a switch with OCaml 5
   opam switch create 5.1.1
   eval $(opam env)
   ```

3. **Lean 4 (via elan)** (for running generated proofs):
   ```bash
   curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh
   ```

#### Building Aeneas

```bash
# Clone the repository
git clone https://github.com/AeneasVerif/aeneas.git
cd aeneas

# Set up Charon (Aeneas includes Charon as a submodule)
make setup-charon

# Build everything
make

# The aeneas binary will be in the build output
# Add it to your PATH or use it directly
```

#### Building Charon Separately

If you want to build Charon independently:

```bash
git clone https://github.com/AeneasVerif/charon.git
cd charon

# Build Charon
cargo build --release

# The charon binary will be in target/release/charon
```

### Verifying Your Installation

Test that everything works:

```bash
# Create a test file
mkdir -p /tmp/aeneas-test/src
cat > /tmp/aeneas-test/Cargo.toml << 'EOF'
[package]
name = "aeneas-test"
version = "0.1.0"
edition = "2021"
EOF

cat > /tmp/aeneas-test/src/lib.rs << 'EOF'
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
EOF

# Run Charon
cd /tmp/aeneas-test
charon cargo --preset=aeneas

# Run Aeneas
aeneas -backend lean aeneas-test.llbc

# Check the output
ls lean/AeneasTest/
# Should see: Types.lean  Funs.lean (or similar)

cat lean/AeneasTest/Funs.lean
# Should see the Lean translation of the add function
```

If you see the generated Lean file with a `def add` function, your
installation is working correctly.

### IDE Setup

For writing Lean 4 proofs, you will want VS Code with the Lean 4
extension:

1. Install VS Code.
2. Install the "lean4" extension from the VS Code marketplace.
3. The extension provides:
   - Syntax highlighting
   - Type information on hover
   - Interactive proof state (see your goals as you write tactics)
   - Error highlighting
   - Go to definition

The interactive proof state is particularly important for Aeneas work.
As you write `step` and `simp` tactics, you can see exactly what the
current goal looks like, what hypotheses you have, and what remains to
be proved.

---

## 13. Research Papers

### Primary Papers

#### "Aeneas: Rust Verification by Functional Translation" (ICFP 2022)

- **Authors**: Son Ho, Jonathan Protzenko
- **Venue**: Proceedings of the ACM on Programming Languages (PACMPL),
  Volume 6, Issue ICFP, August 2022
- **DOI**: https://doi.org/10.1145/3547647

This is the foundational paper. It introduces:
- The forward/backward function decomposition for handling mutable borrows.
- The LLBC intermediate language.
- The translation from LLBC to pure functional code.
- Soundness arguments for the translation.
- Case studies demonstrating the approach.

If you read one paper, read this one. It is well-written and accessible
to anyone with basic PL background.

#### "Sound Borrow-Checking for Rust via Symbolic Semantics" (ICFP 2024)

- **Authors**: Son Ho, Jonathan Protzenko
- **Venue**: Proceedings of the ACM on Programming Languages (PACMPL),
  Volume 8, Issue ICFP, August 2024

This paper addresses a fundamental question: is the Aeneas translation
sound? That is, do the generated pure functions actually model the
behavior of the original Rust program?

The paper introduces a **symbolic semantics** for Rust's borrow system
and uses it to prove that the forward/backward decomposition is correct.
This gives a formal foundation to the translation, beyond the informal
arguments in the original paper.

Key contributions:
- A symbolic operational semantics for a core Rust-like language.
- A formal proof that the functional translation is sound with respect
  to this semantics.
- Extension to handle reborrowing (creating a new borrow from an
  existing one), which is common in real Rust code.

#### "Charon: An Analysis Framework for Rust" (CAV 2025)

- **Authors**: Son Ho, and collaborators
- **Venue**: Computer Aided Verification (CAV), 2025

This paper presents Charon as a standalone contribution. It describes:
- The architecture of Charon and how it hooks into rustc.
- The ULLBC and LLBC intermediate languages in detail.
- The Relooper algorithm for restructuring control flow.
- The design decisions that make Charon reusable beyond Aeneas.
- Evaluation on real-world Rust crates.

### Thesis

#### Son Ho's PhD Thesis (2024)

- **Title**: (Formal title varies; the thesis covers the theoretical
  foundations of Aeneas)
- **Institution**: Inria Paris
- **Year**: 2024

The thesis provides the most comprehensive treatment of the theory behind
Aeneas. It expands on the ICFP papers with:
- More detailed formalization of the borrow calculus.
- Extended proofs of soundness.
- Discussion of design alternatives considered and rejected.
- A thorough treatment of the relationship between Rust's type system
  and the functional translation.

If you want deep theoretical understanding, the thesis is the place to
look.

### Related Papers

The Aeneas project builds on and relates to work from several other
research efforts:

- **Project Everest** (Microsoft Research): Verified HTTPS, using F*.
  Jonathan Protzenko was a key contributor.

- **Electrolysis** (Sebastian Ullrich): An earlier attempt at Rust-to-
  Lean translation. Aeneas builds on lessons learned from this project.

- **RustBelt** (Ralf Jung et al.): Formal foundations for Rust's type
  system using Iris in Coq. Takes a different approach (foundational
  semantics rather than functional translation).

- **Oxide** (Aaron Weiss et al.): A formal model of Rust's ownership
  and borrowing. Related formal foundations.

---

## 14. Resources

### Official Repositories

| Resource             | URL                                              |
|----------------------|--------------------------------------------------|
| Aeneas (main repo)  | https://github.com/AeneasVerif/aeneas            |
| Charon               | https://github.com/AeneasVerif/charon            |
| Documentation        | https://aeneasverif.github.io/                   |
| ICFP Tutorial        | https://github.com/AeneasVerif/icfp-tutorial     |
| Zulip Chat           | https://aeneas-verif.zulipchat.com/              |

### Getting Help

The **Zulip chat** at https://aeneas-verif.zulipchat.com/ is the primary
place to ask questions. The developers (including Son Ho) are active
there and responsive to questions. There are channels for:

- General discussion
- Installation help
- Technical questions about the translation
- Feature requests and bug reports
- Announcements

### Learning Path

If you are starting from scratch, here is a recommended learning path:

1. **This document**: Read it fully to understand the concepts.

2. **Install the tools**: Follow the installation instructions above.
   The Nix method is quickest.

3. **ICFP Tutorial**: Work through the exercises at
   https://github.com/AeneasVerif/icfp-tutorial

4. **Lean 4 basics**: If you are not familiar with Lean 4, work through
   some introductory material:
   - "Theorem Proving in Lean 4" (official documentation)
   - "Mathematics in Lean" (interactive textbook)
   - "Functional Programming in Lean" (David Thrane Christiansen)

5. **Read the ICFP 2022 paper**: Now that you have hands-on experience,
   the paper will make much more sense.

6. **Try your own project**: Pick a small Rust function you have written
   and try to verify it.

7. **Join Zulip**: Ask questions as they come up.

### Documentation Site

The official documentation at https://aeneasverif.github.io/ contains:

- Installation guides.
- Tutorials and walkthroughs.
- API documentation for the Lean 4 library.
- Descriptions of the supported Rust features.
- Changelog and release notes.

### Video Resources

The Aeneas team has given talks at various conferences and workshops:

- **ICFP 2022 talk**: Presentation of the original Aeneas paper.
- **ICFP 2024 talk**: Presentation of the symbolic semantics paper.
- **Various workshop talks**: At Rust Verification Workshop, Lean
  Together, and similar events.

Search for "Aeneas Rust verification" on YouTube to find these. The ICFP
talks are particularly good introductions.

### Related Tools in the Ecosystem

The Aeneas verification ecosystem includes several related tools:

- **Eurydice**: A compiler from a subset of Rust (via Charon/LLBC) to C.
  This enables writing verified Rust code and deploying it as C in
  environments where a Rust compiler is not available. Developed by
  Jonathan Protzenko.

- **hax**: A verification framework for Rust developed at Cryspen, which
  also uses Charon for MIR extraction. hax targets F* and Coq.

- **Aeneas Lean library**: The Lean 4 library that provides types,
  operations, and tactics for working with Aeneas-generated code. This
  is part of the main Aeneas repository under `backends/lean`.

---

## Appendix A: Glossary

| Term                     | Definition                                      |
|--------------------------|-------------------------------------------------|
| **Aeneas**               | The OCaml tool that translates LLBC to pure      |
|                          | functional code in Lean 4 (or Coq, F*, HOL4).   |
| **Backward function**    | The generated function that computes the updated |
|                          | values of mutable references after a function    |
|                          | call.                                            |
| **Borrow checker**       | The part of the Rust compiler that enforces       |
|                          | ownership and borrowing rules.                   |
| **CFG**                  | Control-Flow Graph. A representation of a program|
|                          | as a graph of basic blocks connected by edges.   |
| **Charon**               | The tool that extracts MIR from rustc and         |
|                          | produces ULLBC/LLBC.                             |
| **Eurydice**             | A compiler from Rust (via LLBC) to C.            |
| **Extrinsic proof**      | A proof written separately from the code it is   |
|                          | about, as opposed to inline annotations.         |
| **Forward function**     | The generated function that computes the return   |
|                          | value of a Rust function.                        |
| **Fuel**                 | A natural number parameter used to ensure         |
|                          | termination of potentially-infinite loops in      |
|                          | the translation.                                 |
| **LLBC**                 | (Structured) Low-Level Borrow Calculus. The       |
|                          | structured AST output of Charon.                 |
| **Lean 4**               | A proof assistant and programming language         |
|                          | developed at Microsoft Research.                 |
| **MIR**                  | Mid-level Intermediate Representation. An         |
|                          | internal representation used by the Rust          |
|                          | compiler.                                        |
| **Monadic encoding**     | The technique of wrapping return values in        |
|                          | `Result` to handle potential failures (overflow,  |
|                          | panics).                                         |
| **Proof assistant**      | A tool that checks mathematical proofs written    |
|                          | in a formal language. Also called an interactive  |
|                          | theorem prover.                                  |
| **Relooper**             | An algorithm that converts a CFG into structured  |
|                          | control flow (if/else, loops).                   |
| **Result**               | The type `Result T = Except Error T` used in the  |
|                          | Aeneas Lean library.                             |
| **step tactic**          | A Lean tactic that decomposes monadic operations  |
|                          | in Aeneas-generated code.                        |
| **ULLBC**                | Unstructured Low-Level Borrow Calculus. The CFG   |
|                          | representation before restructuring.             |
| **Verification**         | Mathematical proof that a program satisfies a     |
|                          | specification for ALL possible inputs.           |

---

## Appendix B: Common Lean 4 Types in Aeneas Output

When you look at Aeneas-generated Lean code, you will encounter these
types frequently. Here is a quick reference:

### Integer Types

```lean
-- Unsigned integers
U8    -- 0 to 255
U16   -- 0 to 65535
U32   -- 0 to 4294967295
U64   -- 0 to 18446744073709551615
U128  -- 0 to 2^128 - 1
Usize -- platform-dependent (modeled as U32 or U64)

-- Signed integers
I8    -- -128 to 127
I16   -- -32768 to 32767
I32   -- -2147483648 to 2147483647
I64   -- -9223372036854775808 to 9223372036854775807
I128  -- -2^127 to 2^127 - 1
Isize -- platform-dependent
```

### Container Types

```lean
-- Vector (models Rust's Vec<T>)
Vec T    -- a list with a bounded length

-- Array (models Rust's [T; N])
Array T N  -- a fixed-size array

-- Slice (models Rust's &[T] / &mut [T])
Slice T    -- a view into a contiguous sequence

-- Option (models Rust's Option<T>)
Option T   -- Lean's built-in Option type (some / none)
```

### The Result Monad

```lean
-- Error type
inductive Error where
  | panic
  | outOfFuel

-- Result type (all Aeneas functions return this)
def Result (T : Type) := Except Error T

-- Success constructor
ok : T -> Result T

-- Failure constructor
fail : Result T    -- shorthand for Except.error Error.panic
```

### Integer Literals

In Aeneas-generated Lean code, integer literals use a suffix notation:

```lean
0#u32     -- U32 literal with value 0
42#u64    -- U64 literal with value 42
1#usize   -- Usize literal with value 1
-1#i32    -- I32 literal with value -1
```

---

## Appendix C: Frequently Asked Questions

### Q: Do I need to annotate my Rust code?

**A**: No. Aeneas works with unmodified Rust code. You write standard
Rust, run it through Charon and Aeneas, and get Lean code. All proof
work happens in Lean, not in Rust.

This is what "extrinsic verification" means: the proofs are external to
(extrinsic to) the code they are about.

### Q: Is the translation proven correct?

**A**: The ICFP 2024 paper ("Sound Borrow-Checking for Rust via Symbolic
Semantics") provides a formal proof that the forward/backward
decomposition is sound for a core calculus. This is strong evidence but
not a machine-checked end-to-end proof of the full tool chain. The
translation of the full Rust feature set supported by Aeneas relies on
additional arguments that extend the core result.

In practice, the translation is well-tested and has been used for
significant verification efforts (like SymCrypt), which gives additional
empirical confidence.

### Q: Can I verify `unsafe` code?

**A**: No. Aeneas fundamentally relies on the borrow checker's
guarantees. Unsafe code can violate these guarantees, so the translation
would be unsound. If you need to verify unsafe Rust, look at tools like
Kani or RustBelt.

### Q: How do I handle standard library functions?

**A**: The Aeneas Lean library provides models (axiomatized specifications)
for commonly used standard library functions like `Vec::push`,
`Vec::len`, `Vec::index`, etc. For standard library functions that are
not modeled, you can:

1. Provide your own axiomatized specification in Lean.
2. Reimplement the function in your own code so Aeneas can translate it.
3. Request support on the Zulip chat.

### Q: How long do proofs take to write?

**A**: This varies enormously depending on:

- The complexity of the function.
- The complexity of the property you are proving.
- Your experience with Lean 4.

A simple property of a simple function (like "add is commutative") might
take 5 minutes. A complex property of a complex algorithm (like "this
sorting function produces a sorted permutation of the input") might take
days or weeks.

As a rough rule of thumb for someone experienced with the tools: expect
to spend 5-10x as long on proofs as you spent writing the Rust code.
For beginners, expect 20-50x until you build proficiency.

### Q: Can I use Mathlib (Lean's math library)?

**A**: Yes. Since Aeneas generates standard Lean 4 code, you can import
and use Mathlib in your proofs. This is useful when your verification
involves mathematical concepts that Mathlib already formalizes (number
theory, linear algebra, etc.).

### Q: What about performance? Does Aeneas affect runtime performance?

**A**: Aeneas has zero effect on runtime performance. Your Rust code is
compiled by `rustc` as normal. Aeneas only generates a Lean model for
proving properties. The Lean code is never executed in production; it
exists only for verification purposes.

### Q: Can I incrementally verify a large codebase?

**A**: Yes. Charon's graceful degradation means you can extract and verify
individual functions or modules even if other parts of the codebase use
unsupported features. You can incrementally verify the most critical
functions first and expand coverage over time.

### Q: What about termination? Lean requires all functions to terminate.

**A**: Aeneas handles this in several ways:

1. For loops with obvious termination (like iterating over a range), the
   generated recursive function has a structurally decreasing argument
   or a clear termination measure.

2. For loops where termination is not obvious, Aeneas may use a "fuel"
   parameter (a natural number that decreases on each iteration). You
   then need to prove that sufficient fuel exists for your use case.

3. You can also provide custom termination measures via Lean's
   `termination_by` annotation on the generated functions.

### Q: Is Aeneas production-ready?

**A**: Aeneas is best described as a research tool that is being used in
increasingly serious projects. It has been used for real verification
work (SymCrypt, ML-KEM) by experienced teams. However, the tool is still
evolving, the feature set is expanding, and you should expect some rough
edges, especially with complex Rust code.

For critical verification projects with dedicated verification engineers,
Aeneas is a viable choice. For casual use by developers who are not
willing to invest significant time in learning Lean 4 and debugging tool
issues, it may be premature.

---

## Appendix D: Cheat Sheet

### Quick Command Reference

```bash
# Extract Rust to LLBC
charon cargo --preset=aeneas

# Translate LLBC to Lean 4
aeneas -backend lean my_crate.llbc

# Translate LLBC to Coq
aeneas -backend coq my_crate.llbc

# Translate LLBC to F*
aeneas -backend fstar my_crate.llbc

# Translate LLBC to HOL4
aeneas -backend hol4 my_crate.llbc

# Build Lean project (check proofs)
lake build

# Run via Nix (no installation needed)
nix run github:aeneasverif/charon -L -- cargo --preset=aeneas
nix run github:aeneasverif/aeneas -L -- -backend lean my_crate.llbc
```

### Quick Lean Proof Template

```lean
import Aeneas
import MyCrate.Funs
open Aeneas Primitives
open my_crate

-- Specification theorem for my_function
@[step]
theorem my_function_spec
    (input : U32)
    (h_pre : input.val < 1000)  -- precondition
    (h_call : my_function input = ok result)  -- the call succeeds
    :
    result.val = input.val * 2  -- the postcondition
    := by
  simp [my_function] at h_call
  step*
  omega
```

### Common Tactic Patterns

```lean
-- Unfold a definition
simp [function_name]

-- Step through one monadic operation
step
step as ⟨x, hx⟩

-- Step through all monadic operations
step*

-- Case split on if/then/else or match
split

-- Linear arithmetic on bounded integers
omega

-- Simplify with all hypotheses
simp_all

-- Rewrite using a hypothesis
rw [hypothesis_name]

-- Prove by contradiction
contradiction

-- Introduce universally quantified variables
intro x y z
```

---

*This document was written as a comprehensive reference for developers
approaching Aeneas for the first time. For the most up-to-date
information, always check the official repositories and documentation
at https://aeneasverif.github.io/ and the Zulip chat at
https://aeneas-verif.zulipchat.com/.*

---

[← Back to README](README.md) | [← Lean Reference](LEAN.md) | [Start Tutorial 01 →](tutorials/01-setup-hello-proof/README.md)
