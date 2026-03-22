// app_view.rs — Pure view function: AppState -> ViewTree.
//
// `app_view` turns the current application state into a tree of cells that
// the shell can blit to the terminal.  It performs no I/O; the shell is
// responsible for actually writing the cells to the screen.

use crate::verified_core::app_state::AppState;
use crate::verified_core::integration_types::PaneId;

/// A single character cell in the output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewCell {
    pub x: u16,
    pub y: u16,
    pub ch: u8,
}

/// The complete rendered output as a flat list of cells.
///
/// The shell iterates `cells` and writes each character at the given
/// coordinates.  This representation is deliberately simple so that the
/// Lean translation is straightforward.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewTree {
    pub cells: Vec<ViewCell>,
}

/// Render the entire application state into a `ViewTree`.
///
/// The screen is divided into four regions:
///
/// ```text
/// ┌──────────────────────────┬──────────────┐
/// │                          │  Agent       │
/// │  Conversation View       │  Status      │
/// │                          │  Panel       │
/// ├──────────────────────────┤              │
/// │  Chat Input              ├──────────────┤
/// │                          │  Debug Panel │
/// └──────────────────────────┴──────────────┘
/// ```
///
/// Each pane is rendered by a helper that writes cells into the vector.
pub fn app_view(state: &AppState, width: u16, height: u16) -> ViewTree {
    let mut cells = Vec::new();

    if width < 4 || height < 4 {
        // Too small to render anything useful.
        return ViewTree { cells };
    }

    // Split: left panes take 2/3 width, right panes take the rest.
    let left_w = (width * 2) / 3;
    let right_w = width - left_w;

    // Top half / bottom half.
    let top_h = (height * 3) / 4;
    let bot_h = height - top_h;

    // Right panes split vertically.
    let right_top_h = height / 2;
    let right_bot_h = height - right_top_h;

    // ── Conversation view (top-left) ──
    render_conversation(state, &mut cells, 0, 0, left_w, top_h);

    // ── Chat input (bottom-left) ──
    render_input(state, &mut cells, 0, top_h, left_w, bot_h);

    // ── Agent status panel (top-right) ──
    render_agent_status(state, &mut cells, left_w, 0, right_w, right_top_h);

    // ── Debug reasoning panel (bottom-right) ──
    render_debug(state, &mut cells, left_w, right_top_h, right_w, right_bot_h);

    ViewTree { cells }
}

// ── Pane renderers ─────────────────────────────────────────────────────────

/// Write a horizontal label at (ox, oy).
fn write_label(cells: &mut Vec<ViewCell>, ox: u16, oy: u16, label: &[u8], max_w: u16) {
    for (i, &ch) in label.iter().enumerate() {
        if (i as u16) >= max_w {
            break;
        }
        cells.push(ViewCell {
            x: ox + i as u16,
            y: oy,
            ch,
        });
    }
}

fn render_conversation(
    state: &AppState,
    cells: &mut Vec<ViewCell>,
    ox: u16,
    oy: u16,
    w: u16,
    h: u16,
) {
    let label = if state.active_pane == PaneId::ConversationView {
        b"[Conversation]*" as &[u8]
    } else {
        b"[Conversation]" as &[u8]
    };
    write_label(cells, ox, oy, label, w);

    // Render the last (h-1) conversation entries, one per row.
    let max_rows = (h as usize).saturating_sub(1);
    let start = state.conversations.len().saturating_sub(max_rows);
    for (i, entry) in state.conversations[start..].iter().enumerate() {
        // Render a compact representation: "A<agent_id>:<content_id>"
        let tag = match entry.role {
            0 => b'U',
            1 => b'A',
            _ => b'S',
        };
        let row = oy + 1 + i as u16;
        if row >= oy + h {
            break;
        }
        cells.push(ViewCell { x: ox, y: row, ch: tag });
        // Agent id digit (simplified: single digit).
        cells.push(ViewCell {
            x: ox + 1,
            y: row,
            ch: b'0' + (entry.agent_id % 10) as u8,
        });
        cells.push(ViewCell { x: ox + 2, y: row, ch: b':' });
    }
}

fn render_input(
    state: &AppState,
    cells: &mut Vec<ViewCell>,
    ox: u16,
    oy: u16,
    w: u16,
    _h: u16,
) {
    let label = if state.active_pane == PaneId::ChatInput {
        b"[Input]*" as &[u8]
    } else {
        b"[Input]" as &[u8]
    };
    write_label(cells, ox, oy, label, w);

    // Render buffer contents on the next row.
    for (i, &ch) in state.input_buffer.iter().enumerate() {
        if (i as u16 + ox) >= ox + w {
            break;
        }
        cells.push(ViewCell {
            x: ox + i as u16,
            y: oy + 1,
            ch,
        });
    }
}

fn render_agent_status(
    state: &AppState,
    cells: &mut Vec<ViewCell>,
    ox: u16,
    oy: u16,
    w: u16,
    h: u16,
) {
    let label = if state.active_pane == PaneId::AgentStatusPanel {
        b"[Agents]*" as &[u8]
    } else {
        b"[Agents]" as &[u8]
    };
    write_label(cells, ox, oy, label, w);

    // One row per agent (up to h-1).
    for i in 0..state.agent_count {
        let row = oy + 1 + i as u16;
        if row >= oy + h {
            break;
        }
        let marker = if i == state.selected_agent { b'>' } else { b' ' };
        cells.push(ViewCell { x: ox, y: row, ch: marker });
        cells.push(ViewCell {
            x: ox + 1,
            y: row,
            ch: b'0' + (i % 10) as u8,
        });
    }
}

fn render_debug(
    state: &AppState,
    cells: &mut Vec<ViewCell>,
    ox: u16,
    oy: u16,
    w: u16,
    _h: u16,
) {
    if !state.debug_visible {
        write_label(cells, ox, oy, b"[Debug: hidden]", w);
        return;
    }
    let label = if state.active_pane == PaneId::DebugReasoningPanel {
        b"[Debug]*" as &[u8]
    } else {
        b"[Debug]" as &[u8]
    };
    write_label(cells, ox, oy, label, w);
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verified_core::app_state::AppState;

    #[test]
    fn view_produces_cells_for_empty_state() {
        let s = AppState::new(3, 10);
        let v = app_view(&s, 80, 24);
        // Should have at least the pane labels.
        assert!(!v.cells.is_empty());
    }

    #[test]
    fn view_too_small_returns_empty() {
        let s = AppState::new(1, 5);
        let v = app_view(&s, 2, 2);
        assert!(v.cells.is_empty());
    }

    #[test]
    fn view_reflects_input_buffer() {
        let mut s = AppState::new(1, 5);
        s.input_buffer = vec![b'A', b'B'];
        let v = app_view(&s, 80, 24);
        let ab_cells: Vec<_> = v.cells.iter().filter(|c| c.ch == b'A' || c.ch == b'B').collect();
        assert!(ab_cells.len() >= 2);
    }

    #[test]
    fn view_shows_active_pane_marker() {
        let s = AppState::new(1, 5);
        let v = app_view(&s, 80, 24);
        // ChatInput is active by default — look for the '*' marker.
        let stars: Vec<_> = v.cells.iter().filter(|c| c.ch == b'*').collect();
        assert!(!stars.is_empty(), "active pane should have a * marker");
    }
}
