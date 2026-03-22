// shell/mod.rs — Imperative shell layer (NOT verified).
//
// These modules perform real I/O: terminal rendering via crossterm, HTTP
// requests via ureq, and the main event loop.  None of this code is
// translated by Aeneas.  The Lean proofs treat the shell as a trusted
// boundary, described only by axioms in IOBoundary.lean.
//
// In this tutorial the shell is implemented as documented stubs that compile
// but do not perform actual I/O (since crossterm and ureq are not workspace
// dependencies).

pub mod terminal_io;
pub mod http_client;
pub mod event_loop;
pub mod adapters;

// `main.rs` lives in this directory but is not a module — it would be the
// binary entry point if this crate were built with `cargo run`.
