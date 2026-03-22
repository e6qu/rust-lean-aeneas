//! A gap buffer for efficient text editing at the cursor position.
//!
//! The gap buffer stores text as a flat byte array with a "gap" (unused region)
//! at the cursor position. Inserts and deletes at the cursor are O(1) because
//! they simply shrink or grow the gap. Moving the cursor shifts bytes across
//! the gap boundary.
//!
//! Logical content = `buffer[0..gap_start] ++ buffer[gap_end..buffer.len()]`

/// A gap buffer holding a byte sequence with a gap at the cursor.
///
/// Invariant: `gap_start <= gap_end <= buffer.len()`
///
/// The logical content (what the user sees) is formed by concatenating
/// the bytes before the gap with the bytes after the gap.
pub struct GapBuffer {
    buffer: Vec<u8>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    /// Create a new gap buffer with the given total capacity.
    ///
    /// Initially, the entire buffer is gap (no content).
    pub fn new(capacity: usize) -> Self {
        let mut buffer: Vec<u8> = Vec::new();
        let mut i: usize = 0;
        while i < capacity {
            buffer.push(0);
            i += 1;
        }
        GapBuffer {
            buffer,
            gap_start: 0,
            gap_end: capacity,
        }
    }

    /// Insert a byte at the current cursor position (gap_start).
    ///
    /// The byte is placed at `gap_start` and the gap shrinks by one from the left.
    /// Does nothing if the gap is empty (buffer is full).
    pub fn insert(&mut self, ch: u8) {
        if self.gap_start == self.gap_end {
            // Buffer is full, cannot insert
            return;
        }
        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
    }

    /// Delete the byte immediately before the cursor (backspace).
    ///
    /// Returns `true` if a byte was deleted, `false` if the cursor was at
    /// the start (nothing to delete).
    pub fn delete_before(&mut self) -> bool {
        if self.gap_start == 0 {
            return false;
        }
        self.gap_start -= 1;
        true
    }

    /// Delete the byte immediately after the cursor (delete key).
    ///
    /// Returns `true` if a byte was deleted, `false` if the cursor was at
    /// the end (nothing to delete).
    pub fn delete_after(&mut self) -> bool {
        if self.gap_end == self.buffer.len() {
            return false;
        }
        self.gap_end += 1;
        true
    }

    /// Move the cursor one position to the left.
    ///
    /// This shifts one byte from before the gap to after the gap.
    /// Returns `true` if the cursor moved, `false` if already at the start.
    pub fn move_left(&mut self) -> bool {
        if self.gap_start == 0 {
            return false;
        }
        self.gap_end -= 1;
        self.gap_start -= 1;
        self.buffer[self.gap_end] = self.buffer[self.gap_start];
        true
    }

    /// Move the cursor one position to the right.
    ///
    /// This shifts one byte from after the gap to before the gap.
    /// Returns `true` if the cursor moved, `false` if already at the end.
    pub fn move_right(&mut self) -> bool {
        if self.gap_end == self.buffer.len() {
            return false;
        }
        self.buffer[self.gap_start] = self.buffer[self.gap_end];
        self.gap_start += 1;
        self.gap_end += 1;
        true
    }

    /// Return the current cursor position (same as gap_start).
    pub fn cursor_pos(&self) -> usize {
        self.gap_start
    }

    /// Return the length of the logical content (total bytes minus gap size).
    pub fn content_len(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }

    /// Materialize the logical content as a new `Vec<u8>`.
    ///
    /// Copies bytes before the gap, then bytes after the gap, using
    /// explicit while loops (Aeneas-friendly, no iterators).
    pub fn to_vec(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        // Copy pre-gap bytes
        let mut i: usize = 0;
        while i < self.gap_start {
            result.push(self.buffer[i]);
            i += 1;
        }

        // Copy post-gap bytes
        let mut j: usize = self.gap_end;
        while j < self.buffer.len() {
            result.push(self.buffer[j]);
            j += 1;
        }

        result
    }

    /// Return the gap start index (for testing/inspection).
    pub fn gap_start(&self) -> usize {
        self.gap_start
    }

    /// Return the gap end index (for testing/inspection).
    pub fn gap_end(&self) -> usize {
        self.gap_end
    }

    /// Return the total buffer size (for testing/inspection).
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }
}
