use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use crate::fuzzy_search::FuzzySearchState;

pub struct FuzzySearchWidget<'a> {
    pub state: &'a FuzzySearchState,
}

impl<'a> FuzzySearchWidget<'a> {
    pub fn new(state: &'a FuzzySearchState) -> Self {
        Self { state }
    }

    pub fn calculate_position(terminal_width: u16, terminal_height: u16) -> Rect {
        let width = 60.min(terminal_width.saturating_sub(4));
        let height = 12.min(terminal_height.saturating_sub(4));

        let x = (terminal_width - width) / 2;
        let y = (terminal_height - height) / 2;

        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl<'a> Widget for FuzzySearchWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area
        Clear.render(area, buf);

        // Create the main block
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Fuzzy Search")
            .title_alignment(Alignment::Center);

        // Split the area: search input + results list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input area
                Constraint::Min(1),    // Results list
            ])
            .split(block.inner(area));

        // Render the block
        block.render(area, buf);

        // Search input area
        let search_block = Block::default()
            .borders(Borders::NONE)
            .title("Search:");

        let search_text = format!("> {}", self.state.query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        search_paragraph.render(chunks[0], buf);

        // Results list
        let results_block = Block::default()
            .borders(Borders::NONE);

        let mut result_lines = Vec::new();

        // Show current path
        let path_display = format!("{}", self.state.current_path.display());
        result_lines.push(Line::from(vec![Span::styled(
            path_display,
            Style::default().fg(Color::Cyan),
        )]));

        // Empty line
        result_lines.push(Line::from(""));

        // Show filtered results (up to 8 items)
        let start_idx = self.state.scroll_offset;
        let end_idx = (start_idx + 8).min(self.state.filtered_items.len());

        for (i, item) in self.state.filtered_items[start_idx..end_idx].iter().enumerate() {
            let global_idx = start_idx + i;
            let is_selected = global_idx == self.state.selected_index;

            let prefix = if item.is_dir { "[DIR] " } else { "[FILE] " };
            let name = if item.name == ".." {
                ".. (parent)".to_string()
            } else {
                item.name.clone()
            };

            let mut spans = vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::raw(name),
            ];

            if item.is_hidden {
                spans.push(Span::styled(" (hidden)", Style::default().fg(Color::Gray)));
            }

            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            result_lines.push(Line::from(spans).style(style));
        }

        // Show item count
        if !self.state.filtered_items.is_empty() {
            result_lines.push(Line::from(""));
            let count_text = format!("{} items", self.state.filtered_items.len());
            result_lines.push(Line::from(vec![Span::styled(
                count_text,
                Style::default().fg(Color::Gray),
            )]));
        }

        let results_paragraph = Paragraph::new(result_lines)
            .block(results_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        results_paragraph.render(chunks[1], buf);
    }
}