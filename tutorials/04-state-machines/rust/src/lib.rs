//! State Machines — generic trait with two concrete instances.
//!
//! This crate defines a `StateMachine` trait and implements it for:
//! - **DoorLock**: a door with a numeric code, alarm after 3 wrong attempts
//! - **TrafficLight**: a 4-phase traffic light cycle
//!
//! The code is written in an Aeneas-friendly style: no closures, no iterators,
//! explicit while loops, no dynamic dispatch.

// ---------------------------------------------------------------------------
// Generic StateMachine trait
// ---------------------------------------------------------------------------

/// A deterministic state machine parameterised by State, Event, and Action.
pub trait StateMachine {
    type State: Clone;
    type Event;
    type Action: Clone;

    /// Given the current state and an event, produce the next state and a
    /// list of actions (observable outputs).
    fn transition(state: &Self::State, event: &Self::Event) -> (Self::State, Vec<Self::Action>);
}

/// Run a state machine from `initial` over a slice of events, collecting
/// every action produced along the way.
///
/// Uses an explicit while loop (no iterators) so that Aeneas can translate it
/// into a recursive Lean function.
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
        // Append actions — explicit loop, no extend/iterators
        let mut j: usize = 0;
        while j < actions.len() {
            all_actions.push(actions[j].clone());
            j += 1;
        }
        i += 1;
    }
    (state, all_actions)
}

// ---------------------------------------------------------------------------
// Door Lock state machine
// ---------------------------------------------------------------------------

/// The three possible states of the door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Locked,
    Unlocked,
    Alarmed,
}

/// Events that the door lock can receive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoorEvent {
    /// User enters a numeric code.
    EnterCode(u32),
    /// User turns the door handle.
    TurnHandle,
    /// Admin resets the door lock.
    Reset,
}

/// Observable actions produced by the door lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoorAction {
    Beep,
    SoundAlarm,
    OpenDoor,
    Lock,
}

/// Full state of the door lock, including the consecutive wrong-attempt counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoorLockState {
    pub door: DoorState,
    pub wrong_attempts: u32,
}

/// The correct code for the door lock.
pub const CORRECT_CODE: u32 = 1234;

/// Marker type for the DoorLock state machine.
pub struct DoorLock;

impl StateMachine for DoorLock {
    type State = DoorLockState;
    type Event = DoorEvent;
    type Action = DoorAction;

    fn transition(state: &DoorLockState, event: &DoorEvent) -> (DoorLockState, Vec<DoorAction>) {
        match event {
            DoorEvent::EnterCode(code) => {
                match state.door {
                    DoorState::Locked => {
                        if *code == CORRECT_CODE {
                            let new_state = DoorLockState {
                                door: DoorState::Unlocked,
                                wrong_attempts: 0,
                            };
                            (new_state, vec![DoorAction::Beep])
                        } else {
                            let attempts = state.wrong_attempts + 1;
                            if attempts >= 3 {
                                let new_state = DoorLockState {
                                    door: DoorState::Alarmed,
                                    wrong_attempts: attempts,
                                };
                                (new_state, vec![DoorAction::SoundAlarm])
                            } else {
                                let new_state = DoorLockState {
                                    door: DoorState::Locked,
                                    wrong_attempts: attempts,
                                };
                                (new_state, vec![DoorAction::Beep])
                            }
                        }
                    }
                    DoorState::Unlocked => {
                        // Code entry while unlocked — ignored, just beep
                        (state.clone(), vec![DoorAction::Beep])
                    }
                    DoorState::Alarmed => {
                        // Code entry while alarmed — ignored
                        (state.clone(), vec![DoorAction::SoundAlarm])
                    }
                }
            }
            DoorEvent::TurnHandle => {
                match state.door {
                    DoorState::Unlocked => {
                        (state.clone(), vec![DoorAction::OpenDoor])
                    }
                    DoorState::Locked => {
                        (state.clone(), vec![DoorAction::Beep])
                    }
                    DoorState::Alarmed => {
                        (state.clone(), vec![DoorAction::SoundAlarm])
                    }
                }
            }
            DoorEvent::Reset => {
                let new_state = DoorLockState {
                    door: DoorState::Locked,
                    wrong_attempts: 0,
                };
                (new_state, vec![DoorAction::Lock])
            }
        }
    }
}

/// Default initial state for the door lock.
pub fn door_initial() -> DoorLockState {
    DoorLockState {
        door: DoorState::Locked,
        wrong_attempts: 0,
    }
}

// ---------------------------------------------------------------------------
// Traffic Light state machine
// ---------------------------------------------------------------------------

