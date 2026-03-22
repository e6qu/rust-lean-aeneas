//! AppModel: Elm-style pure application model with focus management.

use crate::action::Action;
use crate::event::{Event, KeyCode};
use crate::geometry::{Cell, Rect};
use crate::layout::{SplitDir, split};
use crate::widget::{WidgetKind, render_widget, scroll_clamp};

/// The entire UI state as a flat arena.
pub struct AppModel {
    pub widgets: Vec<WidgetKind>,
    pub focus_index: usize,
    pub areas: Vec<Rect>,
    pub screen_width: u16,
    pub screen_height: u16,
}

/// Map a raw event to a semantic action based on current focus.
pub fn event_to_action(event: &Event, _model: &AppModel) -> Action {
    match event {
        Event::Key(keycode, mods) => {
            if mods.ctrl() {
                match keycode {
                    KeyCode::Char(b'c') | KeyCode::Char(b'q') => return Action::Quit,
                    _ => {}
                }
            }
            match keycode {
                KeyCode::Tab => Action::FocusNext,
                KeyCode::BackTab => Action::FocusPrev,
                KeyCode::Enter => Action::Submit,
                KeyCode::Char(c) => Action::InsertChar(*c),
                KeyCode::Backspace => Action::DeleteChar,
                KeyCode::Delete => Action::DeleteChar,
                KeyCode::Left => Action::MoveCursor(-1),
                KeyCode::Right => Action::MoveCursor(1),
                KeyCode::Up => Action::ScrollUp,
                KeyCode::Down => Action::ScrollDown,
                KeyCode::Escape => Action::Quit,
                _ => Action::Noop,
            }
        }
        Event::Resize(_, _) => Action::Redraw,
        Event::Tick => Action::Noop,
    }
}

