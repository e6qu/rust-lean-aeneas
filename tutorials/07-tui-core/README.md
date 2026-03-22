[← Previous: Tutorial 06](../06-buffer-management/README.md) | [Index](../README.md) | [Next: Tutorial 08 →](../08-llm-client-core/README.md)

# Tutorial 07: TUI Core

Pure terminal UI model with layout, widgets, events, and focus — zero terminal I/O.

## Overview

This tutorial builds a complete TUI model layer that can be formally verified. Every component is a pure function: layout produces rectangles, widgets produce cells, and the app model processes events through an Elm-style update loop. No terminal I/O appears anywhere — rendering produces a flat `Vec<Cell>` that a future tutorial will paint to the screen.

The key design decision is the **arena pattern**: widgets reference children by index into a flat `Vec<WidgetKind>`, rather than using recursive `Box<Widget>` types. This makes the Aeneas translation straightforward — Lean sees list indexing instead of inductive recursion.

### What You Will Learn

- Modeling a TUI with pure geometry (Rect, Position, Cell)
- Building a layout engine with provable non-overlap and coverage properties
- The arena pattern for tree structures that Aeneas can translate
- Elm-style state management (Model → Event → Action → Model)
- Focus management with modular arithmetic
- Proving render bounds, focus validity, scroll safety, and input routing

### Prerequisites

- Tutorial 01 (Setup and Hello Proof) — Lean 4 and Aeneas basics
- Tutorial 04 (State Machines) — Elm-style pure state machine pattern
- Tutorial 06 (Buffer Management) — The TextBox widget conceptually wraps an InputBuffer

## Part 1: Geometry

All layout and rendering starts with three types.

### Rect

A rectangle in terminal coordinates. The origin (0, 0) is the top-left corner of the terminal. Width and height are in character cells.

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
```

The key methods on `Rect` are:

- **`contains(pos)`**: Point-in-rectangle test. A position is inside if `x <= pos.x < x + width` and `y <= pos.y < y + height`.
- **`area()`**: Returns `width * height` as `u32` to avoid `u16` overflow.
- **`intersects(other)`**: Two rectangles overlap if they share at least one position.
- **`inner(margin)`**: Shrink by `margin` on each side. Used by borders to compute the child area.

### Position and Cell

```rust
pub struct Position { pub x: u16, pub y: u16 }
pub struct Cell { pub pos: Position, pub ch: u8, pub style: u8 }
```

A `Cell` is the atomic unit of rendering: a character at a position with a style byte. The entire rendering pipeline produces `Vec<Cell>` — no terminal escape codes, no buffered writes, just data.

**Why `u8` for characters?** ASCII is sufficient for a TUI framework tutorial. This keeps the Lean translation simple (`U8` instead of `Char`), and `u8` avoids Unicode complexities that would distract from the verification story.

### Aeneas Translation

In the generated Lean, these become:

```lean
structure Rect where
  x : U16;  y : U16;  width : U16;  height : U16

structure Position where
  x : U16;  y : U16

structure Cell where
  pos : Position;  ch : U8;  style : U8
```

The `U16` type is Aeneas's bounded natural — it carries a proof that the value fits in 16 bits. This is what enables arithmetic proofs about rectangle coordinates.

## Part 2: Events

Terminal input is modeled as a pure enum with no I/O dependency.

```rust
pub enum KeyCode {
    Char(u8), Enter, Backspace, Tab, BackTab, Escape,
    Left, Right, Up, Down, Home, End, Delete, PageUp, PageDown,
}

pub struct Modifiers { pub bits: u8 }  // CTRL=1, ALT=2, SHIFT=4

