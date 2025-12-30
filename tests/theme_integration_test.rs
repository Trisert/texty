// tests/theme_integration_test.rs - Integration tests for theme system

use texty::ui::{Theme, ThemeChangeResult, ThemeManager};

#[test]
fn test_theme_discovery() {
    let manager = ThemeManager::new();
    let themes = manager.list_available_themes();

    assert!(!themes.is_empty(), "Should discover at least one theme");
    assert!(
        themes.contains(&"monokai".to_string()),
        "Should contain monokai theme"
    );
}

#[test]
fn test_theme_switching_workflow() {
    let manager = ThemeManager::new();

    if manager.theme_exists("monokai") {
        let result = manager.switch_theme("monokai");
        assert!(matches!(result, ThemeChangeResult::Switched(_)));

        let current = manager.get_current_theme();
        assert_eq!(current.named_theme, Some("monokai".to_string()));
        assert!(current.loaded_syntax_theme.is_some());

        if manager.theme_exists("dracula") {
            manager.switch_theme("dracula");
            let new_current = manager.get_current_theme();
            assert_eq!(new_current.named_theme, Some("dracula".to_string()));
        }
    }
}

#[test]
fn test_terminal_palette_workflow() {
    let manager = ThemeManager::new();

    manager.use_terminal_palette();
    let theme = manager.get_current_theme();

    assert!(theme.use_terminal_palette);
    assert!(theme.terminal_palette.is_some());
}

#[test]
fn test_theme_info_retrieval() {
    let manager = ThemeManager::new();

    if let Some(info) = manager.get_theme_info("monokai") {
        assert!(info.contains("monokai"));
    }
}

#[test]
fn test_nonexistent_theme() {
    let manager = ThemeManager::new();

    let result = manager.switch_theme("nonexistent_theme_xyz");
    assert!(matches!(result, ThemeChangeResult::Error(_)));

    let theme = manager.get_current_theme();
    assert_eq!(theme.named_theme, None);
}

#[test]
fn test_theme_reload() {
    let manager = ThemeManager::new();
    let initial_count = manager.list_available_themes().len();

    manager.reload_themes();
    let after_count = manager.list_available_themes().len();

    assert_eq!(initial_count, after_count);
}

#[test]
fn test_syntax_color_fallback() {
    let theme = Theme::default();

    let keyword_color = theme.syntax_color("keyword");
    assert_ne!(keyword_color, theme.general.foreground);

    let unknown_color = theme.syntax_color("unknown.capture.name");
    assert_eq!(unknown_color, theme.general.foreground);
}

#[test]
fn test_editor_theme_elements() {
    let theme = Theme::default();

    assert_ne!(theme.editor.background, theme.editor.line_number_bg);
    assert_ne!(theme.editor.selection_bg, theme.editor.primary_selection_bg);

    let style = theme.get_line_number_style(true, true);
    assert_eq!(style.fg, Some(theme.editor.line_number_current_fg));
}

#[test]
fn test_popup_theme_elements() {
    let theme = Theme::default();

    assert_ne!(theme.popup.background, theme.popup.border_color);
    assert_ne!(theme.popup.highlight_bg, theme.popup.background);
}

#[test]
fn test_ui_theme_elements() {
    let theme = Theme::default();

    assert_ne!(theme.ui.status_bar_bg, theme.ui.gutter_fg);
    assert_ne!(theme.ui.diagnostic_error, theme.ui.diagnostic_warning);
}

#[test]
fn test_multiple_theme_switches() {
    let manager = ThemeManager::new();

    let themes = manager.list_available_themes();

    for theme_name in themes.iter().take(3) {
        if manager.theme_exists(theme_name) {
            let result = manager.switch_theme(theme_name);
            if matches!(result, ThemeChangeResult::Switched(_)) {
                let current = manager.get_current_theme();
                assert_eq!(current.named_theme, Some(theme_name.clone()));
            }
        }
    }
}

#[test]
fn test_theme_persistence_after_switch() {
    let manager = ThemeManager::new();

    let custom_theme = Theme::with_terminal_palette();
    manager.set_theme(custom_theme.clone());

    let retrieved = manager.get_current_theme();
    assert_eq!(
        retrieved.use_terminal_palette,
        custom_theme.use_terminal_palette
    );
}

#[test]
fn test_syntax_capture_hierarchy() {
    let theme = Theme::default();

    let function_color = theme.syntax_color("function");
    let _builtin_color = theme.syntax_color("function.builtin");
    let method_color = theme.syntax_color("function.method");

    assert_eq!(function_color, method_color);
}
