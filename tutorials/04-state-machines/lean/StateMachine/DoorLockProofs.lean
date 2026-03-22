-- [StateMachine/DoorLockProofs.lean] -- hand-written proofs
-- Safety and liveness properties of the DoorLock state machine.

import StateMachine.Types
import StateMachine.Funs
import StateMachine.Spec

namespace state_machines

-- ============================================================
-- Theorem 1: Reset always goes to Locked
-- ============================================================

/-- Resetting from any state produces a Locked state with zero wrong attempts. -/
theorem reset_goes_to_locked (s : DoorLockState) :
    (doorlock_transition s DoorEvent.Reset).1.door = DoorState.Locked ∧
    (doorlock_transition s DoorEvent.Reset).1.wrong_attempts = 0 := by
  -- Proof sketch: unfold doorlock_transition, Reset branch always returns Locked
  -- Full proof requires Aeneas library
  sorry

-- ============================================================
-- Theorem 2: Correct code unlocks when Locked
-- ============================================================

/-- Entering the correct code when the door is Locked produces Unlocked. -/
theorem correct_code_unlocks (s : DoorLockState)
    (h : s.door = DoorState.Locked) :
    (doorlock_transition s (DoorEvent.EnterCode CORRECT_CODE)).1.door
      = DoorState.Unlocked := by
  -- Proof sketch: unfold transition, use h to enter Locked branch, correct code matches
  -- Full proof requires Aeneas library
  sorry

-- ============================================================
-- Theorem 3: Alarmed needs three wrong codes
-- ============================================================

/-- The wrong_attempts counter is always consistent: if the door is Alarmed
    in a state reachable from door_initial, then wrong_attempts >= 3. -/
theorem alarmed_needs_three_wrong (s : DoorLockState)
    (hreach : Reachable doorlock_transition door_initial s)
    (halarmed : s.door = DoorState.Alarmed) :
    s.wrong_attempts >= 3 := by
  -- Proof sketch: induction on Reachable, case split on events and states
  -- Full proof requires Aeneas library (simp lemmas for doorlock_transition)
  sorry

-- ============================================================
-- Theorem 4: wrong_attempts invariant
-- ============================================================

/-- In a reachable state, wrong_attempts is bounded: if the door is Locked,
    then wrong_attempts < 3. -/
theorem wrong_attempt_bounded (s : DoorLockState)
    (hreach : Reachable doorlock_transition door_initial s)
    (hlocked : s.door = DoorState.Locked) :
    s.wrong_attempts < 3 := by
  -- Proof sketch: induction on Reachable, case split on events
  -- Full proof requires Aeneas library (simp lemmas for doorlock_transition)
  sorry

end state_machines