/// Colors for a traffic light.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightColor {
    Red,
    Yellow,
    Green,
}

/// Compass direction for a traffic light.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    NorthSouth,
    EastWest,
}

/// Full state of the traffic intersection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrafficState {
    pub ns_light: LightColor,
    pub ew_light: LightColor,
}

/// The only event for the traffic light: a timer tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrafficEvent {
    Timer,
}

/// Observable actions produced by the traffic light.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrafficAction {
    ChangeLight(Direction, LightColor),
}

/// Marker type for the TrafficLight state machine.
pub struct TrafficLight;

impl StateMachine for TrafficLight {
    type State = TrafficState;
    type Event = TrafficEvent;
    type Action = TrafficAction;

    /// 4-phase cycle:
    ///   Phase 0: NS=Green,  EW=Red    → (Timer) → NS=Yellow, EW=Red
    ///   Phase 1: NS=Yellow, EW=Red    → (Timer) → NS=Red,    EW=Green
    ///   Phase 2: NS=Red,    EW=Green  → (Timer) → NS=Red,    EW=Yellow
    ///   Phase 3: NS=Red,    EW=Yellow → (Timer) → NS=Green,  EW=Red
    fn transition(
        state: &TrafficState,
        _event: &TrafficEvent,
    ) -> (TrafficState, Vec<TrafficAction>) {
        match (&state.ns_light, &state.ew_light) {
            (LightColor::Green, LightColor::Red) => {
                let new_state = TrafficState {
                    ns_light: LightColor::Yellow,
                    ew_light: LightColor::Red,
                };
                (new_state, vec![TrafficAction::ChangeLight(Direction::NorthSouth, LightColor::Yellow)])
            }
            (LightColor::Yellow, LightColor::Red) => {
                let new_state = TrafficState {
                    ns_light: LightColor::Red,
                    ew_light: LightColor::Green,
                };
                (new_state, vec![
                    TrafficAction::ChangeLight(Direction::NorthSouth, LightColor::Red),
                    TrafficAction::ChangeLight(Direction::EastWest, LightColor::Green),
                ])
            }
            (LightColor::Red, LightColor::Green) => {
                let new_state = TrafficState {
                    ns_light: LightColor::Red,
                    ew_light: LightColor::Yellow,
                };
                (new_state, vec![TrafficAction::ChangeLight(Direction::EastWest, LightColor::Yellow)])
            }
            (LightColor::Red, LightColor::Yellow) => {
                let new_state = TrafficState {
                    ns_light: LightColor::Green,
                    ew_light: LightColor::Red,
                };
                (new_state, vec![
                    TrafficAction::ChangeLight(Direction::NorthSouth, LightColor::Green),
                    TrafficAction::ChangeLight(Direction::EastWest, LightColor::Red),
                ])
            }
            // Any other combination: no change (shouldn't happen in normal operation)
            _ => {
                (state.clone(), vec![])
            }
        }
    }
}

