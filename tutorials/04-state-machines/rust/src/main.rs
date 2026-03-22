//! Interactive CLI for the state machine simulator.

use state_machines::*;
use std::io::{self, BufRead, Write};

fn main() {
    println!("=== State Machine Simulator ===");
    println!("Choose a machine:");
    println!("  1) Door Lock");
    println!("  2) Traffic Light");
    print!("> ");
    io::stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let choice = lines.next().unwrap_or(Ok(String::new())).unwrap_or_default();

    match choice.trim() {
        "1" => run_door(&mut lines),
        "2" => run_traffic(&mut lines),
        _ => println!("Unknown choice."),
    }
}

fn run_door(lines: &mut impl Iterator<Item = io::Result<String>>) {
    let mut state = door_initial();
    println!("Door Lock (code = {CORRECT_CODE}). Commands: code <n>, handle, reset, quit");
    loop {
        println!("  State: {:?}", state);
        print!("> ");
        io::stdout().flush().unwrap();
        let line = match lines.next() {
            Some(Ok(l)) => l,
            _ => break,
        };
        let event = match line.trim() {
            s if s.starts_with("code ") => {
                let n: u32 = s[5..].trim().parse().unwrap_or(0);
                DoorEvent::EnterCode(n)
            }
            "handle" => DoorEvent::TurnHandle,
            "reset" => DoorEvent::Reset,
            "quit" => break,
            _ => { println!("  Unknown command."); continue; }
        };
        let (next, actions) = DoorLock::transition(&state, &event);
        println!("  Actions: {:?}", actions);
        state = next;
    }
}

fn run_traffic(lines: &mut impl Iterator<Item = io::Result<String>>) {
    let mut state = traffic_initial();
    println!("Traffic Light. Commands: tick, quit");
    loop {
        println!("  NS={:?}  EW={:?}", state.ns_light, state.ew_light);
        print!("> ");
        io::stdout().flush().unwrap();
        let line = match lines.next() {
            Some(Ok(l)) => l,
            _ => break,
        };
        match line.trim() {
            "tick" => {
                let (next, actions) = TrafficLight::transition(&state, &TrafficEvent::Timer);
                println!("  Actions: {:?}", actions);
                state = next;
            }
            "quit" => break,
            _ => println!("  Unknown command."),
        }
    }
}
