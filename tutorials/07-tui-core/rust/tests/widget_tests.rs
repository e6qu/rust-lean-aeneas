//! Tests for widget rendering.

use tui_core::geometry::{Position, Rect};
use tui_core::layout::SplitDir;
use tui_core::widget::{WidgetKind, render_widget, scroll_clamp};

#[test]
fn render_textbox_basic() {
    let widgets = vec![WidgetKind::TextBox {
        content: b"hello".to_vec(),
        cursor: 5,
    }];
    let area = Rect::new(0, 0, 20, 1);
    let cells = render_widget(&widgets[0], area, &widgets);
    assert_eq!(cells.len(), 5);
    assert_eq!(cells[0].ch, b'h');
    assert_eq!(cells[4].ch, b'o');
    assert_eq!(cells[0].pos, Position { x: 0, y: 0 });
}

#[test]
fn render_textbox_truncated() {
    let widgets = vec![WidgetKind::TextBox {
        content: b"abcdefghij".to_vec(),
        cursor: 0,
    }];
    let area = Rect::new(0, 0, 5, 1);
    let cells = render_widget(&widgets[0], area, &widgets);
    // Only 5 chars fit
    assert_eq!(cells.len(), 5);
    assert_eq!(cells[4].ch, b'e');
}

#[test]
fn render_textbox_empty() {
    let widgets = vec![WidgetKind::TextBox {
        content: Vec::new(),
        cursor: 0,
    }];
    let area = Rect::new(0, 0, 20, 1);
    let cells = render_widget(&widgets[0], area, &widgets);
    assert!(cells.is_empty());
}

#[test]
fn render_scrollable_list() {
    let widgets = vec![WidgetKind::ScrollableList {
        items: vec![b"alpha".to_vec(), b"beta".to_vec(), b"gamma".to_vec()],
        selected: 1,
        scroll_offset: 0,
    }];
    let area = Rect::new(0, 0, 10, 3);
    let cells = render_widget(&widgets[0], area, &widgets);
    // 5 + 4 + 5 = 14 cells
    assert_eq!(cells.len(), 14);
    // selected item (beta) should have style 1
    let beta_cells: Vec<_> = cells.iter().filter(|c| c.style == 1).collect();
    assert_eq!(beta_cells.len(), 4); // "beta" has 4 chars
}

#[test]
fn render_scrollable_list_with_scroll() {
    let widgets = vec![WidgetKind::ScrollableList {
        items: vec![
            b"a".to_vec(),
            b"b".to_vec(),
            b"c".to_vec(),
            b"d".to_vec(),
            b"e".to_vec(),
        ],
        selected: 3,
        scroll_offset: 2,
    }];
    let area = Rect::new(0, 0, 5, 2); // only 2 visible rows
    let cells = render_widget(&widgets[0], area, &widgets);
    // scroll_offset=2, visible=2, so we see items[2]="c" and items[3]="d"
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0].ch, b'c');
    assert_eq!(cells[1].ch, b'd');
    // items[3] is selected
    assert_eq!(cells[1].style, 1);
}

#[test]
fn render_status_bar() {
    let widgets = vec![WidgetKind::StatusBar {
        left_text: b"Ready".to_vec(),
        right_text: b"OK".to_vec(),
    }];
    let area = Rect::new(0, 0, 20, 1);
    let cells = render_widget(&widgets[0], area, &widgets);
    // "Ready" = 5 chars + "OK" = 2 chars = 7 total
    assert_eq!(cells.len(), 7);
    assert_eq!(cells[0].ch, b'R');
}

#[test]
fn render_border() {
    let widgets = vec![
        WidgetKind::TextBox {
            content: b"hi".to_vec(),
            cursor: 2,
        },
        WidgetKind::Border {
            title: b"T".to_vec(),
            child_index: 0,
        },
    ];
    let area = Rect::new(0, 0, 10, 5);
    let cells = render_widget(&widgets[1], area, &widgets);
    // Should have border chars + child chars
    assert!(!cells.is_empty());
    // Top-left corner
    let top_left: Vec<_> = cells
        .iter()
        .filter(|c| c.pos == Position { x: 0, y: 0 })
        .collect();
    assert!(!top_left.is_empty());
}

#[test]
fn render_container() {
    let widgets = vec![
        WidgetKind::TextBox {
            content: b"A".to_vec(),
            cursor: 1,
        },
        WidgetKind::TextBox {
            content: b"B".to_vec(),
            cursor: 1,
        },
        WidgetKind::Container {
            dir: SplitDir::Horizontal,
            children: vec![0, 1],
        },
    ];
    let area = Rect::new(0, 0, 20, 1);
    let cells = render_widget(&widgets[2], area, &widgets);
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0].ch, b'A');
    assert_eq!(cells[1].ch, b'B');
}

#[test]
fn render_zero_area() {
    let widgets = vec![WidgetKind::TextBox {
        content: b"x".to_vec(),
        cursor: 0,
    }];
    let area = Rect::new(0, 0, 0, 0);
    let cells = render_widget(&widgets[0], area, &widgets);
    assert!(cells.is_empty());
}

#[test]
fn render_in_bounds() {
    // All cells should be within the allocated area
    let widgets = vec![
        WidgetKind::TextBox {
            content: b"hello world".to_vec(),
            cursor: 0,
        },
        WidgetKind::ScrollableList {
            items: vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()],
            selected: 0,
            scroll_offset: 0,
        },
        WidgetKind::StatusBar {
            left_text: b"left".to_vec(),
            right_text: b"right".to_vec(),
        },
        WidgetKind::Border {
            title: b"Input".to_vec(),
            child_index: 0,
        },
        WidgetKind::Container {
            dir: SplitDir::Vertical,
            children: vec![3, 1, 2],
        },
    ];
    let area = Rect::new(0, 0, 40, 20);
    let cells = render_widget(&widgets[4], area, &widgets);
    let mut i: usize = 0;
    while i < cells.len() {
        assert!(
            area.contains(cells[i].pos),
            "Cell at ({}, {}) is outside area {:?}",
            cells[i].pos.x,
            cells[i].pos.y,
            area,
        );
        i += 1;
    }
}

#[test]
fn scroll_clamp_basic() {
    assert_eq!(scroll_clamp(0, 10, 5), 0);
    assert_eq!(scroll_clamp(5, 10, 5), 5);
    assert_eq!(scroll_clamp(6, 10, 5), 5);
    assert_eq!(scroll_clamp(100, 10, 5), 5);
}

#[test]
fn scroll_clamp_items_less_than_visible() {
    assert_eq!(scroll_clamp(5, 3, 10), 0);
    assert_eq!(scroll_clamp(0, 0, 5), 0);
}
