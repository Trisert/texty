// ui/theme.rs - Theme system for UI styling

use ratatui::style::Color;

/// Theme configuration
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct Theme {
    pub general: GeneralTheme,
    pub syntax: SyntaxTheme,
    pub ui: UiTheme,
}

#[derive(Debug, Clone)]
pub struct GeneralTheme {
    pub background: Color,
    pub foreground: Color,
}

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub keyword: Color,
    pub function: Color,
    pub r#type: Color,
    pub string: Color,
    pub comment: Color,
    pub variable: Color,
}

#[derive(Debug, Clone)]
pub struct UiTheme {
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub gutter_fg: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub diagnostic_error: Color,
    pub diagnostic_warning: Color,
    pub diagnostic_info: Color,
    pub diagnostic_hint: Color,
}


impl Default for GeneralTheme {
    fn default() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
        }
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            keyword: Color::Cyan,
            function: Color::Green,
            r#type: Color::Yellow,
            string: Color::Red,
            comment: Color::Blue,
            variable: Color::White,
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            status_bar_bg: Color::Blue,
            status_bar_fg: Color::White,
            gutter_fg: Color::DarkGray,
            cursor_bg: Color::Black,
            cursor_fg: Color::White,
            diagnostic_error: Color::Red,
            diagnostic_warning: Color::Yellow,
            diagnostic_info: Color::Blue,
            diagnostic_hint: Color::Cyan,
        }
    }
}

impl Theme {
    /// Get syntax color for a highlight kind
    pub fn syntax_color(&self, kind: &crate::syntax::HighlightKind) -> Color {
        match kind {
            crate::syntax::HighlightKind::Keyword => self.syntax.keyword,
            crate::syntax::HighlightKind::Function => self.syntax.function,
            crate::syntax::HighlightKind::Type => self.syntax.r#type,
            crate::syntax::HighlightKind::String => self.syntax.string,
            crate::syntax::HighlightKind::Comment => self.syntax.comment,
            crate::syntax::HighlightKind::Variable => self.syntax.variable,
        }
    }
}
