// ui/renderer.rs - Ratatui-based renderer for the text editor

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
};
use std::io::Stdout;

use crate::editor::Editor;
use crate::ui::theme::Theme;
use crate::ui::widgets::editor_pane::EditorPane;
use crate::ui::widgets::fuzzy_search::FuzzySearchWidget;
use crate::ui::widgets::gutter::Gutter;
use crate::ui::widgets::hover::HoverWindow;
use crate::ui::widgets::menu::CodeActionMenu;
use crate::ui::widgets::status_bar::StatusBar;

/// Ratatui-based renderer for the text editor
pub struct TuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    theme: Theme,
}

impl TuiRenderer {
    /// Create a new TuiRenderer
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        let mut theme = Theme::default();

        // Try to load syntax theme
        if let Ok(loaded_theme) = crate::syntax::Theme::from_file("runtime/themes/default.toml") {
            theme.loaded_syntax_theme = Some(loaded_theme);
        }

        Ok(Self { terminal, theme })
    }

    /// Draw the editor UI
    pub fn draw(&mut self, editor: &mut Editor) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            let size = f.size();

            // Check if fuzzy search is active
            let fuzzy_search_active = editor.fuzzy_search.is_some();

            let (_fuzzy_area, content_area) = if fuzzy_search_active {
                let show_preview = editor
                    .fuzzy_search
                    .as_ref()
                    .map(|_| false)
                    .unwrap_or(false);

                if show_preview {
                    // When preview is enabled, fuzzy search takes full screen
                    if let Some(fuzzy_state) = &mut editor.fuzzy_search {
                        let fuzzy_widget = FuzzySearchWidget::new(fuzzy_state, &self.theme, None);
                        f.render_widget(fuzzy_widget, size);
                    }
                    (None, Rect::default()) // No content area when preview is full screen
                } else {
                    // Original behavior: split screen when no preview
                    let fuzzy_width = FuzzySearchWidget::calculate_width(size.width, show_preview);
                    let main_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(fuzzy_width), // Fuzzy search width
                            Constraint::Min(1),              // Content area (editor)
                        ])
                        .split(size);

                    // Render fuzzy search in left panel
                    if let Some(fuzzy_state) = &mut editor.fuzzy_search {
                        let fuzzy_widget = FuzzySearchWidget::new(fuzzy_state, &self.theme, None);
                        f.render_widget(fuzzy_widget, main_chunks[0]);
                    }

                    (Some(main_chunks[0]), main_chunks[1]) // Return both areas
                }
            } else {
                (None, size) // No fuzzy area, content gets full screen
            };

            // Only render editor if there's a valid content area (not empty when preview is full screen)
            if content_area.width > 0 && content_area.height > 0 {
                // Render editor in content area
                // Create main layout: editor area + status bar
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),    // Editor area
                        Constraint::Length(1), // Status bar (1 line)
                    ])
                    .split(content_area);

                // Split editor area: gutter + text
                let editor_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(4), // Gutter
                        Constraint::Min(1),    // Text area
                    ])
                    .split(vertical_chunks[0]);

                // Render gutter
                f.render_widget(Gutter::new(editor, &self.theme), editor_chunks[0]);

                // Render editor pane
                f.render_widget(EditorPane::new(editor, &self.theme), editor_chunks[1]);

                // Set cursor (only when editor is visible and not in fuzzy search mode)
                if !fuzzy_search_active {
                    let cursor_row = editor
                        .cursor
                        .line
                        .saturating_sub(editor.viewport.offset_line)
                        as u16;
                    let cursor_col =
                        editor.cursor.col.saturating_sub(editor.viewport.offset_col) as u16;
                    if cursor_row < editor_chunks[1].height && cursor_col < editor_chunks[1].width {
                        f.set_cursor(
                            editor_chunks[1].x + cursor_col,
                            editor_chunks[1].y + cursor_row,
                        );
                    }
                }
            }

            // Render status bar at the bottom of the terminal
            let status_bar_area = Rect {
                x: 0,
                y: size.height - 1,
                width: size.width,
                height: 1,
            };

            if editor.mode == crate::mode::Mode::Command {
                // Show command line on status bar line, filling full width
                let command_text = editor.get_command_line_display();
                let padded_command = if command_text.len() < status_bar_area.width as usize {
                    format!(
                        "{}{}",
                        command_text,
                        " ".repeat(status_bar_area.width as usize - command_text.len())
                    )
                } else {
                    command_text
                };
                let command_line = ratatui::text::Line::from(padded_command).style(
                    Style::default()
                        .bg(self.theme.ui.status_bar_bg)
                        .fg(self.theme.ui.status_bar_fg),
                );
                f.buffer_mut()
                    .set_line(0, status_bar_area.y, &command_line, status_bar_area.width);
            } else {
                // Show normal status bar
                f.render_widget(StatusBar::new(editor, &self.theme), status_bar_area);
            }

            // Render overlays (floating windows)
            // Calculate cursor position relative to content area
            let (cursor_x, cursor_y) = if fuzzy_search_active {
                // When fuzzy search is active, cursor is not visible, use center of content area
                (
                    content_area.x + content_area.width / 2,
                    content_area.y + content_area.height / 2,
                )
            } else {
                (
                    content_area.x
                        + 4
                        + editor.cursor.col.saturating_sub(editor.viewport.offset_col) as u16, // +4 for gutter
                    content_area.y
                        + editor
                            .cursor
                            .line
                            .saturating_sub(editor.viewport.offset_line)
                            as u16,
                )
            };

            // Render hover window if active
            if let Some(content) = &editor.hover_content {
                let hover_window = HoverWindow::new(content.clone(), &self.theme);
                let hover_area = hover_window.calculate_position(cursor_x, cursor_y, size);
                f.render_widget(hover_window, hover_area);
            }

            // Render code action menu if active
            if let Some(actions) = &editor.code_actions {
                let mut menu = CodeActionMenu::new(actions.clone(), &self.theme);
                menu.selected_index = editor.code_action_selected;
                let menu_area = menu.calculate_position(cursor_x, cursor_y, size);
                f.render_widget(menu, menu_area);
            }
        })?;
        Ok(())
    }
}

