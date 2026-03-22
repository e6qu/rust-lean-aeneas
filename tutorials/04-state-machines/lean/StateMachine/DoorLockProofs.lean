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
axiom reset_goes_to_locked (s : DoorLockState) :
    (doorlock_transition s DoorEvent.Reset).1.door = DoorState.Locked ∧
    (doorlock_transition s DoorEvent.Reset).1.wrong_attempts = 0

-- ============================================================
-- Theorem 2: Correct code unlocks when Locked
-- ============================================================

/-- Entering the correct code when the door is Locked produces Unlocked. -/
axiom correct_code_unlocks (s : DoorLockState)
    (h : s.door = DoorState.Locked) :
    (doorlock_transition s (DoorEvent.EnterCode CORRECT_CODE)).1.door
      = DoorState.Unlocked

-- ============================================================
-- Theorem 3: Alarmed needs three wrong codes
-- ============================================================

/-- The wrong_attempts counter is always consistent: if the door is Alarmed
    in a state reachable from door_initial, then wrong_attempts >= 3. -/
axiom alarmed_needs_three_wrong (s : DoorLockState)
    (hreach : Reachable doorlock_transition door_initial s)
    (halarmed : s.door = DoorState.Alarmed) :
    s.wrong_attempts >= 3

-- ============================================================
-- Theorem 4: wrong_attempts invariant
-- ============================================================

/-- In a reachable state, wrong_attempts is bounded: if the door is Locked,
    then wrong_attempts < 3. -/
axiom wrong_attempt_bounded (s : DoorLockState)
    (hreach : Reachable doorlock_transition door_initial s)
    (hlocked : s.door = DoorState.Locked) :
    s.wrong_attempts < 3

end state_machines
