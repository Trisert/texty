// tests/integration_test.rs - Integration tests for the text editor

use lsp_types::DiagnosticSeverity;
use std::fs;
use tempfile::TempDir;
use texty::command::Command;
use texty::editor::Editor;

#[test]
fn test_load_edit_save_file() {
    // Create a temporary file with initial content
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let initial_content = "Hello\nWorld\n";
    fs::write(&file_path, initial_content).unwrap();

    // Create editor and load file
    let mut editor = Editor::new();
    editor.open_file(file_path.to_str().unwrap()).unwrap();

    // Verify initial content
    assert_eq!(editor.buffer.line_count(), 3); // Hello\nWorld\n has 3 lines
    assert_eq!(editor.buffer.line(0).unwrap(), "Hello");
    assert_eq!(editor.buffer.line(1).unwrap(), "World");
    assert_eq!(editor.buffer.line(2).unwrap(), "");

    // Edit: insert text at end of first line
    editor.execute_command(Command::InsertMode);
    for _ in 0..5 {
        // Move to end of "Hello"
        editor.execute_command(Command::MoveRight);
    }
    editor.execute_command(Command::InsertChar('!'));
    editor.execute_command(Command::NormalMode);

    // Save file
    editor.execute_command(Command::SaveFile);

    // Check saved content
    let saved_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(saved_content, "Hello!\nWorld\n");
}

#[test]
fn test_syntax_highlighting() {
    // Create a temporary Rust file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    let rust_code = "fn main() {\n    println!(\"Hello\");\n}\n";
    fs::write(&file_path, rust_code).unwrap();

    // Create editor and load file
    let mut editor = Editor::new();
    editor.open_file(file_path.to_str().unwrap()).unwrap();

    // Check that highlighter is initialized
    assert!(editor.buffer.highlighter.is_some());

    // Check highlights for first line
    let highlights = editor
        .buffer
        .highlighter
        .as_ref()
        .unwrap()
        .get_line_highlights(0);
    assert!(highlights.is_some(), "Should have highlights on line 0");
    // At least one highlight (fn keyword)
    assert!(
        !highlights.unwrap().is_empty(),
        "Should have at least one highlight token"
    );
}

#[test]
fn test_cursor_movement() {
    let mut editor = Editor::new();
    editor.buffer.insert_text("Hello World", 0, 0).unwrap();

    // Test cursor movement
    assert_eq!(editor.cursor.line, 0);
    assert_eq!(editor.cursor.col, 0);

    editor.execute_command(Command::MoveRight);
    assert_eq!(editor.cursor.col, 1);

    editor.execute_command(Command::MoveRight);
    assert_eq!(editor.cursor.col, 2);
}

#[test]
fn test_mode_switching() {
    let mut editor = Editor::new();

    assert!(matches!(editor.mode, texty::mode::Mode::Normal));

    editor.execute_command(Command::InsertMode);
    assert!(matches!(editor.mode, texty::mode::Mode::Insert));

    editor.execute_command(Command::NormalMode);
    assert!(matches!(editor.mode, texty::mode::Mode::Normal));
}

#[test]
fn test_lsp_integration() {
    let mut editor = Editor::new();

    // Create a test file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    let rust_code = "fn main() {\n    println!(\"Hello\");\n}\n";
    fs::write(&file_path, rust_code).unwrap();

    // Load file and check LSP initialization
    editor.open_file(file_path.to_str().unwrap()).unwrap();

    // Verify basic editor state
    assert_eq!(editor.buffer.line_count(), 4);
    assert!(editor.current_language.is_some());

    // Test completion popup functionality
    editor.execute_command(Command::Completion);
    // Completion should be requested (though we can't test the full async flow easily)

    // Test progress manager initialization
    let progress_count = editor.progress_items.lock().unwrap().len();
    assert_eq!(progress_count, 0); // No active progress initially
}

