use serde::Deserialize;
use std::collections::HashMap;

/// A color represented as RGB values
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Underline configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Underline {
    pub color: Option<String>,
}

/// Text style attributes - can be a simple color string or a full style table
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ThemeStyle {
    /// Bare color string like `comment = "#6272a4"`
    Bare(String),
    /// Simple style with just fg/bg like `comment = { fg = "#6272a4" }`
    Simple {
        #[serde(default)]
        fg: Option<String>,
        #[serde(default)]
        bg: Option<String>,
    },
    /// Full style with modifiers and underline
    Full {
        fg: Option<String>,
        bg: Option<String>,
        modifiers: Option<Vec<String>>,
        underline: Option<Underline>,
    },
}

impl Default for ThemeStyle {
    fn default() -> Self {
        ThemeStyle::Full {
            fg: None,
            bg: None,
            modifiers: None,
            underline: None,
        }
    }
}

impl ThemeStyle {
    fn fg(&self) -> Option<&String> {
        match self {
            ThemeStyle::Bare(s) => Some(s),
            ThemeStyle::Simple { fg, .. } => fg.as_ref(),
            ThemeStyle::Full { fg, .. } => fg.as_ref(),
        }
    }

    fn bg(&self) -> Option<&String> {
        match self {
            ThemeStyle::Bare(_) => None,
            ThemeStyle::Simple { bg, .. } => bg.as_ref(),
            ThemeStyle::Full { bg, .. } => bg.as_ref(),
        }
    }

    fn modifiers(&self) -> Option<&Vec<String>> {
        match self {
            ThemeStyle::Bare(_) => None,
            ThemeStyle::Simple { .. } => None,
            ThemeStyle::Full { modifiers, .. } => modifiers.as_ref(),
        }
    }

    fn has_underline(&self) -> bool {
        match self {
            ThemeStyle::Bare(_) => false,
            ThemeStyle::Simple { .. } => false,
            ThemeStyle::Full { underline, .. } => underline.is_some(),
        }
    }
}

/// Editor theme configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EditorTheme {
    pub background: Option<ThemeStyle>,
    pub whitespace: Option<ThemeStyle>,
    pub cursor: Option<ThemeStyle>,
    pub line_number: Option<ThemeStyle>,
    pub line_number_selected: Option<ThemeStyle>,
    pub selection: Option<ThemeStyle>,
    pub primary_selection: Option<ThemeStyle>,
    pub indent_guide: Option<ThemeStyle>,
    pub current_line: Option<ThemeStyle>,
}

/// Status theme configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct StatusTheme {
    pub normal: Option<ThemeStyle>,
    pub insert: Option<ThemeStyle>,
    pub select: Option<ThemeStyle>,
}

/// Popup theme configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PopupTheme {
    pub background: Option<ThemeStyle>,
    pub border: Option<ThemeStyle>,
    pub menu: Option<ThemeStyle>,
    pub menu_selected: Option<ThemeStyle>,
    pub scrollbar: Option<ThemeStyle>,
    pub scrollbar_thumb: Option<ThemeStyle>,
}

/// UI theme configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UiTheme {
    pub menu: Option<ThemeStyle>,
    pub menu_selected: Option<ThemeStyle>,
    pub help: Option<ThemeStyle>,
    pub cursorline: Option<ThemeStyle>,
    pub cursorline_primary: Option<ThemeStyle>,
    pub highlight: Option<ThemeStyle>,
    pub window: Option<ThemeStyle>,
    pub window_border: Option<ThemeStyle>,
    pub text_focus: Option<ThemeStyle>,
    pub text_inactive: Option<ThemeStyle>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Theme {
    pub palette: HashMap<String, String>,
    #[serde(flatten)]
    pub styles: HashMap<String, ThemeStyle>,
    #[serde(default)]
    pub editor: Option<EditorTheme>,
    #[serde(default)]
    pub status: Option<StatusTheme>,
    #[serde(default)]
    pub popup: Option<PopupTheme>,
    #[serde(default)]
    pub ui: Option<UiTheme>,
    #[serde(default)]
    pub inherits: Option<String>,
}

