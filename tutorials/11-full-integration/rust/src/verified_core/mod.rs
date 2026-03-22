// verified_core/mod.rs — Pure functional core, suitable for Aeneas translation.
//
// Every function in this module is pure: no I/O, no global state, no unsafe.
// The Lean proofs reason about exactly these functions.

pub mod integration_types;
pub mod app_state;
pub mod app_update;
pub mod app_view;
pub mod message_bridge;
