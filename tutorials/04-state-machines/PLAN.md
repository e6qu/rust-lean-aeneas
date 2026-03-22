# Tutorial 04: State Machines

## Goal

Implement a generic `StateMachine` trait with two concrete instances (DoorLock and TrafficLight); prove safety properties, liveness properties, and a generic invariant preservation theorem that will be reused throughout later tutorials.

## File Structure

```
04-state-machines/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # StateMachine trait, DoorLock, TrafficLight, run_machine
│       └── main.rs             # CLI simulator
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── StateMachine/Types.lean # generated
    ├── StateMachine/Funs.lean  # generated
    ├── StateMachine/Spec.lean  # Reachable, IsInvariant, multi_step
    ├── StateMachine/DoorLockProofs.lean
    ├── StateMachine/TrafficLightProofs.lean
    └── StateMachine/GenericProofs.lean   # invariant_induction theorem
```

## Rust Code Outline (~290 lines)

### Key Types

| Type | Definition | Description |
|------|-----------|-------------|
| `StateMachine` (trait) | `{ type State; type Event; type Action; fn transition(&self, state: &State, event: &Event) -> (State, Action); }` | Generic state machine interface |
| `DoorState` | `enum { Locked, Unlocked, Alarmed }` | Door can be locked, unlocked, or in alarm state |
| `DoorEvent` | `enum { EnterCode(u32), Reset }` | User enters a code or admin resets |
| `DoorAction` | `enum { Granted, Denied, AlarmTriggered, AlarmCleared }` | Observable output of a transition |
| `DoorLockState` | `struct { door: DoorState, correct_code: u32, wrong_attempts: u32 }` | Full state including attempt counter |
| `LightColor` | `enum { Red, Yellow, Green }` | Traffic light colors |
| `TrafficState` | `struct { ns_light: LightColor, ew_light: LightColor, tick: u32 }` | North-South and East-West lights plus cycle counter |
| `TrafficEvent` | `enum { Tick }` | Time-driven transitions only |

### Key Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `DoorLock::transition` | `(&self, &DoorLockState, &DoorEvent) -> (DoorLockState, DoorAction)` | Locks after 3 wrong attempts trigger alarm |
| `TrafficLight::transition` | `(&self, &TrafficState, &TrafficEvent) -> (TrafficState, TrafficAction)` | Cycles through phases; NS and EW never both green |
| `run_machine` | `(machine, state, events: &[Event]) -> (State, Vec<Action>)` | Fold transitions over an event sequence |

## Generated Lean (approximate)

- **Types.lean**: Inductive types for all enums; structures for `DoorLockState` and `TrafficState`.

```lean
-- approximate generated types
structure DoorLockState where
  door : DoorState
  correct_code : U32
  wrong_attempts : U32
```

- **Funs.lean**: Trait methods become regular functions (Aeneas monomorphizes trait calls). `run_machine` becomes a recursive function over the event list.

Note: Aeneas translates the trait into a Lean structure (record of functions), and each impl becomes an instance.

## Theorems to Prove (~300 lines)

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `alarmed_needs_three_wrong` | `Reachable doorlock init s → s.door = Alarmed → s.wrong_attempts ≥ 3` | Induction on `Reachable`; in the step case, case-split on event and current state; only `EnterCode` with wrong code increments attempts, and alarm triggers at 3 |
| `reset_goes_to_locked` | `transition doorlock s Reset = (s', _) → s'.door = Locked ∧ s'.wrong_attempts = 0` | Direct computation: unfold transition for the `Reset` event |
| `traffic_never_both_green` | `Reachable traffic init s → ¬(s.ns_light = Green ∧ s.ew_light = Green)` | Invariant proof: define `safe s := ¬(ns=Green ∧ ew=Green)`, show it holds initially and is preserved by every transition (finite case check) |
| `invariant_induction` | `IsInvariant P init transition → Reachable init s → P s` | **Generic theorem.** Induction on `Reachable`: base case uses `P init`; step case uses invariant preservation. This is the core lemma reused in later tutorials |

### Spec Definitions (hand-written)

- `Reachable : State → Prop` — inductively defined: initial state is reachable; if `s` is reachable and `transition s e = (s', _)`, then `s'` is reachable.
- `IsInvariant : (State → Prop) → State → TransitionFn → Prop` — `P` holds on `init` and is preserved by all transitions.
- `multi_step : State → List Event → State` — fold transitions over event list (pure spec version).

## New Lean Concepts Introduced

- **Traits as Lean structures**: How Aeneas translates Rust traits into Lean structures (records of functions) and impls into instances.
- **Generic proofs**: Proving properties parameterized over any `StateMachine` instance, not just a specific one.
- **State space reasoning**: Thinking about reachable states, invariants, and safety properties.
- **Invariant proofs**: The pattern of (1) define invariant, (2) show it holds initially, (3) show it's preserved by transitions, (4) conclude it holds for all reachable states.
- **`decide` tactic**: Automatically decides propositions on finite types (useful for exhaustive case checks on enum transitions).

## Cross-References

- **Prerequisites**: Tutorials 01-03 provide foundational proof techniques (monadic reasoning, induction, pattern matching).
- **Forward (KEY BUILDING BLOCK)**: The `StateMachine` trait pattern and `invariant_induction` theorem are reused extensively:
  - Tutorial 07-11: Agent-based systems model each agent as a state machine.
  - The `Reachable` / `IsInvariant` definitions become the standard framework for safety proofs.

## Estimated Lines of Code

| Component | Lines |
|-----------|-------|
| Rust source | ~290 |
| Generated Lean (Types + Funs) | ~180 |
| Hand-written specs | ~80 |
| Hand-written proofs | ~300 |
| README | ~350 |
| **Total** | **~1200** |