/// Resolved style with actual colors
#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvedStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub underlined: bool,
    pub reversed: bool,
    pub crossed_out: bool,
    pub slow_blink: bool,
    pub rapid_blink: bool,
    pub hidden: bool,
}

impl Theme {
    /// Load theme from TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;

        #[derive(Deserialize)]
        struct RawTheme {
            palette: HashMap<String, String>,
            #[serde(default)]
            editor: Option<EditorTheme>,
            #[serde(default)]
            status: Option<StatusTheme>,
            #[serde(default)]
            popup: Option<PopupTheme>,
            #[serde(default)]
            ui: Option<UiTheme>,
            #[serde(default)]
            inherits: Option<String>,
            #[serde(default)]
            styles: HashMap<String, ThemeStyle>,
        }

        let raw: RawTheme = toml::from_str(&content)?;

        let theme = Theme {
            palette: raw.palette,
            styles: raw.styles,
            editor: raw.editor,
            status: raw.status,
            popup: raw.popup,
            ui: raw.ui,
            inherits: raw.inherits,
        };

        // Handle theme inheritance
        if let Some(inherits) = &theme.inherits {
            let base_theme = Self::from_file(&format!("runtime/themes/{}.toml", inherits))?;
            return Ok(theme.merge(&base_theme));
        }

