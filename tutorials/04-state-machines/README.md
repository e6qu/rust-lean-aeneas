[← Previous: Tutorial 03](../03-infix-calculator/README.md) | [Index](../README.md) | [Next: Tutorial 05 →](../05-message-protocol/README.md)

# Tutorial 04: State Machines

> **Rust trait** → **Lean structure** → **safety invariants** → **generic proofs**

State machines are everywhere: network protocols, embedded controllers, access
control systems, traffic lights.  When these systems are safety-critical,
"testing a few cases" is not enough — we want *mathematical proof* that bad
states are unreachable.

In this tutorial we:

1. Define a **generic `StateMachine` trait** in Rust.
2. Implement two concrete machines — a **Door Lock** and a **Traffic Light**.
3. Translate the code to Lean (simulating Aeneas output).
4. Write **formal specifications** (reachability, invariants).
5. Prove **safety properties** for both machines.
6. Prove a **generic invariant induction theorem** that works for *any*
   state machine — the key result reused throughout later tutorials.

## Prerequisites

Complete Tutorials 01–03.  You should be comfortable with:

- Lean 4 basics (types, pattern matching, `simp`)
- Monadic reasoning for Aeneas-translated code
- Structural induction

## Project Structure

```
04-state-machines/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs        # StateMachine trait, DoorLock, TrafficLight, run_machine
│       └── main.rs       # Interactive CLI simulator
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── StateMachine.lean                  # top-level imports
    └── StateMachine/
        ├── Types.lean                     # simulated Aeneas: types
        ├── Funs.lean                      # simulated Aeneas: functions
        ├── Spec.lean                      # hand-written specs
        ├── DoorLockProofs.lean            # proofs about the door lock
        ├── TrafficLightProofs.lean        # proofs about the traffic light
        └── GenericProofs.lean             # generic invariant_induction theorem
```

## 1. Concept: Traits as Lean Structures

Aeneas translates a Rust trait into a Lean **structure** (a record of
functions).  Each `impl` block becomes an instance of that structure.

| Rust | Lean |
|------|------|
| `trait StateMachine { type State; ... fn transition(...); }` | `structure StateMachine (Self State Event Action : Type) where transition : State → Event → State × List Action` |
| `impl StateMachine for DoorLock { ... }` | `def DoorLockMachine : StateMachine DoorLock DoorLockState DoorEvent DoorAction := { transition := doorlock_transition }` |

The `Self` parameter is a marker type that distinguishes different
implementations.  Associated types become explicit type parameters.

## 2. Rust: The StateMachine Trait

```rust
pub trait StateMachine {
    type State: Clone;
    type Event;
    type Action: Clone;

    fn transition(state: &Self::State, event: &Self::Event)
        -> (Self::State, Vec<Self::Action>);
}
```

Key design choices for Aeneas compatibility:

- **No `&self` receiver** — the trait has no runtime data; `DoorLock` and
  `TrafficLight` are zero-sized marker types.
- **`Vec<Action>` instead of a single action** — some transitions produce
  multiple observable outputs.
- **`Clone` bounds** — needed for the generic `run_machine` function.

## 3. Rust: The Door Lock

The door lock has three states, three event types, and four action types.

### States

| State | Meaning |
|-------|---------|
| `Locked` | Door is locked; code entry is accepted |
| `Unlocked` | Correct code was entered; handle can open the door |
| `Alarmed` | Three wrong codes; alarm is sounding |

### Transition Logic

| Current State | Event | Condition | Next State | Action |
|--------------|-------|-----------|------------|--------|
| Locked | EnterCode(c) | c == 1234 | Unlocked | Beep |
| Locked | EnterCode(c) | c != 1234, attempts+1 < 3 | Locked (attempts+1) | Beep |
| Locked | EnterCode(c) | c != 1234, attempts+1 >= 3 | Alarmed | SoundAlarm |
| Unlocked | TurnHandle | — | Unlocked | OpenDoor |
| Locked | TurnHandle | — | Locked | Beep |
| Alarmed | TurnHandle | — | Alarmed | SoundAlarm |
| *any* | Reset | — | Locked (attempts=0) | Lock |
| Unlocked | EnterCode(_) | — | Unlocked | Beep |
| Alarmed | EnterCode(_) | — | Alarmed | SoundAlarm |

The `wrong_attempts` counter tracks consecutive wrong code entries.  It resets
to 0 on a correct code or a Reset event.

## 4. Rust: The Traffic Light

The traffic light models a 4-way intersection with North-South and East-West
directions.

### The 4-Phase Cycle

```
Phase 0:  NS=Green   EW=Red     ─Timer─→
Phase 1:  NS=Yellow  EW=Red     ─Timer─→
Phase 2:  NS=Red     EW=Green   ─Timer─→
Phase 3:  NS=Red     EW=Yellow  ─Timer─→  (back to Phase 0)
```

The critical safety property: **NS and EW are never both Green.**

The transition function pattern-matches on `(ns_light, ew_light)` and produces
the next phase.  Any state not in the 4-phase cycle is left unchanged (a
defensive catch-all).

