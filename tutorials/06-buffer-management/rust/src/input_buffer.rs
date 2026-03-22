//! A higher-level input buffer wrapping a GapBuffer.
//!
//! Provides editing operations suitable for a text input field:
//! character insertion, backspace, word deletion, and cursor movement.

use crate::gap_buffer::GapBuffer;

/// An input buffer backed by a GapBuffer, providing editor-like operations.
pub struct InputBuffer {
    gap: GapBuffer,
}

impl InputBuffer {
    /// Create a new input buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        InputBuffer {
            gap: GapBuffer::new(capacity),
        }
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, ch: u8) {
        self.gap.insert(ch);
    }

    /// Delete the character before the cursor (backspace).
    /// Returns `true` if a character was deleted.
    pub fn backspace(&mut self) -> bool {
        self.gap.delete_before()
    }

    /// Delete the word before the cursor.
    ///
    /// First skips any whitespace immediately before the cursor, then deletes
    /// non-whitespace characters until a space or the start of the buffer is
    /// reached. Returns `true` if any characters were deleted.
    pub fn delete_word(&mut self) -> bool {
        let mut deleted_any = false;

        // Phase 1: skip whitespace before cursor
        while self.gap.cursor_pos() > 0 {
            // Peek at the character before cursor by checking gap_buffer internals
            // We need to look at what's at gap_start - 1
            let content = self.gap.to_vec();
            let pos = self.gap.cursor_pos();
            if pos == 0 {
                break;
            }
            if content[pos - 1] != b' ' {
                break;
            }
            self.gap.delete_before();
            deleted_any = true;
        }

        // Phase 2: delete non-whitespace characters (the word itself)
        while self.gap.cursor_pos() > 0 {
            let content = self.gap.to_vec();
            let pos = self.gap.cursor_pos();
            if pos == 0 {
                break;
            }
            if content[pos - 1] == b' ' {
                break;
            }
            self.gap.delete_before();
            deleted_any = true;
        }

        deleted_any
    }

    /// Move the cursor one position to the left.
    /// Returns `true` if the cursor moved.
    pub fn move_cursor_left(&mut self) -> bool {
        self.gap.move_left()
    }

    /// Move the cursor one position to the right.
    /// Returns `true` if the cursor moved.
    pub fn move_cursor_right(&mut self) -> bool {
        self.gap.move_right()
    }

    /// Move the cursor to the start of the buffer.
    pub fn move_to_start(&mut self) {
        while self.gap.cursor_pos() > 0 {
            self.gap.move_left();
        }
    }

    /// Move the cursor to the end of the buffer.
    pub fn move_to_end(&mut self) {
        while self.gap.cursor_pos() < self.gap.content_len() {
            self.gap.move_right();
        }
    }

    /// Get the current content of the buffer as a byte vector.
    pub fn get_content(&self) -> Vec<u8> {
        self.gap.to_vec()
    }

    /// Get the current cursor position.
    pub fn cursor_position(&self) -> usize {
        self.gap.cursor_pos()
    }
}