pub enum Event {
    Key(KeyCode, Modifiers),
    Resize(u16, u16),
    Tick,
}
```

The `Modifiers` struct uses bitflags in a single `u8`. This maps cleanly to Lean's `U8` with bitwise operations. The `Event` enum has three variants:

- **`Key`**: A keypress with modifiers. Tab and BackTab are separate keycodes (not Shift+Tab) for simpler pattern matching.
- **`Resize`**: The terminal changed size. Triggers a relayout.
- **`Tick`**: A timer tick for animations. Produces `Noop` in the current implementation.

### Event-to-Action Mapping

The `event_to_action` function maps raw events to semantic `Action` values:

```rust
pub fn event_to_action(event: &Event, _model: &AppModel) -> Action {
    match event {
        Event::Key(KeyCode::Tab, _) => Action::FocusNext,
        Event::Key(KeyCode::BackTab, _) => Action::FocusPrev,
        Event::Key(KeyCode::Enter, _) => Action::Submit,
        Event::Key(KeyCode::Char(c), mods) if mods.ctrl() => {
            match c { b'c' | b'q' => Action::Quit, _ => Action::Noop }
        }
        // ... more mappings
    }
}
```

This is a pure function — no state mutation, no I/O. The Lean translation is a large `match` that we can reason about by direct computation.

## Part 3: Layout Engine

The layout engine splits rectangles into sub-rectangles. Two functions handle all cases.

### `split(area, dir, count) -> Vec<Rect>`

Divides `area` into `count` equal pieces along `dir`. Integer division means pieces may not be exactly equal — the last piece gets the remainder pixels.

```rust
pub fn split(area: Rect, dir: SplitDir, count: usize) -> Vec<Rect> {
    let piece_size = total / (count as u16);
    let mut i: usize = 0;
    while i < count {
        let size = if i == count - 1 { total - offset } else { piece_size };
        // ... create rect at current offset with `size`
        offset += piece_size;
        i += 1;
    }
}
```

**Why explicit while loops?** Aeneas translates `while` loops into recursive Lean functions with termination proofs. Rust iterators use closures and trait objects that Aeneas cannot handle.

### `split_at(area, dir, offset) -> (Rect, Rect)`

Splits at a specific pixel offset. The offset is clamped to the area dimension.

```rust
let clamped = if offset > area.width { area.width } else { offset };
let left = Rect { x: area.x, y: area.y, width: clamped, height: area.height };
let right = Rect { x: area.x + clamped, y: area.y,
                    width: area.width - clamped, height: area.height };
```

### Properties We Will Prove

1. **`split_no_overlap`**: Sub-rectangles are pairwise non-overlapping
2. **`split_covers`**: Total dimension sums to the original
3. **`split_at_partition`**: `left.width + right.width == area.width`

## Part 4: Proving Layout Correctness

### split_at_partition

The simplest layout theorem. In Lean:

```lean
theorem split_at_partition_horizontal (r : Rect) (offset : U16) :
    let (left, right) := split_at r SplitDir.Horizontal offset
    left.width.val + right.width.val = r.width.val := by
  simp [split_at]
  split
  · omega   -- offset > width case: clamped = width, remainder = 0
  · omega   -- normal case: clamped + (width - clamped) = width
```

The `omega` tactic handles the linear arithmetic automatically. The `split` tactic handles the `if` branch in `split_at`.

### split_at_no_overlap

```lean
theorem split_at_no_overlap_horizontal (r : Rect) (offset : U16) :
    let (left, right) := split_at r SplitDir.Horizontal offset
    rect_intersects left right = false := by
  simp [split_at, rect_intersects]
  split <;> omega
