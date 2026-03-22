[← Back to README](README.md) | [Aeneas Reference →](AENEAS.md)

# Lean 4: A Comprehensive Reference

A guide for programmers who know basic algorithms but are new to theorem proving.

---

## Table of Contents

1. [What is Lean 4](#1-what-is-lean-4)
2. [Core Theory: The Calculus of Inductive Constructions](#2-core-theory-the-calculus-of-inductive-constructions)
3. [Lean as a Programming Language](#3-lean-as-a-programming-language)
4. [Lean's Type System for Verification](#4-leans-type-system-for-verification)
5. [Proof Writing in Lean 4](#5-proof-writing-in-lean-4)
6. [Essential Tactics Reference](#6-essential-tactics-reference)
7. [Practical Formal Verification](#7-practical-formal-verification)
8. [Mathlib](#8-mathlib)
9. [Tooling](#9-tooling)
10. [Installation and First Project](#10-installation-and-first-project)
11. [Resources](#11-resources)

---

## 1. What is Lean 4

### 1.1 Overview

Lean 4 is a programming language and a theorem prover in one package. You can
write ordinary programs in it — web servers, compilers, command-line tools — and
you can also write mathematical proofs in it. The same language, the same
compiler, the same file. This dual nature is what makes Lean 4 distinctive.

When you write a function in Lean, the compiler checks that your code is
type-correct. When you write a proof, the compiler checks that your reasoning
is logically valid. Both activities are, under the hood, the same operation:
type checking. This is not a coincidence. It is the central insight of Lean's
design, rooted in a deep connection between logic and computation called the
Curry-Howard correspondence (covered in Section 2).

### 1.2 History

**Leonardo de Moura** started the Lean project at **Microsoft Research** in
**2013**. De Moura is a Brazilian computer scientist who had previously worked
on the Z3 SMT solver, one of the most widely used automated reasoning tools in
the world. His goal with Lean was to build a proof assistant that was
simultaneously:

- Trustworthy (small, auditable kernel)
- Expressive (powerful type theory)
- Practical (usable as a real programming language)
- Extensible (user-defined tactics and automation)

The project went through several major versions:

| Version | Year | Notes |
|---------|------|-------|
| Lean 1 | 2014 | Initial prototype, exploratory |
| Lean 2 | 2015 | HoTT (Homotopy Type Theory) support, research tool |
| Lean 3 | 2017 | First widely used version, Mathlib started here |
| Lean 4 | 2021 | Complete rewrite, self-hosted, general-purpose language |

Each version was essentially a rewrite. Lean 2 experimented with Homotopy
Type Theory but moved away from it. Lean 3 gained a significant user base,
particularly among mathematicians, and the Mathlib library (the largest
single coherent body of formalized mathematics in any proof assistant) was
built on it. However, Lean 3 was implemented in C++ and had limitations in
terms of performance and extensibility.

**Lean 4 was a ground-up rewrite.** It is **self-hosted** — the Lean 4
compiler is written in Lean 4 itself. The compiler translates Lean code to C,
which is then compiled to a native binary by a standard C compiler. This means
Lean 4 programs run at native speed, not interpreted.

### 1.3 Lean FRO

In **2023**, the **Lean Focused Research Organization (Lean FRO)** was
established, with **Sebastian Ullrich** (the primary architect of Lean 4's
compiler) and Leonardo de Moura as the leaders. The FRO is a nonprofit
dedicated to the long-term development and sustainability of Lean. It is funded
by the Simons Foundation, Amazon, and others. This gave Lean a stable
institutional home independent of Microsoft Research.

### 1.4 Current Status (Early 2026)

As of early 2026:

- Lean is at **version 4.28+** (versions are released frequently; check
  `elan` for the latest stable toolchain).
- Lean won the **ACM SIGPLAN Programming Languages Software Award in 2025**,
  recognizing it as a significant contribution to programming language
  technology. Past recipients of this award include GCC, LLVM, Coq, and the
  Java HotSpot compiler — placing Lean in distinguished company.
- The Mathlib library has grown to over **1.9 million lines of code** with more
  than **500 contributors**.
- Lean has been used for industrial verification at **Amazon (Cedar policy
  language)**, **Microsoft (SymCrypt cryptographic library, via Aeneas)**, and
  other organizations.
- In **2025**, Lean-based AI systems earned **gold medals at the International
  Mathematical Olympiad**, demonstrating the power of combining Lean with
  machine learning.

### 1.5 Why Lean 4 Matters

There are other proof assistants: Coq, Agda, Isabelle/HOL, HOL4, Mizar. What
sets Lean 4 apart:

1. **It is a real programming language.** You can write performant, compiled
   programs, not just proofs. The `do` notation, monads, IO, and FFI make it
   feel like a modern functional language.

2. **Extensible by design.** The tactic framework, macro system, and
   elaboration system are all written in Lean itself and exposed to users. You
   can define new notations, new tactics, and new language extensions as
   ordinary Lean code.

3. **Self-hosted.** The compiler, the elaborator, the tactic framework — they
   are all Lean code. You can read and modify them.

4. **Active community.** The Zulip chat is one of the friendliest and most
   helpful proof assistant communities. Mathlib has a well-organized
   contribution process.

5. **Strong tooling.** The VS Code extension provides real-time feedback: as
   you type a proof, you see the current proof state update live. This
   interactive experience is crucial for learning and for productive work.

---

## 2. Core Theory: The Calculus of Inductive Constructions

This section covers the theoretical foundations. If you are eager to write code,
you can skip to Section 3 and return here later. However, understanding these
foundations — even at a high level — will make everything else in Lean click
into place.

### 2.1 Types and Terms

In most programming languages you are used to, values have types:

```
42       : Int
"hello"  : String
true     : Bool
[1,2,3]  : List Int
```

Lean is the same, but it goes further. In Lean, **types themselves have types**:

```lean
#check Nat        -- Nat : Type
#check Bool       -- Bool : Type
#check String     -- String : Type
```

So `42` is a value of type `Nat`, and `Nat` is a value of type `Type`. But
what is the type of `Type`?

```lean
#check Type       -- Type : Type 1
#check Type 1     -- Type 1 : Type 2
#check Type 2     -- Type 2 : Type 3
```

This is the **universe hierarchy**. It goes on forever. This might seem strange,
but it is necessary to avoid logical paradoxes (similar to Russell's paradox in
set theory — "the set of all sets" leads to contradictions).

### 2.2 The Universe Hierarchy

Lean organizes its types into an infinite hierarchy of "universes":

```
Sort 0 = Prop          -- the type of propositions
Sort 1 = Type 0 = Type -- the type of ordinary data types
Sort 2 = Type 1        -- the type of Type
Sort 3 = Type 2        -- the type of Type 1
...
```

Let us look at each level.

#### Level 0: `Prop` (the universe of propositions)

`Prop` is the type of propositions — things that can be true or false in a
logical sense. When you write a statement like `2 + 2 = 4` in Lean, its type
is `Prop`:

```lean
#check 2 + 2 = 4         -- 2 + 2 = 4 : Prop
#check ∀ n : Nat, n = n  -- ∀ (n : Nat), n = n : Prop
```

A **proof** of a proposition `P : Prop` is a term of type `P`. So if you can
construct a value of type `2 + 2 = 4`, you have proved that `2 + 2 = 4`.
More on this below.

#### Level 1: `Type` (the universe of ordinary types)

`Type` (which is shorthand for `Type 0`) is where your everyday data types
live:

```lean
#check Nat      -- Nat : Type
#check String   -- String : Type
#check Bool     -- Bool : Type
#check List Nat -- List Nat : Type
```

#### Higher Levels

`Type 1` is the type of `Type`, `Type 2` is the type of `Type 1`, and so on.
In practice, you rarely need to think about anything above `Type`. The
hierarchy exists for foundational soundness.

Lean also supports **universe polymorphism**, which lets you write definitions
that work at any universe level:

```lean
-- This definition works for types at any universe level
def id' {α : Sort u} (a : α) : α := a
```

The `u` is a universe variable. You will see this in Mathlib but rarely need
to write it yourself as a beginner.

### 2.3 `Prop` is Special: Proof Irrelevance and Impredicativity

`Prop` has two special properties that distinguish it from `Type`:

#### Proof Irrelevance

If `P : Prop` and you have two proofs `h1 : P` and `h2 : P`, then `h1 = h2`.
All proofs of the same proposition are considered equal. The *content* of a
proof does not matter — only *that* a proof exists.

This is analogous to how, in everyday math, nobody cares *which* proof of the
Pythagorean theorem you use — they are all equivalent as justifications.

Concretely, this means:

```lean
-- These two proofs of True are definitionally equal
example : (trivial : True) = (trivial : True) := rfl

-- In general, for any P : Prop and h1 h2 : P, we have h1 = h2.
-- This is an axiom in Lean's type theory.
```

Why does this matter? It means proofs carry no computational content. The
compiler can erase them entirely. In a verified program, the proofs vanish at
compile time — they are only checked during type checking.

#### Impredicativity

`Prop` is **impredicative**: a universal quantification over all propositions
is itself a proposition. That is, if you quantify over `Prop`, you stay in
`Prop`:

```lean
-- This universally quantifies over ALL propositions P, and it is itself a Prop
#check ∀ P : Prop, P → P   -- ∀ (P : Prop), P → P : Prop
```

By contrast, `Type` is **predicative**: quantifying over `Type` bumps you up
a universe level:

```lean
#check ∀ α : Type, α → α   -- ∀ (α : Type), α → α : Type 1
```

This distinction is important for the logical foundations, but as a beginner,
the key takeaway is: `Prop` is the world of logic, `Type` is the world of
data, and they interact through the Curry-Howard correspondence.

### 2.4 Inductive Types

Almost every concrete type in Lean is defined as an **inductive type**. An
inductive type is defined by listing its **constructors** — the ways to build
values of that type.

Here are the most fundamental examples:

#### Natural Numbers

```lean
inductive Nat where
  | zero : Nat
  | succ : Nat → Nat
```

This says: a natural number is either `zero`, or it is `succ n` for some
natural number `n` (the successor of `n`, i.e., `n + 1`). So:

- `0` is `Nat.zero`
- `1` is `Nat.succ Nat.zero`
- `2` is `Nat.succ (Nat.succ Nat.zero)`
- and so on

This is Peano arithmetic encoded as a data type.

#### Booleans

```lean
inductive Bool where
  | false : Bool
  | true : Bool
```

Two constructors, no arguments. A Boolean is either `true` or `false`.

#### Lists

```lean
inductive List (α : Type) where
  | nil : List α
  | cons : α → List α → List α
```

A list is either empty (`nil`) or it is an element followed by another list
(`cons head tail`). This is the classic linked list.

#### Option

```lean
inductive Option (α : Type) where
  | none : Option α
  | some : α → Option α
```

An optional value is either nothing (`none`) or something (`some value`).

#### Propositions as Inductive Types

Even logical connectives are inductive types!

```lean
-- Logical AND
inductive And (P Q : Prop) : Prop where
  | intro : P → Q → And P Q

-- Logical OR
inductive Or (P Q : Prop) : Prop where
  | inl : P → Or P Q
  | inr : Q → Or P Q

-- Logical TRUE
inductive True : Prop where
  | intro : True

-- Logical FALSE (no constructors — you cannot build a proof of False!)
inductive False : Prop where
```

`False` has **no constructors**. This means there is no way to create a term of
type `False`. This is exactly right: `False` is the proposition that has no
proof.

#### The Principle of Induction

When you define an inductive type, Lean automatically generates a **recursor**
(elimination principle). For `Nat`, this is essentially the principle of
mathematical induction:

> To prove a property holds for all natural numbers, prove it for zero (base
> case) and prove that if it holds for `n` then it holds for `n + 1` (inductive
> step).

This is why they are called "inductive" types.

### 2.5 The Curry-Howard Correspondence

This is the single most important idea for understanding Lean. It is the bridge
between programming and proving.

The **Curry-Howard correspondence** (also called the "proofs-as-programs"
interpretation) is the observation that:

| Logic | Programming |
|-------|-------------|
| Proposition | Type |
| Proof | Program (term) |
| Implication P → Q | Function type P → Q |
| Conjunction P ∧ Q | Pair type P × Q |
| Disjunction P ∨ Q | Sum type P ⊕ Q |
| Universal ∀ x, P x | Dependent function (x : α) → P x |
| Existential ∃ x, P x | Dependent pair (x : α) × P x |
| True | Unit type (has one value) |
| False | Empty type (has no values) |

Let us walk through several examples to make this concrete.

#### Example 1: Implication is a function

The proposition "if it is raining, then the ground is wet" has the logical form
`P → Q`. In Lean, a proof of `P → Q` is literally a function that takes a
proof of `P` and returns a proof of `Q`:

```lean
-- The proposition: for any propositions P and Q,
-- if P implies Q and P is true, then Q is true.
-- (This is called "modus ponens.")
theorem modus_ponens (P Q : Prop) (hpq : P → Q) (hp : P) : Q :=
  hpq hp  -- apply the function hpq to the argument hp
```

The proof is just function application! `hpq` is a function from `P` to `Q`,
`hp` is a value of type `P`, so `hpq hp` is a value of type `Q`. Done.

#### Example 2: Conjunction is a pair

To prove `P ∧ Q`, you must provide a proof of `P` and a proof of `Q` — a pair:

```lean
theorem and_intro (P Q : Prop) (hp : P) (hq : Q) : P ∧ Q :=
  ⟨hp, hq⟩  -- angle brackets build a pair (an anonymous constructor)
```

To use a proof of `P ∧ Q`, you extract the two components:

```lean
theorem and_left (P Q : Prop) (hpq : P ∧ Q) : P :=
  hpq.1  -- first component of the pair
```

#### Example 3: Disjunction is a sum

To prove `P ∨ Q`, you must provide either a proof of `P` or a proof of `Q`:

```lean
theorem or_inl (P Q : Prop) (hp : P) : P ∨ Q :=
  Or.inl hp  -- inject on the left

theorem or_inr (P Q : Prop) (hq : Q) : P ∨ Q :=
  Or.inr hq  -- inject on the right
```

#### Example 4: Negation is a function to False

`¬P` is defined as `P → False`. A proof of `¬P` is a function that takes any
proof of `P` and produces a proof of `False` — which is impossible to produce
directly. So the existence of such a function means `P` cannot be proved (in
the constructive reading).

```lean
-- ¬P is definitionally equal to P → False
-- Here is a proof that False implies anything
theorem false_elim' (P : Prop) (h : False) : P :=
  h.elim  -- False has no constructors, so we can "match" on it with zero cases
```

#### Example 5: Universal quantification is a dependent function

"For all natural numbers n, n equals n" has type `∀ n : Nat, n = n`. A proof
is a function that takes any `n` and returns a proof that `n = n`:

```lean
theorem every_nat_eq_itself : ∀ n : Nat, n = n :=
  fun n => rfl  -- for any n, reflexivity gives us n = n
```

Here `rfl` is the proof of `a = a` for any `a` — it is the constructor of the
equality type.

#### Why This Matters

The Curry-Howard correspondence means:

1. **Writing a proof is writing a program.** If you can program, you can prove
   theorems — you just need to learn the idioms.

2. **Type checking is proof checking.** When Lean type-checks your code, it is
   simultaneously verifying your proofs. There is no separate proof checker.

3. **The compiler is the verifier.** If your Lean file compiles without errors,
   your proofs are correct (assuming Lean's kernel is correct, which is small
   and well-audited).

### 2.6 Axioms

Lean's core type theory is constructive, but Lean adds a few axioms by default:

1. **Propositional extensionality** (`propext`): Two propositions that are
   logically equivalent are equal. That is, if `P ↔ Q` then `P = Q`.

2. **Quotient types**: The ability to form quotient types (types modulo an
   equivalence relation).

3. **Classical choice** (`Classical.choice`): Given that a type is nonempty,
   you can produce an element of it. This is nonconstructive — it asserts
   existence without giving a witness. Combined with `propext`, this gives you
   the full **law of excluded middle**: for any proposition `P`, either `P` or
   `¬P`.

These axioms are imported via `Classical` in Lean. Mathlib uses classical logic
throughout. If you are doing pure programming (not proving theorems), axioms
are irrelevant because they are erased at compile time.

### 2.7 Summary of Core Theory

```
                    Sort 3  (= Type 2)
                      |
                    Sort 2  (= Type 1)
                      |
                    Sort 1  (= Type 0 = Type)
                      |
                    Sort 0  (= Prop)

  Every value has a type.
  Every type lives in some universe.
  Types are first-class: they can be arguments and return values.
  Propositions are types. Proofs are values.
  Inductive types define both data structures and logical connectives.
```

---

## 3. Lean as a Programming Language

Lean 4 is a full-featured, general-purpose, compiled functional programming
language. If you know Haskell, OCaml, F#, or Rust (to some extent), you will
find many familiar concepts. If you only know imperative languages like Python
or Java, this section introduces the functional style.

### 3.1 Basic Definitions

#### Defining Values

```lean
def greeting : String := "Hello, world!"

def ultimateAnswer : Nat := 42

-- Type annotations are usually optional; Lean can infer them
def pi := 3.14159
```

#### Defining Functions

```lean
def double (n : Nat) : Nat := n * 2

def add (a b : Nat) : Nat := a + b

-- Multi-argument functions are curried by default
#check add      -- add : Nat → Nat → Nat
#check add 3    -- add 3 : Nat → Nat  (partial application)
```

#### Anonymous Functions (Lambdas)

```lean
def double' : Nat → Nat := fun n => n * 2

-- Shorter notation with the `·` placeholder
def double'' : Nat → Nat := (· * 2)

-- Multi-argument lambda
def add' : Nat → Nat → Nat := fun a b => a + b
```

### 3.2 `let`, `where`, and Local Definitions

```lean
def circleArea (radius : Float) : Float :=
  let pi := 3.14159
  let rSquared := radius * radius
  pi * rSquared

-- `where` lets you define helpers after the main expression
def circleArea' (radius : Float) : Float :=
  pi * radius * radius
where
  pi := 3.14159
```

### 3.3 Basic Data Types

Lean has the types you would expect:

```lean
-- Natural numbers (arbitrary precision)
#check (42 : Nat)

-- Integers (arbitrary precision)
#check (-7 : Int)

-- Unsigned fixed-width integers
#check (255 : UInt8)
#check (1000 : UInt32)
#check (0 : UInt64)

-- Floating point
#check (3.14 : Float)

-- Characters and strings
#check ('a' : Char)
#check ("hello" : String)

-- Booleans
#check (true : Bool)
#check (false : Bool)

-- Unit (a type with one value)
#check (() : Unit)
```

### 3.4 Conditional Expressions

```lean
def abs (n : Int) : Int :=
  if n < 0 then -n else n

-- `if` is an expression, not a statement — it returns a value
def max (a b : Nat) : Nat :=
  if a >= b then a else b
```

### 3.5 Pattern Matching

Pattern matching is one of the most important features of functional languages.
It lets you inspect the structure of a value and act accordingly.

```lean
def isZero (n : Nat) : Bool :=
  match n with
  | 0 => true
  | _ => false

-- Matching on multiple patterns
def and' (a b : Bool) : Bool :=
  match a, b with
  | true, true => true
  | _, _ => false

-- Matching on lists
def head? (xs : List α) : Option α :=
  match xs with
  | [] => none
  | x :: _ => some x

-- Nested patterns
def sumFirstTwo (xs : List Nat) : Nat :=
  match xs with
  | a :: b :: _ => a + b
  | [a] => a
  | [] => 0
```

### 3.6 Recursion

Lean requires all functions to be total (they must terminate on all inputs).
The compiler checks this by verifying that recursive calls are made on
structurally smaller arguments.

```lean
def factorial : Nat → Nat
  | 0 => 1
  | n + 1 => (n + 1) * factorial n

def length : List α → Nat
  | [] => 0
  | _ :: tail => 1 + length tail

def map (f : α → β) : List α → List β
  | [] => []
  | x :: xs => f x :: map f xs

def filter (p : α → Bool) : List α → List α
  | [] => []
  | x :: xs => if p x then x :: filter p xs else filter p xs
```

#### Structural Recursion

By default, Lean expects the recursive argument to get structurally smaller
in each call. For `Nat`, this means the argument decreases by at least one.
For `List`, it means the list gets shorter.

```lean
-- This works: n decreases on each recursive call
def fib : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fib (n + 1) + fib n
```

#### Well-Founded Recursion

When structural recursion is not enough, you can use `termination_by` to
specify a measure that decreases:

```lean
def gcd (a b : Nat) : Nat :=
  if b == 0 then a
  else gcd b (a % b)
termination_by b
decreasing_by
  simp_all
  omega
```

The `termination_by` clause tells Lean what quantity is decreasing, and
`decreasing_by` provides a proof that it actually decreases. Often Lean
can figure this out automatically.

#### Partial Functions

If you truly need a function that might not terminate (e.g., for prototyping),
you can use `partial`:

```lean
partial def loop : Unit → Unit := fun _ => loop ()
```

`partial` functions cannot be used in proofs — they are computationally
opaque to the kernel.

### 3.7 Structures

Structures are product types (records) with named fields:

```lean
structure Point where
  x : Float
  y : Float
deriving Repr

-- Creating a value
def origin : Point := { x := 0.0, y := 0.0 }
def p : Point := { x := 1.0, y := 2.0 }

-- Accessing fields
#eval p.x  -- 1.0
#eval p.y  -- 2.0

-- Updating fields (creates a new value; structures are immutable)
def p' : Point := { p with y := 3.0 }
-- p' = { x := 1.0, y := 3.0 }
```

Structures can have default values, computed fields, and can extend other
structures:

```lean
structure Point3D extends Point where
  z : Float
deriving Repr

def q : Point3D := { x := 1.0, y := 2.0, z := 3.0 }
#eval q.x  -- 1.0 (inherited from Point)
#eval q.z  -- 3.0
```

### 3.8 Enumerations and Sum Types

```lean
-- Simple enumeration
inductive Color where
  | red
  | green
  | blue
deriving Repr

-- Sum type with data
inductive Shape where
  | circle (radius : Float)
  | rectangle (width height : Float)
  | triangle (base height : Float)
deriving Repr

def area : Shape → Float
  | .circle r => 3.14159 * r * r
  | .rectangle w h => w * h
  | .triangle b h => 0.5 * b * h
```

### 3.9 Typeclasses

Typeclasses in Lean work similarly to typeclasses in Haskell or traits in Rust.
They define interfaces that types can implement.

```lean
-- Defining a typeclass
class Printable (α : Type) where
  toString : α → String

-- Implementing it for specific types
instance : Printable Bool where
  toString b := if b then "true" else "false"

instance : Printable Nat where
  toString n := s!"{n}"

-- Using the typeclass
def print [Printable α] (value : α) : String :=
  Printable.toString value

#eval print true    -- "true"
#eval print 42      -- "42"
```

The square brackets `[Printable α]` denote an **instance-implicit** argument:
Lean automatically searches for a `Printable` instance for the given type.

Lean has many built-in typeclasses:

```lean
-- Some commonly used typeclasses
#check @Add         -- defines `+`
#check @Mul         -- defines `*`
#check @BEq         -- defines `==` (Boolean equality)
#check @Ord         -- defines `compare`
#check @ToString    -- defines conversion to String
#check @Repr        -- defines representation for #eval output
#check @Inhabited   -- defines a default value
#check @Hashable    -- defines hashing
```

### 3.10 Monads and `do` Notation

If you have used Haskell or Rust's `Result`/`Option` chaining, this will be
familiar. If not, here is the key idea: a **monad** is a design pattern for
sequencing computations that have some kind of effect (I/O, failure, state,
etc.).

#### The `Option` Monad (Handling Failure)

```lean
def safeDivide (a b : Nat) : Option Nat :=
  if b == 0 then none else some (a / b)

-- Without do notation: manual chaining
def computation1 : Option Nat :=
  match safeDivide 10 2 with
  | none => none
  | some x =>
    match safeDivide x 3 with
    | none => none
    | some y => some (x + y)

-- With do notation: much cleaner
def computation2 : Option Nat := do
  let x ← safeDivide 10 2
  let y ← safeDivide x 3
  return x + y
```

The `←` operator extracts the value from the monad (or short-circuits on
failure). `return` wraps a value back into the monad.

#### The `IO` Monad (Input/Output)

All interaction with the outside world (reading files, printing, network
access) happens in the `IO` monad:

```lean
def main : IO Unit := do
  IO.println "What is your name?"
  let name ← IO.getStdin >>= (·.getLine)
  IO.println s!"Hello, {name.trim}!"
```

A Lean program's entry point is `def main : IO Unit`. The `IO` monad ensures
that side effects are tracked in the type system.

#### More Monadic Operations

```lean
-- for loops in do notation
def printNumbers : IO Unit := do
  for i in [1, 2, 3, 4, 5] do
    IO.println s!"Number: {i}"

-- mutable variables (local to the do block, using StateT under the hood)
def sumList (xs : List Nat) : Nat := Id.run do
  let mut total := 0
  for x in xs do
    total := total + x
  return total

-- try/catch for error handling
def safeParse (s : String) : IO Nat := do
  let some n := s.toNat?
    | throw (IO.Error.userError s!"Cannot parse '{s}' as Nat")
  return n
```

The `Id.run do` pattern lets you use `do` notation with mutable variables in
pure (non-IO) code. Under the hood, Lean transforms this into functional code,
but syntactically it looks imperative.

### 3.11 Arrays and Efficient Data Structures

Lean provides `Array α`, an efficient dynamically-sized array. Unlike `List`,
which is a linked list with O(n) indexing, `Array` uses contiguous memory with
O(1) indexing and amortized O(1) append.

```lean
def xs : Array Nat := #[1, 2, 3, 4, 5]

-- Indexing (returns the element; panics on out of bounds in unchecked version)
#eval xs[0]!   -- 1

-- Safe indexing
#eval xs[0]?   -- some 1
#eval xs[10]?  -- none

-- Functional operations
#eval xs.map (· * 2)      -- #[2, 4, 6, 8, 10]
#eval xs.filter (· > 3)   -- #[4, 5]
#eval xs.foldl (· + ·) 0  -- 15

-- Building arrays
#eval Array.range 5            -- #[0, 1, 2, 3, 4]
#eval #[1, 2] ++ #[3, 4]      -- #[1, 2, 3, 4]
#eval #[1, 2, 3].push 4       -- #[1, 2, 3, 4]
```

Lean uses **reference counting** for memory management (not garbage collection).
When an array has a reference count of one (i.e., it is not shared), mutations
are performed in place. This means functional code that "copies" an array often
runs as fast as imperative code that mutates it.

### 3.12 String Interpolation and Formatting

```lean
def name := "world"
def n := 42

-- String interpolation with s!
#eval s!"Hello, {name}!"           -- "Hello, world!"
#eval s!"The answer is {n + 1}"    -- "The answer is 43"

-- Multi-line strings
def poem := "Roses are red,
Violets are blue,
Lean is great,
And so are you."
```

### 3.13 Namespaces and Imports

```lean
-- Defining a namespace
namespace MyMath

def square (n : Nat) : Nat := n * n
def cube (n : Nat) : Nat := n * n * n

end MyMath

-- Using qualified names
#eval MyMath.square 5  -- 25

-- Opening a namespace
open MyMath in
#eval square 5  -- 25

-- Opening for the rest of the file
open MyMath

-- Importing other files
import Mathlib.Tactic  -- imports tactics from Mathlib
```

### 3.14 Error Handling Patterns

```lean
-- Option: for computations that might not produce a value
def find? (xs : List Nat) (pred : Nat → Bool) : Option Nat :=
  xs.find? pred

-- Except: for computations that might fail with an error
def parseAge (s : String) : Except String Nat :=
  match s.toNat? with
  | some n => if n < 150 then .ok n else .error "Age too large"
  | none => .error "Not a number"

-- Chaining with do notation
def validateInput (name age : String) : Except String (String × Nat) := do
  if name.isEmpty then throw "Name cannot be empty"
  let age ← parseAge age
  return (name, age)
```

### 3.15 The Compiler Pipeline

When you compile a Lean program, here is what happens:

```
Lean source code (.lean)
    ↓  (parsing)
Lean Syntax tree
    ↓  (elaboration — type checking, tactic evaluation, etc.)
Lean Core/Kernel terms
    ↓  (compilation)
LCNF (Lean Compiler Normal Form)
    ↓  (optimization passes: inlining, dead code elimination, etc.)
Optimized LCNF
    ↓  (code generation)
C code (.c files)
    ↓  (C compiler: gcc or clang)
Native binary
```

Key points:
- **Elaboration** is the big step. This is where Lean resolves implicit
  arguments, synthesizes typeclass instances, executes tactics, and checks
  types. Most of the "intelligence" of Lean lives here.
- **LCNF** is an intermediate representation designed for optimization.
- **Proofs are erased.** Any term whose type is in `Prop` is replaced with a
  dummy value during compilation, because proofs carry no computational content
  (proof irrelevance means we never need to inspect them at runtime).
- The generated C code uses Lean's runtime library, which provides reference
  counting, memory management, and a lean-specific ABI.

### 3.16 Metaprogramming and the Macro System

Lean 4 has a powerful metaprogramming system. There are several levels:

#### Macros (Syntax to Syntax transformations)

Macros are the simplest form of metaprogramming. They transform syntax before
elaboration:

```lean
-- Define a new syntax/notation
macro "assert! " cond:term : term =>
  `(if $cond then () else panic! "assertion failed")

-- Use it
def test : Unit :=
  assert! 2 + 2 == 4
```

Lean's macros are **hygienic**: variables introduced by macros do not
accidentally capture variables from the surrounding code.

#### Elaboration-Time Metaprogramming

For more power, you can write **elaborators** that run during type checking.
These have access to the full Lean environment:

```lean
-- A simple custom tactic
elab "my_trivial" : tactic => do
  let goal ← Lean.Elab.Tactic.getMainGoal
  goal.assumption
```

#### Macro Expansion Type

Macros have the type `Syntax → MacroM Syntax`. They take a syntax tree and
produce a new syntax tree, possibly with effects in the `MacroM` monad (which
handles hygiene and error reporting).

```lean
-- A slightly more involved macro
syntax "repeat_n " num " : " term : term

macro_rules
  | `(repeat_n 0 : $body) => `(())
  | `(repeat_n $n : $body) => do
    let n' := n.getNat - 1
    let n_lit := Lean.Syntax.mkNumLit (toString n')
    `(do $body; repeat_n $n_lit : $body)
```

This metaprogramming system is what allows Lean's tactic framework to be
written in Lean itself. Tactics like `simp`, `omega`, and `ring` are all
implemented as Lean metaprograms.

### 3.17 Notation

Lean allows defining custom notation very flexibly:

```lean
-- Infix notation
infixl:65 " ⊕ " => Nat.xor

-- Prefix notation
prefix:100 "√" => Float.sqrt

-- Postfix notation
postfix:max "!" => factorial

-- You can even define mixfix notation
notation "⟦" a ", " b "⟧" => (a, b)

#eval ⟦1, 2⟧  -- (1, 2)
```

### 3.18 `#eval` and `#check`: Your Best Friends

While developing, you will constantly use these commands:

```lean
-- #eval evaluates an expression and prints the result
#eval 2 + 3              -- 5
#eval "hello".length     -- 5
#eval [1,2,3].map (· * 2) -- [2, 4, 6]

-- #check shows the type of an expression WITHOUT evaluating it
#check 2 + 3             -- 2 + 3 : Nat
#check Nat.add            -- Nat.add : Nat → Nat → Nat
#check @List.map          -- List.map : {α β : Type} → (α → β) → List α → List β

-- #print shows the definition
#print Nat.add
```

---

## 4. Lean's Type System for Verification

This section covers the features of Lean's type system that go beyond what you
find in mainstream programming languages. These features are what enable formal
verification.

### 4.1 Dependent Types

A **dependent type** is a type that depends on a value. This is the defining
feature that separates Lean from languages like Haskell or OCaml.

In most languages, you can say "a list of integers" (`List Int`), but you
cannot say "a list of integers of length 5" in the type system. In Lean, you
can:

```lean
-- A vector: a list with its length encoded in the type
inductive Vector (α : Type) : Nat → Type where
  | nil  : Vector α 0
  | cons : α → Vector α n → Vector α (n + 1)
```

Look carefully at the type: `Vector α n` where `n : Nat`. The type depends on
the natural number `n`. A `Vector String 3` is a *different type* from
`Vector String 5` — you cannot use one where the other is expected.

This means the type system can enforce length constraints:

```lean
-- This function can ONLY be called on non-empty vectors
-- because the type requires n + 1 (which is at least 1)
def Vector.head : Vector α (n + 1) → α
  | .cons x _ => x

-- This CANNOT compile:
-- def bad := Vector.head Vector.nil
-- Error: type mismatch, Vector α 0 does not match Vector α (n + 1)
```

The compiler **rejects** the call at compile time. Not a runtime error — a
compile-time type error. The program with the bug simply cannot be compiled.

### 4.2 Pi Types (Dependent Function Types)

A **Pi type** (also written `Π` or using `∀`) is a function type where the
return type can depend on the input value:

```lean
-- A non-dependent function type (ordinary function):
-- Nat → String
-- "takes a Nat, returns a String"

-- A dependent function type:
-- (n : Nat) → Vector String n
-- "takes a Nat n, returns a Vector of Strings of length n"
```

The return type `Vector String n` mentions `n`, which is the input. This is a
Pi type.

In Lean notation:

```lean
-- These are all Pi types:
(n : Nat) → Vector String n           -- explicit dependent function
(α : Type) → α → α                    -- polymorphic identity
∀ (n : Nat), n + 0 = n                -- universal quantification (same thing!)
{α : Type} → [BEq α] → α → α → Bool  -- with implicit/instance args
```

The `∀` keyword and `→` with a named argument are both syntax for Pi types.
When the return type does not depend on the input, `α → β` is just shorthand
for `(_ : α) → β`.

### 4.3 Sigma Types (Dependent Pairs)

A **Sigma type** is a pair where the type of the second component depends on
the value of the first:

```lean
-- An ordinary pair (non-dependent):
-- Nat × String — first is a Nat, second is a String

-- A dependent pair (Sigma type):
-- (n : Nat) × Vector String n
-- "a Nat n, paired with a Vector of n Strings"
```

The second component's type (`Vector String n`) depends on the first
component's value (`n`). This is useful for existential statements:

```lean
-- "There exists a natural number n such that n * n = 9"
-- This is a Sigma type: a witness paired with evidence
example : ∃ n : Nat, n * n = 9 :=
  ⟨3, rfl⟩  -- The witness is 3, and the proof is by reflexivity
```

Technically, `∃ (n : Nat), P n` is syntactic sugar for `Exists (fun n => P n)`,
which is similar to a Sigma type but lives in `Prop`.

### 4.4 Propositional Equality

Equality in Lean is defined as an inductive type:

```lean
-- (Simplified) The actual definition in Lean's core
inductive Eq : α → α → Prop where
  | refl : Eq a a
```

There is one constructor, `refl` (reflexivity), which says "any value is equal
to itself." The notation `a = b` is sugar for `Eq a b`.

The only way to **construct** a proof of `a = b` is if `a` and `b` are
actually the same (definitionally equal, or provably so). `rfl` is the proof
term:

```lean
-- These work because both sides reduce to the same value
example : 2 + 3 = 5 := rfl
example : "hello".length = 5 := rfl

-- This does NOT work with rfl alone because n is a variable
-- example : ∀ n : Nat, n + 0 = n := fun n => rfl  -- Error!
-- It requires induction (see Section 5)
```

### 4.5 Definitional vs Propositional Equality

This is a subtle but important distinction:

**Definitional equality** (also called "judgmental equality") means two
expressions reduce to the same thing by computation. The kernel can check this
automatically:

```lean
-- 2 + 3 and 5 are definitionally equal (the kernel computes 2 + 3 = 5)
example : 2 + 3 = 5 := rfl

-- List.length [1,2,3] and 3 are definitionally equal
example : [1, 2, 3].length = 3 := rfl
```

**Propositional equality** means two expressions are equal, but proving it
requires reasoning (tactics, induction, lemmas). The kernel cannot just compute
it:

```lean
-- n + 0 = n is NOT definitional for a variable n
-- because Nat.add is defined by recursion on the SECOND argument,
-- so `n + 0` does not reduce further when n is a variable
theorem add_zero (n : Nat) : n + 0 = n := by
  induction n with
  | zero => rfl
  | succ n ih => simp [Nat.add_succ, ih]
```

### 4.6 Decidable Equality and `DecidableEq`

**Decidable equality** means there is an algorithm that can determine whether
two values are equal. Not all types have this — in general, equality of
functions is undecidable.

```lean
-- Nat has decidable equality
#check (inferInstance : DecidableEq Nat)

-- String has decidable equality
#check (inferInstance : DecidableEq String)

-- You can use `if` with decidable propositions
def isThree (n : Nat) : Bool :=
  if n = 3 then true else false

-- This works because DecidableEq Nat is available, which makes
-- the proposition `n = 3` decidable.
```

The `DecidableEq` typeclass provides an algorithm for deciding equality. When
you write `if h : x = y then ... else ...`, Lean uses the `DecidableEq`
instance to decide the equality and bind the proof/refutation to `h`.

### 4.7 Proof Irrelevance in Practice

Recall that any two proofs of the same `Prop` are considered equal. Here is
why this matters in practice:

```lean
-- Suppose we define a type of positive natural numbers
structure PosNat where
  val : Nat
  pos : val > 0

-- These are the SAME PosNat, even though the proofs differ
-- (proof irrelevance makes the proofs equal)
example (h1 h2 : 1 > 0) : (PosNat.mk 1 h1) = (PosNat.mk 1 h2) := rfl
```

Without proof irrelevance, you would need to show that `h1 = h2` to conclude
the two `PosNat` values are equal. Proof irrelevance gives you this for free.

### 4.8 Subtypes

A **subtype** is a type equipped with a predicate. It lets you refine a type
to a subset:

```lean
-- The subtype of natural numbers greater than zero
def PosNat' := { n : Nat // n > 0 }

-- Creating a subtype value
def three : PosNat' := ⟨3, by omega⟩

-- Accessing the value and the proof
#eval three.val    -- 3
-- three.property is a proof of 3 > 0
```

The notation `{ x : α // P x }` is syntactic sugar for `Subtype (fun x => P x)`.
Subtypes are one of the primary mechanisms for attaching specifications to
data.

### 4.9 Quotient Types

Quotient types let you define a type by declaring that certain elements are
considered equal:

```lean
-- Define integers as pairs of naturals modulo the relation
-- (a, b) ~ (c, d) iff a + d = c + b
-- (intuitively, (a, b) represents a - b)

-- Lean provides Quotient as a built-in, which creates a new type
-- where the equivalence classes are the elements.
```

Quotient types are used extensively in Mathlib for constructions like the
integers (as equivalence classes of pairs of naturals), the rationals, and
more abstract algebra.

---

## 5. Proof Writing in Lean 4

Now we arrive at the heart of Lean as a theorem prover. This section shows you
how to actually write proofs.

### 5.1 Three Modes of Proof

Lean offers three styles for writing proofs. You can mix them freely.

#### Term Mode

Write the proof as a direct expression (a lambda, an application, etc.). This
is like writing a program:

```lean
-- Prove: for all propositions P, P implies P
theorem identity (P : Prop) (h : P) : P := h

-- Prove: for all P Q, P ∧ Q implies Q ∧ P
theorem and_comm' (P Q : Prop) (h : P ∧ Q) : Q ∧ P :=
  ⟨h.2, h.1⟩
```

Term mode proofs are concise but can be hard to read for complex proofs.

#### Tactic Mode

Write the proof as a sequence of instructions (tactics) that manipulate a
**proof state**. Enter tactic mode with `by`:

```lean
theorem identity' (P : Prop) (h : P) : P := by
  exact h

theorem and_comm'' (P Q : Prop) (h : P ∧ Q) : Q ∧ P := by
  constructor       -- splits the goal into two subgoals: Q and P
  · exact h.2       -- prove Q using the second component of h
  · exact h.1       -- prove P using the first component of h
```

Tactic mode is the most common style for non-trivial proofs. Each tactic
transforms the proof state (the list of goals to prove, along with available
hypotheses).

#### Calc Mode

For equational or inequality reasoning, `calc` lets you write a chain of
steps:

```lean
theorem calc_example (a b c : Nat) (h1 : a = b) (h2 : b = c) : a = c := by
  calc a = b := h1
    _ = c := h2

-- A more interesting example
theorem sum_formula (n : Nat) :
    2 * (List.range (n + 1)).sum = n * (n + 1) := by
  induction n with
  | zero => simp
  | succ n ih =>
    simp [List.range_succ, List.sum_append]
    omega
```

`calc` mode is especially readable for proofs involving chains of equalities
or inequalities.

### 5.2 The Goal View

When you write a tactic proof in VS Code, the **Lean Infoview** panel shows
you the current proof state. This is the most important tool for proof writing.

Here is what you see. Given this partial proof:

```lean
theorem example_proof (n : Nat) (h : n > 0) : n ≠ 0 := by
  intro heq
  -- cursor here
```

The infoview shows:

```
n : Nat
h : n > 0
heq : n = 0
⊢ False
```

This tells you:

- **Context (hypotheses):** You have `n` (a natural number), `h` (a proof that
  `n > 0`), and `heq` (a proof that `n = 0` — this came from `intro` on
  the `≠` goal, since `n ≠ 0` is defined as `n = 0 → False`).

- **Goal:** You need to prove `False`. That is, you need to derive a
  contradiction from the hypotheses.

This interactive feedback loop — write a tactic, see the state change, write
the next tactic — is how you develop proofs in practice. It is like a
conversation between you and the proof checker.

### 5.3 Anatomy of a Theorem

```lean
theorem my_theorem           -- the keyword and name
    (n : Nat)                -- universally quantified variable
    (h1 : n > 0)             -- hypothesis: n is positive
    (h2 : n < 100)           -- hypothesis: n is less than 100
    : n ≥ 1 ∧ n ≤ 99 := by  -- conclusion (what we want to prove)
  constructor                -- tactic proof begins
  · omega                    -- first subgoal: n ≥ 1
  · omega                    -- second subgoal: n ≤ 99
```

The parts:

1. **`theorem` keyword**: Declares a theorem. (You can also use `lemma` —
   it is identical. Convention: `lemma` for small helper results, `theorem`
   for main results.)

2. **Name**: `my_theorem`. Other definitions can refer to this name.

3. **Parameters**: `(n : Nat)`, `(h1 : n > 0)`, `(h2 : n < 100)`. These are
   the universally quantified variables and assumptions.

4. **Conclusion**: `n ≥ 1 ∧ n ≤ 99`. The proposition to prove.

5. **`:= by`**: Enters tactic mode.

6. **Tactics**: The sequence of proof steps.

You can also use `def` instead of `theorem`. The difference is semantic:
`theorem` marks the definition as a proof (so the compiler erases it), while
`def` keeps it as a computable definition. Use `theorem` for proofs and `def`
for programs.

### 5.4 Worked Example: Proving `∀ n : Nat, 0 + n = n`

Let us prove step by step that adding zero on the left gives back the same
number. (Note: `n + 0 = n` is the harder direction because of how addition is
defined. `0 + n = n` is actually true by definition.)

```lean
-- First, let's understand WHY this is true by definition.
-- Nat.add is defined as:
--   Nat.add m 0     = m           (base case: adding zero)
--   Nat.add m (n+1) = (m + n) + 1 (recursive case)
-- Wait — that is recursion on the SECOND argument.
-- So Nat.add 0 n recurses on n:
--   Nat.add 0 0     = 0
--   Nat.add 0 (n+1) = (Nat.add 0 n) + 1
-- This means Nat.add 0 n = n is true, but only by INDUCTION on n.

-- Actually, in Lean 4's implementation, 0 + n does reduce to n
-- by the definitional reduction rules. Let's verify:
example : ∀ n : Nat, 0 + n = n := by
  intro n
  rfl  -- works! 0 + n reduces to n definitionally

-- But n + 0 = n requires a proof:
theorem add_zero' : ∀ n : Nat, n + 0 = n := by
  intro n        -- introduce n, goal becomes: n + 0 = n
  induction n with
  | zero =>
    -- goal: 0 + 0 = 0
    rfl
  | succ n ih =>
    -- n : Nat
    -- ih : n + 0 = n  (induction hypothesis)
    -- goal: (n + 1) + 0 = n + 1
    -- By definition, (n + 1) + 0 = (n + 0) + 1... wait, let's
    -- just use simp with the induction hypothesis
    simp [ih]
```

Let us also do a slightly more involved example:

```lean
-- Prove: addition is commutative
-- This is a classic exercise in proof by induction
theorem add_comm' : ∀ (m n : Nat), m + n = n + m := by
  intro m n
  induction n with
  | zero =>
    -- goal: m + 0 = 0 + m
    simp
  | succ n ih =>
    -- ih : m + n = n + m
    -- goal: m + (n + 1) = (n + 1) + m
    -- i.e., m + n + 1 = n + 1 + m
    omega  -- omega can handle linear arithmetic over Nat
```

In practice, `omega` handles many arithmetic goals automatically. But
understanding how to do it manually (via `induction` and `rw`) is important for
building intuition.

### 5.5 The Proof State: Goals, Hypotheses, Context

Let us formalize what we mean by "proof state."

A **proof state** consists of zero or more **goals**. Each goal has:

- A **context** (also called **local context**): a list of local declarations.
  Each declaration is either:
  - A variable: `(n : Nat)` — we know `n` exists and has type `Nat`
  - A hypothesis: `(h : n > 0)` — we have a proof named `h` of `n > 0`
  - A local definition: `(x : Nat := 5)` — `x` is defined as `5`

- A **target** (also called **goal type**): the proposition we need to prove.

When you start a tactic proof, there is one goal with no hypotheses and the
target is the theorem statement. As you apply tactics:

- Some tactics **transform** the current goal (e.g., `intro` moves a
  universally quantified variable into the context).
- Some tactics **split** a goal into subgoals (e.g., `constructor` on an `∧`
  goal creates two subgoals).
- Some tactics **close** a goal (e.g., `exact` provides a proof term).

The proof is complete when there are **zero goals remaining**.

### 5.6 Common Proof Patterns

#### Proving an implication: introduce and apply

```lean
-- To prove P → Q:
-- 1. Introduce the hypothesis P
-- 2. Use it to prove Q
theorem imp_example (P Q : Prop) (hpq : P → Q) (hp : P) : Q := by
  apply hpq   -- goal becomes: P
  exact hp     -- provide the proof of P
```

#### Proving a conjunction: split and prove each part

```lean
theorem conj_example (n : Nat) (h : n = 5) : n = 5 ∧ n < 10 := by
  constructor
  · exact h
  · omega
```

#### Proving a disjunction: choose a side

```lean
theorem disj_example (n : Nat) (h : n = 5) : n = 5 ∨ n = 6 := by
  left      -- choose the left disjunct
  exact h
```

#### Proving by contradiction

```lean
theorem by_contra_example (n : Nat) (h : n > 0) : n ≠ 0 := by
  intro heq     -- assume n = 0 (since ≠ is defined as → False)
  omega          -- derive contradiction: n > 0 and n = 0

-- Or using `by_contra`:
theorem by_contra_example' (P : Prop) (h : ¬¬P) : P := by
  by_contra hp   -- assume ¬P, goal becomes False
  exact h hp     -- apply h to hp to get False
```

#### Proving by cases

```lean
theorem cases_example (n : Nat) : n = 0 ∨ n > 0 := by
  cases n with
  | zero => left; rfl
  | succ n => right; omega
```

#### Proving universal statements: introduce and prove

```lean
theorem forall_example : ∀ n : Nat, n + 0 = n := by
  intro n     -- introduce an arbitrary n
  omega       -- prove n + 0 = n
```

#### Proving existential statements: provide a witness

```lean
theorem exists_example : ∃ n : Nat, n * n = 9 := by
  exact ⟨3, rfl⟩    -- the witness is 3, and 3 * 3 = 9 by rfl

-- Or using `use`:
theorem exists_example' : ∃ n : Nat, n * n = 9 := by
  use 3   -- provides the witness; remaining goal is 3 * 3 = 9
          -- which Lean closes automatically
```

#### Proof by induction

```lean
theorem induction_example (n : Nat) : n * 0 = 0 := by
  induction n with
  | zero => rfl                    -- base case: 0 * 0 = 0
  | succ k ih =>                   -- inductive step
    -- ih : k * 0 = 0
    -- goal: (k + 1) * 0 = 0
    simp [Nat.succ_mul, ih]
```

#### Rewriting with equations

```lean
theorem rewrite_example (a b c : Nat) (h1 : a = b) (h2 : b = c) : a = c := by
  rw [h1]      -- rewrite a to b in the goal; goal becomes b = c
  exact h2     -- finish with h2 : b = c

-- rw [← h1] rewrites in the reverse direction (b to a)
```

---

## 6. Essential Tactics Reference

This section provides a comprehensive reference for Lean 4 tactics. For each
tactic, we give a brief description, the typical usage pattern, and an example.

### 6.1 Introduction Tactics

These tactics move quantified variables and hypotheses from the goal into the
context.

#### `intro`

Introduces one or more variables/hypotheses.

```lean
-- Before: ⊢ ∀ (n : Nat), n = n
-- After intro n: n : Nat ⊢ n = n
example : ∀ (n : Nat), n = n := by
  intro n
  rfl

-- Introduces a hypothesis from an implication
-- Before: ⊢ P → P
-- After intro h: h : P ⊢ P
example (P : Prop) : P → P := by
  intro h
  exact h

-- Introduce multiple variables at once
example : ∀ (a b : Nat), a + b = b + a := by
  intro a b
  omega
```

#### `intros`

Introduces all leading universal quantifiers and implications at once:

```lean
-- Introduces everything it can
example : ∀ (a b c : Nat), a = b → b = c → a = c := by
  intros a b c h1 h2
  rw [h1, h2]
```

#### `rintro`

A more powerful version of `intro` that allows pattern matching during
introduction. (Available with Mathlib.)

```lean
-- Destructure a conjunction
-- Before: ⊢ P ∧ Q → Q ∧ P
example (P Q : Prop) : P ∧ Q → Q ∧ P := by
  rintro ⟨hp, hq⟩    -- destructures the P ∧ Q into hp : P and hq : Q
  exact ⟨hq, hp⟩

-- Destructure an existential
-- Before: ⊢ (∃ n, P n) → ...
example (P : Nat → Prop) : (∃ n, P n) → ∃ n, P n := by
  rintro ⟨n, hn⟩     -- destructures into n : Nat and hn : P n
  exact ⟨n, hn⟩

-- Case split on a disjunction
example (P Q : Prop) : P ∨ Q → Q ∨ P := by
  rintro (hp | hq)
  · right; exact hp
  · left; exact hq
```

### 6.2 Application Tactics

These tactics make progress by applying existing terms to the goal.

#### `exact`

Closes the goal by providing an exact proof term:

```lean
-- The term must have exactly the type of the goal
example (P : Prop) (h : P) : P := by
  exact h

example : 2 + 3 = 5 := by
  exact rfl

-- You can use any expression, including function applications
example (P Q : Prop) (hpq : P → Q) (hp : P) : Q := by
  exact hpq hp
```

#### `apply`

Applies a function/lemma to the goal, creating new goals for the arguments:

```lean
-- If the goal is Q and you have hpq : P → Q,
-- `apply hpq` changes the goal to P
example (P Q : Prop) (hpq : P → Q) (hp : P) : Q := by
  apply hpq    -- goal becomes P
  exact hp

-- Works with multi-argument functions too
example (P Q R : Prop) (f : P → Q → R) (hp : P) (hq : Q) : R := by
  apply f      -- creates two goals: P and Q
  · exact hp
  · exact hq

-- Commonly used with library lemmas
example (a b : Nat) (h : a = b) : b = a := by
  apply Eq.symm  -- goal becomes a = b
  exact h
```

#### `refine`

Like `apply`, but lets you leave holes (written `?_`) for goals you want to
prove later:

```lean
example (P Q R : Prop) (hq : Q) : P → (P → Q → R) → R := by
  refine fun hp f => f ?_ ?_
  · exact hp
  · exact hq
```

### 6.3 Case Analysis Tactics

#### `cases`

Performs case analysis on an inductive type:

```lean
-- Case analysis on a natural number
example (n : Nat) : n = 0 ∨ n ≥ 1 := by
  cases n with
  | zero => left; rfl
  | succ m => right; omega

-- Case analysis on a Boolean
example (b : Bool) : b = true ∨ b = false := by
  cases b with
  | true => left; rfl
  | false => right; rfl

-- Case analysis on a hypothesis
example (P Q : Prop) (h : P ∨ Q) : Q ∨ P := by
  cases h with
  | inl hp => right; exact hp
  | inr hq => left; exact hq

-- Case analysis on a conjunction
example (P Q : Prop) (h : P ∧ Q) : Q := by
  cases h with
  | intro hp hq => exact hq
```

#### `rcases` (Mathlib)

A more flexible version of `cases` that allows recursive destructuring:

```lean
-- Destructure nested structures
example (P Q R : Prop) (h : P ∧ (Q ∧ R)) : R := by
  rcases h with ⟨_, _, hr⟩
  exact hr

-- Case split and destructure simultaneously
example (P Q R : Prop) (h : (P ∧ Q) ∨ R) : Q ∨ R := by
  rcases h with ⟨_, hq⟩ | hr
  · left; exact hq
  · right; exact hr
```

#### `obtain` (Mathlib)

Like `rcases` but often used to destructure existential statements and
introduce new variables:

```lean
example (h : ∃ n : Nat, n * n = 9) : ∃ m : Nat, m * m = 9 := by
  obtain ⟨n, hn⟩ := h     -- introduces n : Nat and hn : n * n = 9
  exact ⟨n, hn⟩
```

### 6.4 Induction Tactics

#### `induction`

Performs structural induction:

```lean
-- Induction on natural numbers
theorem zero_add (n : Nat) : 0 + n = n := by
  induction n with
  | zero => rfl
  | succ n ih =>
    -- ih : 0 + n = n
    -- goal: 0 + (n + 1) = n + 1
    rw [Nat.add_succ, ih]

-- Induction on lists
theorem length_append (xs ys : List α) :
    (xs ++ ys).length = xs.length + ys.length := by
  induction xs with
  | nil => simp
  | cons x xs ih =>
    simp [List.cons_append, List.length_cons, ih]
    omega
```

#### Strong Induction

For when you need the induction hypothesis to apply to ALL smaller values,
not just the immediate predecessor:

```lean
-- Using Nat.strongRecOn or well-founded recursion
theorem strong_induction_example (n : Nat) (h : n > 0) : n ≥ 1 := by
  omega  -- trivial here, but the pattern is:
  -- You can use `Nat.strong_rec_on` or `termination_by` in recursive defs
```

In practice, `omega` handles many arithmetic goals that would classically
require strong induction. For structural strong induction, you can use
well-founded recursion in the definition itself.

### 6.5 Rewriting Tactics

#### `rw` (rewrite)

Rewrites the goal using an equation:

```lean
-- rw [h] replaces the left side of h with the right side
example (a b : Nat) (h : a = b) : a + 1 = b + 1 := by
  rw [h]   -- goal becomes b + 1 = b + 1, which is closed by rfl

-- rw [← h] rewrites in the reverse direction
example (a b : Nat) (h : a = b) : b + 1 = a + 1 := by
  rw [← h]

-- Multiple rewrites at once
example (a b c : Nat) (h1 : a = b) (h2 : b = c) : a = c := by
  rw [h1, h2]

-- Rewrite in a hypothesis (not the goal)
example (a b : Nat) (h1 : a = b) (h2 : a > 0) : b > 0 := by
  rw [← h1]    -- No! This rewrites the goal.
  -- Instead:
  sorry

example (a b : Nat) (h1 : a = b) (h2 : a > 0) : b > 0 := by
  rw [h1] at h2  -- rewrites h2 to b > 0
  exact h2
```

#### `simp` (simplification)

The simplification tactic. It applies a set of rewrite rules repeatedly until
no more apply. This is one of the most powerful and frequently used tactics.

```lean
-- simp knows many basic simplification rules
example (n : Nat) : n + 0 = n := by simp
example (xs : List Nat) : ([] ++ xs) = xs := by simp
example : ¬False := by simp

-- You can add extra lemmas for simp to use
example (a b : Nat) (h : a = b) : a + 1 = b + 1 := by
  simp [h]

-- simp can simplify complex expressions
example (xs ys : List Nat) : (xs ++ []).length = xs.length := by
  simp

-- simp only [...] restricts simp to specific lemmas
example (n : Nat) : 0 + n = n := by
  simp only [Nat.zero_add]
```

#### `simp_all`

Like `simp` but also simplifies all hypotheses, and uses hypotheses as
rewrite rules:

```lean
example (a b : Nat) (h1 : a = 0) (h2 : b = a + 1) : b = 1 := by
  simp_all
```

#### `dsimp` (definitional simplification)

Like `simp` but only uses definitional equalities (does not use propositional
lemmas). Useful when you want to unfold definitions without using lemmas:

```lean
def myDouble (n : Nat) : Nat := n + n

example : myDouble 3 = 6 := by
  dsimp [myDouble]   -- unfolds myDouble; goal becomes 3 + 3 = 6
  rfl
```

### 6.6 Arithmetic Tactics

#### `omega`

Solves goals in **Presburger arithmetic** — linear arithmetic over natural
numbers and integers, with quantifiers. This is a decision procedure: if the
goal is in its fragment, `omega` will solve it or fail definitively.

```lean
-- Linear inequalities
example (n : Nat) (h : n > 5) : n ≥ 6 := by omega
example (a b : Nat) (h1 : a ≤ b) (h2 : b ≤ a) : a = b := by omega

-- Modular arithmetic and divisibility (to some extent)
example (n : Nat) : n + 1 > 0 := by omega

-- Complex linear constraints
example (x y z : Nat) (h1 : x + y ≤ z) (h2 : z < x + y + 3) : z - x ≤ y + 2 := by
  omega

-- Works with Int too
example (n : Int) (h : n > 0) : n ≥ 1 := by omega
```

`omega` is your go-to tactic for any goal involving linear combinations of
natural numbers or integers with `+`, `-`, `*` (by a constant), `<`, `≤`,
`=`, `≥`, `>`.

#### `linarith`

Like `omega` but works over ordered fields (including rationals and reals)
and can use hypotheses more flexibly. Available with Mathlib.

```lean
-- Works with rationals and reals
example (x y : Rat) (h1 : x ≤ y) (h2 : y ≤ x + 1) : y - x ≤ 1 := by
  linarith

-- Can combine multiple hypotheses
example (a b c : Nat) (h1 : a ≤ b) (h2 : b ≤ c) (h3 : c ≤ a + 2) :
    c - a ≤ 2 := by
  omega  -- omega also works here since these are Nats
```

#### `ring`

Solves equalities in commutative (semi)rings. Handles polynomial arithmetic:

```lean
-- Polynomial identities
example (a b : Int) : (a + b) * (a + b) = a * a + 2 * a * b + b * b := by
  ring

example (x : Int) : (x + 1) * (x - 1) = x ^ 2 - 1 := by
  ring

-- Works for any commutative ring
example (a b c : Int) : a * (b + c) = a * b + a * c := by
  ring
```

#### `norm_num`

Evaluates numerical expressions and proves numerical facts:

```lean
example : (2 : Nat) + 3 = 5 := by norm_num
example : (7 : Nat) < 10 := by norm_num
example : (100 : Nat) % 7 = 2 := by norm_num
example : Nat.Prime 17 := by norm_num

-- Also works with more complex numerical goals
example : (3 : Rat) / 4 + 1 / 4 = 1 := by norm_num
```

#### `norm_cast`

Handles goals involving coercions between number types (e.g., `Nat → Int`,
`Int → Rat`):

```lean
example (n : Nat) : (↑n : Int) ≥ 0 := by
  exact Int.ofNat_nonneg n

-- norm_cast simplifies coercions
example (m n : Nat) : (↑(m + n) : Int) = ↑m + ↑n := by
  norm_cast
```

### 6.7 Decision Tactics

#### `decide`

Uses the `Decidable` typeclass to automatically decide a proposition. Works
for finite/decidable propositions:

```lean
example : 2 + 2 = 4 := by decide
example : ¬(3 = 4) := by decide
example : 10 < 20 := by decide
example : True := by decide
example : ¬False := by decide

-- Can decide propositions about finite types
example : ∀ b : Bool, b = true ∨ b = false := by decide
```

`decide` works by computation: it evaluates the decision procedure and checks
the result. It is limited to propositions that are actually decidable and where
the computation terminates quickly.

#### `native_decide`

Like `decide` but uses native code execution (compiled C) instead of the
kernel's built-in evaluator. Much faster for large computations, but slightly
less trustworthy (trusts the compiler):

```lean
-- Faster for large computations
example : Nat.Prime 104729 := by native_decide
```

### 6.8 Automation Tactics

#### `grind`

A relatively recent (Lean 4.x) general-purpose automation tactic that combines
congruence closure, E-matching, and other techniques. It can handle many
first-order reasoning goals:

```lean
-- Handles equational reasoning and basic logic
example (f : Nat → Nat) (a b : Nat) (h1 : a = b) (h2 : f a = 0) : f b = 0 := by
  grind

-- Handles propositional logic
example (P Q R : Prop) (h1 : P → Q) (h2 : Q → R) (h3 : P) : R := by
  grind
```

#### `solve_by_elim`

Tries to close the goal by repeatedly applying hypotheses and lemmas:

```lean
example (P Q R : Prop) (hp : P) (hpq : P → Q) (hqr : Q → R) : R := by
  solve_by_elim
```

#### `tauto`

Solves propositional tautologies. Available with Mathlib.

```lean
-- Classical propositional tautologies
example (P Q : Prop) : P ∨ ¬P := by tauto
example (P Q : Prop) : ¬(P ∧ Q) ↔ ¬P ∨ ¬Q := by tauto
example (P Q : Prop) : (P → Q) → (¬Q → ¬P) := by tauto
```

### 6.9 Structural Tactics

#### `constructor`

Applies the constructor of the goal type. For `∧`, splits into two subgoals.
For `∃`, asks for a witness. For any inductive type with a single constructor,
applies it.

```lean
example (P Q : Prop) (hp : P) (hq : Q) : P ∧ Q := by
  constructor    -- splits into goals P and Q
  · exact hp
  · exact hq

-- For Iff (if and only if)
example (P : Prop) : P ↔ P := by
  constructor
  · intro h; exact h
  · intro h; exact h
```

#### `ext` (extensionality)

Proves equality of functions or structures by proving equality at each point
or field:

```lean
-- Function extensionality
example (f g : Nat → Nat) (h : ∀ n, f n = g n) : f = g := by
  ext n       -- introduces n : Nat, goal becomes f n = g n
  exact h n

-- Structure extensionality
example (p q : Point) (hx : p.x = q.x) (hy : p.y = q.y) : p = q := by
  ext
  · exact hx
  · exact hy
```

#### `funext`

Specifically for function extensionality (proving two functions are equal):

```lean
example : (fun n : Nat => n + 0) = (fun n => n) := by
  funext n       -- goal becomes n + 0 = n
  omega
```

#### `congr`

Proves a goal of the form `f a = f b` by creating a subgoal `a = b` (or more
generally, matches up corresponding arguments):

```lean
example (a b : Nat) (h : a = b) : a + 1 = b + 1 := by
  congr 1    -- the `1` says to go one level deep; goal becomes a = b
  exact h
```

### 6.10 Control Flow Tactics

#### `have`

Introduces an intermediate result (a sub-lemma) into the context:

```lean
example (n : Nat) (h : n > 10) : n > 5 := by
  have h' : n > 7 := by omega   -- prove an intermediate result
  -- now h' : n > 7 is in the context
  omega
```

`have` is like writing a helper lemma inline. It creates a subgoal for the
intermediate result, and then after proving it, the result is available as a
hypothesis.

#### `let`

Like `have`, but introduces a definition (not just a proof):

```lean
example : 10 = 10 := by
  let x := 5
  -- x : Nat := 5 is in the context
  show 10 = 10   -- the goal is unchanged
  rfl
```

#### `show`

Changes the goal to a definitionally equal expression. Useful for clarifying
what you are proving:

```lean
example (n : Nat) : n + 0 = n := by
  show n + 0 = n    -- no actual change; just makes it explicit
  omega
```

#### `suffices`

Like `have` but in reverse: you state what would be sufficient to prove the
goal, prove the goal assuming that, and then prove the sufficient condition:

```lean
example (n : Nat) (h : n > 10) : n > 5 := by
  suffices n > 7 by omega   -- if n > 7, then n > 5 (by omega)
  omega                      -- now prove n > 7
```

#### `calc`

For equational reasoning chains:

```lean
example (a b c d : Nat) (h1 : a = b) (h2 : b + 1 = c) (h3 : c = d) :
    a + 1 = d := by
  calc a + 1 = b + 1 := by rw [h1]
    _ = c := h2
    _ = d := h3
```

Each step in a `calc` block proves one link in the chain. The `_` refers to
the right-hand side of the previous step.

### 6.11 The `@[simp]` Attribute

The `@[simp]` attribute marks a lemma as a simplification rule. When you call
`simp`, it uses all lemmas tagged with `@[simp]` (plus any you pass
explicitly).

```lean
@[simp]
theorem myAdd_zero (n : Nat) : myAdd n 0 = n := by
  -- ...
  sorry

@[simp]
theorem myAdd_succ (n m : Nat) : myAdd n (m + 1) = myAdd n m + 1 := by
  -- ...
  sorry

-- Now simp knows about myAdd
example (n : Nat) : myAdd (myAdd n 0) 0 = n := by
  simp   -- uses the @[simp] lemmas above to simplify
```

Guidelines for `@[simp]` lemmas:

1. They should rewrite from complicated to simple (left side should be more
   complex than right side).
2. They should be unconditionally applicable (no hypotheses, or only simple
   ones).
3. They should terminate (no infinite loops like `a = b` and `b = a` both
   being `@[simp]`).

Mathlib has thousands of `@[simp]` lemmas. When you call `simp` in a file that
imports Mathlib, all of those lemmas are available.

### 6.12 Other Useful Tactics

#### `trivial`

Tries a few simple strategies: `rfl`, `assumption`, `contradiction`, `decide`:

```lean
example : True := by trivial
example (h : P) : P := by trivial
```

#### `assumption`

Searches the context for a hypothesis that exactly matches the goal:

```lean
example (P Q : Prop) (hp : P) (hq : Q) : P := by assumption
```

#### `contradiction`

Searches for contradictory hypotheses:

```lean
example (h1 : P) (h2 : ¬P) : Q := by contradiction
example (h : False) : P := by contradiction
```

#### `exfalso`

Changes any goal to `False` (because from `False` anything follows):

```lean
example (h : False) : 2 + 2 = 5 := by
  exfalso
  exact h
```

#### `push_neg`

Pushes negations inward through quantifiers and connectives. Available with
Mathlib.

```lean
-- ¬ ∀ x, P x  becomes  ∃ x, ¬ P x
-- ¬ (a < b)   becomes  b ≤ a
example : ¬ (∀ n : Nat, n > 0) := by
  push_neg       -- goal becomes ∃ n, n ≤ 0
  exact ⟨0, le_refl 0⟩
```

#### `use`

Provides a witness for an existential goal:

```lean
example : ∃ n : Nat, n + n = 10 := by
  use 5   -- goal becomes 5 + 5 = 10, which is closed automatically
```

#### `specialize`

Specializes a universally quantified hypothesis:

```lean
example (h : ∀ n : Nat, n > 0 → n ≥ 1) : 5 ≥ 1 := by
  specialize h 5    -- h becomes: 5 > 0 → 5 ≥ 1
  apply h
  omega
```

#### `left` and `right`

Choose a side of a disjunction:

```lean
example (P Q : Prop) (hp : P) : P ∨ Q := by
  left; exact hp

example (P Q : Prop) (hq : Q) : P ∨ Q := by
  right; exact hq
```

#### `simp_arith` / `simp with arith`

Combines `simp` with arithmetic simplification:

```lean
example (n : Nat) : 0 < n + 1 := by simp_arith
```

#### `positivity` (Mathlib)

Proves goals of the form `0 ≤ x` or `0 < x`:

```lean
-- example (a : Real) : 0 ≤ a ^ 2 := by positivity
```

#### `field_simp` (Mathlib)

Clears denominators in field expressions:

```lean
-- example (a b : Rat) (hb : b ≠ 0) : a / b * b = a := by field_simp
```

---

## 7. Practical Formal Verification

Lean is not just an academic tool. It is increasingly used for verifying real
software. This section explains how.

### 7.1 Specifications as Types

The core idea of formal verification is: **express what you want to prove as a
type, then write a term of that type.** The types serve as specifications and
the terms serve as proofs that the specifications are met.

#### Pre-conditions and Post-conditions

```lean
-- A verified division function:
-- Pre-condition: divisor is not zero
-- Post-condition: result * divisor + remainder = dividend
def divmod (a b : Nat) (hb : b > 0) :
    { p : Nat × Nat // p.1 * b + p.2 = a ∧ p.2 < b } := by
  exact ⟨(a / b, a % b), by omega, by omega⟩
```

Here the return type is a subtype: it is a pair `(quotient, remainder)` along
with a proof that `quotient * b + remainder = a` and `remainder < b`. The
function must produce both the result AND the proof. If the implementation is
wrong, it will not type-check.

#### Sorting Example

```lean
-- A specification for what it means to be sorted
def IsSorted : List Nat → Prop
  | [] => True
  | [_] => True
  | a :: b :: rest => a ≤ b ∧ IsSorted (b :: rest)

-- A verified sort would have this signature:
-- def verifiedSort (xs : List Nat) :
--     { ys : List Nat // IsSorted ys ∧ ys.Perm xs }
-- (ys is sorted AND ys is a permutation of xs)
```

### 7.2 Intrinsic vs Extrinsic Verification

There are two main approaches to verification:

#### Extrinsic Verification

Write the function first, then prove properties about it separately:

```lean
-- Step 1: Write the function
def reverse : List α → List α
  | [] => []
  | x :: xs => reverse xs ++ [x]

-- Step 2: Prove properties
theorem reverse_reverse (xs : List α) : reverse (reverse xs) = xs := by
  induction xs with
  | nil => simp [reverse]
  | cons x xs ih => simp [reverse, ih]
```

This approach is more flexible and more common. You write clean code and prove
facts about it.

#### Intrinsic Verification

Encode the property directly in the type:

```lean
-- The type FORCES the function to preserve length
def map' (f : α → β) : Vector α n → Vector β n
  | .nil => .nil
  | .cons x xs => .cons (f x) (map' f xs)
```

Because `map'` takes a `Vector α n` and returns a `Vector β n`, the compiler
guarantees the output has the same length as the input. You do not need a
separate theorem.

Intrinsic verification produces more type-safe code but can be harder to work
with for complex properties.

### 7.3 Notable Verification Projects

#### AWS Cedar (Amazon)

Amazon Web Services developed **Cedar**, a policy language for access control.
Cedar's semantics and core authorization logic have been formally verified in
Lean 4. This ensures that Cedar's authorization decisions match their
specification — critical for security-sensitive cloud infrastructure.

The Cedar Lean formalization includes:
- The abstract syntax of Cedar policies
- The evaluation semantics
- Proofs of soundness of the authorization algorithm
- Proofs that policy analysis operations are correct

#### SymCrypt Verification (Microsoft, via Aeneas)

Microsoft's **SymCrypt** is a core cryptographic library used across Windows,
Azure, and other Microsoft products. Using **Aeneas** (a tool that translates
Rust code into Lean), researchers have verified portions of SymCrypt's
implementations. Aeneas takes Rust code (via its MIR intermediate
representation) and produces equivalent Lean definitions, which can then be
formally verified.

This is relevant to the repository you are working in (rust-lean-aeneas) — the
pipeline is:

```
Rust source code
    ↓  (Rust compiler frontend)
MIR (Mid-level IR)
    ↓  (Aeneas)
Lean 4 definitions
    ↓  (Lean 4 proof assistant)
Formal verification proofs
```

#### IMO 2025 Gold Medals

In 2025, AI systems using Lean as a backend earned gold medal scores at the
International Mathematical Olympiad. The systems generated candidate proofs,
and Lean verified their correctness. This demonstrated that:

1. Lean is expressive enough to state and verify competition-level mathematics.
2. Lean's automation is sufficient to fill in routine details.
3. The combination of AI and formal verification is powerful.

#### Other Projects

- **Liquid Tensor Experiment**: Verified a result of Peter Scholze in
  condensed mathematics, using Lean 4 (via Mathlib). This was a landmark
  achievement in formalized mathematics.
- **Sphere packing in 24 dimensions**: Formalized in Lean.
- **Reservoir** (https://reservoir.lean-lang.org): A growing package registry
  for Lean projects.

### 7.4 The "Functional Core, Imperative Shell" Pattern

A common architecture for verified programs:

1. **Functional Core**: Pure functions with no side effects. These are the
   parts you verify. They take inputs and return outputs, with specifications
   encoded in their types.

2. **Imperative Shell**: IO-performing code that calls the functional core.
   This handles user interaction, file I/O, networking, etc. It is typically
   NOT verified (or verified to a lesser degree).

```lean
-- Functional core (verified)
def processData (input : InputData) : { output : OutputData // valid output } :=
  sorry -- ... verified implementation ...

-- Imperative shell (not verified, handles IO)
def main : IO Unit := do
  let raw ← IO.FS.readFile "input.txt"
  let input ← parseInput raw
  let ⟨output, _proof⟩ := processData input
  IO.FS.writeFile "output.txt" (formatOutput output)
```

The proofs apply to the functional core. The imperative shell is trusted but
kept thin.

### 7.5 Lean for Software Engineers

If you are a software engineer interested in using Lean for verification, here
is a practical workflow:

1. **Write your algorithm in Lean** as a pure function.
2. **State the properties** you want to verify as theorems.
3. **Prove the theorems** using tactics.
4. **Compile and run** — the proofs are erased, so the compiled code is
   efficient.

For verifying code written in other languages (like Rust), tools like Aeneas
can translate the code into Lean for verification.

---

## 8. Mathlib

### 8.1 Overview

**Mathlib** is the mathematical library for Lean 4. It is the largest
coherent library of formalized mathematics in any proof assistant, and it is
one of the main reasons people choose Lean.

Key facts:
- **1.9+ million lines** of Lean code (as of early 2026)
- **500+ contributors**
- Covers a vast range of mathematics: algebra, analysis, number theory,
  combinatorics, category theory, topology, measure theory, probability, and
  more
- Actively maintained with multiple PRs merged daily
- Has extensive CI: every PR is tested against the full library

### 8.2 What Mathlib Contains

Here is a sampling of what Mathlib covers, organized by area:

#### Algebra
- Groups, rings, fields, modules, vector spaces
- Polynomial rings, power series
- Linear algebra: matrices, eigenvalues, determinants
- Galois theory
- Homological algebra

#### Analysis
- Real analysis: limits, continuity, derivatives, integrals
- Complex analysis
- Measure theory and Lebesgue integration
- Functional analysis: Banach spaces, Hilbert spaces
- Fourier analysis

#### Number Theory
- Divisibility, primes, GCD
- Modular arithmetic
- Quadratic reciprocity
- p-adic numbers

#### Combinatorics
- Graph theory
- Generating functions
- Binomial coefficients and identities
- Matroids

#### Category Theory
- Categories, functors, natural transformations
- Limits and colimits
- Adjunctions
- Abelian categories

#### Topology
- Topological spaces, compactness, connectedness
- Metric spaces
- Uniform spaces
- Manifolds (in progress)

#### Order Theory
- Partial orders, lattices, complete lattices
- Galois connections
- Fixed-point theorems

### 8.3 Notable Achievements

#### The Liquid Tensor Experiment

In 2020, Fields Medalist Peter Scholze challenged the formalization community
to verify a key technical result in his theory of condensed mathematics.
A team led by Johan Commelin formalized the result in Lean (starting in
Lean 3, eventually ported to Lean 4). This was completed successfully and
was considered a major milestone for interactive theorem proving.

#### Sphere Packing

The proof that the E8 lattice gives the densest sphere packing in 24
dimensions was formalized in Lean.

#### Continuous Growth

Mathlib has been growing rapidly. The porting effort from Lean 3 to Lean 4
(called "Mathlib4") was a massive community undertaking completed in 2023-2024.
Since then, new mathematics has been added at an accelerating pace.

### 8.4 Using Mathlib as a Dependency

To use Mathlib in your project:

1. **Add it to your `lakefile.toml`** (or `lakefile.lean`):

```toml
# lakefile.toml
[package]
name = "my-project"
leanOptions = [{ name = "autoImplicit", value = false }]

[[require]]
name = "mathlib"
scope = "leanprover-community"
```

Or in `lakefile.lean`:

```lean
import Lake
open Lake DSL

package «my-project» where
  leanOptions := #[⟨`autoImplicit, false⟩]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4" @ "master"
```

2. **Update dependencies**:

```bash
lake update
```

This downloads Mathlib and its transitive dependencies. Mathlib is large, so
this step downloads precompiled binaries (called "oleans") to save time.

3. **Import what you need**:

```lean
import Mathlib.Data.Nat.Basic
import Mathlib.Tactic

-- Now you have access to thousands of theorems and tactics
```

Mathlib is organized into a hierarchy of files. Common imports:

```lean
import Mathlib.Tactic              -- all tactics
import Mathlib.Data.Nat.Basic      -- basic natural number theory
import Mathlib.Data.List.Basic     -- basic list theory
import Mathlib.Data.Real.Basic     -- real numbers
import Mathlib.Algebra.Group.Basic -- group theory
import Mathlib.Analysis.SpecificLimits.Basic -- analysis limits
import Mathlib.Topology.Basic      -- topological spaces
```

### 8.5 Finding Lemmas in Mathlib

Mathlib is huge. Finding the right lemma is a skill. Here are your tools:

#### `exact?` (Search for exact matches)

```lean
-- When you do not know the name of a lemma:
example (a b : Nat) (h : a ≤ b) : a ≤ b + 1 := by
  exact?   -- Lean searches for a lemma that closes this goal
           -- and suggests: exact Nat.le_succ_of_le h
```

#### `apply?` (Search for applicable lemmas)

```lean
example (a b : Nat) : a + b = b + a := by
  apply?   -- suggests: exact Nat.add_comm a b
```

#### `rw?` (Search for rewrite lemmas)

```lean
example (a : Nat) : a + 0 = a := by
  rw?      -- suggests: rw [Nat.add_zero]
```

#### Naming Conventions

Mathlib follows consistent naming conventions. If you want a lemma about
`List.map` and `List.length`, try `List.length_map`. Common patterns:

```
Nat.add_comm          -- commutativity of addition on Nat
Nat.add_assoc         -- associativity of addition on Nat
Nat.mul_comm          -- commutativity of multiplication on Nat
List.length_append    -- length of appended lists
List.map_map          -- map composed with map
Nat.le_of_lt          -- a < b implies a ≤ b
Nat.lt_of_le_of_lt    -- a ≤ b and b < c implies a < c
```

The general pattern is: `TypeName.operation_property` or
`TypeName.relation_of_relation`.

#### Online Search

- **Loogle** (https://loogle.lean-lang.org): Search by type signature.
  Type in a pattern like `List.length (List.map _ _) = _` and it finds
  matching lemmas.
- **Moogle** (https://www.moogle.ai): Natural language search for Mathlib
  lemmas. Type "length of reversed list" and it finds the right lemma.

### 8.6 Contributing to Mathlib

Mathlib welcomes contributions. The process:

1. Read the contribution guidelines on the Mathlib4 repository.
2. Join the Lean Zulip and introduce yourself.
3. Pick an issue or propose a new addition.
4. Follow the style guide (naming conventions, documentation, etc.).
5. Submit a PR. It will be reviewed by Mathlib maintainers.

---

## 9. Tooling

### 9.1 elan: The Lean Version Manager

**elan** is to Lean what `rustup` is to Rust or `nvm` is to Node.js. It
manages Lean toolchains (compiler versions).

#### Installation

```bash
# Linux / macOS
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh

# Windows (PowerShell)
# Download and run the installer from https://github.com/leanprover/elan/releases

# After installation, restart your shell or run:
source ~/.elan/env
```

#### Basic Usage

```bash
# Check the installed version
elan --version

# List installed toolchains
elan toolchain list

# Install a specific toolchain
elan toolchain install leanprover/lean4:v4.28.0

# Set the default toolchain
elan default leanprover/lean4:stable

# Update to the latest stable version
elan update

# Show the active toolchain
elan show
```

Each Lean project specifies its toolchain in a `lean-toolchain` file:

```
leanprover/lean4:v4.28.0
```

When you enter a project directory, elan automatically switches to the
specified version. If that version is not installed, elan downloads and
installs it automatically.

### 9.2 Lake: The Build System and Package Manager

**Lake** (Lean Make) is Lean's build system and package manager. It is
bundled with the Lean toolchain.

#### Creating a New Project

```bash
# Create a new library project
lake new my-project

# Create a new executable project
lake new my-app exe

# This creates:
# my-project/
# ├── Main.lean            (entry point for executables)
# ├── MyProject.lean       (root file for libraries)
# ├── MyProject/
# │   └── Basic.lean       (a module)
# ├── lakefile.toml         (build configuration)
# └── lean-toolchain        (Lean version)
```

#### Project Configuration: `lakefile.toml`

The modern (and recommended) format for Lean project configuration:

```toml
[package]
name = "my-project"
version = "0.1.0"
keywords = ["math", "verification"]
# Lean options that apply to all files
leanOptions = [
  { name = "autoImplicit", value = false }
]

# A library target
[[lean_lib]]
name = "MyProject"
# srcDir = "." # default

# An executable target
[[lean_exe]]
name = "my-app"
root = "Main"

# Dependencies
[[require]]
name = "mathlib"
scope = "leanprover-community"

[[require]]
name = "aesop"
scope = "leanprover-community"
```

#### Project Configuration: `lakefile.lean`

The older Lean DSL format (still supported and used by many projects):

```lean
import Lake
open Lake DSL

package «my-project» where
  version := v!"0.1.0"

lean_lib «MyProject» where
  -- library configuration

@[default_target]
lean_exe «my-app» where
  root := `Main

require mathlib from git
  "https://github.com/leanprover-community/mathlib4" @ "master"
```

#### Lake Commands

```bash
# Build the project (compiles all .lean files)
lake build

# Build a specific target
lake build MyProject

# Run an executable
lake exe my-app

# Clean build artifacts
lake clean

# Update dependencies
lake update

# Fetch dependencies without building
lake update

# Start the Lean language server (used by VS Code)
lake serve

# Initialize a new project in the current directory
lake init my-project

# Get help
lake help
lake help build
```

#### How Lake Resolves Dependencies

When you run `lake build`, Lake:

1. Reads `lakefile.toml` (or `lakefile.lean`).
2. Resolves dependencies (downloading them if necessary).
3. For Mathlib, downloads pre-built `.olean` files (binary caches) to avoid
   recompiling the entire library (which would take hours).
4. Compiles your project's `.lean` files into `.olean` files.
5. If there are executable targets, links them into native binaries.

### 9.3 VS Code Extension

The **Lean 4 VS Code extension** is the primary IDE for Lean development. It
provides:

- **Real-time type checking**: Errors and warnings appear as you type.
- **Infoview panel**: Shows the current proof state (goals and hypotheses).
- **Go to definition**: Jump to the definition of any symbol.
- **Hover information**: Hover over any expression to see its type.
- **Code completion**: Autocomplete for tactics, theorems, and definitions.
- **Semantic highlighting**: Different colors for types, terms, tactics, etc.
- **Unicode input**: Type `\alpha` and it converts to `α`, type `\to` for `→`,
  `\forall` for `∀`, `\exists` for `∃`, `\and` for `∧`, `\or` for `∨`, etc.

#### Installation

1. Install VS Code: https://code.visualstudio.com
2. Install the extension: search for `leanprover.lean4` in the Extensions
   marketplace, or run:
   ```
   code --install-extension leanprover.lean4
   ```
3. Open a Lean project folder. The extension starts automatically.

#### Key Keybindings and Features

```
Ctrl+Shift+Enter (Cmd+Shift+Enter on Mac):
    Open the Lean Infoview panel

Hover over any term:
    Shows its type

Ctrl+Click (Cmd+Click on Mac):
    Go to definition

Type \ followed by a name:
    Unicode input (e.g., \forall → ∀, \lam → λ, \in → ∈)
```

#### Common Unicode Input Sequences

```
\to      →    (function arrow / implication)
\forall  ∀    (universal quantifier)
\exists  ∃    (existential quantifier)
\and     ∧    (logical and)
\or      ∨    (logical or)
\not     ¬    (negation)
\ne      ≠    (not equal)
\le      ≤    (less than or equal)
\ge      ≥    (greater than or equal)
\sub     ⊂    (subset)
\in      ∈    (element of)
\notin   ∉    (not element of)
\union   ∪    (set union)
\inter   ∩    (set intersection)
\alpha   α    \beta    β    \gamma   γ
\delta   δ    \epsilon ε    \lambda  λ
\mu      μ    \sigma   σ    \phi     φ
\langle  ⟨    \rangle  ⟩    (anonymous constructor brackets)
\x       ×    (product type)
\N       ℕ    (shorthand for Nat in some contexts)
\Z       ℤ    \Q       ℚ    \R       ℝ
```

### 9.4 The Language Server Protocol (LSP)

Lean 4 implements the Language Server Protocol, which means any editor that
supports LSP can work with Lean. The server is started with:

```bash
lake serve
```

This is usually handled automatically by the VS Code extension. But if you use
Neovim, Emacs, or another editor, you can configure it to use `lake serve` as
the LSP server.

#### Neovim Setup

If you prefer Neovim, you can use `nvim-lspconfig` with the `lean4` server:

```lua
-- In your Neovim config (e.g., init.lua)
require('lspconfig').lean4.setup{}
```

There is also a dedicated plugin: `Julian/lean.nvim`.

#### Emacs Setup

Use `lean4-mode` for Emacs:
```
M-x package-install lean4-mode
```

### 9.5 Other Tools

#### `#check`, `#eval`, `#print` (Interactive Commands)

These are not standalone tools but Lean commands you write in `.lean` files:

```lean
#check Nat.add       -- shows the type
#eval 2 + 3          -- evaluates and shows the result
#print Nat.add       -- shows the full definition
#print axioms myThm  -- shows which axioms a theorem depends on
```

#### `lake env printPaths`

Shows the Lean search paths for the current project:

```bash
lake env printPaths
```

#### Documentation Generation

Lean can generate HTML documentation from your code:

```bash
lake build MyProject:docs
```

Mathlib's documentation is generated this way and hosted at
https://leanprover-community.github.io/mathlib4_docs/

---

## 10. Installation and First Project

This section walks you through installing Lean 4 and creating your first
project, step by step.

### 10.1 Install elan and Lean

#### macOS / Linux

```bash
# Install elan (this also installs the latest stable Lean)
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh

# Follow the prompts (press Enter to accept defaults)

# Restart your terminal, or run:
source ~/.elan/env

# Verify installation
lean --version
# Should print something like: leanprover/lean4:v4.28.0

lake --version
# Should print the Lake version
```

#### Windows

1. Install **Git for Windows** if you do not already have it.
2. Install **Visual Studio Build Tools** (needed for the C compiler).
3. Download the elan installer from
   https://github.com/leanprover/elan/releases and run it.
4. Restart your terminal.
5. Verify: `lean --version`

#### Verify Everything Works

```bash
lean --version
lake --version
elan show
```

### 10.2 Create Your First Project

```bash
# Create a new project called "my-first-lean"
lake new my-first-lean

# Enter the project directory
cd my-first-lean

# See what was created
ls -la
# Output:
# lakefile.toml
# lean-toolchain
# Main.lean
# MyFirstLean.lean
# MyFirstLean/
#   Basic.lean
```

### 10.3 Explore the Generated Files

#### `lean-toolchain`

```
leanprover/lean4:v4.28.0
```

Specifies the exact Lean version this project uses.

#### `lakefile.toml`

```toml
[package]
name = "my-first-lean"
version = "0.1.0"

[[lean_lib]]
name = "MyFirstLean"

[[lean_exe]]
name = "my-first-lean"
root = "Main"
```

#### `Main.lean`

```lean
import MyFirstLean

def main : IO Unit :=
  IO.println s!"Hello, {hello}!"
```

#### `MyFirstLean/Basic.lean`

```lean
def hello := "world"
```

### 10.4 Build and Run

```bash
# Build the project
lake build

# Run the executable
lake exe my-first-lean
# Output: Hello, world!
```

### 10.5 Write Your First Theorem

Open `MyFirstLean/Basic.lean` in VS Code and replace its contents with:

```lean
-- Our first definitions
def double (n : Nat) : Nat := n + n

-- Our first theorem: doubling a number gives an even result
-- (We define "even" as "divisible by 2")
theorem double_is_even (n : Nat) : ∃ k, double n = 2 * k := by
  use n            -- the witness is n itself
  unfold double    -- expand the definition of double
  ring             -- solve the arithmetic: n + n = 2 * n

-- A simple arithmetic theorem
theorem add_is_commutative (a b : Nat) : a + b = b + a := by
  omega

-- Prove that zero is a left identity for addition
theorem zero_add_eq (n : Nat) : 0 + n = n := by
  rfl   -- true by definition

-- A slightly harder theorem: if n > 0, then n has a predecessor
theorem has_pred (n : Nat) (h : n > 0) : ∃ m, n = m + 1 := by
  cases n with
  | zero => omega          -- n = 0 contradicts n > 0
  | succ m => exact ⟨m, rfl⟩  -- n = m + 1, so m is the predecessor

-- Let us prove something about our double function
theorem double_pos (n : Nat) (h : n > 0) : double n > 0 := by
  unfold double
  omega

-- A proof about lists
theorem length_repeat (a : α) (n : Nat) :
    (List.replicate n a).length = n := by
  induction n with
  | zero => simp [List.replicate]
  | succ n ih => simp [List.replicate, ih]
```

Save the file. If you have the VS Code extension installed, you will see
green checkmarks (no errors) next to each theorem. The Infoview panel will
show the proof state at your cursor position.

### 10.6 Build and Verify

```bash
lake build
```

If the build succeeds with no errors, all your proofs are verified. Lean's
type checker has mechanically confirmed that every theorem is correct.

### 10.7 Interactive Exploration

Here are things to try in VS Code:

1. **Place your cursor inside a `by` block** and look at the Infoview panel.
   You will see the current goals and hypotheses.

2. **Delete a tactic** (e.g., remove `omega` from `add_is_commutative`) and
   see the error message. It will show you the unsolved goal.

3. **Try `#check`**:
   ```lean
   #check Nat.add_comm    -- see the type of a library lemma
   #check @List.map        -- see the full signature with implicit args
   ```

4. **Try `#eval`**:
   ```lean
   #eval double 21         -- 42
   #eval List.replicate 3 "hello"  -- ["hello", "hello", "hello"]
   ```

5. **Try `example`** to experiment with proofs without naming them:
   ```lean
   example : 2 + 2 = 4 := by rfl
   example : ∀ n : Nat, n ≤ n := by intro n; omega
   ```

### 10.8 Adding Mathlib

If you want to use Mathlib's extensive library, edit your `lakefile.toml`:

```toml
[package]
name = "my-first-lean"
version = "0.1.0"
leanOptions = [{ name = "autoImplicit", value = false }]

[[lean_lib]]
name = "MyFirstLean"

[[lean_exe]]
name = "my-first-lean"
root = "Main"

[[require]]
name = "mathlib"
scope = "leanprover-community"
```

Then run:

```bash
lake update
lake build
```

The first `lake update` will download Mathlib (this can take a while, but it
downloads precompiled binaries rather than compiling from source).

Now you can import Mathlib modules:

```lean
import Mathlib.Tactic
import Mathlib.Data.Nat.Prime.Basic

-- Now you have access to all Mathlib tactics and theorems
example : Nat.Prime 7 := by decide

example (P Q : Prop) : P ∨ ¬P := by tauto
```

---

## 11. Resources

### 11.1 Official Resources

| Resource | URL |
|----------|-----|
| **Lean Homepage** | https://lean-lang.org |
| **Lean 4 GitHub** | https://github.com/leanprover/lean4 |
| **Lean FRO** | https://lean-fro.org |
| **Lean Toolchain Releases** | https://github.com/leanprover/lean4/releases |
| **Lean Documentation** | https://lean-lang.org/documentation/ |
| **Reservoir** (Package Registry) | https://reservoir.lean-lang.org |

### 11.2 Books

These are all freely available online:

#### Theorem Proving in Lean 4

- **URL**: https://lean-lang.org/theorem_proving_in_lean4/
- **Audience**: Beginners to Lean and theorem proving
- **Covers**: The foundations of Lean's type theory, propositions and proofs,
  tactics, inductive types, structures, and typeclasses
- **Recommended as**: Your first book. Read it cover to cover.

#### Functional Programming in Lean

- **URL**: https://lean-lang.org/functional_programming_in_lean/
- **Audience**: Programmers learning Lean as a programming language
- **Covers**: Basic types, polymorphism, typeclasses, monads, IO, do-notation,
  arrays, dependent types for programming, macros
- **Recommended for**: If you want to write programs (not just proofs) in Lean

#### Mathematics in Lean

- **URL**: https://leanprover-community.github.io/mathematics_in_lean/
- **Audience**: Mathematicians learning to formalize math
- **Covers**: Logic, sets, functions, order, algebra, topology, analysis —
  all formalized in Lean with extensive exercises
- **Recommended for**: If your goal is formalizing mathematics

#### Metaprogramming in Lean 4

- **URL**: https://leanprover-community.github.io/lean4-metaprogramming-book/
- **Audience**: Advanced users who want to write tactics, macros, and
  elaborators
- **Covers**: The Lean metaprogramming API, `Syntax`, `Expr`, `MetaM`,
  `TacticM`, custom tactics, custom elaborators
- **Recommended for**: Once you are comfortable with basic Lean and want to
  extend it

### 11.3 Community

| Resource | URL |
|----------|-----|
| **Lean Zulip Chat** | https://leanprover.zulipchat.com |
| **Lean Community Site** | https://leanprover-community.github.io |
| **Mathlib4 GitHub** | https://github.com/leanprover-community/mathlib4 |
| **Mathlib Documentation** | https://leanprover-community.github.io/mathlib4_docs/ |
| **Lean Zulip "new members" stream** | Best place to ask beginner questions |

The Lean Zulip is exceptionally friendly and responsive. It is the primary
communication channel for the Lean community. If you are stuck, ask there.

### 11.4 Search Tools

| Tool | URL | Purpose |
|------|-----|---------|
| **Loogle** | https://loogle.lean-lang.org | Search Mathlib by type signature |
| **Moogle** | https://www.moogle.ai | Natural language search for Lean/Mathlib |

### 11.5 Practice and Learning

- **Lean Game Server** (https://adam.math.hhu.de/): Interactive browser-based
  games that teach you Lean through progressively harder proof challenges.
  Includes the Natural Number Game (highly recommended for beginners).

- **Lean 4 exercises**: Many books above include exercises with solutions.

- **Mathlib contributions**: Once comfortable, contributing small PRs to
  Mathlib is an excellent way to improve.

### 11.6 Related Tools

| Tool | Description |
|------|-------------|
| **Aeneas** | Translates Rust (MIR) to Lean for verification |
| **LLMlean** | Integration of LLMs with Lean for proof automation |
| **doc-gen4** | Documentation generator for Lean 4 projects |
| **std4 / Batteries** | Standard library additions (many now merged into core Lean or Mathlib) |

---

## Appendix A: Quick Reference — Tactic Cheat Sheet

```
INTRODUCING HYPOTHESES
  intro x          — introduce a universally quantified variable or hypothesis
  intros x y z     — introduce multiple at once
  rintro ⟨a, b⟩   — introduce with pattern matching (Mathlib)

CLOSING GOALS
  exact e          — provide an exact proof term
  rfl              — prove a = a
  trivial          — try rfl, assumption, decide
  assumption       — find a matching hypothesis
  contradiction    — find contradictory hypotheses
  decide           — decide a decidable proposition by computation
  native_decide    — like decide but uses native code (faster)
  omega            — Presburger arithmetic (linear nat/int goals)
  linarith         — linear arithmetic over ordered fields
  norm_num         — numerical normalization
  ring             — polynomial ring equalities
  simp             — simplification using simp lemmas
  tauto            — propositional tautologies (Mathlib)
  grind            — general-purpose automation

TRANSFORMING GOALS
  apply f          — backward reasoning: goal Q becomes P if f : P → Q
  refine e         — like apply but with holes (?_)
  rw [h]           — rewrite goal using equation h (left to right)
  rw [← h]         — rewrite using h right to left
  rw [h] at hyp    — rewrite in a hypothesis
  simp [h1, h2]    — simplify using specific lemmas
  simp_all         — simplify goal and all hypotheses
  push_neg         — push negations inward (Mathlib)
  norm_cast        — simplify coercions between number types
  field_simp       — clear denominators (Mathlib)
  unfold f         — unfold a definition

STRUCTURING PROOFS
  constructor      — apply the constructor (split ∧, ∃, ↔)
  left / right     — choose a side of ∨
  use e            — provide a witness for ∃
  ext              — prove equality by extensionality
  funext x         — function extensionality
  congr n          — prove f a = f b by proving a = b
  exfalso          — change any goal to False
  by_contra h      — proof by contradiction

CASE ANALYSIS AND INDUCTION
  cases h          — case split on h
  rcases h with .. — recursive case split (Mathlib)
  obtain ⟨x, hx⟩ := h  — destructure an existential (Mathlib)
  induction n with ..   — induction on n

INTERMEDIATE STEPS
  have h : P := ..   — prove an intermediate result
  let x := e         — introduce a local definition
  suffices h : P by ..  — state what suffices
  show P             — clarify the goal type
  calc               — equational/inequality chain

SEARCHING
  exact?           — search for a lemma that closes the goal
  apply?           — search for a lemma that applies
  rw?              — search for a rewrite lemma
  simp?            — show which simp lemmas were used
```

---

## Appendix B: Quick Reference — Common Symbols

```
LOGIC
  →     implication / function type         \to
  ↔     if and only if                      \iff
  ∧     and                                 \and
  ∨     or                                  \or
  ¬     not                                 \not
  ∀     for all                             \forall
  ∃     exists                              \exists
  ⊢     turnstile (proves / entails)        \vdash
  ⊥     false / bottom                      \bot
  ⊤     true / top                          \top

EQUALITY AND ORDER
  =     equality
  ≠     not equal                           \ne
  ≤     less or equal                       \le
  ≥     greater or equal                    \ge
  <     less than
  >     greater than

TYPES AND TERMS
  ×     product type                        \x or \times
  ⊕     sum type                            \oplus
  ⟨⟩    anonymous constructor               \langle \rangle
  λ     lambda                              \lam
  Π     Pi type                             \Pi
  Σ     Sigma type                          \Sigma
  α β γ type variables                      \alpha \beta \gamma

SETS
  ∈     element of                          \in
  ∉     not element of                      \notin
  ⊂     subset                              \sub
  ⊆     subset or equal                     \subseteq
  ∪     union                               \union
  ∩     intersection                        \inter
  ∅     empty set                           \empty

NUMBERS
  ℕ     natural numbers                     \N
  ℤ     integers                            \Z
  ℚ     rationals                           \Q
  ℝ     reals                               \R
  ℂ     complex numbers                     \C
```

---

## Appendix C: Common Patterns and Idioms

### Pattern: "Suffices to show"

```lean
theorem my_theorem (n : Nat) (h : n > 10) : n > 5 := by
  suffices n > 7 by omega
  omega
```

### Pattern: "Chain of equalities"

```lean
theorem chain (a b c d : Nat) (h1 : a = b + 1) (h2 : b = c) (h3 : c = d - 1) :
    a = d := by
  calc a = b + 1 := h1
    _ = c + 1 := by rw [h2]
    _ = (d - 1) + 1 := by rw [h3]
    _ = d := by omega
```

### Pattern: "Prove two goals from a conjunction"

```lean
theorem from_conj (P Q : Prop) (h : P ∧ Q) : Q ∧ P := by
  exact ⟨h.2, h.1⟩
```

### Pattern: "Case split on decidable property"

```lean
theorem case_split (n : Nat) : n * n ≥ n := by
  rcases Nat.eq_or_gt_of_le (Nat.zero_le n) with rfl | h
  · simp
  · exact Nat.le_mul_of_pos_left n h
```

### Pattern: "Induction on a list"

```lean
theorem map_length (f : α → β) (xs : List α) :
    (xs.map f).length = xs.length := by
  induction xs with
  | nil => simp
  | cons x xs ih => simp [ih]
```

### Pattern: "Unfold, simplify, close"

```lean
def myFunction (n : Nat) : Nat := n * 2 + 1

theorem myFunction_pos (n : Nat) : myFunction n ≥ 1 := by
  unfold myFunction
  omega
```

### Pattern: "Use a local helper lemma"

```lean
theorem main_result (n : Nat) : n ^ 2 + n ≥ n := by
  have h : n ^ 2 ≥ 0 := Nat.zero_le _
  omega
```

### Pattern: "Existential from computation"

```lean
theorem constructive_exists : ∃ n : Nat, n > 100 ∧ n < 200 ∧ n % 7 = 0 := by
  exact ⟨105, by omega, by omega, by omega⟩
```

---

## Appendix D: Glossary

**Axiom**: A statement accepted without proof. Lean has a small number of
axioms (propositional extensionality, quotient types, classical choice).

**CIC**: Calculus of Inductive Constructions. The type theory underlying Lean.

**Constructive**: A form of mathematics where "existence" means you can
produce a witness. Lean's core is constructive, but classical axioms are
available.

**Curry-Howard Correspondence**: The observation that types correspond to
propositions and programs correspond to proofs.

**Decidable**: A proposition is decidable if there is an algorithm that
determines whether it is true or false. `DecidableEq Nat` says equality of
natural numbers is decidable.

**Definitional Equality**: Two terms are definitionally equal if they reduce
to the same normal form by computation. Checked automatically by the kernel.

**Dependent Type**: A type that depends on a value. Example: `Vector α n`
depends on the natural number `n`.

**elan**: Lean's toolchain version manager (analogous to `rustup`).

**Elaboration**: The process by which Lean fills in implicit arguments,
resolves typeclasses, evaluates tactics, and type-checks your code.

**Elimination**: Using a value of an inductive type (via pattern matching
or recursion). The dual of introduction (construction).

**Functional Programming**: Programming with pure functions, immutable data,
and expressions rather than statements.

**Goal**: In tactic mode, a proposition that remains to be proved.

**Hypothesis**: In tactic mode, an assumption available in the local context.

**Impredicativity**: The property of `Prop` that universal quantification over
all propositions is itself a proposition.

**Inductive Type**: A type defined by its constructors. Lean's primary
mechanism for defining data types and logical connectives.

**Introduction**: Constructing a value of a type (using a constructor). The
dual of elimination (using/destructuring).

**Kernel**: The small, trusted core of Lean that checks proofs. If the kernel
accepts a proof, it is correct (assuming the kernel itself is correct).

**Lake**: Lean's build system and package manager (analogous to `cargo`).

**Lambda**: An anonymous function. Written `fun x => body` in Lean 4.

**LCNF**: Lean Compiler Normal Form. An intermediate representation used by
the Lean compiler.

**Lean FRO**: The Lean Focused Research Organization, a nonprofit supporting
Lean's development.

**Mathlib**: The mathematical library for Lean 4. Over 1.9 million lines of
formalized mathematics.

**Monad**: A typeclass that provides a way to sequence computations with
effects. Examples: `IO`, `Option`, `Except`.

**Olean**: A compiled Lean file (`.olean`). Contains the checked definitions
in binary format. Analogous to `.o` or `.pyc` files.

**Pi Type**: A dependent function type. Written `(a : α) → β a`. Non-dependent
functions `α → β` are a special case.

**Proof Irrelevance**: The principle that any two proofs of the same
proposition are equal.

**Prop**: The type of propositions (Sort 0). Types in `Prop` are proof-
irrelevant and impredicative.

**Propositional Equality**: Equality that requires a proof (potentially using
induction, lemmas, or tactics). Contrasted with definitional equality.

**Recursor**: The elimination principle automatically generated for each
inductive type. For `Nat`, this is the principle of mathematical induction.

**Sigma Type**: A dependent pair type. Written `(a : α) × β a`.

**Tactic**: A command in tactic mode that transforms the proof state.
Examples: `intro`, `apply`, `rw`, `simp`.

**Term Mode**: Writing proofs directly as expressions, without tactics.

**Type**: A classifier for values. In Lean, types are first-class values that
themselves have types (forming the universe hierarchy).

**Typeclass**: An interface that types can implement. Used for ad-hoc
polymorphism (operator overloading, etc.). Similar to Haskell typeclasses or
Rust traits.

**Universe**: A level in the type hierarchy. `Prop` (Sort 0), `Type` (Sort 1),
`Type 1` (Sort 2), etc.

**Well-Founded Recursion**: Recursion that is guaranteed to terminate because
some measure decreases at each step. Used when structural recursion is not
enough.

---

## Appendix E: Comparison with Other Proof Assistants

| Feature | Lean 4 | Coq | Agda | Isabelle/HOL |
|---------|--------|-----|------|--------------|
| Type Theory | CIC + quotients | CIC | Martin-Lof + extensions | Simple type theory (HOL) |
| Programming Language | Yes (compiled, efficient) | Limited (extraction) | Limited | Limited (code generation) |
| Self-Hosted | Yes | No (OCaml) | No (Haskell) | No (ML, Scala) |
| Tactic Language | Lean (metaprogramming) | Ltac2 / OCaml | No tactics (term mode) | Isar / Eisbach |
| Main Math Library | Mathlib | MathComp / Std | agda-stdlib | AFP |
| IDE | VS Code (excellent) | VS Code / CoqIDE | Emacs | jEdit |
| Classical Logic | Default (via axiom) | Optional | Not default | Built-in |
| Computation | Native (compiles to C) | Slow (kernel reduction) | Moderate | Code generation to ML/Haskell |
| Community Size | Large and growing rapidly | Large, established | Small | Medium, established |

---

*This document was written as a reference for the rust-lean-aeneas project.*
*Last updated: 2026-03-22.*

---

[← Back to README](README.md) | [Aeneas Reference →](AENEAS.md) | [Start Tutorial 01 →](tutorials/01-setup-hello-proof/README.md)
