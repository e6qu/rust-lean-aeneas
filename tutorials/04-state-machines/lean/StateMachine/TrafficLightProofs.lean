-- [StateMachine/TrafficLightProofs.lean] -- hand-written proofs
-- Safety properties of the TrafficLight state machine.

import StateMachine.Types
import StateMachine.Funs
import StateMachine.Spec

namespace state_machines

-- ============================================================
-- Valid traffic states
-- ============================================================

/-- A traffic state is valid if NS and EW are not both Green. -/
def valid_traffic_state (s : TrafficState) : Prop :=
  ¬ (s.ns_light = LightColor.Green ∧ s.ew_light = LightColor.Green)

-- ============================================================
-- Theorem 1: Traffic lights are never both Green
-- ============================================================

/-- The traffic light transition preserves validity: if the current state
    is valid, so is the next state. -/
theorem traffic_transition_preserves_valid (s : TrafficState) (e : TrafficEvent)
    (h : valid_traffic_state s) :
    valid_traffic_state (traffic_transition s e).1 := by
  simp [valid_traffic_state, traffic_transition] at *
  cases e with
  | Timer =>
    cases s.ns_light <;> cases s.ew_light <;> simp_all

/-- NS and EW are never both Green in any reachable state. -/
theorem traffic_never_both_green (s : TrafficState)
    (hreach : Reachable traffic_transition traffic_initial s) :
    valid_traffic_state s := by
  induction hreach with
  | init_reach =>
    simp [valid_traffic_state, traffic_initial]
  | step hprev htrans ih =>
    rename_i prev s' event
    rw [← htrans]
    exact traffic_transition_preserves_valid prev event ih

-- ============================================================
-- Theorem 2: Traffic cycle returns after 4 Timer events
-- ============================================================

/-- After 4 Timer events the traffic light returns to its original state,
    provided it starts in one of the 4 valid phases. -/
theorem traffic_cycle_returns (s : TrafficState)
    (h : s = traffic_initial ∨
         s = { ns_light := LightColor.Yellow, ew_light := LightColor.Red } ∨
         s = { ns_light := LightColor.Red, ew_light := LightColor.Green } ∨
         s = { ns_light := LightColor.Red, ew_light := LightColor.Yellow }) :
    multi_step traffic_transition s
      [TrafficEvent.Timer, TrafficEvent.Timer, TrafficEvent.Timer, TrafficEvent.Timer] = s := by
  rcases h with h | h | h | h <;> subst h <;>
    simp [multi_step, traffic_transition, traffic_initial]

end state_machines