```

The key insight: `left` ends at `x + clamped` and `right` starts at `x + clamped`. They share no x-coordinate, so they cannot overlap.

### split_no_overlap and split_covers

These theorems about the `split` function require induction on the recursive helper. The full proofs are deferred to Aeneas integration (marked with `sorry`), but the proof strategy is:

1. Show that consecutive sub-rectangles have adjacent coordinates
2. Use `Nat.div_add_mod` to show the last piece absorbs the remainder
3. Sum all piece sizes to recover the original dimension

## Part 5: The Widget Enum (Arena Pattern)

### Why Not Recursive Types?

A natural Rust design for widgets would be:

```rust
enum Widget {
    TextBox { ... },
    Border { child: Box<Widget> },
    Container { children: Vec<Box<Widget>> },
}
```

But `Box<Widget>` creates heap-allocated recursive types that Aeneas cannot translate. Instead, we use the **arena pattern**: all widgets live in a flat `Vec<WidgetKind>`, and children are referenced by index.

### The Arena Pattern

```rust
pub enum WidgetKind {
    TextBox { content: Vec<u8>, cursor: usize },
    ScrollableList { items: Vec<Vec<u8>>, selected: usize, scroll_offset: usize },
    StatusBar { left_text: Vec<u8>, right_text: Vec<u8> },
    Border { title: Vec<u8>, child_index: usize },
    Container { dir: SplitDir, children: Vec<usize> },
}
```

A `Border` stores `child_index: usize` — the index of its child in the arena. A `Container` stores `children: Vec<usize>` — indices of its children. This is a tree encoded as an adjacency list.

In Lean, indices become `Nat` with bounds checked via `List.get?`:

```lean
inductive WidgetKind where
  | TextBox (content : List U8) (cursor : Nat)
  | Border (title : List U8) (child_index : Nat)
  | Container (dir : SplitDir) (children : List Nat)
  -- ...
```

### Widget Tree Validity

The `valid_widget_tree` predicate ensures all indices are in range:

```lean
def widget_indices_valid (w : WidgetKind) (arena_len : Nat) : Prop :=
  match w with
  | WidgetKind.Border _ child_index => child_index < arena_len
  | WidgetKind.Container _ children =>
      ∀ (idx : Nat), idx ∈ children → idx < arena_len
  | _ => True

def valid_widget_tree (widgets : List WidgetKind) : Prop :=
  ∀ (w : WidgetKind), w ∈ widgets → widget_indices_valid w widgets.length
```

This is the arena pattern's equivalent of "well-formed tree." If this predicate holds, every index lookup will succeed.

### Widget Tree Depth

To reason about termination of rendering, we define a depth function with explicit fuel:

```lean
def widget_tree_depth (widgets : List WidgetKind) (root : Nat) (fuel : Nat) : Nat :=
  match fuel with
  | 0 => 0
  | fuel' + 1 =>
    match widgets.get? root with
    | none => 0
    | some (WidgetKind.Border _ child_index) =>
        1 + widget_tree_depth widgets child_index fuel'
    | some (WidgetKind.Container _ children) =>
        1 + max_child_depth children
    | _ => 1
```

The fuel parameter ensures Lean can prove termination. In practice, the arena length serves as an upper bound on meaningful depth.

## Part 6: Rendering

Each widget variant renders cells within its allocated area.

### TextBox

Places characters starting at `area.x, area.y`, limited by `area.width`:

```rust
let mut i: usize = 0;
while i < content.len() && i < max_chars {
    cells.push(Cell {
        pos: Position { x: area.x + i as u16, y: area.y },
        ch: content[i],
        style: 0,
    });
    i += 1;
}
```

### ScrollableList

Renders visible items starting from `scroll_offset`, highlighting the `selected` item:

```rust
while row < visible_rows && (clamped_offset + row) < items.len() {
    let item = &items[clamped_offset + row];
    let style = if item_idx == selected { 1 } else { 0 };
    // render item characters in this row
}
```

### StatusBar

Renders left-aligned text and right-aligned text on a single row.

### Border

Draws `+`, `-`, `|` characters around the perimeter, renders the title on the top edge, then delegates to the child widget in the `inner(1)` area.

### Container

Splits its area using `split()`, then renders each child in its sub-area:

```rust
let sub_areas = split(area, dir, children.len());
while i < children.len() && i < sub_areas.len() {
    let child_cells = render_widget(&widgets[children[i]], sub_areas[i], widgets);
    // append child_cells to output
}
```

### The `render_widget` Dispatcher

```rust
pub fn render_widget(kind: &WidgetKind, area: Rect, widgets: &[WidgetKind]) -> Vec<Cell> {
    if area.width == 0 || area.height == 0 { return Vec::new(); }
    match kind {
        WidgetKind::TextBox { .. } => render_textbox(content, area),
        WidgetKind::Border { .. } => render_border(title, child_index, area, widgets),
        WidgetKind::Container { .. } => render_container(dir, children, area, widgets),
        // ...
    }
}
```

The `widgets` parameter (the full arena) is passed through so that `Border` and `Container` can look up children by index.

## Part 7: Proving Render Bounds

The central rendering theorem: every cell produced by rendering is within the screen rectangle.

### cells_in_bounds Predicate

```lean
def cells_in_bounds (cells : List Cell) (area : Rect) : Prop :=
  ∀ (c : Cell), c ∈ cells → rect_contains area c.pos = true
