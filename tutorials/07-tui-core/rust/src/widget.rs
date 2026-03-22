//! Widget types using the arena pattern (flat Vec with index references).

use crate::geometry::{Cell, Position, Rect};
use crate::layout::{SplitDir, split};

/// Arena-pattern widget kinds. Children are referenced by index into a flat
/// `Vec<WidgetKind>`, avoiding recursive Box types that Aeneas cannot handle.
#[derive(Clone, PartialEq, Debug)]
pub enum WidgetKind {
    TextBox {
        content: Vec<u8>,
        cursor: usize,
    },
    ScrollableList {
        items: Vec<Vec<u8>>,
        selected: usize,
        scroll_offset: usize,
    },
    StatusBar {
        left_text: Vec<u8>,
        right_text: Vec<u8>,
    },
    Border {
        title: Vec<u8>,
        child_index: usize,
    },
    Container {
        dir: SplitDir,
        children: Vec<usize>,
    },
}

/// Clamp scroll offset so it stays in valid range.
pub fn scroll_clamp(offset: usize, item_count: usize, visible: usize) -> usize {
    if item_count <= visible {
        return 0;
    }
    let max_offset = item_count - visible;
    if offset > max_offset {
        max_offset
    } else {
        offset
    }
}

/// Render a single widget into cells within the given area.
/// `widgets` is the full arena, used for looking up children by index.
pub fn render_widget(kind: &WidgetKind, area: Rect, widgets: &[WidgetKind]) -> Vec<Cell> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    match kind {
        WidgetKind::TextBox { content, cursor: _ } => render_textbox(content, area),
        WidgetKind::ScrollableList {
            items,
            selected,
            scroll_offset,
        } => render_scrollable_list(items, *selected, *scroll_offset, area),
        WidgetKind::StatusBar {
            left_text,
            right_text,
        } => render_status_bar(left_text, right_text, area),
        WidgetKind::Border { title, child_index } => {
            render_border(title, *child_index, area, widgets)
        }
        WidgetKind::Container { dir, children } => render_container(*dir, children, area, widgets),
    }
}

fn render_textbox(content: &[u8], area: Rect) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    let max_chars = area.width as usize;
    let mut i: usize = 0;
    while i < content.len() && i < max_chars {
        cells.push(Cell {
            pos: Position {
                x: area.x.wrapping_add(i as u16),
                y: area.y,
            },
            ch: content[i],
            style: 0,
        });
        i += 1;
    }
    cells
}

fn render_scrollable_list(
    items: &[Vec<u8>],
    selected: usize,
    scroll_offset: usize,
    area: Rect,
) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    let visible_rows = area.height as usize;
    let clamped_offset = scroll_clamp(scroll_offset, items.len(), visible_rows);

    let mut row: usize = 0;
    while row < visible_rows && (clamped_offset + row) < items.len() {
        let item_idx = clamped_offset + row;
        let item = &items[item_idx];
        let style: u8 = if item_idx == selected { 1 } else { 0 };
        let max_cols = area.width as usize;
        let mut col: usize = 0;
        while col < item.len() && col < max_cols {
            cells.push(Cell {
                pos: Position {
                    x: area.x.wrapping_add(col as u16),
                    y: area.y.wrapping_add(row as u16),
                },
                ch: item[col],
                style,
            });
            col += 1;
        }
        row += 1;
    }
    cells
}

fn render_status_bar(left_text: &[u8], right_text: &[u8], area: Rect) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    let w = area.width as usize;

    // Render left text
    let mut i: usize = 0;
    while i < left_text.len() && i < w {
        cells.push(Cell {
            pos: Position {
                x: area.x.wrapping_add(i as u16),
                y: area.y,
            },
            ch: left_text[i],
            style: 2,
        });
        i += 1;
    }

    // Render right text, right-aligned
    if right_text.len() <= w {
        let start = w - right_text.len();
        let mut j: usize = 0;
        while j < right_text.len() {
            let col = start + j;
            cells.push(Cell {
                pos: Position {
                    x: area.x.wrapping_add(col as u16),
                    y: area.y,
                },
                ch: right_text[j],
                style: 2,
            });
            j += 1;
        }
    }

    cells
}

fn render_border(
    title: &[u8],
    child_index: usize,
    area: Rect,
    widgets: &[WidgetKind],
) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();

    // Draw top border
    let mut col: u16 = 0;
    while col < area.width {
        let ch = if col == 0 || col == area.width - 1 {
            b'+'
        } else {
            b'-'
        };
        cells.push(Cell {
            pos: Position {
                x: area.x.wrapping_add(col),
                y: area.y,
            },
            ch,
            style: 3,
        });
        col += 1;
    }

    // Draw title on top border
    let mut ti: usize = 0;
    while ti < title.len() && (ti + 2) < area.width as usize {
        cells.push(Cell {
            pos: Position {
                x: area.x.wrapping_add(2).wrapping_add(ti as u16),
                y: area.y,
            },
            ch: title[ti],
            style: 3,
        });
        ti += 1;
    }

    // Draw bottom border
    if area.height > 1 {
        let bottom_y = area.y.wrapping_add(area.height - 1);
        let mut col2: u16 = 0;
        while col2 < area.width {
            let ch = if col2 == 0 || col2 == area.width - 1 {
                b'+'
            } else {
                b'-'
            };
            cells.push(Cell {
                pos: Position {
                    x: area.x.wrapping_add(col2),
                    y: bottom_y,
                },
                ch,
                style: 3,
            });
            col2 += 1;
        }
    }

    // Draw left and right borders
    let mut row: u16 = 1;
    while row < area.height.wrapping_sub(1) {
        cells.push(Cell {
            pos: Position {
                x: area.x,
                y: area.y.wrapping_add(row),
            },
            ch: b'|',
            style: 3,
        });
        if area.width > 1 {
            cells.push(Cell {
                pos: Position {
                    x: area.x.wrapping_add(area.width - 1),
                    y: area.y.wrapping_add(row),
                },
                ch: b'|',
                style: 3,
            });
        }
        row += 1;
    }

    // Render child inside the border
    let inner = area.inner(1);
    if child_index < widgets.len() && inner.width > 0 && inner.height > 0 {
        let child_cells = render_widget(&widgets[child_index], inner, widgets);
        let mut ci: usize = 0;
        while ci < child_cells.len() {
            cells.push(child_cells[ci].clone());
            ci += 1;
        }
    }

    cells
}

fn render_container(
    dir: SplitDir,
    children: &[usize],
    area: Rect,
    widgets: &[WidgetKind],
) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    let sub_areas = split(area, dir, children.len());

    let mut i: usize = 0;
    while i < children.len() && i < sub_areas.len() {
        let child_idx = children[i];
        if child_idx < widgets.len() {
            let child_cells = render_widget(&widgets[child_idx], sub_areas[i], widgets);
            let mut j: usize = 0;
            while j < child_cells.len() {
                cells.push(child_cells[j].clone());
                j += 1;
            }
        }
        i += 1;
    }

    cells
}
