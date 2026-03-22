[← Previous: Tutorial 05](../05-message-protocol/README.md) | [Index](../README.md) | [Next: Tutorial 07 →](../07-tui-core/README.md)

# Tutorial 06: Buffer Management

In this tutorial, we implement three buffer data structures that power text editing in TUI applications: a **ring buffer** for bounded FIFO queues, a **gap buffer** for efficient cursor-based editing, and an **input buffer** that provides higher-level editing operations. We then prove key properties in Lean: capacity invariants, FIFO ordering, content preservation under edits, and cursor validity.

## Prerequisites

Before starting this tutorial, you should have completed:

- **Tutorial 01**: Rust + Lean project setup, basic Aeneas workflow
- **Tutorial 04**: State machine invariants and invariant induction (we reuse the same pattern here for buffer well-formedness)
- **Tutorial 05**: Message protocol proofs (familiarity with Vec and List reasoning)

You will need:

- Rust (edition 2024)
- Lean 4 with the Aeneas library
- Familiarity with modular arithmetic concepts

## Table of Contents

1. [Introduction](#1-introduction)
2. [Ring Buffers](#2-ring-buffers)
3. [Implementing RingBuffer](#3-implementing-ringbuffer)
4. [Modular Arithmetic in Lean](#4-modular-arithmetic-in-lean)
5. [Proving RingBuffer Properties](#5-proving-ringbuffer-properties)
6. [Gap Buffers](#6-gap-buffers)
7. [Implementing GapBuffer](#7-implementing-gapbuffer)
8. [Array and Vec Proofs in Lean](#8-array-and-vec-proofs-in-lean)
9. [Proving GapBuffer Invariants](#9-proving-gapbuffer-invariants)
10. [InputBuffer and Word Deletion](#10-inputbuffer-and-word-deletion)
11. [Loop Invariants for Fixpoints](#11-loop-invariants-for-fixpoints)
12. [Exercises](#12-exercises)
13. [Looking Ahead](#13-looking-ahead)

---

## 1. Introduction

Text editors and TUI (terminal user interface) applications need specialized data structures for managing buffers of data. A chat application needs a bounded message history (ring buffer). A text input field needs efficient insert and delete at the cursor (gap buffer). And the user-facing input widget needs higher-level operations like "delete the last word" (input buffer).

In this tutorial we implement all three in Rust using Aeneas-friendly patterns, then prove their key properties in Lean. The gap buffer and input buffer we build here will be imported directly by Tutorial 07's TUI framework.

### What We Will Prove

| Property | Data Structure | Why It Matters |
|----------|---------------|----------------|
| Capacity invariant | RingBuffer | Buffer never exceeds allocated size |
| FIFO ordering | RingBuffer | Messages come out in the order they went in |
| Push/pop roundtrip | RingBuffer | Pushing then popping returns the same value |
| Content preservation | GapBuffer | Cursor movement does not alter text |
| Insert correctness | GapBuffer | Insert splices byte at the right position |
| Cursor validity | GapBuffer | Cursor always stays in bounds |
| Delete-word correctness | InputBuffer | Removes exactly one word before cursor |
| Move-to-start/end | InputBuffer | Cursor reaches the expected boundary |

---

## 2. Ring Buffers

A **ring buffer** (also called a circular buffer) is a fixed-size FIFO queue that wraps around when it reaches the end of its backing storage. It is the ideal data structure for bounded histories: chat message logs, command histories, or event queues.

### How It Works

Imagine a fixed-size array arranged in a circle:

```
Capacity = 5, len = 3, head = 1, tail = 4

    Index:  0     1     2     3     4
          [   ] [ A ] [ B ] [ C ] [   ]
                  ^                 ^
                 head              tail
```

- **head** points to the next element to read (pop)
- **tail** points to the next empty slot to write (push)
- **len** tracks how many elements are stored

When `tail` reaches the end of the array, it wraps around to index 0:

```
After pushing D and E (tail wraps):

    Index:  0     1     2     3     4
          [ E ] [ A ] [ B ] [ C ] [ D ]
                  ^     ^
                 head  tail (wrapped!)

len = 5 (full)
```

The key operation is modular arithmetic: `tail = (tail + 1) % capacity`. This is what makes the buffer "circular" without ever reallocating memory.

---

## 3. Implementing RingBuffer

Our `RingBuffer<T>` uses Aeneas-friendly patterns throughout:

### Pre-allocation with Default Values

Instead of using `Option<T>` slots (which creates a nested generic enum that complicates Aeneas translation), we pre-allocate the entire vector with `T::default()` values:

```rust
pub fn new(capacity: usize) -> Self {
    let mut data: Vec<T> = Vec::new();
    let mut i: usize = 0;
    while i < capacity {
        data.push(T::default());
        i += 1;
    }
    RingBuffer { data, capacity, head: 0, tail: 0, len: 0 }
}
```

Note the explicit `while` loop instead of `vec![T::default(); capacity]` or an iterator -- Aeneas needs to see the loop structure to translate it into a recursive Lean function.

### Tuple Returns Instead of Option

For `pop` and `peek`, we return `(bool, T)` instead of `Option<T>`:

```rust
pub fn pop(&mut self) -> (bool, T) {
    if self.len == 0 {
        return (false, T::default());
    }
    let item = self.data[self.head].clone();
    self.data[self.head] = T::default();
    self.head = (self.head + 1) % self.capacity;
    self.len -= 1;
    (true, item)
}
```

In the Lean translation, this becomes `Result (Bool x T x RingBuffer T)` -- a simple product type that is straightforward to destructure in proofs.

### Modular Index Advancement

The core of the ring buffer is the index update:

```rust
self.tail = (self.tail + 1) % self.capacity;
```

In Aeneas-generated Lean, this becomes a `Usize` modular operation that we reason about using the lemmas in `ArithUtils.lean`.

---

## 4. Modular Arithmetic in Lean

Ring buffer proofs require reasoning about expressions like `(head + 1) % capacity`. Lean's `omega` tactic handles many linear arithmetic goals, but modular arithmetic needs additional lemmas.

Our `ArithUtils.lean` provides the building blocks:

### Core Lemma: mod_lt

```lean
theorem mod_lt (a n : Nat) (hn : n > 0) : a % n < n
```

This is the most-used lemma: after any modular operation, the result is within bounds. It directly proves that ring buffer indices stay within `[0, capacity)`.

### Wrap-Around Lemma

```lean
theorem add_one_mod_cases (a n : Nat) (hn : n > 0) (ha : a < n) :
    (a + 1) % n = if a + 1 < n then a + 1 else 0
```

This captures the two cases of incrementing a ring buffer index: either it stays in range, or it wraps to 0.

### Full Cycle

```lean
theorem wrap_full_cycle (start capacity : Nat) (hc : capacity > 0)
    (hs : start < capacity) :
    (start + capacity) % capacity = start
```

After exactly `capacity` steps, we return to the starting index. This is key to proving FIFO ordering: the indices visited by `push` and `pop` align correctly.

---

## 5. Proving RingBuffer Properties

### The Invariant

```lean
def ring_inv (T : Type) (rb : RingBuffer T) : Prop :=
  rb.capacity > 0 ∧
  rb.len ≤ rb.capacity ∧
  rb.head < rb.capacity ∧
  rb.tail < rb.capacity ∧
  rb.data.length = rb.capacity ∧
  rb.tail = (rb.head + rb.len) % rb.capacity
```

The last conjunct is crucial: it ties `tail` to `head` and `len` through modular arithmetic. This means we do not need to track `tail` independently -- it is determined by `head` and `len`.

### The Abstraction Function

```lean
def ring_to_list (T : Type) (rb : RingBuffer T) (hcap : rb.capacity > 0) : List T :=
  ring_to_list_aux T rb.data rb.capacity rb.head rb.len hcap
```

This converts a ring buffer into the logical `List T` it represents, reading `len` elements starting from `head` with wrap-around. All our correctness theorems are stated in terms of this abstraction.

### Proof Strategy

The proofs follow the **invariant induction** pattern from Tutorial 04:

1. **Base case**: `new` produces a well-formed ring buffer
2. **Push preservation**: if `ring_inv rb` and `push` succeeds, then `ring_inv rb'`
3. **Pop preservation**: if `ring_inv rb` and `pop` succeeds, then `ring_inv rb'`
4. **FIFO**: under the invariant, `ring_to_list` after push appends to the end, and `ring_to_list` after pop removes from the front

The arithmetic reasoning is handled by `omega` combined with our `ArithUtils` lemmas.

---

## 6. Gap Buffers

A **gap buffer** is the classic data structure for text editors. It stores text as a flat array with a "gap" (contiguous unused region) at the cursor position. Inserts and deletes at the cursor are O(1) because they simply shrink or grow the gap.

### How It Works

```
Content: "Hello World"  with cursor after "Hello"

    buffer:  H  e  l  l  o  _  _  _  _  W  o  r  l  d
             0  1  2  3  4  5  6  7  8  9 10 11 12 13
                            ^              ^
                        gap_start       gap_end

Logical content = buffer[0..5] ++ buffer[9..14] = "Hello World"
                = "Hello" ++ " World"         (the gap is invisible)
```

- **Inserting** a character at the cursor writes into `buffer[gap_start]` and increments `gap_start`. The gap shrinks by one from the left.
- **Deleting** before the cursor (backspace) decrements `gap_start`. The gap grows by one to the left.
- **Moving** the cursor left copies `buffer[gap_start - 1]` to `buffer[gap_end - 1]`, then decrements both indices. The gap "slides" left.

The key insight: only cursor movement is O(distance moved). Insert and delete at the cursor are always O(1). This makes gap buffers ideal for text editing where the user typically types at one position.

### The Invariant

```
gap_start <= gap_end <= buffer.length
```

This simple invariant is maintained by every operation and ensures the gap is a valid sub-range of the buffer.

---

## 7. Implementing GapBuffer

### Construction

A new gap buffer starts with the gap spanning the entire buffer:

```rust
pub fn new(capacity: usize) -> Self {
    let mut buffer: Vec<u8> = Vec::new();
    let mut i: usize = 0;
    while i < capacity {
        buffer.push(0);
        i += 1;
    }
    GapBuffer { buffer, gap_start: 0, gap_end: capacity }
}
```

### Insert

```rust
pub fn insert(&mut self, ch: u8) {
    if self.gap_start == self.gap_end {
        return;  // buffer full
    }
    self.buffer[self.gap_start] = ch;
    self.gap_start += 1;
}
```

When the gap is empty (`gap_start == gap_end`), the buffer is full and we cannot insert. Otherwise, we write the byte into the first gap slot and advance `gap_start`.

### Cursor Movement

Moving left "slides" the gap left by transferring one byte from before the gap to after the gap:

```rust
pub fn move_left(&mut self) -> bool {
    if self.gap_start == 0 { return false; }
    self.gap_end -= 1;
    self.gap_start -= 1;
    self.buffer[self.gap_end] = self.buffer[self.gap_start];
    true
}
```

This preserves the logical content while changing the cursor position.

### Materializing Content

The `to_vec` method copies the pre-gap and post-gap regions into a new vector:

```rust
pub fn to_vec(&self) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut i: usize = 0;
    while i < self.gap_start {
        result.push(self.buffer[i]);
        i += 1;
    }
    let mut j: usize = self.gap_end;
    while j < self.buffer.len() {
        result.push(self.buffer[j]);
        j += 1;
    }
    result
}
```

Again, explicit while loops for Aeneas compatibility.

---

## 8. Array and Vec Proofs in Lean

Gap buffer proofs require reasoning about `Vec` (Aeneas's vector type) in terms of Lean's `List`. The key operations are:

### List.take and List.drop

The gap buffer abstraction function uses `take` and `drop`:

```lean
def gap_content (gb : GapBuffer) : List U8 :=
  (gb.buffer.val.take gb.gap_start) ++ (gb.buffer.val.drop gb.gap_end)
```

Key lemmas from Lean's standard library:

- `List.length_take`: `(xs.take n).length = min n xs.length`
- `List.length_drop`: `(xs.drop n).length = xs.length - n`
- `List.take_append_drop`: `xs.take n ++ xs.drop n = xs`
- `List.length_append`: `(xs ++ ys).length = xs.length + ys.length`

### Vec.index_mut_update

When we write `self.buffer[self.gap_start] = ch` in Rust, Aeneas translates this as `Vec.index_mut_update`. The key property is:

- Reading at the updated index returns the new value
- Reading at any other index returns the old value

This "read-over-write" reasoning is essential for proving that insert places the byte at the correct position and that cursor movement preserves content.

---

## 9. Proving GapBuffer Invariants

### Content Preservation Under Movement

The most important gap buffer theorem: moving the cursor does not change the logical content.

```lean
theorem move_left_preserves_content (gb gb' : GapBuffer)
    (h_inv : gap_inv gb)
    (h_mv : GapBuffer.move_left gb = .ok (true, gb')) :
    gap_content gb' = gap_content gb
```

The proof works by showing that `move_left` takes one byte from the end of `buffer[0..gap_start]` and places it at the start of `buffer[gap_end..buffer.length]`. The `take` region shrinks by one and the `drop` region grows by one with the same byte, so the concatenation is unchanged.

### Insert Correctness

```lean
theorem insert_preserves_content (gb : GapBuffer) (ch : U8) (gb' : GapBuffer)
    (h_inv : gap_inv gb) (h_not_full : gb.gap_start ≠ gb.gap_end)
    (h_ins : GapBuffer.insert gb ch = .ok gb') :
    gap_content gb' =
      (gap_content gb).take gb.gap_start ++ [ch] ++
      (gap_content gb).drop gb.gap_start
```

This says: after inserting `ch`, the new content is the old content with `ch` spliced in at the cursor position. The proof uses `List.take_append_drop` to decompose the original content and `Vec.index_mut_update` to show the new byte appears at `gap_start`.

### Cursor Validity

```lean
theorem cursor_always_valid (gb : GapBuffer) (h_inv : gap_inv gb) :
    gb.gap_start ≤ gap_content_len gb
```

This ensures the cursor is always within the content bounds. The proof is a direct consequence of the gap invariant (`gap_start ≤ gap_end ≤ buffer.length`) and the definition of `gap_content_len`, solved by `omega`.

---

## 10. InputBuffer and Word Deletion

The `InputBuffer` wraps a `GapBuffer` with higher-level editing operations. The most interesting is `delete_word`, which implements Ctrl+Backspace behavior: delete the word before the cursor.

### The Algorithm

```rust
pub fn delete_word(&mut self) -> bool {
    let mut deleted_any = false;
    // Phase 1: skip whitespace before cursor
    while self.gap.cursor_pos() > 0 {
        let content = self.gap.to_vec();
        let pos = self.gap.cursor_pos();
        if pos == 0 { break; }
        if content[pos - 1] != b' ' { break; }
        self.gap.delete_before();
        deleted_any = true;
    }
    // Phase 2: delete non-whitespace (the word)
    while self.gap.cursor_pos() > 0 {
        let content = self.gap.to_vec();
        let pos = self.gap.cursor_pos();
        if pos == 0 { break; }
        if content[pos - 1] == b' ' { break; }
        self.gap.delete_before();
        deleted_any = true;
    }
    deleted_any
}
```

Phase 1 strips trailing spaces, Phase 2 strips the word characters. Together they implement standard word-deletion behavior.

### Move to Start/End

These use while loops that repeatedly call `move_left` or `move_right`:

```rust
pub fn move_to_start(&mut self) {
    while self.gap.cursor_pos() > 0 {
        self.gap.move_left();
    }
}
```

---

## 11. Loop Invariants for Fixpoints

When Aeneas translates a `while` loop, it becomes a recursive Lean function marked with `@[rust_loop] partial`. Proving properties of these loops requires identifying a **loop invariant** -- a property that holds before and after each iteration.

### delete_word Loop Invariant

For the whitespace-skipping phase:

```
Invariant: gap_inv ib.gap ∧
           ∀ i, cursor_pos ≤ i < original_cursor_pos →
             content_at i = ' '
```

Each iteration:
1. Checks `cursor_pos > 0` (termination condition)
2. Reads the byte before the cursor
3. If it is a space, calls `delete_before` (which preserves `gap_inv`)
4. Otherwise, exits the loop

The invariant says: everything we have deleted so far was a space character. The non-whitespace phase has a symmetric invariant.

### Termination

These loops terminate because `gap_start` strictly decreases (for backward deletion/movement) or strictly increases toward `content_len` (for forward movement). Since `gap_start` is bounded by `[0, buffer.length]`, termination is guaranteed.

In Lean, we mark the functions `partial` (Aeneas convention for loops). In a full verification, one could provide a termination proof via a decreasing measure (e.g., `gap_start` for backward loops, `content_len - gap_start` for forward loops).

---

## 12. Exercises

### Exercise 1: move_word_left

Implement `move_word_left` in `InputBuffer`: move the cursor to the start of the previous word (skip spaces, then skip non-spaces). This mirrors the movement of `delete_word` but uses `move_left` instead of `delete_before`.

**Hint**: The structure is identical to `delete_word` -- two while loops with the same conditions, but calling `move_left` instead of `delete_before`.

### Exercise 2: move_word_right

Implement `move_word_right`: move the cursor to the end of the next word (skip spaces forward, then skip non-spaces forward).

**Hint**: You will need to compare the byte *after* the cursor (at `cursor_pos` in the content) and use `move_right`.

### Exercise 3: Prove insert_delete_roundtrip

Complete the proof of `delete_insert_inverse` in `GapBufferProofs.lean`:

```lean
theorem delete_insert_inverse (gb : GapBuffer) (ch : U8)
    (gb1 gb2 : GapBuffer) ...
    : gap_content gb2 = gap_content gb
```

**Hint**: Unfold the definitions of `insert` and `delete_before`. Show that `gap_start` returns to its original value, and the byte written by `insert` is "unwritten" by `delete_before` growing the gap back over it.

### Exercise 4: Ring buffer with u32 keys

Modify `RingBuffer` to store `(u32, T)` pairs where the `u32` is a monotonically increasing sequence number. Add a `find_by_key` method that scans the buffer for a given key. Prove that if keys are inserted in order, `find_by_key` always returns the correct value.

---

## 13. Looking Ahead

The `GapBuffer` and `InputBuffer` we built in this tutorial are the foundation of the text editing widget in **Tutorial 07: TUI Core**. There, we will:

- Wrap `InputBuffer` in a `TextBox` widget with rendering logic
- Use the `ring_buffer` for bounded message/event histories
- Import our `gap_inv` and content preservation theorems to verify that the TUI's editing operations are correct

The abstraction function pattern (`gap_content`) and the invariant preservation proofs (`insert_preserves_inv`, `move_left_preserves_content`) will be composed with the TUI framework's own invariants to build a verified text editor, step by step.

---

## File Structure

```
06-buffer-management/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs               # Module re-exports
│   │   ├── ring_buffer.rs       # RingBuffer<T>: circular FIFO queue
│   │   ├── gap_buffer.rs        # GapBuffer: text editing buffer
│   │   └── input_buffer.rs      # InputBuffer: editor operations
│   └── tests/
│       ├── ring_buffer_tests.rs  # Push/pop, FIFO, capacity tests
│       ├── gap_buffer_tests.rs   # Insert, delete, move, content tests
│       └── input_buffer_tests.rs # Word deletion, cursor movement tests
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── BufferManagement.lean          # Top-level imports
    └── BufferManagement/
        ├── Types.lean                 # Simulated Aeneas types
        ├── Funs.lean                  # Simulated Aeneas functions
        ├── ArithUtils.lean            # Modular arithmetic lemmas
        ├── RingBufferSpec.lean        # ring_to_list, ring_inv
        ├── RingBufferProofs.lean      # Capacity, FIFO, roundtrip proofs
        ├── GapBufferSpec.lean         # gap_content, gap_inv
        ├── GapBufferProofs.lean       # Content preservation, cursor validity
        └── InputBufferProofs.lean     # delete_word, move_to_start/end
```

## Running

### Rust Tests

```bash
cd rust
cargo test
```

### Lean (requires Aeneas library)

```bash
cd lean
lake build
```

---

[← Previous: Tutorial 05](../05-message-protocol/README.md) | [Index](../README.md) | [Next: Tutorial 07 →](../07-tui-core/README.md)