```

### Per-Widget Bounds

Each widget variant has a theorem showing its cells stay within the allocated area. For example:

```lean
theorem textbox_render_in_bounds (content : List U8) (area : Rect) (cells : List Cell)
    (h_render : ∀ (c : Cell), c ∈ cells →
      c.pos.x.val >= area.x.val ∧ c.pos.x.val < area.x.val + area.width.val ∧
      c.pos.y.val = area.y.val) :
    cells_in_bounds cells area
```

### Border Composition

Border rendering combines border chars (within the outer area) and child cells (within the inner area). The `inner_subset` lemma bridges them:

```lean
theorem inner_subset (r : Rect) (margin : U16) (p : Position)
    (h : rect_contains (rect_inner r margin) p = true) :
    rect_contains r p = true
```

This lets us compose: child cells are in the inner area, inner area is a subset of the outer area, therefore child cells are in the outer area.

### Top-Level render_in_bounds

```lean
theorem render_in_bounds (screen : Rect) (cells : List Cell) (areas : List Rect)
    (h_areas_in_screen : ∀ (a : Rect), a ∈ areas →
      ∀ (p : Position), rect_contains a p = true → rect_contains screen p = true)
    (h_cells_in_areas : ∀ (c : Cell), c ∈ cells →
      ∃ (a : Rect), a ∈ areas ∧ rect_contains a c.pos = true) :
    cells_in_bounds cells screen
```

The proof chains two hypotheses: (1) all widget areas are within the screen, and (2) all cells are within their widget's area. Composition gives cells within the screen.

## Part 8: AppModel

The `AppModel` is the central state container, following the Elm architecture from Tutorial 04.

```rust
pub struct AppModel {
    pub widgets: Vec<WidgetKind>,   // The arena
    pub focus_index: usize,          // Which widget receives input
    pub areas: Vec<Rect>,           // Computed layout areas (parallel to widgets)
    pub screen_width: u16,
    pub screen_height: u16,
}
```

### Default Layout

`AppModel::new(width, height)` creates a five-widget arena:

| Index | Widget | Role |
|-------|--------|------|
| 0 | TextBox | Text input field |
| 1 | ScrollableList | Message/item display |
| 2 | StatusBar | Status line at bottom |
| 3 | Border | Frame around the TextBox (child_index = 0) |
| 4 | Container | Root: vertical split of [3, 1, 2] |

### The Update Loop

```rust
pub fn update(&mut self, event: Event) -> Action {
    let action = event_to_action(&event, self);
    match &action {
        Action::FocusNext => self.focus_next(),
        Action::InsertChar(c) => { /* modify focused TextBox */ },
        Action::Submit => { /* move TextBox content to ScrollableList */ },
        Action::Redraw => { self.layout(); },
        // ...
    }
    action
}
```

Each action modifies only the relevant widget. `InsertChar` and `DeleteChar` only touch the focused TextBox. `ScrollUp` and `ScrollDown` only touch the focused ScrollableList. This locality is what the input routing proof establishes.

### Layout Recomputation

```rust
pub fn layout(&mut self) {
    // Walk from root widget, assign areas using split()
    self.layout_widget(root_index, screen_rect);
}
```

The `layout_widget` method recursively assigns areas: containers split their area among children, borders compute `inner(1)` for their child, leaf widgets simply accept their area.

## Part 9: Focus Management

Focus determines which widget receives keyboard input. Only `TextBox` and `ScrollableList` are focusable.

### focus_next and focus_prev

```rust
pub fn focus_next(&mut self) {
    let len = self.widgets.len();
    let mut i: usize = 1;
    while i <= len {
        let idx = (self.focus_index + i) % len;
        if is_focusable(&self.widgets[idx]) {
            self.focus_index = idx;
            return;
        }
        i += 1;
    }
}
```

The modular arithmetic wraps around the arena. The loop visits every widget exactly once. If no widget is focusable, focus stays put.

### Proving Focus Validity

```lean
theorem focus_next_valid (widgets : List WidgetKind) (current : Nat)
    (h : current < widgets.length) (hne : widgets.length > 0) :
    focus_next widgets current < widgets.length
