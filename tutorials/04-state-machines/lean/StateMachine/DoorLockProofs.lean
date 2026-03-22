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
  simp [doorlock_transition]

-- ============================================================
-- Theorem 2: Correct code unlocks when Locked
-- ============================================================

/-- Entering the correct code when the door is Locked produces Unlocked. -/
theorem correct_code_unlocks (s : DoorLockState)
    (h : s.door = DoorState.Locked) :
    (doorlock_transition s (DoorEvent.EnterCode CORRECT_CODE)).1.door
      = DoorState.Unlocked := by
  simp [doorlock_transition, h, CORRECT_CODE]

-- ============================================================
-- Theorem 3: Alarmed needs three wrong codes
-- ============================================================

/-- The wrong_attempts counter is always consistent: if the door is Alarmed
    in a state reachable from door_initial, then wrong_attempts >= 3. -/
theorem alarmed_needs_three_wrong (s : DoorLockState)
    (hreach : Reachable doorlock_transition door_initial s)
    (halarmed : s.door = DoorState.Alarmed) :
    s.wrong_attempts >= 3 := by
  induction hreach with
  | init_reach =>
    -- Initial state has door = Locked, contradiction with Alarmed
    simp [door_initial] at halarmed
  | step hprev htrans ih =>
    -- s' is reached by one transition from some reachable state prev
    rename_i prev s' event
    -- We need to case-split on the event and previous state
    simp [doorlock_transition] at htrans
    split at htrans <;> simp_all
    · -- EnterCode
      split at htrans <;> simp_all
      · -- prev.door = Locked
        split at htrans <;> simp_all
        · -- code == CORRECT_CODE: new door = Unlocked, contradiction
          obtain ⟨h1, _⟩ := htrans
          rw [h1] at halarmed; contradiction
        · -- code ≠ CORRECT_CODE
          split at htrans <;> simp_all
          · -- attempts >= 3: new door = Alarmed, wrong_attempts = attempts
            obtain ⟨_, h2⟩ := htrans
            rw [h2]; assumption
          · -- attempts < 3: new door = Locked, contradiction
            obtain ⟨h1, _⟩ := htrans
            rw [h1] at halarmed; contradiction
      · -- prev.door = Unlocked: state unchanged
        obtain ⟨h1, _⟩ := htrans
        rw [h1] at halarmed
        exact ih halarmed
      · -- prev.door = Alarmed: state unchanged
        obtain ⟨h1, _⟩ := htrans
        rw [h1] at halarmed
        exact ih halarmed
    · -- TurnHandle: state unchanged in all cases
      split at htrans <;> simp_all
      all_goals (obtain ⟨h1, _⟩ := htrans; rw [h1] at halarmed; try exact ih halarmed)
    · -- Reset: new door = Locked, contradiction
      obtain ⟨h1, _⟩ := htrans
      rw [h1] at halarmed; contradiction

-- ============================================================
-- Theorem 4: wrong_attempts invariant
-- ============================================================

/-- In a reachable state, wrong_attempts is bounded: if the door is Locked,
    then wrong_attempts < 3. -/
theorem wrong_attempt_bounded (s : DoorLockState)
    (hreach : Reachable doorlock_transition door_initial s)
    (hlocked : s.door = DoorState.Locked) :
    s.wrong_attempts < 3 := by
  induction hreach with
  | init_reach =>
    simp [door_initial]
  | step hprev htrans ih =>
    rename_i prev s' event
    simp [doorlock_transition] at htrans
    split at htrans <;> simp_all
    · -- EnterCode
      split at htrans <;> simp_all
      · -- prev.door = Locked
        split at htrans <;> simp_all
        · -- correct code → Unlocked, contradiction
          obtain ⟨h1, _⟩ := htrans
          rw [h1] at hlocked; contradiction
        · -- wrong code
          split at htrans <;> simp_all
          · -- attempts >= 3 → Alarmed, contradiction
            obtain ⟨h1, _⟩ := htrans
            rw [h1] at hlocked; contradiction
          · -- attempts < 3 → Locked with attempts
            obtain ⟨_, h2⟩ := htrans
            rw [h2]; assumption
      · -- prev.door = Unlocked
        obtain ⟨h1, _⟩ := htrans
        rw [h1] at hlocked; contradiction
      · -- prev.door = Alarmed
        obtain ⟨h1, _⟩ := htrans
        rw [h1] at hlocked; contradiction
    · -- TurnHandle
      split at htrans <;> simp_all
      all_goals (obtain ⟨h1, h2⟩ := htrans; rw [h1] at hlocked; try (rw [h2]; exact ih hlocked); try contradiction)
    · -- Reset
      obtain ⟨_, h2⟩ := htrans
      rw [h2]; simp

end state_machines
