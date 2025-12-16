// src/ui/widgets/hover.rs - Hover information floating window

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Widget},
};

use crate::ui::theme::Theme;

/// Hover information window that displays LSP hover content
pub struct HoverWindow<'a> {
    pub content: Vec<String>,
    pub theme: &'a Theme,
}

impl<'a> HoverWindow<'a> {
    pub fn new(content: Vec<String>, theme: &'a Theme) -> Self {
        Self { content, theme }
    }

    /// Calculate the position for the hover window relative to cursor
    pub fn calculate_position(&self, cursor_x: u16, cursor_y: u16, area: Rect) -> Rect {
        let width = 60.min(area.width.saturating_sub(4)); // Max width with padding
        let height = (self.content.len() as u16 + 2).min(area.height.saturating_sub(4)); // Content + borders + padding

        let mut x = cursor_x.saturating_sub(width / 2); // Center horizontally on cursor
        let mut y = cursor_y.saturating_sub(height + 1); // Position above cursor

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

impl Widget for HoverWindow<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first
        Clear.render(area, buf);

        // Create the hover content
        let lines: Vec<Line> = self
            .content
            .iter()
            .map(|line| {
                Line::from(vec![Span::styled(
                    line.clone(),
                    Style::default().fg(self.theme.general.foreground),
                )])
            })
            .collect();

        // Create the block with borders
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.ui.gutter_fg))
            .title(" Hover ")
            .title_style(
                Style::default()
                    .fg(self.theme.syntax.comment)
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