        Ok(theme)
    }

    /// Merge this theme with a base theme (base takes precedence for missing values)
    fn merge(self, base: &Self) -> Self {
        let mut merged_palette = base.palette.clone();
        merged_palette.extend(self.palette);

        let mut merged_styles = base.styles.clone();
        merged_styles.extend(self.styles);

        Theme {
            palette: merged_palette,
            styles: merged_styles,
            editor: Some(
                self.editor
                    .unwrap_or_default()
                    .merge(&base.editor.clone().unwrap_or_default()),
            ),
            status: Some(
                self.status
                    .unwrap_or_default()
                    .merge(&base.status.clone().unwrap_or_default()),
            ),
            popup: Some(
                self.popup
                    .unwrap_or_default()
                    .merge(&base.popup.clone().unwrap_or_default()),
            ),
            ui: Some(
                self.ui
                    .unwrap_or_default()
                    .merge(&base.ui.clone().unwrap_or_default()),
            ),
            inherits: base.inherits.clone(),
        }
    }

    /// Get resolved style for a capture name
    pub fn get_style(&self, capture_name: &str) -> ResolvedStyle {
        if let Some(theme_style) = self.styles.get(capture_name) {
            self.resolve_style(theme_style)
        } else {
            ResolvedStyle::default()
        }
    }

    /// Resolve a ThemeStyle to a ResolvedStyle
    fn resolve_style(&self, theme_style: &ThemeStyle) -> ResolvedStyle {
        let modifiers = theme_style.modifiers().map(|m| m.as_slice()).unwrap_or(&[]);

        ResolvedStyle {
            fg: theme_style.fg().and_then(|c| self.parse_color(c)),
            bg: theme_style.bg().and_then(|c| self.parse_color(c)),
            bold: modifiers.contains(&"bold".to_string()),
            italic: modifiers.contains(&"italic".to_string()),
            dim: modifiers.contains(&"dim".to_string()),
            underlined: theme_style.has_underline(),
            reversed: modifiers.contains(&"reversed".to_string()),
            crossed_out: modifiers.contains(&"crossed_out".to_string()),
            slow_blink: modifiers.contains(&"slow_blink".to_string()),
            rapid_blink: modifiers.contains(&"rapid_blink".to_string()),
            hidden: modifiers.contains(&"hidden".to_string()),
        }
    }

    /// Get editor theme style
    pub fn get_editor_style(&self, key: &str) -> ResolvedStyle {
        if let Some(editor) = &self.editor {
            let theme_style = match key {
                "background" => editor.background.as_ref(),
                "whitespace" => editor.whitespace.as_ref(),
                "cursor" => editor.cursor.as_ref(),
                "line_number" => editor.line_number.as_ref(),
                "line_number_selected" => editor.line_number_selected.as_ref(),
                "selection" => editor.selection.as_ref(),
                "primary_selection" => editor.primary_selection.as_ref(),
                "indent_guide" => editor.indent_guide.as_ref(),
                "current_line" => editor.current_line.as_ref(),
                _ => return ResolvedStyle::default(),
            };

            if let Some(style) = theme_style {
                return self.resolve_style(style);
            }
        }
        ResolvedStyle::default()
    }

    /// Get status theme style
    pub fn get_status_style(&self, mode: &str) -> ResolvedStyle {
        if let Some(status) = &self.status {
            let theme_style = match mode {
                "normal" => status.normal.as_ref(),
                "insert" => status.insert.as_ref(),
                "select" => status.select.as_ref(),
                _ => return ResolvedStyle::default(),
            };

            if let Some(style) = theme_style {
                return self.resolve_style(style);
            }
        }
        ResolvedStyle::default()
    }

    /// Get popup theme style
    pub fn get_popup_style(&self, key: &str) -> ResolvedStyle {
        if let Some(popup) = &self.popup {
            let theme_style = match key {
                "background" => popup.background.as_ref(),
                "border" => popup.border.as_ref(),
                "menu" => popup.menu.as_ref(),
                "menu_selected" => popup.menu_selected.as_ref(),
                "scrollbar" => popup.scrollbar.as_ref(),
                "scrollbar_thumb" => popup.scrollbar_thumb.as_ref(),
                _ => return ResolvedStyle::default(),
            };

            if let Some(style) = theme_style {
                return self.resolve_style(style);
            }
        }
        ResolvedStyle::default()
    }

    /// Get UI theme style
    pub fn get_ui_style(&self, key: &str) -> ResolvedStyle {
        if let Some(ui) = &self.ui {
            let theme_style = match key {
                "menu" => ui.menu.as_ref(),
                "menu_selected" => ui.menu_selected.as_ref(),
                "help" => ui.help.as_ref(),
                "cursorline" => ui.cursorline.as_ref(),
                "cursorline_primary" => ui.cursorline_primary.as_ref(),
                "highlight" => ui.highlight.as_ref(),
                "window" => ui.window.as_ref(),
                "window_border" => ui.window_border.as_ref(),
                "text_focus" => ui.text_focus.as_ref(),
                "text_inactive" => ui.text_inactive.as_ref(),
                _ => return ResolvedStyle::default(),
            };

            if let Some(style) = theme_style {
                return self.resolve_style(style);
            }
        }
        ResolvedStyle::default()
    }

    /// Parse color from hex string or palette name
    fn parse_color(&self, color_str: &str) -> Option<Color> {
        // First check if it's a palette color
        if let Some(hex) = self.palette.get(color_str) {
            return Self::hex_to_color(hex);
        }

        // Otherwise parse as hex directly
        Self::hex_to_color(color_str)
    }

    /// Convert hex string to Color
    fn hex_to_color(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color { r, g, b })
        } else {
            None
        }
    }
}