```

The key insight: `(current + 1 + i) % len < len` always holds by `Nat.mod_lt`. Since `focus_next` either returns such an index or returns `current` (which is already valid), the result is always in bounds.

The `single_focus` theorem states the obvious but important property: there is exactly one `focus_index`, so at most one widget is focused at any time.

## Part 10: Input Routing

Input routing proofs verify the connection between events and actions.

### Tab and BackTab

```lean
theorem tab_maps_to_focus_next (model : AppModel) :
    event_to_action (Event.Key KeyCode.Tab ⟨0⟩) model = Action.FocusNext := by
  simp [event_to_action]
```

These proofs are by direct computation — `simp` unfolds the `match` in `event_to_action` and the result is immediate.

### Character Events Target Focus

```lean
theorem char_produces_insert (c : U8) (model : AppModel) :
    event_to_action (Event.Key (KeyCode.Char c) ⟨0⟩) model = Action.InsertChar c
```

Combined with the fact that `InsertChar` only modifies `widgets[focus_index]`, this establishes that character input only affects the focused widget.

## Part 11: Scroll Bounds

The `scroll_clamp` function ensures scroll offsets stay valid:

```rust
pub fn scroll_clamp(offset: usize, item_count: usize, visible: usize) -> usize {
    if item_count <= visible { return 0; }
    let max_offset = item_count - visible;
    if offset > max_offset { max_offset } else { offset }
}
```

### Theorems

```lean
theorem scroll_clamp_upper_bound (offset item_count visible : Nat)
    (h : item_count > visible) :
    scroll_clamp offset item_count visible <= item_count - visible

theorem scroll_clamp_idempotent (offset item_count visible : Nat) :
    scroll_clamp (scroll_clamp offset item_count visible) item_count visible
      = scroll_clamp offset item_count visible
