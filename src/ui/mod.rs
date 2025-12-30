// ui/mod.rs - UI module definitions

pub mod renderer;
pub mod system_theme;
pub mod theme;
pub mod theme_loader;
pub mod theme_manager;
pub mod widgets;

// Re-export commonly used types
pub use system_theme::{SystemTheme, get_system_theme_colors};
pub use theme::{EditorTheme, PopupTheme, Theme};
pub use theme_loader::{ThemeInfo, ThemeLoader, ThemeLoaderError};
pub use theme_manager::{ThemeChangeResult, ThemeManager};