/// Default initial state for the traffic light: NS green, EW red.
pub fn traffic_initial() -> TrafficState {
    TrafficState {
        ns_light: LightColor::Green,
        ew_light: LightColor::Red,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Door Lock tests --

    #[test]
    fn test_correct_code_unlocks() {
        let init = door_initial();
        let (state, actions) = DoorLock::transition(&init, &DoorEvent::EnterCode(CORRECT_CODE));
        assert_eq!(state.door, DoorState::Unlocked);
        assert_eq!(state.wrong_attempts, 0);
        assert_eq!(actions, vec![DoorAction::Beep]);
    }

    #[test]
    fn test_wrong_code_increments_attempts() {
        let init = door_initial();
        let (state, _) = DoorLock::transition(&init, &DoorEvent::EnterCode(0000));
        assert_eq!(state.door, DoorState::Locked);
        assert_eq!(state.wrong_attempts, 1);
    }

    #[test]
    fn test_three_wrong_codes_alarm() {
        let events = vec![
            DoorEvent::EnterCode(1111),
            DoorEvent::EnterCode(2222),
            DoorEvent::EnterCode(3333),
        ];
        let (state, actions) = run_machine::<DoorLock>(&door_initial(), &events);
        assert_eq!(state.door, DoorState::Alarmed);
        assert_eq!(state.wrong_attempts, 3);
        assert_eq!(actions.last(), Some(&DoorAction::SoundAlarm));
    }

    #[test]
    fn test_turn_handle_unlocked_opens() {
        let unlocked = DoorLockState {
            door: DoorState::Unlocked,
            wrong_attempts: 0,
        };
        let (_, actions) = DoorLock::transition(&unlocked, &DoorEvent::TurnHandle);
        assert_eq!(actions, vec![DoorAction::OpenDoor]);
    }

    #[test]
    fn test_turn_handle_locked_beeps() {
        let init = door_initial();
        let (state, actions) = DoorLock::transition(&init, &DoorEvent::TurnHandle);
        assert_eq!(state.door, DoorState::Locked);
        assert_eq!(actions, vec![DoorAction::Beep]);
    }

    #[test]
    fn test_reset_from_alarmed() {
        let alarmed = DoorLockState {
            door: DoorState::Alarmed,
            wrong_attempts: 5,
        };
        let (state, actions) = DoorLock::transition(&alarmed, &DoorEvent::Reset);
        assert_eq!(state.door, DoorState::Locked);
        assert_eq!(state.wrong_attempts, 0);
        assert_eq!(actions, vec![DoorAction::Lock]);
    }

    #[test]
    fn test_reset_from_unlocked() {
        let unlocked = DoorLockState {
            door: DoorState::Unlocked,
            wrong_attempts: 0,
        };
        let (state, actions) = DoorLock::transition(&unlocked, &DoorEvent::Reset);
        assert_eq!(state.door, DoorState::Locked);
        assert_eq!(state.wrong_attempts, 0);
        assert_eq!(actions, vec![DoorAction::Lock]);
    }

    #[test]
    fn test_run_machine_door_sequence() {
        let events = vec![
            DoorEvent::EnterCode(9999),   // wrong #1
            DoorEvent::EnterCode(CORRECT_CODE), // correct → unlock
            DoorEvent::TurnHandle,         // open door
        ];
        let (state, actions) = run_machine::<DoorLock>(&door_initial(), &events);
        assert_eq!(state.door, DoorState::Unlocked);
        assert_eq!(actions, vec![
            DoorAction::Beep,     // wrong code beep
            DoorAction::Beep,     // correct code beep
            DoorAction::OpenDoor, // handle turn
        ]);
    }

    // -- Traffic Light tests --

    #[test]
    fn test_traffic_phase_0_to_1() {
        let init = traffic_initial();
        let (state, _) = TrafficLight::transition(&init, &TrafficEvent::Timer);
        assert_eq!(state.ns_light, LightColor::Yellow);
        assert_eq!(state.ew_light, LightColor::Red);
    }

    #[test]
    fn test_traffic_phase_1_to_2() {
        let phase1 = TrafficState {
            ns_light: LightColor::Yellow,
            ew_light: LightColor::Red,
        };
        let (state, _) = TrafficLight::transition(&phase1, &TrafficEvent::Timer);
        assert_eq!(state.ns_light, LightColor::Red);
        assert_eq!(state.ew_light, LightColor::Green);
    }

    #[test]
    fn test_traffic_phase_2_to_3() {
        let phase2 = TrafficState {
            ns_light: LightColor::Red,
            ew_light: LightColor::Green,
        };
        let (state, _) = TrafficLight::transition(&phase2, &TrafficEvent::Timer);
        assert_eq!(state.ns_light, LightColor::Red);
        assert_eq!(state.ew_light, LightColor::Yellow);
    }

    #[test]
    fn test_traffic_phase_3_to_0() {
        let phase3 = TrafficState {
            ns_light: LightColor::Red,
            ew_light: LightColor::Yellow,
        };
        let (state, _) = TrafficLight::transition(&phase3, &TrafficEvent::Timer);
        assert_eq!(state.ns_light, LightColor::Green);
        assert_eq!(state.ew_light, LightColor::Red);
    }

    #[test]
    fn test_traffic_full_cycle_returns() {
        let init = traffic_initial();
        let events = vec![
            TrafficEvent::Timer,
            TrafficEvent::Timer,
            TrafficEvent::Timer,
            TrafficEvent::Timer,
        ];
        let (state, _) = run_machine::<TrafficLight>(&init, &events);
        assert_eq!(state, init);
    }

    #[test]
    fn test_traffic_never_both_green() {
        // Exhaustively check all 4 phases
        let init = traffic_initial();
        let mut state = init.clone();
        let mut i = 0;
        while i < 8 {
            assert!(
                !(state.ns_light == LightColor::Green && state.ew_light == LightColor::Green),
                "Both lights green at step {i}!"
            );
            let (next, _) = TrafficLight::transition(&state, &TrafficEvent::Timer);
            state = next;
            i += 1;
        }
    }

    #[test]
    fn test_run_machine_empty_events() {
        let init = door_initial();
        let (state, actions) = run_machine::<DoorLock>(&init, &[]);
        assert_eq!(state, init);
        assert!(actions.is_empty());
    }
}
