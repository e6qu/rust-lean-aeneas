//! TUI Core — pure terminal UI model with layout, widgets, events, and focus.
//!
//! This crate provides a complete TUI model layer with zero terminal I/O:
//! - **Geometry**: `Rect`, `Position`, `Cell` primitives for spatial reasoning
//! - **Events**: `Event`, `KeyCode`, `Modifiers` for input handling
//! - **Layout**: `split` and `split_at` for dividing screen regions
//! - **Widgets**: Arena-pattern `WidgetKind` enum (no recursive Box types)
//! - **AppModel**: Elm-style update loop with focus management
//!
//! All code is written in an Aeneas-friendly style: no closures, no iterators,
//! no trait objects, explicit while loops, and `Vec<u8>` instead of `String`.

pub mod geometry;
pub mod event;
pub mod layout;
pub mod widget;
pub mod app_model;
pub mod action;