## 5. Rust: Generic run_machine

```rust
pub fn run_machine<M: StateMachine>(
    initial: &M::State,
    events: &[M::Event],
) -> (M::State, Vec<M::Action>) {
    let mut state = initial.clone();
    let mut all_actions: Vec<M::Action> = Vec::new();
    let mut i: usize = 0;
    while i < events.len() {
        let (next_state, actions) = M::transition(&state, &events[i]);
        state = next_state;
        let mut j: usize = 0;
        while j < actions.len() {
            all_actions.push(actions[j].clone());
            j += 1;
        }
        i += 1;
    }
    (state, all_actions)
}
```

This uses explicit `while` loops (no iterators, no `for`) so that Aeneas can
translate it into a recursive Lean function.  The inner loop appends actions
one by one instead of using `.extend()`.

## 6. Translation to Lean

### Types.lean (simulated Aeneas output)

Each Rust `enum` becomes a Lean `inductive`:

```lean
inductive DoorState where
  | Locked
  | Unlocked
  | Alarmed
```

Each Rust `struct` becomes a Lean `structure`:

```lean
structure DoorLockState where
  door : DoorState
  wrong_attempts : UInt32
```

The trait itself becomes a parameterised structure:

```lean
structure StateMachine (Self State Event Action : Type) where
  transition : State → Event → (State × List Action)
```

### Funs.lean (simulated Aeneas output)

Trait method implementations become standalone functions.  The `match`
structure mirrors the Rust code exactly:

```lean
def doorlock_transition (state : DoorLockState) (event : DoorEvent)
    : DoorLockState × List DoorAction :=
  match event with
  | DoorEvent.EnterCode code =>
    match state.door with
    | DoorState.Locked =>
      if code == CORRECT_CODE then ...
      else ...
    | DoorState.Unlocked => ...
    | DoorState.Alarmed  => ...
  | DoorEvent.TurnHandle => ...
  | DoorEvent.Reset => ...
```

The generic `run_machine` becomes a recursive function over the event list:

```lean
def run_machine (machine : StateMachine Self State Event Action)
    (state : State) (events : List Event) : State × List Action :=
  match events with
  | [] => (state, [])
  | e :: es =>
    let (next_state, actions) := machine.transition state e
    let (final_state, rest_actions) := run_machine machine next_state es
    (final_state, actions ++ rest_actions)
```

## 7. Concept: State Space Reasoning

When we verify a state machine, we reason about its **state space** — the set
of all states the machine can reach from its initial state.

| Concept | Definition |
|---------|-----------|
| **Reachable state** | A state that can be reached by zero or more transitions from the initial state |
| **Safety property** | "Bad thing never happens" — a property that holds for all reachable states |
| **Liveness property** | "Good thing eventually happens" — the system makes progress |
| **Invariant** | A property that (1) holds initially and (2) is preserved by every transition |

The key insight: **every invariant is a safety property**.  If we can show that
P is an invariant, then P holds for *every* reachable state.

## 8. Specifications in Lean

### Reachable (inductive predicate)

```lean
inductive Reachable (trans : State → Event → State × List α) (init : State)
    : State → Prop where
  | init_reach : Reachable trans init init
  | step {s s'} {e} : Reachable trans init s →
      (trans s e).1 = s' → Reachable trans init s'
```

This says: the initial state is reachable, and if `s` is reachable and we take
one transition, the result is also reachable.

### IsInvariant

```lean
def IsInvariant (P : State → Prop) (trans : ...) (init : State) : Prop :=
  P init ∧ ∀ s e, P s → P (trans s e).1
```

An invariant is a property that holds initially and is preserved by every
possible transition.

### multi_step

```lean
def multi_step (trans : ...) (state : State) : List Event → State
  | [] => state
  | e :: es => multi_step trans (trans state e).1 es
```

A pure specification function that folds transitions over a list of events.
Useful for stating theorems like "after 4 Timer events, the traffic light
returns to its original state."

## 9. Proof: Reset Liveness (Warmup)

Our first proof is simple: Reset always produces a Locked state.

```lean
theorem reset_goes_to_locked (s : DoorLockState) :
    (doorlock_transition s DoorEvent.Reset).1.door = DoorState.Locked ∧
    (doorlock_transition s DoorEvent.Reset).1.wrong_attempts = 0 := by
  simp [doorlock_transition]
```

This is a **direct computation** proof.  We unfold the definition of
`doorlock_transition` for the Reset case, and `simp` handles the rest.  No
induction needed — this is a property of a single transition, not of reachable
states.

## 10. Proof: Correct Code Unlocks

```lean
theorem correct_code_unlocks (s : DoorLockState)
    (h : s.door = DoorState.Locked) :
    (doorlock_transition s (DoorEvent.EnterCode CORRECT_CODE)).1.door
      = DoorState.Unlocked := by
  simp [doorlock_transition, h, CORRECT_CODE]
```

Again a single-step computation.  The hypothesis `h : s.door = Locked` is
needed because code entry only unlocks from the Locked state.

## 11. Proof: Alarmed Safety