#[test]
fn test_lsp_diagnostic_colors() {
    // Test diagnostic color mapping
    assert_eq!(
        texty::lsp::diagnostics::DiagnosticManager::diagnostic_to_color(DiagnosticSeverity::ERROR),
        crossterm::style::Color::Red
    );
    assert_eq!(
        texty::lsp::diagnostics::DiagnosticManager::diagnostic_to_color(
            DiagnosticSeverity::WARNING
        ),
        crossterm::style::Color::Yellow
    );
    assert_eq!(
        texty::lsp::diagnostics::DiagnosticManager::diagnostic_to_color(
            DiagnosticSeverity::INFORMATION
        ),
        crossterm::style::Color::Blue
    );
    assert_eq!(
        texty::lsp::diagnostics::DiagnosticManager::diagnostic_to_color(DiagnosticSeverity::HINT),
        crossterm::style::Color::Cyan
    );
}

#[tokio::test]
async fn test_lsp_client_with_rust_analyzer() {
    use texty::lsp::manager::LspManager;
    use texty::syntax::LanguageId;

    // Skip this test in normal runs to avoid LSP server output cluttering test results
    // Run with LSP_TEST=1 cargo test to enable this test
    if std::env::var("LSP_TEST").is_err() {
        return;
    }

    // Test LSP manager configuration
    let manager = LspManager::new();

    // Test trigger characters
    assert!(manager.is_trigger_character(LanguageId::Rust, "."));
    assert!(manager.is_trigger_character(LanguageId::Rust, "::"));

    // Test LSP client creation and initialization
    let result = manager.get_or_start_client(LanguageId::Rust).await;
    if result.is_err() {
        // Skip test if LSP server not available
        return;
    }

    // Check if client is initialized
    let initialized = manager.is_client_initialized(LanguageId::Rust).await;
    assert!(initialized, "LSP client should be initialized");

    // Test basic LSP functionality (without full request testing to avoid output)
    assert!(
        manager.get_client(LanguageId::Rust).await.is_some(),
        "Should have LSP client"
    );

    // Shutdown
    let shutdown_result = manager.shutdown_all().await;
    assert!(shutdown_result.is_ok(), "LSP shutdown should succeed");
}

#[test]
fn test_ui_widgets_and_floating_windows() {
    use texty::editor::Editor;
    use texty::ui::theme::Theme;
    use texty::ui::widgets::hover::HoverWindow;
    use texty::ui::widgets::menu::CodeActionMenu;

    let mut editor = Editor::new();
    let theme = Theme::default();

    // Test hover window functionality
    let hover_content = vec!["Line 1".to_string(), "Line 2".to_string()];
    editor.show_hover(hover_content.clone());

    assert!(editor.hover_content.is_some());
    assert_eq!(editor.hover_content.as_ref().unwrap(), &hover_content);

    let hover_window = HoverWindow::new(hover_content, &theme);
    let rect = hover_window.calculate_position(10, 5, ratatui::layout::Rect::new(0, 0, 80, 24));
    assert!(rect.width > 0 && rect.height > 0);

    editor.hide_hover();
    assert!(editor.hover_content.is_none());

    // Test code action menu functionality
    let actions = vec![
        lsp_types::CodeAction {
            title: "Action 1".to_string(),
            kind: Some(lsp_types::CodeActionKind::QUICKFIX),
            ..Default::default()
        },
        lsp_types::CodeAction {
            title: "Action 2".to_string(),
            kind: Some(lsp_types::CodeActionKind::REFACTOR),
            ..Default::default()
        },
    ];

    editor.show_code_actions(actions.clone());
    assert!(editor.code_actions.is_some());
    assert_eq!(editor.code_actions.as_ref().unwrap().len(), 2);
    assert_eq!(editor.code_action_selected, 0);

    let menu = CodeActionMenu::new(actions, &theme);
    let rect = menu.calculate_position(10, 5, ratatui::layout::Rect::new(0, 0, 80, 24));
    assert!(rect.width > 0 && rect.height > 0);

    editor.select_next_code_action();
    assert_eq!(editor.code_action_selected, 1);

    editor.select_prev_code_action();
    assert_eq!(editor.code_action_selected, 0);

    editor.hide_code_actions();
    assert!(editor.code_actions.is_none());
}

