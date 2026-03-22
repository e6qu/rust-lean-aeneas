-- [StateMachine/GenericProofs.lean] -- hand-written GENERIC proofs
-- The key theorem: invariant_induction, and its instantiation for traffic lights.

import StateMachine.Types
import StateMachine.Funs
import StateMachine.Spec
import StateMachine.TrafficLightProofs

namespace state_machines

-- ============================================================
-- Generic invariant induction theorem
-- ============================================================

/-- **The core generic theorem.**
    If P is an invariant (holds on init, preserved by transitions),
    then P holds for every reachable state.

    This is the fundamental proof pattern for state machine verification:
    prove an invariant by induction on reachability. -/
theorem invariant_induction {State Event : Type} {α : Type}
    {trans : State → Event → State × List α}
    {init : State}
    {P : State → Prop}
    (hinv : IsInvariant P trans init)
    {s : State}
    (hreach : Reachable trans init s) :
    P s := by
  induction hreach with
  | init_reach => exact hinv.1
  | step _ htrans ih =>
    rw [← htrans]
    exact hinv.2 _ _ ih

-- ============================================================
-- Instantiation: traffic light invariant via generic theorem
-- ============================================================

/-- The traffic light validity is an IsInvariant. -/
theorem traffic_valid_is_invariant :
    IsInvariant valid_traffic_state traffic_transition traffic_initial := by
  constructor
  · -- P holds on initial state
    simp [valid_traffic_state, traffic_initial]
  · -- P is preserved by all transitions
    intro s e hs
    exact traffic_transition_preserves_valid s e hs

/-- Traffic lights are never both Green — proved via the generic
    invariant_induction theorem instead of direct induction.
    This demonstrates how generic proofs work: prove the invariant
    structure once, then instantiate for specific machines. -/
theorem traffic_invariant_via_generic (s : TrafficState)
    (hreach : Reachable traffic_transition traffic_initial s) :
    valid_traffic_state s :=
  invariant_induction traffic_valid_is_invariant hreach

end state_machines
