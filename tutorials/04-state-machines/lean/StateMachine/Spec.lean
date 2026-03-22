-- [StateMachine/Spec.lean] -- hand-written specifications
-- Defines Reachable, IsInvariant, multi_step, and helper functions.

import StateMachine.Types
import StateMachine.Funs

namespace state_machines

-- ============================================================
-- Reachable states
-- ============================================================

/-- A state is reachable from `init` via transition function `trans`
    if it is the initial state, or it can be reached by taking one
    transition from a reachable state. -/
inductive Reachable {α : Type} {State Event : Type}
    (trans : State → Event → State × List α)
    (init : State) : State → Prop where
  | init_reach : Reachable trans init init
  | step {s s' : State} {e : Event} :
      Reachable trans init s →
      (trans s e).1 = s' →
      Reachable trans init s'

-- ============================================================
-- multi_step: fold transitions over a list of events
-- ============================================================

/-- Run transitions over a list of events, returning the final state. -/
def multi_step {α : Type} {State Event : Type}
    (trans : State → Event → State × List α)
    (state : State) : List Event → State
  | [] => state
  | e :: es => multi_step trans (trans state e).1 es

/-- multi_step produces reachable states. -/
axiom multi_step_reachable {α : Type} {State Event : Type}
    {trans : State → Event → State × List α}
    {init : State} (events : List Event) :
    Reachable trans init (multi_step trans init events)

-- ============================================================
-- IsInvariant
-- ============================================================

/-- A property P is an invariant of a state machine if:
    1. P holds on the initial state, and
    2. For every state s and event e, if P(s) then P((trans s e).1). -/
def IsInvariant {α : Type} {State Event : Type}
    (P : State → Prop)
    (trans : State → Event → State × List α)
    (init : State) : Prop :=
  P init ∧ ∀ (s : State) (e : Event), P s → P (trans s e).1

-- ============================================================
-- Helpers: extract next state (ignoring actions)
-- ============================================================

/-- Extract the next door lock state from a transition. -/
def door_next (s : DoorLockState) (e : DoorEvent) : DoorLockState :=
  (doorlock_transition s e).1

/-- Extract the next traffic state from a transition. -/
def traffic_next (s : TrafficState) (e : TrafficEvent) : TrafficState :=
  (traffic_transition s e).1

end state_machines
