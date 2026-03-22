# Tutorial 07: TUI Core

## Goal

Implement a pure TUI model (layout, widgets, events, focus) with zero terminal I/O; prove layout correctness and render bounds.

## File Structure

```
07-tui-core/
├── README.md
├── PLAN.md
├── rust/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── geometry.rs         # Rect, Position, Cell
│   │   ├── event.rs            # Event, KeyCode, Modifiers
│   │   ├── widget.rs           # WidgetKind enum (arena pattern)
│   │   ├── layout.rs           # split, split_at
│   │   ├── app_model.rs        # AppModel, update, render, focus_next/prev
│   │   └── action.rs           # Action enum
│   └── tests/
│       ├── layout_tests.rs
│       ├── widget_tests.rs
│       └── app_model_tests.rs
└── lean/
    ├── lakefile.lean
    ├── lean-toolchain
    ├── TuiCore/Types.lean
    ├── TuiCore/Funs.lean
    ├── TuiCore/RectLemmas.lean         # Rect arithmetic
    ├── TuiCore/LayoutProof.lean        # split correctness
    ├── TuiCore/RenderBoundsProof.lean
    ├── TuiCore/FocusProof.lean
    ├── TuiCore/InputRoutingProof.lean
    ├── TuiCore/ScrollProof.lean
    └── TuiCore/WidgetTreeSpec.lean
```

## Rust Code Outline

### Types

- **`Rect`** — Rectangle with `x: u16, y: u16, width: u16, height: u16`. Core geometric primitive for layout.
- **`Position`** — `x: u16, y: u16`. A single coordinate in the terminal grid.
- **`Cell`** — `pos: Position, ch: u8, style: u8`. A single rendered cell with character and style byte.
- **`Event`** — Enum: `Key(KeyCode, Modifiers) | Resize(u16, u16) | Tick`. Input events with no I/O dependency.
- **`KeyCode`** — Enum: `Char(u8) | Enter | Esc | Tab | BackTab | Up | Down | Left | Right | Backspace | Delete`.
- **`Modifiers`** — `u8` bitflags: `NONE`, `CTRL`, `ALT`, `SHIFT`.
- **`WidgetKind`** — Enum (arena pattern): `TextBox(InputBuffer) | ScrollableList { items: Vec<Vec<u8>>, offset: u32, selected: u32 } | StatusBar(Vec<u8>) | Border { title: Vec<u8>, child: u32 } | Container { children: Vec<u32>, direction: Direction }`. Children referenced by `u32` index into a flat `Vec<WidgetKind>`.
- **`Direction`** — Enum: `Horizontal | Vertical`.
- **`Action`** — Enum: `Noop | InsertChar(u8) | DeleteChar | MoveCursor(i32) | ScrollUp | ScrollDown | FocusNext | FocusPrev | Submit | Quit`.
- **`AppModel`** — `widgets: Vec<WidgetKind>, areas: Vec<Rect>, focus_index: u32`. The entire UI state as a flat arena.

### Functions

- **`rect_contains(rect: &Rect, pos: &Position) -> bool`** — Point-in-rectangle test.
- **`rect_intersection(a: &Rect, b: &Rect) -> Option<Rect>`** — Compute intersection of two rectangles.
- **`split(rect: &Rect, direction: Direction, count: u32) -> Vec<Rect>`** — Split a rectangle into `count` equal parts along `direction`.
- **`split_at(rect: &Rect, direction: Direction, at: u16) -> (Rect, Rect)`** — Split at an absolute coordinate.
- **`event_to_action(event: &Event, model: &AppModel) -> Action`** — Map raw events to semantic actions based on focus.
- **`app_update(model: AppModel, action: Action) -> AppModel`** — Pure Elm-style update; returns new model.
- **`app_render(model: &AppModel) -> Vec<Cell>`** — Pure render: produces a flat list of cells, all within bounds.
- **`focus_next(model: AppModel) -> AppModel`** — Advance focus to next focusable widget (wraps around).
- **`focus_prev(model: AppModel) -> AppModel`** — Move focus to previous focusable widget (wraps around).
- **`widget_render(widget: &WidgetKind, area: &Rect) -> Vec<Cell>`** — Render a single widget into its allocated area.
- **`scroll_clamp(offset: u32, item_count: u32, visible: u32) -> u32`** — Clamp scroll offset to valid range.

