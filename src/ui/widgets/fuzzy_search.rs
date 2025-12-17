use crate::fuzzy_search::{FuzzySearchState, PreviewCache};
use crate::ui::theme::Theme;
use crate::ui::widgets::preview::{PreviewBuffer, render_preview_content};
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
        // Create the main block with minimal styling
        let block = Block::default().style(Style::default().bg(Color::Black));

        // Check if we should show preview (following Helix's pattern)
        const MIN_AREA_WIDTH_FOR_PREVIEW: u16 = 72;
        let show_preview = self.state.show_preview && area.width > MIN_AREA_WIDTH_FOR_PREVIEW;

        let inner_area = block.inner(area);

        if show_preview {
            // Horizontal layout: file list + preview
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40), // File list
                    Constraint::Percentage(60), // Preview
                ])
                .split(inner_area);

            self.render_file_list(chunks[0], buf);
            self.render_preview(chunks[1], buf);
        } else {
            // Vertical layout: search input + file list (original behavior)
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Search input area
                    Constraint::Min(1),    // File list area
                ])
                .split(inner_area);

            self.render_search_input(vertical_chunks[0], buf);
            self.render_file_list(vertical_chunks[1], buf);
        }
    }
}

impl<'a> FuzzySearchWidget<'a> {
    fn render_search_input(&self, area: Rect, buf: &mut Buffer) {
        let search_block = Block::default().borders(Borders::NONE).title("Search:");

        let search_text = format!("> {}", self.state.query);
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(Style::default().fg(Color::White));

        search_paragraph.render(area, buf);
    }

    fn render_file_list(&self, area: Rect, buf: &mut Buffer) {
        // Search input area (when in preview mode)
        let search_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 3,
        };
        self.render_search_input(search_area, buf);

