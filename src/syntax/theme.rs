use serde::Deserialize;
use std::collections::HashMap;

/// A color represented as RGB values
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Text style attributes
#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Theme configuration loaded from TOML
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub palette: HashMap<String, String>, // color name -> hex
    #[serde(flatten)]
    pub styles: HashMap<String, ThemeStyle>, // capture name -> style
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeStyle {
    pub fg: Option<String>, // color name or hex
    pub bg: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}

/// Resolved style with actual colors
#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvedStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Theme {
    /// Load theme from TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&content)?;
        Ok(theme)
    }

    /// Get resolved style for a capture name
    pub fn get_style(&self, capture_name: &str) -> ResolvedStyle {
        if let Some(theme_style) = self.styles.get(capture_name) {
            ResolvedStyle {
                fg: theme_style.fg.as_ref().and_then(|c| self.parse_color(c)),
                bg: theme_style.bg.as_ref().and_then(|c| self.parse_color(c)),
                bold: theme_style.bold.unwrap_or(false),
                italic: theme_style.italic.unwrap_or(false),
                underline: theme_style.underline.unwrap_or(false),
            }
        } else {
            ResolvedStyle::default()
        }
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
        assert!(comment_style.fg.is_some());
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
}
