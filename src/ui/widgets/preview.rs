// src/ui/widgets/preview.rs - File preview buffer and rendering

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::path::PathBuf;

use crate::formatter::get_formatter_config;
use crate::syntax::{LanguageId, SyntaxHighlighter, get_language_config_by_extension};
use crate::ui::theme::Theme;

/// Preview buffer containing formatted, highlighted file content
#[derive(Debug, Clone)]
pub struct PreviewBuffer {
    pub content: String,
    pub language: Option<LanguageId>,
    pub syntax_highlights: Vec<crate::syntax::HighlightToken>,
}

impl PreviewBuffer {
    /// Load and prepare file content for preview using editor-style logic
    pub fn load_from_file(file_path: &PathBuf) -> Result<Self, String> {
        // 1. Load file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read file: {}", e)),
        };

        // 2. Detect language from extension
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let language_config = get_language_config_by_extension(extension);
        let language = language_config.as_ref().map(|config| config.id);

        // 3. Apply formatting if formatter is available
        let formatted_content = if let Some(lang) = language {
            if let Some(formatter_config) = get_formatter_config(lang) {
                match crate::formatter::external::Formatter::new(formatter_config) {
                    Ok(formatter) => {
                        match formatter.format_text(&content) {
                            Ok(formatted) => formatted,
                            Err(_) => content, // Fallback to original on format error
                        }
                    }
                    Err(_) => content, // Fallback to original if formatter unavailable
                }
            } else {
                content // No formatter for this language
            }
        } else {
            content // No language detected
        };

        // 4. Generate syntax highlights
        let syntax_highlights = if let Some(config) = language_config {
            match SyntaxHighlighter::new(config) {
                Ok(mut highlighter) => {
                    if highlighter.parse(&formatted_content).is_err() {
                        Vec::new() // Fallback on parse error
                    } else {
                        // Collect highlights for all lines (limited for performance)
                        let mut all_highlights = Vec::new();
                        for line_idx in 0..formatted_content.lines().count().min(1000) {
                            if let Some(line_highlights) = highlighter.get_line_highlights(line_idx)
                            {
                                all_highlights.extend(line_highlights.iter().cloned());
                            }
                        }
                        all_highlights
                    }
                }
                Err(_) => Vec::new(), // No syntax highlighting
            }
        } else {
            Vec::new() // No language support
        };

        Ok(Self {
            content: formatted_content,
            language,
            syntax_highlights,
        })
    }
}

/// Render preview buffer content using editor-style highlighting
pub fn render_preview_content(
    preview_buffer: &PreviewBuffer,
    theme: &Theme,
    area: Rect,
) -> Paragraph<'static> {
    let lines: Vec<Line> = preview_buffer
        .content
        .lines()
        .enumerate()
        .take(area.height as usize) // Limit to visible area
        .map(|(line_idx, line_content)| {
            // Filter highlights for this specific line
            let line_highlights: Vec<&crate::syntax::HighlightToken> = preview_buffer
                .syntax_highlights
                .iter()
                .filter(|h| {
                    // Calculate which line this highlight belongs to
                    let before_highlight = &preview_buffer.content[..h.start];
                    let highlight_line = before_highlight.chars().filter(|&c| c == '\n').count();
                    highlight_line == line_idx
                })
                .collect();

            if !line_highlights.is_empty() {
                // Apply syntax highlighting like the editor does
                let mut spans = Vec::new();
                let mut pos = 0;

                for highlight in line_highlights {
                    // Calculate positions within the line
                    let line_start_byte = preview_buffer
                        .content
                        .lines()
                        .take(line_idx)
                        .map(|l| l.len() + 1)
                        .sum();

                    let relative_start = highlight.start.saturating_sub(line_start_byte);
                    let relative_end = highlight.end.saturating_sub(line_start_byte);

                    if relative_start >= line_content.len() {
                        continue; // Highlight is beyond this line
                    }

                    let actual_end = relative_end.min(line_content.len());

                    // Add unhighlighted text before highlight
                    if relative_start > pos {
                        let before_text = line_content[pos..relative_start].to_string();
                        spans.push(Span::styled(before_text, Style::default().fg(Color::White)));
                    }

                    // Add highlighted text
                    if actual_end > relative_start {
                        let highlight_text = line_content[relative_start..actual_end].to_string();
                        let color = theme.syntax_color(&highlight.capture_name);
                        spans.push(Span::styled(highlight_text, Style::default().fg(color)));
                    }

                    pos = actual_end;
                }

                // Add remaining unhighlighted text
                if pos < line_content.len() {
                    let remaining = line_content[pos..].to_string();
                    spans.push(Span::styled(remaining, Style::default().fg(Color::White)));
                }

                Line::from(spans)
            } else {
                // No highlights for this line
                Line::from(Span::styled(
                    line_content.to_owned(),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect();

    Paragraph::new(lines)
}
