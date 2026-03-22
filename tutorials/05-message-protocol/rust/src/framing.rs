use crate::deserialize::deserialize;
use crate::message::{Message, ParseError};

/// Accumulates bytes from a stream and extracts fully framed messages.
///
/// The `feed` method accepts arbitrary chunks of bytes (which may contain
/// partial messages, exactly one message, or multiple messages) and returns
/// all messages that could be successfully decoded.
pub struct FrameAccumulator {
    buffer: Vec<u8>,
}

impl FrameAccumulator {
    /// Create a new, empty accumulator.
    pub fn new() -> Self {
        FrameAccumulator { buffer: Vec::new() }
    }

    /// Append `data` to the internal buffer and attempt to extract complete
    /// messages.
    ///
    /// Returns a vector of results: `Ok(msg)` for each successfully decoded
    /// message, or `Err(e)` for each parse error encountered (the offending
    /// byte is skipped so parsing can continue).
    pub fn feed(&mut self, data: &[u8]) -> Vec<Result<Message, ParseError>> {
        // Append incoming data
        let mut i: usize = 0;
        while i < data.len() {
            self.buffer.push(data[i]);
            i += 1;
        }

        let mut results: Vec<Result<Message, ParseError>> = Vec::new();

        loop {
            if self.buffer.is_empty() {
                break;
            }

            match deserialize(&self.buffer) {
                Ok((msg, consumed)) => {
                    results.push(Ok(msg));
                    // Drain the consumed bytes
                    let mut new_buf = Vec::new();
                    let mut j: usize = consumed;
                    while j < self.buffer.len() {
                        new_buf.push(self.buffer[j]);
                        j += 1;
                    }
                    self.buffer = new_buf;
                }
                Err(ParseError::NotEnoughData) => {
                    // Wait for more data
                    break;
                }
                Err(e) => {
                    results.push(Err(e));
                    // Skip one byte and try again
                    let mut new_buf = Vec::new();
                    let mut j: usize = 1;
                    while j < self.buffer.len() {
                        new_buf.push(self.buffer[j]);
                        j += 1;
                    }
                    self.buffer = new_buf;
                }
            }
        }

        results
    }
}
