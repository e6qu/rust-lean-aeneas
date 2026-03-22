-- [buffer_management]: function definitions
-- This file simulates the Aeneas-generated function translations for
-- the buffer management data structures. All methods become standalone
-- functions operating monadically in the Result type.
import BufferManagement.Types
import Aeneas

open Aeneas Primitives
open Result

namespace buffer_management

-- ==========================================================================
-- RingBuffer functions
-- ==========================================================================

/-- Create a new ring buffer with the given capacity.
    Pre-allocates `capacity` slots filled with a default value. -/
@[rust_loop]
partial def RingBuffer.new_loop (T : Type) [Inhabited T]
    (capacity : Usize) (data : Vec T) (i : Usize) :
    Result (Vec T) :=
  if i < capacity then do
    let data' ← Vec.push data default
    let i' ← i + 1#usize
    RingBuffer.new_loop T capacity data' i'
  else
    ok data

def RingBuffer.new (T : Type) [Inhabited T] (capacity : Usize) :
    Result (RingBuffer T) := do
  let data ← RingBuffer.new_loop T capacity Vec.new 0#usize
  ok ⟨data, capacity, 0#usize, 0#usize, 0#usize⟩

/-- Push an item into the ring buffer.
    Returns `(true, rb')` on success, `(false, rb)` if full. -/
