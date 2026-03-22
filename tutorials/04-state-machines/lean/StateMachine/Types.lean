-- [StateMachine/Types.lean] -- simulated Aeneas output
-- This file mirrors the types that Aeneas would generate from the Rust code.

namespace state_machines

-- ============================================================
-- Generic StateMachine structure (from Rust trait)
-- ============================================================

/-- A state machine: Aeneas translates Rust traits into Lean structures.
    The `Self` parameter represents the marker type (e.g., DoorLock). -/
structure StateMachine (Self : Type) (State : Type) (Event : Type) (Action : Type) where
  transition : State → Event → (State × List Action)

-- ============================================================
-- Door Lock types
-- ============================================================

/-- Door states. -/
inductive DoorState where
  | Locked
  | Unlocked
  | Alarmed
  deriving DecidableEq, Repr

/-- Door events. -/
inductive DoorEvent where
  | EnterCode (code : UInt32)
  | TurnHandle
  | Reset
  deriving DecidableEq, Repr

/-- Door actions (observable outputs). -/
inductive DoorAction where
  | Beep
  | SoundAlarm
  | OpenDoor
  | Lock
  deriving DecidableEq, Repr

/-- Full state of the door lock. -/
structure DoorLockState where
  door : DoorState
  wrong_attempts : UInt32
  deriving DecidableEq, Repr

/-- Marker type for the DoorLock state machine. -/
inductive DoorLock where | mk

/-- The correct code. -/
def CORRECT_CODE : UInt32 := 1234

/-- Default initial door lock state. -/
def door_initial : DoorLockState :=
  { door := DoorState.Locked, wrong_attempts := 0 }

-- ============================================================
-- Traffic Light types
-- ============================================================

/-- Colors for a traffic light. -/
inductive LightColor where
  | Red
  | Yellow
  | Green
  deriving DecidableEq, Repr

/-- Compass direction. -/
inductive Direction where
  | NorthSouth
  | EastWest
  deriving DecidableEq, Repr

/-- Full state of the traffic intersection. -/
structure TrafficState where
  ns_light : LightColor
  ew_light : LightColor
  deriving DecidableEq, Repr

/-- Traffic light events. -/
inductive TrafficEvent where
  | Timer
  deriving DecidableEq, Repr

/-- Traffic light actions. -/
inductive TrafficAction where
  | ChangeLight (dir : Direction) (color : LightColor)
  deriving DecidableEq, Repr

/-- Marker type for the TrafficLight state machine. -/
inductive TrafficLight where | mk

/-- Default initial traffic state. -/
def traffic_initial : TrafficState :=
  { ns_light := LightColor.Green, ew_light := LightColor.Red }

end state_machines