fn apply_tree_sitter_syntax_highlighting(
    text: &str,
    file_extension: Option<&str>,
) -> Vec<ratatui::text::Line<'static>> {
    // First, try to format the text if a formatter is available
    let formatted_text = if let Some(language_id) = file_extension_to_language_id(file_extension) {
        if let Some(formatter_config) =
            crate::formatter::external::get_formatter_config(language_id)
        {
            if let Ok(formatter) = crate::formatter::external::Formatter::new(formatter_config) {
                if let Ok(formatted) = formatter.format_text(text) {
                    formatted
                } else {
                    text.to_string()
                }
            } else {
                text.to_string()
            }
        } else {
            text.to_string()
        }
    } else {
        text.to_string()
    };

    let mut lines = Vec::new();

    // Try tree-sitter highlighting on the formatted text
    if let Some(language_id) = file_extension_to_language_id(file_extension) {
        let config = crate::syntax::language::get_language_config(language_id);
        if let Ok(mut highlighter) = crate::syntax::SyntaxHighlighter::new(config)
            && highlighter.parse(&formatted_text).is_ok() {
                // Apply tree-sitter highlighting
                for (line_idx, line) in formatted_text.lines().enumerate() {
                    if let Some(tokens) = highlighter.get_line_highlights(line_idx) {
                        let spans =
                            build_highlighted_line_spans(line, tokens, line_idx, &formatted_text);
                        lines.push(ratatui::text::Line::from(spans));
                    } else {
                        lines.push(ratatui::text::Line::from(vec![
                            ratatui::text::Span::styled(
                                line.to_string(),
                                Style::default().fg(Color::White),
                            ),
                        ]));
                    }
                }
                return lines;
            }
    }

    // Fallback to basic regex highlighting on the formatted text
    for line in formatted_text.lines() {
        let spans = match file_extension {
            Some("rs") => highlight_line_rust(line),
            Some("py") => highlight_line_python(line),
            Some("js") | Some("ts") => highlight_line_javascript(line),
            _ => vec![ratatui::text::Span::styled(
                line.to_string(),
                Style::default().fg(Color::White),
            )],
        };
        lines.push(ratatui::text::Line::from(spans));
    }

    lines
}

