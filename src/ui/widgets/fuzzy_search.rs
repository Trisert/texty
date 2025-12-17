use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
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
        // Use 80% of terminal width, with min 60 and max 100
        let width = ((terminal_width as f32 * 0.8) as u16)
            .max(60)
            .min(100)
            .min(terminal_width.saturating_sub(4));

        // Use 70% of terminal height, with min 16 and max 30
        let height = ((terminal_height as f32 * 0.7) as u16)
            .max(16)
            .min(30)
            .min(terminal_height.saturating_sub(4));

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
        // Create the main block with dark purple background
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(255, 102, 0))) // Orange border
            .style(Style::default().bg(Color::Rgb(42, 26, 62))) // Dark purple background
            .title("Fuzzy Search")
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(Color::White));

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
            .style(Style::default().fg(Color::Rgb(224, 224, 224))); // Light gray #e0e0e0

        search_paragraph.render(chunks[0], buf);

        // Results list
        let results_block = Block::default()
            .borders(Borders::NONE);

        let mut result_lines = Vec::new();

        // Show filtered results (up to 10 items)
        let start_idx = self.state.scroll_offset;
        let end_idx = (start_idx + 10).min(self.state.filtered_items.len());

        for (i, item) in self.state.filtered_items[start_idx..end_idx].iter().enumerate() {
            let global_idx = start_idx + i;
            let is_selected = global_idx == self.state.selected_index;

            let prefix = if item.is_dir { "📁 " } else { "📄 " };
            let display_path = item.path.display().to_string();

            let mut spans = vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::raw(display_path),
            ];

            if item.is_hidden {
                spans.push(Span::styled(" (hidden)", Style::default().fg(Color::Rgb(128, 128, 128)))); // Darker gray
            }

            let style = if is_selected {
                Style::default().bg(Color::Rgb(255, 102, 0)).fg(Color::White).add_modifier(Modifier::BOLD) // Orange highlight
            } else {
                Style::default().fg(Color::Rgb(224, 224, 224)) // Light gray text
            };

            result_lines.push(Line::from(spans).style(style));
        }

        // Show item count
        if !self.state.filtered_items.is_empty() {
            result_lines.push(Line::from(""));
            let count_text = format!("{} items", self.state.filtered_items.len());
            result_lines.push(Line::from(vec![Span::styled(
                count_text,
                Style::default().fg(Color::Rgb(160, 160, 160)), // Medium gray
            )]));
        }

        let results_paragraph = Paragraph::new(result_lines)
            .block(results_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        results_paragraph.render(chunks[1], buf);
    }
}