// Full Integration — Tutorial 11
//
// This crate demonstrates the "functional core, imperative shell" architecture
// for a multi-agent LLM TUI application.  The `verified_core` module contains
// pure functions that Aeneas can translate to Lean for formal verification.
// The `shell` module contains I/O stubs (crossterm, HTTP) that are NOT verified.
// The `deps` module re-exports stub types representing prior tutorials (05-10).

pub mod deps;
pub mod verified_core;
pub mod shell;