fn file_extension_to_language_id(ext: Option<&str>) -> Option<crate::syntax::LanguageId> {
    match ext {
        Some("rs") => Some(crate::syntax::LanguageId::Rust),
        Some("py") => Some(crate::syntax::LanguageId::Python),
        Some("js") => Some(crate::syntax::LanguageId::JavaScript),
        Some("ts") => Some(crate::syntax::LanguageId::TypeScript),
        _ => None,
    }
}

fn build_highlighted_line_spans(
    line: &str,
    tokens: &[crate::syntax::highlighter::HighlightToken],
    line_idx: usize,
    full_text: &str,
) -> Vec<ratatui::text::Span<'static>> {
    // Calculate the byte position where this line starts in the full text
    let mut line_start_byte = 0;
    for (i, line_text) in full_text.lines().enumerate() {
        if i == line_idx {
            break;
        }
        line_start_byte += line_text.len() + 1; // +1 for the newline character
    }

    let mut spans = Vec::new();
    let mut pos = 0;

    // Sort tokens by start position
    let mut sorted_tokens = tokens.to_vec();
    sorted_tokens.sort_by_key(|t| t.start);

    for token in sorted_tokens {
        // Check if this token belongs to the current line
        if token.start < line_start_byte || token.start >= line_start_byte + line.len() {
            continue;
        }

        // Convert absolute byte positions to relative byte positions within the line
        let rel_start_byte = token.start - line_start_byte;
        let rel_end_byte = token.end - line_start_byte;

        // Convert byte positions to character positions
        let char_start = line
            .char_indices()
            .take_while(|(i, _)| *i < rel_start_byte)
            .count();
        let char_end = line
            .char_indices()
            .take_while(|(i, _)| *i < rel_end_byte)
            .count();

        // Skip if we're past the current position
        if char_start < pos {
            continue;
        }

        if char_start > pos {
            // Add unhighlighted text before the token
            spans.push(ratatui::text::Span::styled(
                line.chars()
                    .skip(pos)
                    .take(char_start - pos)
                    .collect::<String>(),
                Style::default().fg(Color::White),
            ));
        }

        // Add highlighted token
        let color = syntax_capture_to_color(&token.capture_name);
        spans.push(ratatui::text::Span::styled(
            line.chars()
                .skip(char_start)
                .take(char_end - char_start)
                .collect::<String>(),
            Style::default().fg(color),
        ));

        pos = char_end;
    }

    // Add remaining unhighlighted text
    if pos < line.chars().count() {
        spans.push(ratatui::text::Span::styled(
            line.chars().skip(pos).collect::<String>(),
            Style::default().fg(Color::White),
        ));
    }

    spans
}

fn syntax_capture_to_color(capture_name: &str) -> Color {
    // Fallback colors for syntax highlighting (similar to theme.rs but without theme dependency)
    match capture_name {
        "keyword" | "keyword.control" | "keyword.function" => Color::Rgb(255, 102, 0), // Orange
        "function" | "function.macro" | "function.method" => Color::Rgb(0, 255, 255),  // Cyan
        "type" | "type.builtin" => Color::Rgb(0, 255, 0),                              // Green
        "string" | "string.escape" => Color::Rgb(255, 102, 0),                         // Orange
        "comment" | "comment.line" | "comment.block" => Color::Rgb(128, 128, 128),     // Gray
        "variable" | "variable.member" | "variable.parameter" => Color::Rgb(255, 255, 255), // White
        "constant" | "constant.builtin" | "constant.numeric" => Color::Rgb(255, 255, 0), // Yellow
        "operator" => Color::Rgb(255, 255, 255),                                       // White
        "punctuation" | "punctuation.bracket" => Color::Rgb(255, 255, 255),            // White
        _ => Color::White,                                                             // Default
    }
}