impl AppModel {
    /// Create a default layout: a container with a bordered text box,
    /// a scrollable list, and a status bar.
    pub fn new(width: u16, height: u16) -> Self {
        // Arena layout:
        // 0: TextBox (the input field)
        // 1: ScrollableList
        // 2: StatusBar
        // 3: Border around TextBox (child_index = 0)
        // 4: Container (root) [3, 1, 2] vertical
        let widgets: Vec<WidgetKind> = vec![
            WidgetKind::TextBox {
                content: Vec::new(),
                cursor: 0,
            },
            WidgetKind::ScrollableList {
                items: Vec::new(),
                selected: 0,
                scroll_offset: 0,
            },
            WidgetKind::StatusBar {
                left_text: b"Ready".to_vec(),
                right_text: b"07-tui-core".to_vec(),
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

        let mut model = AppModel {
            widgets,
            focus_index: 0,
            areas: Vec::new(),
            screen_width: width,
            screen_height: height,
        };
        model.layout();
        model
    }

    /// Recompute areas from the root widget (last in arena).
    pub fn layout(&mut self) {
        let screen = Rect::new(0, 0, self.screen_width, self.screen_height);
        let widget_count = self.widgets.len();

        // Initialize areas to the screen rect
        self.areas = Vec::new();
        let mut i: usize = 0;
        while i < widget_count {
            self.areas.push(screen);
            i += 1;
        }

        // Walk from root (last widget) and assign areas
        if widget_count > 0 {
            self.layout_widget(widget_count - 1, screen);
        }
    }

    fn layout_widget(&mut self, idx: usize, area: Rect) {
        if idx >= self.widgets.len() {
            return;
        }
        self.areas[idx] = area;

        // We need to extract child info without borrowing self immutably
        // while we also need &mut self for recursive calls.
        // Clone just the necessary data.
        let widget = self.widgets[idx].clone();
        match widget {
            WidgetKind::Border { child_index, .. } => {
                let inner = area.inner(1);
                self.layout_widget(child_index, inner);
            }
            WidgetKind::Container { dir, ref children } => {
                let sub_areas = split(area, dir, children.len());
                let mut ci: usize = 0;
                while ci < children.len() && ci < sub_areas.len() {
                    self.layout_widget(children[ci], sub_areas[ci]);
                    ci += 1;
                }
            }
            _ => {}
        }
    }

    /// Process an action and update the model. Returns the resulting action
    /// for the caller to interpret (e.g., Quit, Submit).
    pub fn update(&mut self, event: Event) -> Action {
        let action = event_to_action(&event, self);

        match &action {
            Action::Quit => {}
            Action::FocusNext => {
                self.focus_next();
            }
            Action::FocusPrev => {
                self.focus_prev();
            }
            Action::InsertChar(c) => {
                let c = *c;
                let fi = self.focus_index;
                if fi < self.widgets.len()
                    && let WidgetKind::TextBox {
                        ref mut content,
                        ref mut cursor,
                    } = self.widgets[fi]
                {
                    if *cursor > content.len() {
                        *cursor = content.len();
                    }
                    content.insert(*cursor, c);
                    *cursor += 1;
                }
            }
            Action::DeleteChar => {
                let fi = self.focus_index;
                if fi < self.widgets.len()
                    && let WidgetKind::TextBox {
                        ref mut content,
                        ref mut cursor,
                    } = self.widgets[fi]
                    && *cursor > 0
                    && !content.is_empty()
                {
                    *cursor -= 1;
                    content.remove(*cursor);
                }
            }
            Action::MoveCursor(delta) => {
                let delta = *delta;
                let fi = self.focus_index;
                if fi < self.widgets.len()
                    && let WidgetKind::TextBox {
                        ref content,
                        ref mut cursor,
                    } = self.widgets[fi]
                {
                    if delta < 0 {
                        let abs_delta = (-delta) as usize;
                        if *cursor >= abs_delta {
                            *cursor -= abs_delta;
                        } else {
                            *cursor = 0;
                        }
                    } else {
                        *cursor += delta as usize;
                        if *cursor > content.len() {
                            *cursor = content.len();
                        }
                    }
                }
            }
            Action::ScrollUp => {
                let fi = self.focus_index;
                if fi < self.widgets.len()
                    && let WidgetKind::ScrollableList {
                        ref mut selected,
                        ref items,
                        ref mut scroll_offset,
                    } = self.widgets[fi]
                {
                    if *selected > 0 {
                        *selected -= 1;
                    }
                    let visible = if fi < self.areas.len() {
                        self.areas[fi].height as usize
                    } else {
                        1
                    };
                    if *selected < *scroll_offset {
                        *scroll_offset = *selected;
                    }
                    *scroll_offset = scroll_clamp(*scroll_offset, items.len(), visible);
                }
            }
            Action::ScrollDown => {
                let fi = self.focus_index;
                if fi < self.widgets.len()
                    && let WidgetKind::ScrollableList {
                        ref mut selected,
                        ref items,
                        ref mut scroll_offset,
                    } = self.widgets[fi]
                {
                    if *selected + 1 < items.len() {
                        *selected += 1;
                    }
                    let visible = if fi < self.areas.len() {
                        self.areas[fi].height as usize
                    } else {
                        1
                    };
                    if *selected >= *scroll_offset + visible {
                        *scroll_offset = *selected - visible + 1;
                    }
                    *scroll_offset = scroll_clamp(*scroll_offset, items.len(), visible);
                }
            }
            Action::Submit => {
                // Extract text from focused TextBox and add to list widget
                let fi = self.focus_index;
                if fi < self.widgets.len()
                    && let WidgetKind::TextBox { ref content, .. } = self.widgets[fi]
                {
                    let text = content.clone();
                    if !text.is_empty() {
                        // Find the first ScrollableList and add to it
                        let mut si: usize = 0;
                        while si < self.widgets.len() {
                            if let WidgetKind::ScrollableList { .. } = self.widgets[si] {
                                break;
                            }
                            si += 1;
                        }
                        if si < self.widgets.len()
                            && let WidgetKind::ScrollableList {
                                ref mut items,
                                ref mut selected,
                                ..
                            } = self.widgets[si]
                        {
                            items.push(text);
                            *selected = items.len() - 1;
                        }
                        // Clear the textbox
                        if let WidgetKind::TextBox {
                            ref mut content,
                            ref mut cursor,
                        } = self.widgets[fi]
                        {
                            content.clear();
                            *cursor = 0;
                        }
                    }
                }
            }
            Action::Redraw => {
                if let Event::Resize(w, h) = event {
                    self.screen_width = w;
                    self.screen_height = h;
                }
                self.layout();
            }
            Action::Noop => {}
        }

        action
    }

    /// Render all widgets into a flat list of cells.
    pub fn render(&self) -> Vec<Cell> {
        if self.widgets.is_empty() {
            return Vec::new();
        }
        let root_idx = self.widgets.len() - 1;
        let root_area = if root_idx < self.areas.len() {
            self.areas[root_idx]
        } else {
            Rect::new(0, 0, self.screen_width, self.screen_height)
        };
        render_widget(&self.widgets[root_idx], root_area, &self.widgets)
    }

    /// Advance focus to the next focusable widget (TextBox or ScrollableList).
    pub fn focus_next(&mut self) {
        let len = self.widgets.len();
        if len == 0 {
            return;
        }
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

    /// Move focus to the previous focusable widget.
    pub fn focus_prev(&mut self) {
        let len = self.widgets.len();
        if len == 0 {
            return;
        }
        let mut i: usize = 1;
        while i <= len {
            let idx = (self.focus_index + len - i) % len;
            if is_focusable(&self.widgets[idx]) {
                self.focus_index = idx;
                return;
            }
            i += 1;
        }
    }
}

/// Check if a widget kind is focusable.
fn is_focusable(kind: &WidgetKind) -> bool {
    matches!(
        kind,
        WidgetKind::TextBox { .. } | WidgetKind::ScrollableList { .. }
    )
}
