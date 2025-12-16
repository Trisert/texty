// src/ui/widgets/menu.rs - Code action menu widget

use lsp_types::CodeAction;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget},
};

use crate::ui::theme::Theme;

/// Code action menu that displays available code actions
pub struct CodeActionMenu<'a> {
    pub actions: Vec<CodeAction>,
    pub selected_index: usize,
    pub theme: &'a Theme,
}

impl<'a> CodeActionMenu<'a> {
    pub fn new(actions: Vec<CodeAction>, theme: &'a Theme) -> Self {
        Self {
            actions,
            selected_index: 0,
            theme,
        }
    }

    pub fn select_next(&mut self) {
        if !self.actions.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.actions.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.actions.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.actions.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn get_selected_action(&self) -> Option<&CodeAction> {
        self.actions.get(self.selected_index)
    }

    /// Calculate the position for the menu relative to cursor
    pub fn calculate_position(&self, cursor_x: u16, cursor_y: u16, area: Rect) -> Rect {
        let max_title_len = self
            .actions
            .iter()
            .map(|action| action.title.len())
            .max()
            .unwrap_or(20) as u16;

        let width = (max_title_len + 6).min(area.width.saturating_sub(4)); // Title + borders + padding
        let height = (self.actions.len() as u16 + 2).min(area.height.saturating_sub(4)); // Actions + borders

        let mut x = cursor_x.saturating_sub(width / 2); // Center horizontally on cursor
        let mut y = cursor_y + 2; // Position below cursor

        // Adjust if it would go off-screen
        if x + width > area.width {
            x = area.width.saturating_sub(width);
        }
        if y + height > area.height {
            y = cursor_y.saturating_sub(height + 1); // Position above cursor if below doesn't fit
        }

        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl Widget for CodeActionMenu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first
        Clear.render(area, buf);

        // Create the menu content
        let mut lines = Vec::new();

        for (i, action) in self.actions.iter().enumerate() {
            let style = if i == self.selected_index {
                Style::default()
                    .fg(self.theme.general.background)
                    .bg(self.theme.general.foreground)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.general.foreground)
            };

            let prefix = if i == self.selected_index {
                "▶ "
            } else {
                "  "
            };
            let title = format!("{}{}", prefix, action.title);

            lines.push(Line::from(vec![Span::styled(title, style)]));
        }

        // Create the block with borders
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.ui.gutter_fg))
            .title(" Code Actions ")
            .title_style(
                Style::default()
                    .fg(self.theme.syntax.function)
                    .add_modifier(Modifier::BOLD),
            )
            .padding(Padding::horizontal(1));

        // Create the paragraph widget
        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);

        // Render the paragraph
        paragraph.render(area, buf);
    }
}
