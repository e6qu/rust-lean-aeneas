-- [StateMachine/Funs.lean] -- simulated Aeneas output
-- This file mirrors the functions that Aeneas would generate from the Rust code.

import StateMachine.Types

namespace state_machines

-- ============================================================
-- Door Lock transition
-- ============================================================

/-- Transition function for the door lock state machine.
    Aeneas monomorphizes trait method calls, so this is a standalone function. -/
def doorlock_transition (state : DoorLockState) (event : DoorEvent)
    : DoorLockState × List DoorAction :=
  match event with
  | DoorEvent.EnterCode code =>
    match state.door with
    | DoorState.Locked =>
      if code == CORRECT_CODE then
        ({ door := DoorState.Unlocked, wrong_attempts := 0 }, [DoorAction.Beep])
      else
        let attempts := state.wrong_attempts + 1
        if attempts >= 3 then
          ({ door := DoorState.Alarmed, wrong_attempts := attempts }, [DoorAction.SoundAlarm])
        else
          ({ door := DoorState.Locked, wrong_attempts := attempts }, [DoorAction.Beep])
    | DoorState.Unlocked =>
      (state, [DoorAction.Beep])
    | DoorState.Alarmed =>
      (state, [DoorAction.SoundAlarm])
  | DoorEvent.TurnHandle =>
    match state.door with
    | DoorState.Unlocked => (state, [DoorAction.OpenDoor])
    | DoorState.Locked   => (state, [DoorAction.Beep])
    | DoorState.Alarmed  => (state, [DoorAction.SoundAlarm])
  | DoorEvent.Reset =>
    ({ door := DoorState.Locked, wrong_attempts := 0 }, [DoorAction.Lock])

/-- The DoorLock instance of the StateMachine structure. -/
def DoorLockMachine : StateMachine DoorLock DoorLockState DoorEvent DoorAction :=
  { transition := doorlock_transition }

-- ============================================================
-- Traffic Light transition
-- ============================================================

/-- Transition function for the traffic light state machine. -/
def traffic_transition (state : TrafficState) (_event : TrafficEvent)
    : TrafficState × List TrafficAction :=
  match state.ns_light, state.ew_light with
  | LightColor.Green, LightColor.Red =>
    ({ ns_light := LightColor.Yellow, ew_light := LightColor.Red },
     [TrafficAction.ChangeLight Direction.NorthSouth LightColor.Yellow])
  | LightColor.Yellow, LightColor.Red =>
    ({ ns_light := LightColor.Red, ew_light := LightColor.Green },
     [TrafficAction.ChangeLight Direction.NorthSouth LightColor.Red,
      TrafficAction.ChangeLight Direction.EastWest LightColor.Green])
  | LightColor.Red, LightColor.Green =>
    ({ ns_light := LightColor.Red, ew_light := LightColor.Yellow },
     [TrafficAction.ChangeLight Direction.EastWest LightColor.Yellow])
  | LightColor.Red, LightColor.Yellow =>
    ({ ns_light := LightColor.Green, ew_light := LightColor.Red },
     [TrafficAction.ChangeLight Direction.NorthSouth LightColor.Green,
      TrafficAction.ChangeLight Direction.EastWest LightColor.Red])
  | _, _ =>
    (state, [])

/-- The TrafficLight instance of the StateMachine structure. -/
def TrafficLightMachine : StateMachine TrafficLight TrafficState TrafficEvent TrafficAction :=
  { transition := traffic_transition }

-- ============================================================
-- Generic run_machine (from explicit while loop)
-- ============================================================

/-- Run a state machine over a list of events, collecting all actions.
    Aeneas translates the `while` loop into a recursive function. -/
-- @[rust_loop]  -- would be tagged by Aeneas in real translation
def run_machine {Self : Type} {State Event Action : Type}
    (machine : StateMachine Self State Event Action)
    (state : State) (events : List Event)
    : State × List Action :=
  match events with
  | [] => (state, [])
  | e :: es =>
    let (next_state, actions) := machine.transition state e
    let (final_state, rest_actions) := run_machine machine next_state es
    (final_state, actions ++ rest_actions)

end state_machines