fn highlight_line_rust(line: &str) -> Vec<ratatui::text::Span<'static>> {
    let mut result = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = line.chars().collect();

    while i < chars.len() {
        match chars[i] {
            '"' => {
                // String literal
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i] != '"' || (i > 0 && chars[i - 1] == '\\')) {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                result.push(ratatui::text::Span::styled(
                    text,
                    Style::default().fg(Color::Rgb(255, 102, 0)),
                ));
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                // Comment
                let text: String = chars[i..].iter().collect();
                result.push(ratatui::text::Span::styled(
                    text,
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                ));
                break;
            }
            'f' if line[i..].starts_with("fn")
                && (i + 2 >= chars.len() || !chars[i + 2].is_alphanumeric()) =>
            {
                result.push(ratatui::text::Span::styled(
                    "fn".to_string(),
                    Style::default()
                        .fg(Color::Rgb(0, 255, 255))
                        .add_modifier(Modifier::BOLD),
                ));
                i += 2;
            }
            'l' if line[i..].starts_with("let")
                && (i + 3 >= chars.len() || !chars[i + 3].is_alphanumeric()) =>
            {
                result.push(ratatui::text::Span::styled(
                    "let".to_string(),
                    Style::default()
                        .fg(Color::Rgb(0, 255, 255))
                        .add_modifier(Modifier::BOLD),
                ));
                i += 3;
            }
            'i' if line[i..].starts_with("if")
                && (i + 2 >= chars.len() || !chars[i + 2].is_alphanumeric()) =>
            {
                result.push(ratatui::text::Span::styled(
                    "if".to_string(),
                    Style::default()
                        .fg(Color::Rgb(0, 255, 255))
                        .add_modifier(Modifier::BOLD),
                ));
                i += 2;
            }
            _ => {
                result.push(ratatui::text::Span::styled(
                    chars[i].to_string(),
                    Style::default().fg(Color::White),
                ));
                i += 1;
            }
        }
    }

    result
}

fn highlight_line_python(line: &str) -> Vec<ratatui::text::Span<'static>> {
    let mut result = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = line.chars().collect();

    while i < chars.len() {
        match chars[i] {
            '"' | '\'' => {
                // String literal
                let quote = chars[i];
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i] != quote || (i > 0 && chars[i - 1] == '\\')) {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                result.push(ratatui::text::Span::styled(
                    text,
                    Style::default().fg(Color::Rgb(255, 102, 0)),
                ));
            }
            '#' => {
                // Comment
                let text: String = chars[i..].iter().collect();
                result.push(ratatui::text::Span::styled(
                    text,
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                ));
                break;
            }
            'd' if line[i..].starts_with("def")
                && (i + 3 >= chars.len() || !chars[i + 3].is_alphanumeric()) =>
            {
                result.push(ratatui::text::Span::styled(
                    "def".to_string(),
                    Style::default()
                        .fg(Color::Rgb(0, 255, 255))
                        .add_modifier(Modifier::BOLD),
                ));
                i += 3;
            }
            _ => {
                result.push(ratatui::text::Span::styled(
                    chars[i].to_string(),
                    Style::default().fg(Color::White),
                ));
                i += 1;
            }
        }
    }

    result
}

fn highlight_line_javascript(line: &str) -> Vec<ratatui::text::Span<'static>> {
    let mut result = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = line.chars().collect();

    while i < chars.len() {
        match chars[i] {
            '"' | '\'' => {
                // String literal
                let quote = chars[i];
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i] != quote || (i > 0 && chars[i - 1] == '\\')) {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                result.push(ratatui::text::Span::styled(
                    text,
                    Style::default().fg(Color::Rgb(255, 102, 0)),
                ));
            }
            '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                // Comment
                let text: String = chars[i..].iter().collect();
                result.push(ratatui::text::Span::styled(
                    text,
                    Style::default().fg(Color::Rgb(128, 128, 128)),
                ));
                break;
            }
            'f' if line[i..].starts_with("function")
                && (i + 8 >= chars.len() || !chars[i + 8].is_alphanumeric()) =>
            {
                result.push(ratatui::text::Span::styled(
                    "function".to_string(),
                    Style::default()
                        .fg(Color::Rgb(0, 255, 255))
                        .add_modifier(Modifier::BOLD),
                ));
                i += 8;
            }
            _ => {
                result.push(ratatui::text::Span::styled(
                    chars[i].to_string(),
                    Style::default().fg(Color::White),
                ));
                i += 1;
            }
        }
    }

    result
}

// TODO: Implement syntax highlighting for preview
// For now, preview shows plain text but full content
