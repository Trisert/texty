use crate::fuzzy_search::FuzzySearchState;
use crate::ui::theme::Theme;
use crate::ui::widgets::preview::render_preview_content;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct FuzzySearchWidget<'a> {
    pub state: &'a mut FuzzySearchState,
    pub theme: &'a Theme,
}

impl<'a> FuzzySearchWidget<'a> {
    pub fn new(
        state: &'a mut FuzzySearchState,
        theme: &'a Theme,
        _formatter: Option<&'a crate::formatter::external::Formatter>,
    ) -> Self {
        // Note: We ignore the global formatter and create formatters dynamically per file
        Self { state, theme }
    }

    pub fn calculate_width(terminal_width: u16, show_preview: bool) -> u16 {
        if show_preview && terminal_width > 72 {
            // When preview is enabled and terminal is wide enough,
            // use 40% of width for the file list (matching our split ratio)
            ((terminal_width as f32 * 0.4) as u16)
                .clamp(40, 80)
                .min(terminal_width.saturating_sub(4))
        } else {
            // Original behavior: use 35% of terminal width for sidebar-like layout
            ((terminal_width as f32 * 0.35) as u16)
                .clamp(40, 80)
                .min(terminal_width.saturating_sub(4))
        }
    }
}

impl<'a> Widget for FuzzySearchWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().style(Style::default().bg(self.theme.ui.status_bar_bg));
        let inner_area = block.inner(area);

        let show_preview = area.width > 80 && self.state.current_preview.is_some();

        if show_preview {
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(60),
                ])
                .split(inner_area);

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(horizontal_chunks[0]);

            self.render_search_input(vertical_chunks[0], buf);
            self.render_file_list(vertical_chunks[1], buf);
            self.render_preview(horizontal_chunks[1], buf);
        } else {
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(inner_area);

            self.render_search_input(vertical_chunks[0], buf);
            self.render_file_list(vertical_chunks[1], buf);
        }
    }
}