This is our first **reachability** proof.  We show that if a state is Alarmed,
then `wrong_attempts >= 3`.

```lean
theorem alarmed_needs_three_wrong (s : DoorLockState)
    (hreach : Reachable doorlock_transition door_initial s)
    (halarmed : s.door = DoorState.Alarmed) :
    s.wrong_attempts >= 3
```

**Proof strategy:** Induction on `Reachable`.

- **Base case:** The initial state has `door = Locked`, which contradicts
  `door = Alarmed`.
- **Step case:** We case-split on the event and previous door state.  The only
  way to reach `Alarmed` is via `EnterCode` with a wrong code when
  `attempts + 1 >= 3`.  In that case, `wrong_attempts` is set to `attempts`
  which is `>= 3`.

This proof exercises deep case analysis — splitting on every combination of
event and state.

## 12. Proof: Traffic Mutual Exclusion

The key safety property: NS and EW are never both Green.

```lean
def valid_traffic_state (s : TrafficState) : Prop :=
  ¬ (s.ns_light = LightColor.Green ∧ s.ew_light = LightColor.Green)
```

We first prove that transitions preserve validity:

```lean
theorem traffic_transition_preserves_valid (s : TrafficState) (e : TrafficEvent)
    (h : valid_traffic_state s) :
    valid_traffic_state (traffic_transition s e).1
```

Then we lift this to all reachable states by induction on `Reachable`.

## 13. Concept: The `decide` Tactic

For types with `DecidableEq` (which Lean can derive for our enums), the
`decide` tactic can automatically prove or disprove equalities.  Combined with
`simp` and `cases`, it lets us exhaustively check all combinations of enum
values.

In the traffic light proof, `cases s.ns_light <;> cases s.ew_light <;> simp_all`
checks all 3 x 3 = 9 combinations of (ns_light, ew_light) and verifies the
property for each.

This "finite state enumeration" strategy is powerful for small state spaces.

## 14. Concept: Generic Proofs — invariant_induction

The crown jewel of this tutorial is the **generic** invariant induction theorem:

```lean
theorem invariant_induction
    {trans : State → Event → State × List α}
    {init : State}
    {P : State → Prop}
    (hinv : IsInvariant P trans init)
    (hreach : Reachable trans init s) :
    P s
```

This says: if P is an invariant and s is reachable, then P s.  It works for
**any** state machine — not just DoorLock or TrafficLight.

**Proof:** Induction on `Reachable`.
- Base case: `IsInvariant` gives us `P init`.
- Step case: `IsInvariant` gives us preservation, and the induction hypothesis
  gives us `P` on the previous state.

### Instantiation for Traffic Light

```lean
theorem traffic_valid_is_invariant :
    IsInvariant valid_traffic_state traffic_transition traffic_initial := by
  constructor
  · simp [valid_traffic_state, traffic_initial]
  · intro s e hs; exact traffic_transition_preserves_valid s e hs

theorem traffic_invariant_via_generic (s : TrafficState)
    (hreach : Reachable traffic_transition traffic_initial s) :
    valid_traffic_state s :=
  invariant_induction traffic_valid_is_invariant hreach
```

This demonstrates the pattern:
1. **Prove** that your property is an `IsInvariant`.
2. **Apply** `invariant_induction` to get the safety property for free.

This is exactly the pattern we reuse in Tutorials 07–11 for agent-based systems.

## 15. Running the Code

### Rust

```bash
cd rust
cargo test          # run all tests
cargo run           # interactive CLI
```

The CLI lets you pick a machine (Door Lock or Traffic Light) and enter events
interactively to see transitions.

### Lean

```bash
cd lean
lake build          # type-check all proofs
```

## 16. Exercises

1. **Timeout event**: Add a `Timeout` event to the DoorLock.  If the door has
   been Unlocked for too long (track a timer in the state), it should
   automatically re-lock.  Prove that `Reset` still goes to Locked.

2. **Pedestrian crossing**: Add a `PedestrianRequest` event to the
   TrafficLight.  It should force a transition to all-Red.  Prove that the
   "never both Green" invariant still holds.

3. **Generic cycle theorem**: State and prove a generic theorem that if a
   state machine's state space is finite and every state has a successor, then
   `multi_step` eventually revisits a state (pigeonhole principle).

4. **Composition**: Define a `compose` function that runs two state machines
   in parallel.  Prove that if both machines have an invariant, the composed
   machine preserves the conjunction of both invariants.

## 17. What's Next

In **Tutorial 05** we move to data structures — implementing a verified
stack and queue.  The state machine patterns you learned here
(invariants, reachability, generic proofs) will reappear when we verify
data structure invariants like "the stack size is always non-negative" and
"popping from a non-empty stack succeeds."

The `invariant_induction` theorem from this tutorial becomes a workhorse:
instead of re-proving induction on reachability for every new system, we
package the pattern once and instantiate it everywhere.

---

[← Previous: Tutorial 03](../03-infix-calculator/README.md) | [Index](../README.md) | [Next: Tutorial 05 →](../05-message-protocol/README.md)
