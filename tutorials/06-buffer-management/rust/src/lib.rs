//! Buffer Management — ring buffer, gap buffer, and input buffer.
//!
//! This crate provides three buffer data structures commonly used in
//! text editors and TUI applications:
//! - **RingBuffer<T>**: fixed-capacity circular FIFO queue
//! - **GapBuffer**: text buffer with a gap at the cursor for efficient editing
//! - **InputBuffer**: higher-level editing abstraction wrapping a GapBuffer
//!
//! All code is written in an Aeneas-friendly style: no closures, no iterators,
//! no trait objects, explicit while loops, and return types that avoid Option/Result
//! where they would complicate the Lean translation.

pub mod ring_buffer;
pub mod gap_buffer;
pub mod input_buffer;
