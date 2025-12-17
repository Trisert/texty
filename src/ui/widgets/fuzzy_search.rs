use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use crate::fuzzy_search::FuzzySearchState;

pub struct FuzzySearchWidget<'a> {
    pub state: &'a mut FuzzySearchState,
}

impl<'a> FuzzySearchWidget<'a> {
    pub fn new(state: &'a mut FuzzySearchState) -> Self {
        Self { state }
    }

    pub fn calculate_width(terminal_width: u16) -> u16 {
        // Use 35% of terminal width for sidebar-like layout
        ((terminal_width as f32 * 0.35) as u16)
            .max(40)
            .min(80)
            .min(terminal_width.saturating_sub(4))
    }
}

impl<'a> Widget for FuzzySearchWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the main block with minimal styling
        let block = Block::default()
            .style(Style::default().bg(Color::Black));

        // Split the area: search input + file list
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input area
                Constraint::Min(1),    // File list area
            ])
            .split(block.inner(area));

        // Search input area
        let search_block = Block::default()
            .borders(Borders::NONE)
            .title("Search:");

        let search_text = format!("> {}", self.state.query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        search_paragraph.render(vertical_chunks[0], buf);

        // File list
        let file_list_block = Block::default()
            .borders(Borders::NONE)
            .title("Files");

        let mut file_lines = Vec::new();

        // Calculate how many items can fit in the available height
        let available_height = vertical_chunks[1].height as usize;
        let max_visible_items = available_height;

        // Update scroll offset based on selection
        if self.state.selected_index < self.state.scroll_offset {
            self.state.scroll_offset = self.state.selected_index;
        } else if self.state.selected_index >= self.state.scroll_offset + max_visible_items {
            self.state.scroll_offset = self.state.selected_index.saturating_sub(max_visible_items - 1);
        }

        // Show filtered results (up to available height)
        let start_idx = self.state.scroll_offset;
        let end_idx = (start_idx + max_visible_items).min(self.state.filtered_items.len());

        for (i, item) in self.state.filtered_items[start_idx..end_idx].iter().enumerate() {
            let global_idx = start_idx + i;
            let is_selected = global_idx == self.state.selected_index;

            let full_path = item.path.display().to_string();
            let mut spans = Vec::new();

            // For files, show path in gray and filename in white
            if !item.is_dir {
                // Find the last path separator
                if let Some(last_sep) = full_path.rfind('/') {
                    let path_part = &full_path[..last_sep + 1]; // Include the /
                    let file_part = &full_path[last_sep + 1..];

                    spans.push(Span::styled(path_part.to_string(), Style::default().fg(Color::Gray)));
                    spans.push(Span::styled(file_part.to_string(), Style::default().fg(Color::White)));
                } else {
                    // No path separator, just show the filename
                    spans.push(Span::styled(full_path, Style::default().fg(Color::White)));
                }
            } else {
                // For directories, show the full path in white
                spans.push(Span::styled(full_path, Style::default().fg(Color::White)));
            }

            if item.is_hidden {
                spans.push(Span::styled(" (hidden)", Style::default().fg(Color::DarkGray)));
            }

            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            file_lines.push(Line::from(spans).style(style));
        }

        let file_list_paragraph = Paragraph::new(file_lines)
            .block(file_list_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        file_list_paragraph.render(vertical_chunks[1], buf);
    }
}