def RingBuffer.push (T : Type) (rb : RingBuffer T) (item : T) :
    Result (Bool × RingBuffer T) := do
  if rb.len == rb.capacity then
    ok (false, rb)
  else do
    let data' ← Vec.index_mut_update rb.data rb.tail item
    let tail' ← (rb.tail + 1#usize) % rb.capacity
    let len' ← rb.len + 1#usize
    ok (true, { rb with data := data', tail := tail', len := len' })

/-- Pop an item from the front of the ring buffer.
    Returns `(true, value, rb')` if non-empty, `(false, default, rb)` if empty. -/
def RingBuffer.pop (T : Type) [Inhabited T] (rb : RingBuffer T) :
    Result (Bool × T × RingBuffer T) := do
  if rb.len == 0#usize then
    ok (false, default, rb)
  else do
    let item ← Vec.index rb.data rb.head
    let data' ← Vec.index_mut_update rb.data rb.head default
    let head' ← (rb.head + 1#usize) % rb.capacity
    let len' ← rb.len - 1#usize
    ok (true, item, { rb with data := data', head := head', len := len' })

/-- Peek at the front element without removing it.
    Returns `(true, value)` if non-empty, `(false, default)` if empty. -/
def RingBuffer.peek (T : Type) [Inhabited T] (rb : RingBuffer T) :
    Result (Bool × T) := do
  if rb.len == 0#usize then
    ok (false, default)
  else do
    let item ← Vec.index rb.data rb.head
    ok (true, item)

/-- Check if the ring buffer is full. -/
def RingBuffer.is_full (T : Type) (rb : RingBuffer T) : Result Bool :=
  ok (rb.len == rb.capacity)

/-- Check if the ring buffer is empty. -/
def RingBuffer.is_empty (T : Type) (rb : RingBuffer T) : Result Bool :=
  ok (rb.len == 0#usize)

/-- Return the number of elements in the ring buffer. -/
def RingBuffer.len (T : Type) (rb : RingBuffer T) : Result Usize :=
  ok rb.len

/-- Return the total capacity of the ring buffer. -/
def RingBuffer.capacity_ (T : Type) (rb : RingBuffer T) : Result Usize :=
  ok rb.capacity

-- ==========================================================================
-- GapBuffer functions
-- ==========================================================================

/-- Create a new gap buffer with the given total capacity. -/
@[rust_loop]
partial def GapBuffer.new_loop
    (capacity : Usize) (buf : Vec U8) (i : Usize) :
    Result (Vec U8) :=
  if i < capacity then do
    let buf' ← Vec.push buf 0#u8
    let i' ← i + 1#usize
    GapBuffer.new_loop capacity buf' i'
  else
    ok buf

def GapBuffer.new (capacity : Usize) : Result GapBuffer := do
  let buf ← GapBuffer.new_loop capacity Vec.new 0#usize
  ok ⟨buf, 0#usize, capacity⟩

/-- Insert a byte at the cursor position. Does nothing if buffer is full. -/
def GapBuffer.insert (gb : GapBuffer) (ch : U8) :
    Result GapBuffer := do
  if gb.gap_start == gb.gap_end then
    ok gb
  else do
    let buffer' ← Vec.index_mut_update gb.buffer gb.gap_start ch
    let gap_start' ← gb.gap_start + 1#usize
    ok { gb with buffer := buffer', gap_start := gap_start' }

/-- Delete the byte before the cursor (backspace).
    Returns `(true, gb')` on success, `(false, gb)` if at start. -/
def GapBuffer.delete_before (gb : GapBuffer) :
    Result (Bool × GapBuffer) := do
  if gb.gap_start == 0#usize then
    ok (false, gb)
  else do
    let gap_start' ← gb.gap_start - 1#usize
    ok (true, { gb with gap_start := gap_start' })

/-- Delete the byte after the cursor (delete key).
    Returns `(true, gb')` on success, `(false, gb)` if at end. -/
def GapBuffer.delete_after (gb : GapBuffer) :
    Result (Bool × GapBuffer) := do
  if gb.gap_end == gb.buffer.length then
    ok (false, gb)
  else do
    let gap_end' ← gb.gap_end + 1#usize
    ok (true, { gb with gap_end := gap_end' })

/-- Move cursor one position left.
    Returns `(true, gb')` on success, `(false, gb)` if at start. -/
def GapBuffer.move_left (gb : GapBuffer) :
    Result (Bool × GapBuffer) := do
  if gb.gap_start == 0#usize then
    ok (false, gb)
  else do
    let gap_end' ← gb.gap_end - 1#usize
    let gap_start' ← gb.gap_start - 1#usize
    let ch ← Vec.index gb.buffer gap_start'
    let buffer' ← Vec.index_mut_update gb.buffer gap_end' ch
    ok (true, { gb with buffer := buffer', gap_start := gap_start', gap_end := gap_end' })

/-- Move cursor one position right.
    Returns `(true, gb')` on success, `(false, gb)` if at end. -/
def GapBuffer.move_right (gb : GapBuffer) :
    Result (Bool × GapBuffer) := do
  if gb.gap_end == gb.buffer.length then
    ok (false, gb)
  else do
    let ch ← Vec.index gb.buffer gb.gap_end
    let buffer' ← Vec.index_mut_update gb.buffer gb.gap_start ch
    let gap_start' ← gb.gap_start + 1#usize
    let gap_end' ← gb.gap_end + 1#usize
    ok (true, { gb with buffer := buffer', gap_start := gap_start', gap_end := gap_end' })

/-- Return the current cursor position. -/
def GapBuffer.cursor_pos (gb : GapBuffer) : Result Usize :=
  ok gb.gap_start

/-- Return the content length (total bytes minus gap size). -/
def GapBuffer.content_len (gb : GapBuffer) : Result Usize := do
  let gap_size ← gb.gap_end - gb.gap_start
  gb.buffer.length - gap_size

/-- Materialize the logical content: copy pre-gap, then post-gap bytes. -/
@[rust_loop]
partial def GapBuffer.to_vec_pre_loop
    (gb : GapBuffer) (result : Vec U8) (i : Usize) :
    Result (Vec U8) :=
  if i < gb.gap_start then do
    let ch ← Vec.index gb.buffer i
    let result' ← Vec.push result ch
    let i' ← i + 1#usize
    GapBuffer.to_vec_pre_loop gb result' i'
  else
    ok result

@[rust_loop]
partial def GapBuffer.to_vec_post_loop
    (gb : GapBuffer) (result : Vec U8) (j : Usize) :
    Result (Vec U8) :=
  if j < gb.buffer.length then do
    let ch ← Vec.index gb.buffer j
    let result' ← Vec.push result ch
    let j' ← j + 1#usize
    GapBuffer.to_vec_post_loop gb result' j'
  else
    ok result

def GapBuffer.to_vec (gb : GapBuffer) : Result (Vec U8) := do
  let pre ← GapBuffer.to_vec_pre_loop gb Vec.new 0#usize
  GapBuffer.to_vec_post_loop gb pre gb.gap_end

-- ==========================================================================
-- InputBuffer functions
-- ==========================================================================

/-- Create a new input buffer with the given capacity. -/
def InputBuffer.new (capacity : Usize) : Result InputBuffer := do
  let gap ← GapBuffer.new capacity
  ok ⟨gap⟩

/-- Insert a character at cursor position. -/
def InputBuffer.insert_char (ib : InputBuffer) (ch : U8) :
    Result InputBuffer := do
  let gap' ← GapBuffer.insert ib.gap ch
  ok { ib with gap := gap' }

/-- Backspace: delete the character before the cursor. -/
def InputBuffer.backspace (ib : InputBuffer) :
    Result (Bool × InputBuffer) := do
  let (success, gap') ← GapBuffer.delete_before ib.gap
  ok (success, { ib with gap := gap' })

/-- delete_word: skip whitespace backward, then skip non-whitespace backward.
    This is a loop translated as a partial fixpoint. -/

-- Phase 1: skip whitespace before cursor
@[rust_loop]
partial def InputBuffer.delete_word_ws_loop (ib : InputBuffer) (deleted : Bool) :
    Result (InputBuffer × Bool) := do
  if ib.gap.gap_start == 0#usize then
    ok (ib, deleted)
  else do
    let content ← GapBuffer.to_vec ib.gap
    let pos := ib.gap.gap_start
    let idx ← pos - 1#usize
    let ch ← Vec.index content idx
    if ch != 32#u8 then  -- 32 = ASCII space
      ok (ib, deleted)
    else do
      let (_, gap') ← GapBuffer.delete_before ib.gap
      InputBuffer.delete_word_ws_loop { ib with gap := gap' } true

-- Phase 2: delete non-whitespace characters (the word)
@[rust_loop]
partial def InputBuffer.delete_word_nws_loop (ib : InputBuffer) (deleted : Bool) :
    Result (InputBuffer × Bool) := do
  if ib.gap.gap_start == 0#usize then
    ok (ib, deleted)
  else do
    let content ← GapBuffer.to_vec ib.gap
    let pos := ib.gap.gap_start
    let idx ← pos - 1#usize
    let ch ← Vec.index content idx
    if ch == 32#u8 then
      ok (ib, deleted)
    else do
      let (_, gap') ← GapBuffer.delete_before ib.gap
      InputBuffer.delete_word_nws_loop { ib with gap := gap' } true

def InputBuffer.delete_word (ib : InputBuffer) :
    Result (Bool × InputBuffer) := do
  let (ib', deleted1) ← InputBuffer.delete_word_ws_loop ib false
  let (ib'', deleted2) ← InputBuffer.delete_word_nws_loop ib' deleted1
  ok (deleted2, ib'')

/-- Move cursor left. -/
def InputBuffer.move_cursor_left (ib : InputBuffer) :
    Result (Bool × InputBuffer) := do
  let (success, gap') ← GapBuffer.move_left ib.gap
  ok (success, { ib with gap := gap' })

/-- Move cursor right. -/
def InputBuffer.move_cursor_right (ib : InputBuffer) :
    Result (Bool × InputBuffer) := do
  let (success, gap') ← GapBuffer.move_right ib.gap
  ok (success, { ib with gap := gap' })

/-- Move cursor to start. -/
@[rust_loop]
partial def InputBuffer.move_to_start_loop (ib : InputBuffer) :
    Result InputBuffer := do
  if ib.gap.gap_start == 0#usize then
    ok ib
  else do
    let (_, gap') ← GapBuffer.move_left ib.gap
    InputBuffer.move_to_start_loop { ib with gap := gap' }

def InputBuffer.move_to_start (ib : InputBuffer) : Result InputBuffer :=
  InputBuffer.move_to_start_loop ib

/-- Move cursor to end. -/
@[rust_loop]
partial def InputBuffer.move_to_end_loop (ib : InputBuffer) :
    Result InputBuffer := do
  let clen ← GapBuffer.content_len ib.gap
  if ib.gap.gap_start == clen then
    ok ib
  else do
    let (_, gap') ← GapBuffer.move_right ib.gap
    InputBuffer.move_to_end_loop { ib with gap := gap' }

def InputBuffer.move_to_end (ib : InputBuffer) : Result InputBuffer :=
  InputBuffer.move_to_end_loop ib

/-- Get buffer contents. -/
def InputBuffer.get_content (ib : InputBuffer) : Result (Vec U8) :=
  GapBuffer.to_vec ib.gap

/-- Get cursor position. -/
def InputBuffer.cursor_position (ib : InputBuffer) : Result Usize :=
  GapBuffer.cursor_pos ib.gap

end buffer_management