impl ResolvedStyle {
    /// Convert to ratatui Style
    pub fn to_ratatui_style(self) -> ratatui::style::Style {
        use ratatui::style::Modifier;

        let mut style = ratatui::style::Style::default();

        if let Some(fg) = self.fg {
            style = style.fg(ratatui::style::Color::Rgb(fg.r, fg.g, fg.b));
        }

        if let Some(bg) = self.bg {
            style = style.bg(ratatui::style::Color::Rgb(bg.r, bg.g, bg.b));
        }

        if self.bold {
            style = style.add_modifier(Modifier::BOLD);
        }

        if self.italic {
            style = style.add_modifier(Modifier::ITALIC);
        }

        if self.dim {
            style = style.add_modifier(Modifier::DIM);
        }

        if self.underlined {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        if self.reversed {
            style = style.add_modifier(Modifier::REVERSED);
        }

        if self.crossed_out {
            style = style.add_modifier(Modifier::CROSSED_OUT);
        }

        if self.slow_blink {
            style = style.add_modifier(Modifier::SLOW_BLINK);
        }

        if self.rapid_blink {
            style = style.add_modifier(Modifier::RAPID_BLINK);
        }

        if self.hidden {
            style = style.add_modifier(Modifier::HIDDEN);
        }

        style
    }
}

impl EditorTheme {
    fn merge(self, base: &Self) -> Self {
        EditorTheme {
            background: self.background.or(base.background.clone()),
            whitespace: self.whitespace.or(base.whitespace.clone()),
            cursor: self.cursor.or(base.cursor.clone()),
            line_number: self.line_number.or(base.line_number.clone()),
            line_number_selected: self
                .line_number_selected
                .or(base.line_number_selected.clone()),
            selection: self.selection.or(base.selection.clone()),
            primary_selection: self.primary_selection.or(base.primary_selection.clone()),
            indent_guide: self.indent_guide.or(base.indent_guide.clone()),
            current_line: self.current_line.or(base.current_line.clone()),
        }
    }
}

impl StatusTheme {
    fn merge(self, base: &Self) -> Self {
        StatusTheme {
            normal: self.normal.or(base.normal.clone()),
            insert: self.insert.or(base.insert.clone()),
            select: self.select.or(base.select.clone()),
        }
    }
}

impl PopupTheme {
    fn merge(self, base: &Self) -> Self {
        PopupTheme {
            background: self.background.or(base.background.clone()),
            border: self.border.or(base.border.clone()),
            menu: self.menu.or(base.menu.clone()),
            menu_selected: self.menu_selected.or(base.menu_selected.clone()),
            scrollbar: self.scrollbar.or(base.scrollbar.clone()),
            scrollbar_thumb: self.scrollbar_thumb.or(base.scrollbar_thumb.clone()),
        }
    }
}

impl UiTheme {
    fn merge(self, base: &Self) -> Self {
        UiTheme {
            menu: self.menu.or(base.menu.clone()),
            menu_selected: self.menu_selected.or(base.menu_selected.clone()),
            help: self.help.or(base.help.clone()),
            cursorline: self.cursorline.or(base.cursorline.clone()),
            cursorline_primary: self.cursorline_primary.or(base.cursorline_primary.clone()),
            highlight: self.highlight.or(base.highlight.clone()),
            window: self.window.or(base.window.clone()),
            window_border: self.window_border.or(base.window_border.clone()),
            text_focus: self.text_focus.or(base.text_focus.clone()),
            text_inactive: self.text_inactive.or(base.text_inactive.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color() {
        let color = Theme::hex_to_color("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_theme_from_toml() {
        let theme = Theme::from_file("runtime/themes/monokai.toml").unwrap();

        let comment_style = theme.get_style("comment");
        assert!(comment_style.fg.is_some(), "comment style fg is None");
        if let Some(color) = comment_style.fg {
            assert_eq!(color.r, 98);
            assert_eq!(color.g, 114);
            assert_eq!(color.b, 164);
        }

        let keyword_style = theme.get_style("keyword");
        assert!(keyword_style.fg.is_some());
        if let Some(color) = keyword_style.fg {
            assert_eq!(color.r, 255);
            assert_eq!(color.g, 121);
            assert_eq!(color.b, 198);
        }
    }

    #[test]
    fn test_resolved_style_to_ratatui() {
        let style = ResolvedStyle {
            fg: Some(Color { r: 255, g: 0, b: 0 }),
            bg: Some(Color { r: 0, g: 255, b: 0 }),
            bold: true,
            italic: true,
            ..Default::default()
        };

        let ratatui_style = style.to_ratatui_style();
        assert!(matches!(
            ratatui_style.fg,
            Some(ratatui::style::Color::Rgb(255, 0, 0))
        ));
        assert!(matches!(
            ratatui_style.bg,
            Some(ratatui::style::Color::Rgb(0, 255, 0))
        ));
    }
}
