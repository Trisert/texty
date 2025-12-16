// src/ui/widgets/completion.rs - Completion popup widget

use lsp_types::CompletionItem;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget},
};

pub struct CompletionPopup {
    pub items: Vec<CompletionItem>,
    pub selected_index: usize,
    pub max_visible: usize,
    pub scroll_offset: usize,
}

impl Default for CompletionPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionPopup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            max_visible: 10,
            scroll_offset: 0,
        }
    }

    pub fn set_items(&mut self, items: Vec<CompletionItem>) {
        self.items = items;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.items.len();
        self.update_scroll();
    }

    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            self.items.len() - 1
        } else {
            self.selected_index - 1
        };
        self.update_scroll();
    }

    fn update_scroll(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.max_visible {
            self.scroll_offset = self.selected_index.saturating_sub(self.max_visible - 1);
        }
    }

    pub fn selected_item(&self) -> Option<&CompletionItem> {
        self.items.get(self.selected_index)
    }

    pub fn is_visible(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn hide(&mut self) {
        self.items.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn calculate_position(
        &self,
        cursor_x: u16,
        cursor_y: u16,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Rect {
        let max_width = 40;
        let height = self.items.len().min(self.max_visible) as u16 + 2; // +2 for borders

        let mut x = cursor_x;
        let mut y = cursor_y + 1; // Show below cursor

        // Adjust if popup would go off screen
        if x + max_width > terminal_width {
            x = terminal_width.saturating_sub(max_width);
        }

        if y + height > terminal_height {
            y = cursor_y.saturating_sub(height);
            if y == 0 {
                y = 1;
            }
        }

        Rect {
            x,
            y,
            width: max_width,
            height,
        }
    }
}

impl Widget for &CompletionPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.items.is_empty() {
            return;
        }

        // Clear the area first
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Completions")
            .title_alignment(Alignment::Center);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let visible_items = self
            .items
            .iter()
            .skip(self.scroll_offset)
            .take(self.max_visible)
            .enumerate();

        let mut lines = Vec::new();
        for (i, item) in visible_items {
            let actual_index = i + self.scroll_offset;
            let is_selected = actual_index == self.selected_index;

            let label = item.label.as_str();
            let detail = item.detail.as_deref().unwrap_or("");

            let mut spans = vec![Span::styled(
                label,
                if is_selected {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                },
            )];

            if !detail.is_empty() {
                spans.push(Span::styled(
                    format!(" - {}", detail),
                    if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::White)
                            .add_modifier(Modifier::DIM)
                    } else {
                        Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)
                    },
                ));
            }

            lines.push(Line::from(spans));
        }

        let paragraph =
            Paragraph::new(lines).block(Block::default().padding(Padding::horizontal(1)));

        paragraph.render(inner_area, buf);
    }
}