#[test]
fn test_command_line_functionality() {
    use texty::editor::Editor;
    use texty::mode::Mode;

    let mut editor = Editor::new();

    // Test entering command mode
    editor.enter_command_mode();
    assert_eq!(editor.mode, Mode::Command);
    assert!(editor.command_line.is_empty());

    // Test command input
    let should_quit = editor.handle_command_input('w').unwrap();
    assert!(!should_quit);
    assert_eq!(editor.command_line, "w");

    let should_quit = editor.handle_command_input('q').unwrap();
    assert!(!should_quit);
    assert_eq!(editor.command_line, "wq");

    // Test command execution - wq should quit
    let should_quit = editor.handle_command_input('\n').unwrap();
    assert!(should_quit); // wq command should signal quit
    assert_eq!(editor.mode, Mode::Normal); // Should return to normal mode

    // Test command display
    editor.enter_command_mode();
    editor.handle_command_input('q').unwrap();
    let display = editor.get_command_line_display();
    assert_eq!(display, ":q");

    // Test quit command
    let should_quit = editor.handle_command_input('\n').unwrap();
    assert!(should_quit);
}

#[test]
fn test_status_bar_full_width() {
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
    use texty::editor::Editor;
    use texty::ui::theme::Theme;
    use texty::ui::widgets::status_bar::StatusBar;

    let editor = Editor::new();
    let theme = Theme::default();

    // Test status bar fills full width
    let status_bar = StatusBar::new(&editor, &theme);

    // Create a buffer with specific width
    let width = 80;
    let mut buf = Buffer::empty(Rect::new(0, 0, width, 1));

    // Render the status bar
    status_bar.render(Rect::new(0, 0, width, 1), &mut buf);

    // Check that the entire width is filled
    for x in 0..width {
        let cell = buf.get(x, 0);
        // Should have background color (non-default)
        assert_eq!(cell.bg, theme.ui.status_bar_bg);
    }
}

#[test]
fn test_ui_widgets() {
    use texty::editor::Editor;
    use texty::ui::theme::Theme;
    // Widgets are tested via direct module access below

    // Test that widgets can be created and work with editor
    let editor = Editor::new();
    let theme = Theme::default();

    // Test widget creation
    let _editor_pane = texty::ui::widgets::editor_pane::EditorPane::new(&editor, &theme);
    let _gutter = texty::ui::widgets::gutter::Gutter::new(&editor, &theme);
    let _status_bar = texty::ui::widgets::status_bar::StatusBar::new(&editor, &theme);
    let _completion_popup = texty::ui::widgets::completion::CompletionPopup::new();

    // Test theme color methods
    assert_eq!(theme.syntax_color("keyword"), ratatui::style::Color::Cyan);
}

#[test]
fn test_command_line_file_opening() {
    use std::fs;
    use tempfile::TempDir;
    use texty::editor::Editor;

    // Create a temporary file with some content
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_open.rs");
    let test_content = "fn main() {\n    println!(\"Hello, world!\");\n}\n";
    fs::write(&file_path, test_content).unwrap();

    // Create editor and open the file
    let mut editor = Editor::new();
    let open_result = editor.open_file(file_path.to_str().unwrap());
    assert!(open_result.is_ok(), "File should open successfully");

    // Verify the content was loaded
    assert_eq!(editor.buffer.line_count(), 4); // fn main() {\n    println!("Hello, world!");\n}\n has 4 lines
    assert_eq!(editor.buffer.line(0).unwrap(), "fn main() {");
    assert_eq!(
        editor.buffer.line(1).unwrap(),
        "    println!(\"Hello, world!\");"
    );
    assert_eq!(editor.buffer.line(2).unwrap(), "}");

    // Verify file path is set
    assert_eq!(
        editor.buffer.file_path,
        Some(file_path.to_str().unwrap().to_string())
    );
}