impl<'a> FuzzySearchWidget<'a> {
    fn render_search_input(&self, area: Rect, buf: &mut Buffer) {
        let mode_indicator = if self.state.recursive_search {
            " [R]"
        } else {
            ""
        };

        // Show result count
        let result_display = if self.state.result_count > 0 {
            format!(
                " ({}/{})",
                self.state.displayed_count, self.state.result_count
            )
        } else if self.state.is_scanning {
            " (scanning...)".to_string()
        } else {
            "".to_string()
        };

        let pagination_hint = if self.state.has_more_results {
            " - Tab for more"
        } else {
            ""
        };
        let title = format!(
            "Search{}{}{}:",
            mode_indicator, result_display, pagination_hint
        );
        let search_block = Block::default().borders(Borders::NONE).title(title);

        let search_text = format!("> {}", self.state.query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        search_paragraph.render(area, buf);
    }

    fn render_file_list(&self, area: Rect, buf: &mut Buffer) {
        let file_list_area = area;

        let file_list_block = Block::default().borders(Borders::NONE).title("Files");

        let mut file_lines = Vec::new();

        // Calculate how many items can fit in the available height
        let available_height = file_list_area.height as usize;
        let max_visible_items = available_height;

        // Calculate scroll offset locally (don't modify state)
        let scroll_offset = if self.state.selected_index < self.state.scroll_offset {
            self.state.selected_index
        } else if self.state.selected_index >= self.state.scroll_offset + max_visible_items {
            self.state
                .selected_index
                .saturating_sub(max_visible_items - 1)
        } else {
            self.state.scroll_offset
        };

        // Show filtered results (up to available height)
        let start_idx = scroll_offset;
        let end_idx = (start_idx + max_visible_items).min(self.state.filtered_items.len());

        // Auto-load more results when scrolling near bottom
        if self.state.has_more_results && end_idx >= self.state.filtered_items.len() - 5 {
            // Load more results when within 5 items of the bottom
            // Note: In a real implementation, this would need to trigger a callback
            // to the editor to call load_more_results() on the fuzzy state
        }

        for (i, item) in self.state.filtered_items[start_idx..end_idx]
            .iter()
            .enumerate()
        {
            let global_idx = start_idx + i;
            let is_selected = global_idx == self.state.selected_index;

            let full_path = item.path.display().to_string();
            let mut spans = Vec::new();

            if self.state.recursive_search {
                // In recursive mode, show relative path from current_path
                let relative_path =
                    if let Ok(relative) = item.path.strip_prefix(&self.state.current_path) {
                        relative.display().to_string()
                    } else {
                        full_path.clone()
                    };

                if !item.is_dir {
                    // For files: show path in gray, filename in white
                    if let Some(last_sep) = relative_path.rfind('/') {
                        let path_part = &relative_path[..last_sep + 1];
                        let file_part = &relative_path[last_sep + 1..];

                        spans.push(Span::styled(
                            path_part.to_string(),
                            Style::default().fg(Color::Gray),
                        ));
                        spans.push(Span::styled(
                            file_part.to_string(),
                            Style::default().fg(Color::White),
                        ));
                    } else {
                        spans.push(Span::styled(
                            relative_path,
                            Style::default().fg(Color::White),
                        ));
                    }
                } else {
                    // For directories in recursive mode, show relative path in cyan
                    let display_path = if relative_path == ".." {
                        "..".to_string()
                    } else {
                        relative_path + "/"
                    };
                    spans.push(Span::styled(display_path, Style::default().fg(Color::Cyan)));
                }
            } else {
                // Non-recursive mode (original behavior)
                if !item.is_dir {
                    // For files, show path in gray and filename in white
                    if let Some(last_sep) = full_path.rfind('/') {
                        let path_part = &full_path[..last_sep + 1]; // Include the /
                        let file_part = &full_path[last_sep + 1..];

                        spans.push(Span::styled(
                            path_part.to_string(),
                            Style::default().fg(Color::Gray),
                        ));
                        spans.push(Span::styled(
                            file_part.to_string(),
                            Style::default().fg(Color::White),
                        ));
                    } else {
                        // No path separator, just show the filename
                        spans.push(Span::styled(full_path, Style::default().fg(Color::White)));
                    }
                } else {
                    // For directories, show the full path in white
                    spans.push(Span::styled(full_path, Style::default().fg(Color::White)));
                }
            }

            if item.is_hidden {
                spans.push(Span::styled(
                    " (hidden)",
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            file_lines.push(Line::from(spans).style(style));
        }

        // Add directory placeholder if selected item is a directory
        if let Some(selected_item) = self.state.filtered_items.get(self.state.selected_index)
            && selected_item.is_dir
        {
            let dir_name = selected_item
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&selected_item.name);

            let placeholder_text = format!("'{}' is a directory", dir_name);
            let placeholder_line = Line::from(vec![Span::styled(
                placeholder_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            )]);

            file_lines.push(placeholder_line);
        }

        let file_list_paragraph = Paragraph::new(file_lines)
            .block(file_list_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        file_list_paragraph.render(file_list_area, buf);
    }

    fn render_preview(&self, area: Rect, buf: &mut Buffer) {
        if let Some(preview_buffer) = &self.state.current_preview {
            let preview_block = Block::default()
                .borders(Borders::ALL)
                .title("Preview")
                .style(Style::default().bg(self.theme.ui.status_bar_bg));

            let inner_area = preview_block.inner(area);
            preview_block.render(area, buf);

            let preview_paragraph = render_preview_content(preview_buffer, self.theme, inner_area);
            preview_paragraph.render(inner_area, buf);
        } else {
            let no_preview_block = Block::default()
                .borders(Borders::ALL)
                .title("Preview")
                .style(Style::default().bg(self.theme.ui.status_bar_bg));

            let no_preview_text = Paragraph::new("No preview available")
                .style(Style::default().fg(Color::Gray))
                .block(no_preview_block);

            no_preview_text.render(area, buf);
        }
    }
}