### Estimated Lines

~600 lines Rust.

## Generated Lean (Approximate)

Aeneas will produce:

- **`TuiCore/Types.lean`**: Inductive types for `Rect`, `Position`, `Cell`, `Event`, `KeyCode`, `Modifiers`, `WidgetKind`, `Direction`, `Action`, `AppModel`. `WidgetKind` uses `Nat` indices for children (arena pattern translates to list indexing).
- **`TuiCore/Funs.lean`**: All functions as `def` with `Result` return types. `split` and `app_render` produce `List Rect` / `List Cell`. `app_update` is a large match on `Action`. Widget rendering recurses through the arena via index lookup.

Key translation notes:
- `u16` becomes `U16` (bounded natural); `Vec<T>` becomes `List T`.
- Arena `u32` indices become `Nat` with bounds checked via `List.get?`.
- The arena pattern means no actual recursion on `WidgetKind` — just index-based lookups, which Aeneas handles cleanly.

## Theorems to Prove

### `split_no_overlap`
**Statement:** For any `rect` and `direction`, the rectangles returned by `split` are pairwise non-overlapping.
**Proof strategy:** Show that consecutive sub-rectangles have adjacent coordinates with no gap/overlap by induction on the split list and arithmetic on offsets.

### `split_covers`
**Statement:** The union of rectangles from `split rect dir n` equals `rect` (every position in `rect` is in exactly one sub-rectangle).
**Proof strategy:** Show total width/height of sub-rectangles sums to original, combined with `split_no_overlap` gives exact cover.

### `render_in_bounds`
**Statement:** Every `Cell` produced by `app_render model` satisfies `rect_contains (screen_rect model) cell.pos = true`.
**Proof strategy:** Induct on the widget list; each `widget_render` call is bounded by its `area`, and areas are produced by `split` which is bounded by the screen rect.

### `single_focus`
**Statement:** In any `AppModel`, at most one widget index equals `focus_index`, and that index is in bounds.
**Proof strategy:** Direct from the representation — `focus_index` is a single `u32`; show `focus_next`/`focus_prev` preserve the in-bounds invariant using modular arithmetic.

### `key_events_to_focus`
**Statement:** `event_to_action (Key Tab _) model` returns `FocusNext` and `event_to_action (Key BackTab _) model` returns `FocusPrev`.
**Proof strategy:** Unfold `event_to_action` and simplify; direct by computation.

### `scroll_in_bounds`
**Statement:** `scroll_clamp offset item_count visible <= item_count - visible` (when `item_count >= visible`).
**Proof strategy:** Unfold `scroll_clamp` definition; it uses `min`, so the result follows from `Nat.min_le_right`.

### Estimated Lines

~900 lines proofs.

## New Lean Concepts Introduced

- **Widget trees as inductives**: Modeling a widget arena (flat vector with index-based children) as a Lean inductive, reasoning about tree structure via index lookups.
- **Structural proofs on trees**: Proving properties that hold across all widgets in the arena by iterating over the list with invariants.
- **Rect arithmetic lemmas**: Building a small library of rectangle lemmas (containment, splitting, intersection) that compose for layout proofs.
- **Composing verified components**: Combining render-bounds proofs for individual widgets into a whole-screen guarantee.

## Cross-References

- **From Tutorial 06 (Buffer Management):** The `TextBox` widget wraps an `InputBuffer` (gap buffer) from Tutorial 06. Cursor movement and text editing reuse the verified buffer operations.
- **From Tutorial 04 (State Machines):** `app_update` follows the same pure state-machine update pattern introduced in Tutorial 04, now applied to UI state.
- **To Tutorial 11 (Full Integration):** The TUI core becomes the rendering and input layer of the final integrated application.
