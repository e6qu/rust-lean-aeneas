# Tutorial 06: Buffer Management

## Goal

Implement ring buffer, gap buffer, and input buffer data structures in Rust; prove capacity invariants, FIFO ordering, content preservation under edits, and cursor validity — building the verified foundation for the text editor in Tutorial 07.

## File Structure

```
06-buffer-management/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ring_buffer.rs      # RingBuffer<T>: new, push, pop, peek, is_full, is_empty, len
│   │   ├── gap_buffer.rs       # GapBuffer: insert, delete_before, delete_after, move_left/right, to_vec
│   │   └── input_buffer.rs     # InputBuffer: insert_char, backspace, delete_word, move cursor
│   └── tests/
│       ├── ring_buffer_tests.rs
│       ├── gap_buffer_tests.rs
│       └── input_buffer_tests.rs
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── BufferManagement/Types.lean         # generated
    ├── BufferManagement/Funs.lean          # generated
    ├── BufferManagement/RingBufferSpec.lean # abstraction: ring_to_list
    ├── BufferManagement/RingBufferProofs.lean
    ├── BufferManagement/GapBufferSpec.lean  # abstraction: gap_content
    ├── BufferManagement/GapBufferProofs.lean
    ├── BufferManagement/InputBufferProofs.lean
    └── BufferManagement/ArithUtils.lean    # modular arithmetic lemmas
```

## Rust Code Outline (~450 lines)

### Key Types

| Type | Definition | Description |
|------|-----------|-------------|
| `RingBuffer<T>` | `struct { data: Vec<(bool, T)>, head: u32, tail: u32, len: u32, capacity: u32 }` | Fixed-capacity circular buffer. Uses `(bool, T)` pairs where the bool indicates slot occupancy, avoiding `Option<T>` which would require `T: Default` or complicate Aeneas translation |
| `GapBuffer` | `struct { data: Vec<u8>, gap_start: u32, gap_end: u32 }` | Text buffer with a gap at the cursor position. Text = `data[0..gap_start] ++ data[gap_end..data.len()]`. Efficient insert/delete at cursor |
| `InputBuffer` | `struct { buffer: GapBuffer, name: Vec<u8> }` | Wraps GapBuffer with higher-level editing operations (word deletion, line-level cursor movement) |

### RingBuffer Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `new` | `(capacity: u32) -> RingBuffer<T>` | Allocate with given capacity (requires `T: Copy + Default`) |
| `push` | `(&mut self, item: T) -> Result<(), T>` | Enqueue; returns `Err(item)` if full |
| `pop` | `(&mut self) -> Option<T>` | Dequeue from head; returns `None` if empty |
| `peek` | `(&self) -> Option<&T>` | View head without removing |
| `is_full` | `(&self) -> bool` | `len == capacity` |
| `is_empty` | `(&self) -> bool` | `len == 0` |
| `len` | `(&self) -> u32` | Current number of elements |

Design note: `RingBuffer` uses `(bool, T)` instead of `Option<T>` to avoid Aeneas complications with nested generic enums. The bool flag marks whether a slot is occupied.

### GapBuffer Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `new` | `(capacity: u32) -> GapBuffer` | Create with initial gap spanning the full buffer |
| `insert` | `(&mut self, byte: u8) -> Result<(), ()>` | Insert byte at gap_start; fails if gap is empty (buffer full) |
| `delete_before` | `(&mut self) -> Result<u8, ()>` | Delete byte before cursor (backspace); fails if at start |
| `delete_after` | `(&mut self) -> Result<u8, ()>` | Delete byte after cursor (delete key); fails if at end |
| `move_left` | `(&mut self) -> Result<(), ()>` | Move cursor (gap) left by one |
| `move_right` | `(&mut self) -> Result<(), ()>` | Move cursor (gap) right by one |
| `to_vec` | `(&self) -> Vec<u8>` | Extract content as contiguous byte vector |
| `content_len` | `(&self) -> u32` | Number of content bytes (total - gap size) |

### InputBuffer Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `insert_char` | `(&mut self, ch: u8) -> Result<(), ()>` | Insert a character at cursor |
| `backspace` | `(&mut self) -> Result<(), ()>` | Delete character before cursor |
| `delete_word` | `(&mut self) -> Result<(), ()>` | Delete word before cursor (back to previous space or start) |
| `move_cursor_left` | `(&mut self) -> Result<(), ()>` | Move cursor left |
| `move_cursor_right` | `(&mut self) -> Result<(), ()>` | Move cursor right |
| `contents` | `(&self) -> Vec<u8>` | Get buffer contents |

## Generated Lean (approximate)

