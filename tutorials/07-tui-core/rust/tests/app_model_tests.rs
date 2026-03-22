//! Tests for the AppModel.

use tui_core::action::Action;
use tui_core::app_model::AppModel;
use tui_core::event::{Event, KeyCode, Modifiers};
use tui_core::geometry::Rect;

#[test]
fn new_model_has_widgets() {
    let model = AppModel::new(80, 24);
    assert_eq!(model.widgets.len(), 5);
    assert_eq!(model.screen_width, 80);
    assert_eq!(model.screen_height, 24);
}

#[test]
fn new_model_has_areas() {
    let model = AppModel::new(80, 24);
    assert_eq!(model.areas.len(), model.widgets.len());
}

#[test]
fn focus_starts_at_zero() {
    let model = AppModel::new(80, 24);
    assert_eq!(model.focus_index, 0);
}

#[test]
fn focus_next_cycles() {
    let mut model = AppModel::new(80, 24);
    // Focus starts at 0 (TextBox), next focusable is 1 (ScrollableList)
    model.focus_next();
    assert_eq!(model.focus_index, 1);
    // Next focusable wraps back to 0 (TextBox)
    model.focus_next();
    assert_eq!(model.focus_index, 0);
}

#[test]
fn focus_prev_cycles() {
    let mut model = AppModel::new(80, 24);
    // Focus starts at 0, prev should go to 1 (ScrollableList)
    model.focus_prev();
    assert_eq!(model.focus_index, 1);
    model.focus_prev();
    assert_eq!(model.focus_index, 0);
}

#[test]
fn focus_index_always_valid() {
    let mut model = AppModel::new(80, 24);
    let mut i = 0;
    while i < 20 {
        model.focus_next();
        assert!(model.focus_index < model.widgets.len());
        i += 1;
    }
    i = 0;
    while i < 20 {
        model.focus_prev();
        assert!(model.focus_index < model.widgets.len());
        i += 1;
    }
}

#[test]
fn tab_triggers_focus_next() {
    let mut model = AppModel::new(80, 24);
    let action = model.update(Event::Key(KeyCode::Tab, Modifiers::none()));
    assert_eq!(action, Action::FocusNext);
    assert_eq!(model.focus_index, 1);
}

#[test]
fn backtab_triggers_focus_prev() {
    let mut model = AppModel::new(80, 24);
    model.focus_index = 1;
    let action = model.update(Event::Key(KeyCode::BackTab, Modifiers::none()));
    assert_eq!(action, Action::FocusPrev);
    assert_eq!(model.focus_index, 0);
}

#[test]
fn insert_char_into_textbox() {
    let mut model = AppModel::new(80, 24);
    model.update(Event::Key(KeyCode::Char(b'a'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Char(b'b'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Char(b'c'), Modifiers::none()));

    if let tui_core::widget::WidgetKind::TextBox { ref content, cursor } = model.widgets[0] {
        assert_eq!(content, &b"abc".to_vec());
        assert_eq!(cursor, 3);
    } else {
        panic!("widget 0 should be TextBox");
    }
}

#[test]
fn delete_char_from_textbox() {
    let mut model = AppModel::new(80, 24);
    model.update(Event::Key(KeyCode::Char(b'a'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Char(b'b'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Backspace, Modifiers::none()));

    if let tui_core::widget::WidgetKind::TextBox { ref content, cursor } = model.widgets[0] {
        assert_eq!(content, &b"a".to_vec());
        assert_eq!(cursor, 1);
    } else {
        panic!("widget 0 should be TextBox");
    }
}

#[test]
fn submit_moves_text_to_list() {
    let mut model = AppModel::new(80, 24);
    model.update(Event::Key(KeyCode::Char(b'h'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Char(b'i'), Modifiers::none()));
    let action = model.update(Event::Key(KeyCode::Enter, Modifiers::none()));
    assert_eq!(action, Action::Submit);

    // TextBox should be cleared
    if let tui_core::widget::WidgetKind::TextBox { ref content, .. } = model.widgets[0] {
        assert!(content.is_empty());
    } else {
        panic!("widget 0 should be TextBox");
    }

    // ScrollableList should have the item
    if let tui_core::widget::WidgetKind::ScrollableList { ref items, .. } = model.widgets[1] {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], b"hi".to_vec());
    } else {
        panic!("widget 1 should be ScrollableList");
    }
}

#[test]
fn escape_returns_quit() {
    let mut model = AppModel::new(80, 24);
    let action = model.update(Event::Key(KeyCode::Escape, Modifiers::none()));
    assert_eq!(action, Action::Quit);
}

#[test]
fn ctrl_c_returns_quit() {
    let mut model = AppModel::new(80, 24);
    let action = model.update(Event::Key(
        KeyCode::Char(b'c'),
        Modifiers { bits: Modifiers::CTRL },
    ));
    assert_eq!(action, Action::Quit);
}

#[test]
fn resize_triggers_relayout() {
    let mut model = AppModel::new(80, 24);
    let action = model.update(Event::Resize(120, 40));
    assert_eq!(action, Action::Redraw);
    assert_eq!(model.screen_width, 120);
    assert_eq!(model.screen_height, 40);
}

#[test]
fn render_produces_cells() {
    let model = AppModel::new(80, 24);
    let cells = model.render();
    // The default layout has a border, status bar text etc.
    // There should be some cells rendered
    assert!(!cells.is_empty());
}

#[test]
fn render_all_cells_in_screen_bounds() {
    let mut model = AppModel::new(80, 24);
    // Add some content
    model.update(Event::Key(KeyCode::Char(b'x'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Enter, Modifiers::none()));

    let screen = Rect::new(0, 0, 80, 24);
    let cells = model.render();
    let mut i: usize = 0;
    while i < cells.len() {
        assert!(
            screen.contains(cells[i].pos),
            "Cell at ({}, {}) outside screen",
            cells[i].pos.x,
            cells[i].pos.y,
        );
        i += 1;
    }
}

#[test]
fn cursor_movement() {
    let mut model = AppModel::new(80, 24);
    model.update(Event::Key(KeyCode::Char(b'a'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Char(b'b'), Modifiers::none()));
    model.update(Event::Key(KeyCode::Char(b'c'), Modifiers::none()));
    // Cursor at 3
    model.update(Event::Key(KeyCode::Left, Modifiers::none()));
    // Cursor at 2
    if let tui_core::widget::WidgetKind::TextBox { cursor, .. } = model.widgets[0] {
        assert_eq!(cursor, 2);
    }
    model.update(Event::Key(KeyCode::Right, Modifiers::none()));
    if let tui_core::widget::WidgetKind::TextBox { cursor, .. } = model.widgets[0] {
        assert_eq!(cursor, 3);
    }
}

#[test]
fn tick_is_noop() {
    let mut model = AppModel::new(80, 24);
    let action = model.update(Event::Tick);
    assert_eq!(action, Action::Noop);
}
