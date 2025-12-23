// src/ui/widgets/preview.rs - File preview buffer and rendering

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::path::PathBuf;
use std::collections::HashSet;
use lru::LruCache;

use crate::syntax::{LanguageId, SyntaxHighlighter, get_language_config_by_extension, get_language_config};
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Default)]
pub struct HighlightProgress {
    highlighted_lines: HashSet<usize>,
    fully_parsed: bool,
}

impl HighlightProgress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_line_highlighted(&self, line: usize) -> bool {
        self.highlighted_lines.contains(&line)
    }

    pub fn mark_lines_highlighted(&mut self, start_line: usize, line_count: usize) {
        for line in start_line..(start_line + line_count) {
            self.highlighted_lines.insert(line);
        }
    }

    pub fn clear(&mut self) {
        self.highlighted_lines.clear();
        self.fully_parsed = false;
    }

    pub fn is_fully_parsed(&self) -> bool {
        self.fully_parsed
    }

    pub fn set_fully_parsed(&mut self, fully_parsed: bool) {
        self.fully_parsed = fully_parsed;
    }
}

#[derive(Debug, Clone)]
pub struct PreviewBuffer {
    pub content: String,
    pub language: Option<LanguageId>,
    pub syntax_highlights: Option<Vec<crate::syntax::HighlightToken>>,
    pub highlight_progress: HighlightProgress,
}

impl PreviewBuffer {
    pub fn load_from_file(file_path: &PathBuf) -> Result<Self, String> {
        let content = match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read file: {}", e)),
        };

        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let language_config = get_language_config_by_extension(extension);
        let language = language_config.as_ref().map(|config| config.id);

        Ok(Self {
            content,
            language,
            syntax_highlights: None,
            highlight_progress: HighlightProgress::new(),
        })
    }

    pub fn ensure_highlighted(&mut self, start_line: usize, line_count: usize) {
        if self.highlight_progress.is_fully_parsed() {
            return;
        }

        let total_lines = self.content.lines().count();
        let end_line = (start_line + line_count).min(total_lines);

        if let Some(lang) = self.language {
            let config = get_language_config(lang);
            if let Ok(mut highlighter) = SyntaxHighlighter::new(config) {
                if highlighter.parse(&self.content).is_ok() {
                    if self.syntax_highlights.is_none() {
                        self.syntax_highlights = Some(Vec::new());
                    }

                    if let Some(highlights) = &mut self.syntax_highlights {
                        for line_idx in start_line..end_line {
                            if !self.highlight_progress.is_line_highlighted(line_idx) {
                                if let Some(line_highlights) = highlighter.get_line_highlights(line_idx) {
                                    highlights.extend(line_highlights.iter().cloned());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            if self.syntax_highlights.is_none() {
                self.syntax_highlights = Some(Vec::new());
            }
            self.highlight_progress.set_fully_parsed(true);
            return;
        }

        self.highlight_progress.mark_lines_highlighted(start_line, end_line - start_line);

        if end_line >= total_lines {
            self.highlight_progress.set_fully_parsed(true);
        }
    }
}

#[derive(Debug)]
pub struct PreviewCache {
    cache: LruCache<PathBuf, PreviewBuffer>,
    max_size: usize,
}

impl PreviewCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: LruCache::new(std::num::NonZeroUsize::new(max_size).unwrap()),
            max_size,
        }
    }

    pub fn get(&mut self, path: &PathBuf) -> Option<PreviewBuffer> {
        self.cache.get(path).cloned()
    }

    pub fn put(&mut self, path: PathBuf, buffer: PreviewBuffer) {
        self.cache.put(path, buffer);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for PreviewCache {
    fn default() -> Self {
        Self::new(50)
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
            let line_highlights: Vec<&crate::syntax::HighlightToken> = preview_buffer
                .syntax_highlights
                .as_ref()
                .map(|highlights| {
                    highlights
                        .iter()
                        .filter(|h| {
                            let before_highlight = &preview_buffer.content[..h.start];
                            let highlight_line = before_highlight.chars().filter(|&c| c == '\n').count();
                            highlight_line == line_idx
                        })
                        .collect()
                })
                .unwrap_or_default();

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