```

The upper bound theorem follows from `min` semantics. Idempotency ensures that repeated clamping (e.g., after multiple scroll events) doesn't drift.

## Part 12: Composing Verified Components

The power of this tutorial is composition. Individual proofs combine:

1. **Layout** produces non-overlapping areas that cover the screen
2. **Each widget** renders cells within its area
3. **The border** delegates to its child in a subset area (inner_subset)
4. **The container** delegates to children in split sub-areas (split_no_overlap)
5. **Top-level render_in_bounds** chains layout + per-widget bounds

This is the verification pattern for real UI frameworks: prove local properties per widget, then compose them into global guarantees.

### The Arena Pattern Enables Composition

Because widgets are in a flat list, we can state "for all widgets in the arena, rendering stays in bounds" as a simple `∀ w ∈ widgets` quantifier. No structural induction on a recursive type — just list iteration with an index invariant.

## Running the Code

### Rust

```bash
cd rust
cargo test
```

This runs tests for layout (split correctness, no overlap, coverage), widget rendering (all variants, bounds checking), and app model (focus cycling, input handling, submit flow).

### Lean

```bash
cd lean
lake build
```

This type-checks all proofs. Some theorems about `split` use `sorry` — these are deferred to full Aeneas integration where the generated Lean code provides the actual function definitions to reason about.

## Exercises

### Exercise 1: Add a Popup Widget

Add a `WidgetKind::Popup { content: Vec<u8>, child_index: usize }` variant that renders content centered over its child. Update `render_widget` and prove that popup cells are within the area.

### Exercise 2: Proportional Split

Implement `split_proportional(area, dir, ratios: &[u16]) -> Vec<Rect>` that splits according to given ratios (e.g., `[1, 2, 1]` gives 25%/50%/25%). Prove that the output still covers the original area with no overlap.

### Exercise 3: Focus Ring Proof

Prove that starting from any valid focus_index, calling `focus_next` exactly N times (where N is the number of focusable widgets) returns to the original index. This is a cycle-length proof.

### Exercise 4: Widget Tree Acyclicity

Define an `acyclic` predicate for the arena graph and prove that `valid_widget_tree ∧ acyclic` implies `widget_tree_depth widgets root widgets.length` terminates without hitting the fuel limit.

### Exercise 5: Full Render Bounds

Remove the `sorry` markers from `split_no_overlap` and `split_covers` by providing complete proofs. Hint: use `Nat.div_add_mod` for the remainder argument.

## Looking Ahead

This tutorial builds the pure TUI model. In Tutorial 11 (Full Integration), this becomes the rendering and input layer of the complete verified application. The proofs established here — layout correctness, render bounds, focus validity — become assumptions that the integration layer can rely on.

Tutorial 08 (LLM Client Core) builds the other major subsystem: a pure model for LLM API interactions. Together, Tutorials 07 and 08 provide the two halves of the UI: what you see (TUI Core) and what drives it (LLM Client).

## File Reference

### Rust Sources

| File | Lines | Description |
|------|-------|-------------|
| `src/geometry.rs` | ~80 | Rect, Position, Cell types and methods |
| `src/event.rs` | ~55 | KeyCode, Modifiers, Event enums |
| `src/layout.rs` | ~90 | split and split_at functions |
| `src/widget.rs` | ~230 | WidgetKind enum and render functions |
| `src/app_model.rs` | ~250 | AppModel, update, render, focus management |
| `src/action.rs` | ~15 | Action enum |

### Lean Proofs

| File | Description |
|------|-------------|
| `TuiCore/Types.lean` | Type definitions (simulated Aeneas output) |
| `TuiCore/Funs.lean` | Function definitions (simulated Aeneas output) |
| `TuiCore/RectLemmas.lean` | contains_in_range, inner_subset, non_overlapping_disjoint |
| `TuiCore/LayoutProof.lean` | split_no_overlap, split_covers, split_at_partition |
| `TuiCore/RenderBoundsProof.lean` | Per-widget and top-level render_in_bounds |
| `TuiCore/FocusProof.lean` | focus_next_valid, focus_prev_valid, single_focus |
| `TuiCore/InputRoutingProof.lean` | tab/backtab mapping, char routing |
| `TuiCore/ScrollProof.lean` | scroll_clamp bounds and idempotency |
| `TuiCore/WidgetTreeSpec.lean` | valid_widget_tree predicate, widget_tree_depth |

### Test Files

| File | Tests |
|------|-------|
| `tests/layout_tests.rs` | Split correctness, no overlap, coverage, split_at |
| `tests/widget_tests.rs` | All widget variants, bounds checking, scroll_clamp |
| `tests/app_model_tests.rs` | Focus cycling, input handling, submit, resize |

---

[← Previous: Tutorial 06](../06-buffer-management/README.md) | [Index](../README.md) | [Next: Tutorial 08 →](../08-llm-client-core/README.md)