        // File list area
        let file_list_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: area.height.saturating_sub(3),
        };

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

        for (i, item) in self.state.filtered_items[start_idx..end_idx]
            .iter()
            .enumerate()
        {
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

        let file_list_paragraph = Paragraph::new(file_lines)
            .block(file_list_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        file_list_paragraph.render(file_list_area, buf);
    }

    fn render_preview(&self, area: Rect, buf: &mut Buffer) {
        let title = if self.state.show_formatted_preview {
            "Formatted Preview"
        } else {
            "Preview"
        };

        let preview_block = Block::default().borders(Borders::NONE).title(title);

        let preview_widget = if let Some(item) = self.state.get_selected_item() {
            match self.state.get_preview(&item.path) {
                Some(PreviewCache::PlainContent(content)) => {
                    if self.state.show_formatted_preview {
                        // Use PreviewBuffer for editor-style formatted preview
                        self.render_formatted_preview(item, area)
                    } else {
                        // Apply syntax highlighting to raw content
                        match self.highlight_preview_content(content, &item.path) {
                            Ok(highlighted_lines) => {
                                let visible_lines = highlighted_lines
                                    .into_iter()
                                    .take(area.height as usize)
                                    .collect::<Vec<_>>();
                                Paragraph::new(visible_lines)
                            }
                            Err(_) => {
                                let plain_lines = content
                                    .lines()
                                    .take(area.height as usize)
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                Paragraph::new(plain_lines)
                            }
                        }
                    }
                }
                Some(PreviewCache::HighlightedContent(lines)) => {
                    // Already highlighted, just limit to visible area
                    let visible_lines = lines
                        .clone()
                        .into_iter()
                        .take(area.height as usize)
                        .collect::<Vec<_>>();
                    Paragraph::new(visible_lines)
                }
                Some(PreviewCache::FormattedContent(content)) => {
                    // Formatted content - apply syntax highlighting
                    match self.highlight_preview_content(content, &item.path) {
                        Ok(highlighted_lines) => {
                            let visible_lines = highlighted_lines
                                .into_iter()
                                .take(area.height as usize)
                                .collect::<Vec<_>>();
                            Paragraph::new(visible_lines)
                        }
                        Err(_) => {
                            let plain_lines = content
                                .lines()
                                .take(area.height as usize)
                                .collect::<Vec<_>>()
                                .join("\n");
                            Paragraph::new(plain_lines)
                        }
                    }
                }
                Some(PreviewCache::FormattedHighlighted(lines)) => {
                    // Already formatted and highlighted, just limit to visible area
                    let visible_lines = lines
                        .clone()
                        .into_iter()
                        .take(area.height as usize)
                        .collect::<Vec<_>>();
                    Paragraph::new(visible_lines)
                }
                Some(PreviewCache::Directory(entries)) => {
                    let mut dir_content = String::new();
                    for entry in entries.iter().take(50.min(area.height as usize)) {
                        // Limit to visible area
                        dir_content.push_str(entry);
                        dir_content.push('\n');
                    }
                    if entries.len() > 50 {
                        dir_content.push_str("... (truncated)");
                    }
                    Paragraph::new(dir_content)
                }
                Some(PreviewCache::Binary) => Paragraph::new("(Binary file)"),
                Some(PreviewCache::LargeFile) => Paragraph::new("(File too large to preview)"),
                Some(PreviewCache::Error(msg)) => Paragraph::new(format!("(Error: {})", msg)),
                None => Paragraph::new("(Loading...)"),
            }
        } else {
            Paragraph::new("")
        };

        let preview_widget = preview_widget
            .block(preview_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        preview_widget.render(area, buf);
    }

    fn render_formatted_preview(
        &self,
        item: &crate::fuzzy_search::FileItem,
        area: Rect,
    ) -> Paragraph<'_> {
        // Load preview buffer using editor-style logic
        match PreviewBuffer::load_from_file(&item.path) {
            Ok(preview_buffer) => {
                // Render using editor-style logic (caching can be added later)
                render_preview_content(&preview_buffer, self.theme, area)
            }
            Err(error_msg) => {
                // Show error message
                Paragraph::new(format!("❌ Preview error: {}", error_msg))
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
            }
        }
    }

    fn highlight_preview_content(
        &self,
        content: &str,
        file_path: &std::path::Path,
    ) -> Result<Vec<Line<'static>>, Box<dyn std::error::Error>> {
        use crate::syntax::{SyntaxHighlighter, get_language_config_by_extension};

        // Detect language from file extension
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let language_config =
            match crate::syntax::language::get_language_config_by_extension(extension) {
                Some(config) => config,
                None => {
                    // No syntax highlighting support for this file type
                    return Err("No syntax highlighting support for this file type".into());
                }
            };

        // Create syntax highlighter
        let mut highlighter = SyntaxHighlighter::new(language_config)?;

        // Parse the content
        highlighter.parse(content)?;

        // Get lines to highlight (limit to reasonable number for performance)
        let lines: Vec<&str> = content.lines().take(100).collect(); // Preview max 100 lines
        let mut highlighted_lines = Vec::with_capacity(lines.len());

        for (line_idx, line_content) in lines.iter().enumerate() {
            if let Some(tokens) = highlighter.get_line_highlights(line_idx) {
                let mut spans = Vec::new();
                let mut pos = 0;

                // Build spans with syntax highlighting
                for token in tokens {
                    // Skip tokens outside this line
                    let line_start_byte = content.lines().take(line_idx).map(|l| l.len() + 1).sum();
                    let line_end_byte = line_start_byte + line_content.len();

                    if token.start < line_start_byte || token.end > line_end_byte {
                        continue;
                    }

                    // Calculate relative positions within the line
                    let rel_start = token.start - line_start_byte;
                    let rel_end = token.end - line_start_byte;

                    // Add unhighlighted text before token
                    if rel_start > pos {
                        let text_before = &line_content[pos..rel_start];
                        spans.push(Span::styled(
                            text_before.to_string(),
                            Style::default().fg(Color::White),
                        ));
                    }

                    // Add highlighted token
                    let token_text = &line_content[rel_start..rel_end];
                    let color = self.theme.syntax_color(&token.capture_name);
                    spans.push(Span::styled(
                        token_text.to_string(),
                        Style::default().fg(color),
                    ));

                    pos = rel_end;
                }

                // Add remaining unhighlighted text
                if pos < line_content.len() {
                    let remaining_text = &line_content[pos..];
                    spans.push(Span::styled(
                        remaining_text.to_string(),
                        Style::default().fg(Color::White),
                    ));
                }

                highlighted_lines.push(Line::from(spans));
            } else {
                // No highlights for this line, use plain text
                highlighted_lines.push(Line::from(Span::styled(
                    line_content.to_string(),
                    Style::default().fg(Color::White),
                )));
            }
        }
        Ok(highlighted_lines)
    }
}