- **Types.lean**: Structures for `RingBuffer`, `GapBuffer`, `InputBuffer`. The `(bool, T)` pair in `RingBuffer` becomes a product type.

```lean
-- approximate generated types
structure RingBuffer (T : Type) where
  data : Vec (Bool × T)
  head : U32
  tail : U32
  len : U32
  capacity : U32

structure GapBuffer where
  data : Vec U8
  gap_start : U32
  gap_end : U32
```

- **Funs.lean**: All methods translated monadically. Modular arithmetic (`% capacity`) in ring buffer operations will appear as `U32.rem` calls.

## Theorems to Prove (~700 lines)

### RingBuffer Proofs

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `capacity_invariant` | `∀ rb, well_formed rb → rb.len ≤ rb.capacity` | Invariant proof: show `new` satisfies it, show `push`/`pop` preserve it. Uses `omega` for arithmetic |
| `fifo_order` | Push sequence `[a, b, c]` then pop 3 times yields `[a, b, c]` | Unfold `ring_to_list` abstraction; show push appends to the logical list and pop removes from the front. Induction on operation sequence |
| `push_pop_roundtrip` | `¬is_full rb → push rb x = ok rb' → pop rb' = ok (x, rb'')` when `rb` was empty | Direct computation through the monadic translation |

### GapBuffer Proofs

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `insert_preserves_content` | `insert gb b = ok gb' → gap_content gb' = gap_content_at_cursor gb b` | Show that `gap_content` (the abstraction function `data[0..gap_start] ++ data[gap_end..]`) has `b` spliced in at the cursor position. Key lemma: `List.take`/`List.drop` decomposition |
| `cursor_always_valid` | `well_formed gb → move_left gb = ok gb' → well_formed gb'` (and symmetrically for all operations) | Show `gap_start ≤ gap_end ≤ data.len` is preserved by every operation. Uses `omega` |
| `delete_insert_inverse` | `insert gb b = ok gb' → delete_before gb' = ok (b', gb'') → b' = b ∧ gap_content gb'' = gap_content gb` | Composing insert then delete-before restores original content |

### InputBuffer Proofs

| Theorem | Statement | Proof Strategy |
|---------|-----------|----------------|
| `delete_word_correct` | `delete_word ib = ok ib' → contents ib' = remove_last_word (contents ib)` | Induction on the `delete_word` loop (which repeatedly calls `delete_before` until it hits a space or start). Show loop invariant: "all deleted characters were non-space, and we stop at a space boundary" |

### Abstraction Functions (Spec files)

- `ring_to_list : RingBuffer T → List T` — the logical sequence represented by the ring buffer (read from head to tail, wrapping around).
- `gap_content : GapBuffer → List U8` — the logical content: `data[0..gap_start] ++ data[gap_end..data.len()]`.
- `well_formed_ring : RingBuffer T → Prop` — `head < capacity ∧ tail < capacity ∧ len ≤ capacity ∧ data.len() = capacity`.
- `well_formed_gap : GapBuffer → Prop` — `gap_start ≤ gap_end ∧ gap_end ≤ data.len()`.

## New Lean Concepts Introduced

- **Modular arithmetic reasoning**: Proving properties about `(head + 1) % capacity` and similar expressions. The `ArithUtils.lean` file provides reusable lemmas like `mod_lt`, `add_mod_wrap`, etc.
- **Vec/array proofs**: Reasoning about `Vec.index`, `Vec.update`, `Vec.length` and their interactions. Connecting Aeneas's `Vec` to Lean's `List` via abstraction functions.
- **Gap buffer theory**: The abstraction function pattern — defining a pure `List` that a `GapBuffer` represents, then proving all operations are correct with respect to that abstraction.
- **Loop invariants for fixpoints**: When Aeneas translates a loop (e.g., `delete_word`'s while loop), it becomes a recursive fixpoint. Proving correctness requires identifying a loop invariant and showing it's preserved by each iteration.

## Cross-References

- **Prerequisites**: Tutorial 04's invariant proof pattern (`IsInvariant` / `invariant_induction`) is applied here to buffer well-formedness invariants.
- **Forward**: Tutorial 07 (TUI Framework) imports `GapBuffer` and `InputBuffer` directly for the `TextBox` widget. The `well_formed_gap` invariant and `insert_preserves_content` theorem are used to verify text editing operations in the UI.

## Estimated Lines of Code

| Component | Lines |
|-----------|-------|
| Rust source | ~450 |
| Rust tests | ~150 |
| Generated Lean (Types + Funs) | ~300 |
| Hand-written specs + utils | ~150 |
| Hand-written proofs | ~700 |
| README | ~400 |
| **Total** | **~2150** |