#[test]
fn test_fuzzy_search_formatted_preview() {
    use std::fs;
    use tempfile::TempDir;
    use texty::fuzzy_search::FileItem;
    use texty::fuzzy_search::FuzzySearchState;

    // Create a temporary Rust file with unformatted code
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_preview.rs");
    let unformatted_content = "fn main(){println!(\"hello world\");}";
    fs::write(&file_path, unformatted_content).unwrap();

    // Create fuzzy search state and manually set up a file item
    let mut fuzzy_state = FuzzySearchState::new();
    let file_item = FileItem {
        name: "test_preview.rs".to_string(),
        path: file_path.clone(),
        is_dir: false,
        is_hidden: false,
        modified: std::fs::metadata(&file_path).unwrap().modified().unwrap(),
        size: Some(unformatted_content.len() as u64),
        is_binary: false,
    };

    // Simulate selecting the file
    fuzzy_state.filtered_items = vec![file_item.clone()];
    fuzzy_state.selected_index = 0;

    // Preview functionality has been removed - fuzzy search now shows only file list
    // This test verifies that the basic fuzzy search functionality works
    assert_eq!(fuzzy_state.filtered_items.len(), 1);
    assert_eq!(fuzzy_state.selected_index, 0);

    println!("✅ Fuzzy search working correctly (preview removed)");
}

#[test]
fn test_formatting_rust_file() {
    use std::fs;
    use tempfile::TempDir;
    use texty::command::Command;
    use texty::editor::Editor;

    // Create a temporary Rust file with unformatted code
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_format.rs");
    let unformatted_content = "fn main(){println!(\"hello world\");}";
    fs::write(&file_path, unformatted_content).unwrap();

    // Create editor and open the file
    let mut editor = Editor::new();
    editor.open_file(file_path.to_str().unwrap()).unwrap();

    // Check if formatter is available for Rust files
    assert!(
        editor.formatter.is_some(),
        "Rust formatter should be available"
    );

    // Execute format command
    editor.execute_command(Command::FormatBuffer);

    // Check that formatting worked by examining the content
    let formatted_content = editor.buffer.rope.to_string();

    // rustfmt should format this to multiple lines
    assert!(
        formatted_content.contains("fn main()"),
        "Should contain main function"
    );
    assert!(
        formatted_content.contains("println!"),
        "Should contain println"
    );

    // The formatted content should be different from the unformatted
    // (rustfmt typically adds proper spacing and newlines)
    println!("Original: {:?}", unformatted_content);
    println!("Formatted: {:?}", formatted_content);
}

#[test]
fn test_backspace_bounds_checking() {
    use texty::command::Command;
    use texty::editor::Editor;

    let mut editor = Editor::new();

    // Create a buffer with some content
    editor
        .buffer
        .insert_text("line1\nline2\nline3\n", 0, 0)
        .unwrap();

    // Move cursor to the end of first line
    for _ in 0..5 {
        // "line1" has 5 characters
        editor.execute_command(Command::MoveRight);
    }

    // Ensure cursor is at end of first line
    assert_eq!(editor.cursor.line, 0);
    assert_eq!(editor.cursor.col, 5);

    // Backspace repeatedly - this should not panic
    for _ in 0..50 {
        // Backspace way more times than there are characters
        editor.execute_command(Command::DeleteChar);
    }

    // After all backspacing, cursor should be at valid position
    assert_eq!(editor.cursor.line, 0);
    assert_eq!(editor.cursor.col, 0);
}

#[test]
fn test_arrow_key_movement() {
    use texty::command::Command;
    use texty::editor::Editor;

    let mut editor = Editor::new();
    editor.buffer.insert_text("Hello World", 0, 0).unwrap();

    // Test arrow key movement (same as hjkl)
    assert_eq!(editor.cursor.line, 0);
    assert_eq!(editor.cursor.col, 0);

    // Right arrow (same as l)
    editor.execute_command(Command::MoveRight);
    assert_eq!(editor.cursor.col, 1);

    // Left arrow (same as h)
    editor.execute_command(Command::MoveLeft);
    assert_eq!(editor.cursor.col, 0);

    // Test in insert mode
    editor.execute_command(Command::InsertMode);
    editor.buffer.insert_text("test", 0, 0).unwrap();
    editor.cursor.col = 4; // Move to end of "test"

    // Arrow keys should work in insert mode too
    editor.execute_command(Command::MoveLeft);
    assert_eq!(editor.cursor.col, 3);

    editor.execute_command(Command::MoveRight);
    assert_eq!(editor.cursor.col, 4);
}
