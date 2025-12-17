// src/editor.rs - Core editor coordinator

use crate::buffer::Buffer;
use crate::command::Command;
use crate::cursor::Cursor;
use crate::formatter::external::{Formatter, get_formatter_config};
use crate::fuzzy_search::FuzzySearchState;
use crate::lsp::completion::CompletionManager;
use crate::lsp::diagnostics::DiagnosticManager;
use crate::lsp::manager::LspManager;
use crate::lsp::progress::ProgressManager;
use crate::mode::Mode;
use crate::syntax::{LanguageId, LanguageRegistry, load_languages_config};
use crate::ui::widgets::completion::CompletionPopup;
use crate::viewport::Viewport;
use std::path::PathBuf;
use lsp_types::{Diagnostic, Url};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Editor {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub mode: Mode,
    pub viewport: Viewport,
    pub formatter: Option<Formatter>,
    pub lsp_manager: LspManager,
    pub completion_manager: CompletionManager,
    pub diagnostic_manager: DiagnosticManager,
    pub diagnostics: Arc<Mutex<HashMap<Url, Vec<Diagnostic>>>>, // Synchronous access for UI
    pub completion_popup: CompletionPopup,
    pub progress_items: Arc<Mutex<Vec<crate::lsp::progress::ProgressItem>>>, // Synchronous access for UI
    pub progress_manager: Arc<ProgressManager>,
    pub current_language: Option<LanguageId>,
    pub language_registry: LanguageRegistry,
    // Fuzzy search
    pub fuzzy_search: Option<FuzzySearchState>,
    // UI overlays
    pub hover_content: Option<Vec<String>>, // Content for hover window
    pub code_actions: Option<Vec<lsp_types::CodeAction>>, // Available code actions
    pub code_action_selected: usize,        // Selected code action index
    // Command line
    pub command_line: String,         // Current command line input
    pub command_history: Vec<String>, // Command history
    pub command_history_index: usize, // Current position in history
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        let mut buffer = Buffer::new();
        buffer.file_path = Some("buffer.txt".to_string());
        let formatter =
            get_formatter_config(LanguageId::Rust).and_then(|config| Formatter::new(config).ok());

        let language_registry = load_languages_config()
            .map(LanguageRegistry::new)
            .unwrap_or_else(|_| {
                // Fallback to empty registry if config fails to load
                LanguageRegistry::new(crate::syntax::config::LanguagesConfig { language: vec![] })
            });

        Self {
            buffer,
            cursor: Cursor::new(),
            mode: Mode::Normal,
            viewport: Viewport::new(20, 80),
            formatter,
            lsp_manager: LspManager::new(),
            completion_manager: CompletionManager::new(),
            diagnostic_manager: DiagnosticManager::new(),
            diagnostics: Arc::new(Mutex::new(HashMap::new())),
            completion_popup: CompletionPopup::new(),
            progress_items: Arc::new(Mutex::new(Vec::new())),
            progress_manager: Arc::new(ProgressManager::new()),
            current_language: Some(LanguageId::Rust), // Default to Rust for now
            language_registry,
            fuzzy_search: None,
            hover_content: None,
            code_actions: None,
            code_action_selected: 0,
            command_line: String::new(),
            command_history: Vec::new(),
            command_history_index: 0,
        }
    }

    pub fn execute_command(&mut self, cmd: Command) -> bool {
        // Returns true if should quit
        match cmd {
            Command::Quit => return true, // Signal to quit
            Command::MoveLeft => {
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                }
            }
            Command::MoveRight => {
                let line_len = self.buffer.line_len(self.cursor.line);
                if self.cursor.col < line_len {
                    self.cursor.col += 1;
                }
            }
            Command::MoveUp => {
                if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                }
            }
            Command::MoveDown => {
                if self.cursor.line < self.buffer.line_count() - 1 {
                    self.cursor.line += 1;
                }
            }
            Command::InsertChar(c) => {
                if self.mode == Mode::Insert {
                    let _ = self.buffer.insert_char(c, self.cursor.line, self.cursor.col);
                    self.cursor.col += 1;
                    self.notify_text_change();
                } else if self.mode == Mode::Normal && self.fuzzy_search.is_some() {
                    // Handle typing in fuzzy search
                    if let Some(fuzzy) = &mut self.fuzzy_search {
                        fuzzy.query.push(c);
                        fuzzy.update_filter();
                    }
                }
            }
            Command::DeleteChar => {
                if self.mode == Mode::Insert {
                    if self.cursor.col > 0 {
                        let _ = self.buffer.delete_char(self.cursor.line, self.cursor.col - 1);
                        self.cursor.col -= 1;
                    }
                } else if self.mode == Mode::Normal {
                    if self.fuzzy_search.is_some() {
                        // Handle backspace in fuzzy search
                        if let Some(fuzzy) = &mut self.fuzzy_search {
                            fuzzy.query.pop();
                            fuzzy.update_filter();
                        }
                    } else {
                        // Backspace in normal mode: delete previous character
                        if self.cursor.col > 0 {
                            let _ = self.buffer.delete_char(self.cursor.line, self.cursor.col - 1);
                            self.cursor.col -= 1;
                        }
                    }
                }
            }
            Command::OpenFuzzySearch => {
                self.open_fuzzy_search();
            }
            Command::FuzzySearchUp => {
                if let Some(fuzzy) = &mut self.fuzzy_search {
                    fuzzy.select_prev();
                }
            }
            Command::FuzzySearchDown => {
                if let Some(fuzzy) = &mut self.fuzzy_search {
                    fuzzy.select_next();
                }
            }
            Command::FuzzySearchSelect => {
                // Handle directory navigation first
                let mut should_navigate = false;
                let mut nav_path = None;
                if let Some(fuzzy) = &self.fuzzy_search {
                    if let Some(item) = fuzzy.get_selected_item() {
                        if item.is_dir {
                            should_navigate = true;
                            nav_path = Some(item.path.clone());
                        }
                    }
                }

                if should_navigate {
                    if let Some(fuzzy) = &mut self.fuzzy_search {
                        if let Some(path) = nav_path {
                            fuzzy.navigate_to_directory(path);
                        }
                    }
                } else {
                    // Handle file opening
                    let mut should_open = false;
                    let mut open_path = None;
                    if let Some(fuzzy) = &self.fuzzy_search {
                        if let Some(item) = fuzzy.get_selected_item() {
                            if !item.is_dir {
                                should_open = true;
                                open_path = Some(item.path.clone());
                            }
                        }
                    }

                    if should_open {
                        if let Some(path) = open_path {
                            self.open_file(&path.to_string_lossy()).ok();
                            self.fuzzy_search = None;
                            self.mode = Mode::Normal;
                        }
                    }
                }
            }
            Command::FuzzySearchCancel => {
                self.fuzzy_search = None;
                self.mode = Mode::Normal;
            }
            Command::InsertMode => self.mode = Mode::Insert,
            Command::NormalMode => self.mode = Mode::Normal,

            Command::FormatBuffer => {
                if let Some(formatter) = &self.formatter
                    && let Ok((new_line, new_col)) =
                        self.buffer
                            .format_buffer(formatter, self.cursor.line, self.cursor.col)
                {
                    self.cursor.line = new_line;
                    self.cursor.col = new_col;
                }
            }
            Command::Completion => {
                // TODO: Implement async completion with proper UI integration
                // For now, completion is a placeholder
            }
            Command::GotoDefinition => {
                // TODO: Implement LSP goto definition
                eprintln!("LSP goto definition not implemented yet");
            }
            Command::FindReferences => {
                // TODO: Implement LSP find references
                eprintln!("LSP find references not implemented yet");
            }
            Command::Hover => {
                // Toggle hover information
                if self.hover_content.is_some() {
                    self.hide_hover();
                } else {
                    // TODO: Request hover information from LSP server
                    // For now, show a placeholder
                    self.show_hover(vec![
                        "This is hover information".to_string(),
                        "It would show type information,".to_string(),
                        "documentation, etc.".to_string(),
                    ]);
                }
            }
            Command::WorkspaceSymbols => {
                // TODO: Implement LSP workspace symbols
                eprintln!("LSP workspace symbols not implemented yet");
            }
            Command::CodeAction => {
                // Toggle code actions menu
                if self.code_actions.is_some() {
                    self.hide_code_actions();
                } else {
                    // TODO: Request code actions from LSP server
                    // For now, show placeholder actions
                    self.show_code_actions(vec![
                        lsp_types::CodeAction {
                            title: "Add missing import".to_string(),
                            kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                            ..Default::default()
                        },
                        lsp_types::CodeAction {
                            title: "Extract function".to_string(),
                            kind: Some(lsp_types::CodeActionKind::REFACTOR_EXTRACT),
                            ..Default::default()
                        },
                        lsp_types::CodeAction {
                            title: "Sort imports".to_string(),
                            kind: Some(lsp_types::CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
                            ..Default::default()
                        },
                    ]);
                }
            }
            Command::CodeActionNext => {
                self.select_next_code_action();
            }
            Command::CodeActionPrev => {
                self.select_prev_code_action();
            }
            Command::CodeActionAccept => {
                if let Some(action) = self.get_selected_code_action() {
                    // TODO: Execute the selected code action
                    eprintln!("Executing code action: {}", action.title);
                    self.hide_code_actions();
                }
            }
            Command::EnterCommandMode => {
                self.enter_command_mode();
            }
            Command::SaveFile => {
                let path = self.buffer.file_path.as_ref().cloned();
                if let Some(path) = path
                    && self.buffer.save_to_file(&path).is_ok()
                {
                    // TODO: Notify LSP server about file save
                    // Async LSP operations need proper integration with sync UI
                }
            }

             _ => {}
         }
        // Update desired_col
        self.cursor.desired_col = self.cursor.col;
        // Scroll to keep cursor visible
        self.viewport
            .scroll_to_cursor(self.cursor.line, self.cursor.col);
        false // Don't quit by default
    }

    pub fn handle_resize(&mut self, rows: u16, cols: u16) {
        self.viewport.rows = rows as usize - 1; // Leave room for status bar (1 line)
        self.viewport.cols = cols as usize;
    }

    pub fn get_buffer_uri(&self) -> Option<Url> {
        self.buffer
            .file_path
            .as_ref()
            .and_then(|path| Url::from_file_path(path).ok())
    }

    pub fn open_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer.load_from_file(path)?;
        self.buffer.file_path = Some(path.to_string());

        // Update language based on file extension
        let language_config = crate::syntax::language::get_language_config_by_extension(
            std::path::Path::new(path)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or(""),
        );

        if let Some(config) = language_config {
            self.current_language = Some(config.id);

            // Initialize syntax highlighter for this language
            match crate::syntax::highlighter::SyntaxHighlighter::new(config) {
                Ok(highlighter) => {
                    self.buffer.highlighter = Some(highlighter);
                    // Parse the loaded content
                    let _ = self.buffer.update_highlighter();
                }
                Err(_) => {
                    // Syntax highlighting failed to initialize, continue without it
                    self.buffer.highlighter = None;
                }
            }
        } else {
            self.current_language = None;
            self.buffer.highlighter = None;
        }

        // TODO: Notify LSP server about file open
        // Async LSP operations need proper integration with sync UI

        Ok(())
    }

    pub fn notify_text_change(&mut self) {
        // Adjust viewport if it's out of bounds after buffer changes
        let max_line = self.buffer.line_count().saturating_sub(1);
        if self.viewport.offset_line > max_line {
            self.viewport.offset_line = max_line;
        }

        // Adjust cursor if it's out of bounds
        if self.cursor.line > max_line {
            self.cursor.line = max_line;
        }

        // Adjust cursor column if it's beyond the end of the current line
        if let Some(line) = self.buffer.line(self.cursor.line) {
            let line_len = line.len();
            if self.cursor.col > line_len {
                self.cursor.col = line_len;
            }
        }

        // Clear overlays when text changes
        self.hide_hover();
        self.hide_code_actions();

        // TODO: Implement async text change notifications
        // For now, LSP operations are handled asynchronously in command handlers
    }

    /// Show hover information at cursor position
    pub fn show_hover(&mut self, content: Vec<String>) {
        self.hover_content = Some(content);
    }

    /// Hide hover information
    pub fn hide_hover(&mut self) {
        self.hover_content = None;
    }

    /// Show code actions menu
    pub fn show_code_actions(&mut self, actions: Vec<lsp_types::CodeAction>) {
        self.code_actions = Some(actions);
        self.code_action_selected = 0;
    }

    /// Hide code actions menu
    pub fn hide_code_actions(&mut self) {
        self.code_actions = None;
    }

    /// Navigate code actions menu
    pub fn select_next_code_action(&mut self) {
        if let Some(actions) = &self.code_actions
            && !actions.is_empty()
        {
            self.code_action_selected = (self.code_action_selected + 1) % actions.len();
        }
    }

    pub fn select_prev_code_action(&mut self) {
        if let Some(actions) = &self.code_actions
            && !actions.is_empty()
        {
            self.code_action_selected = if self.code_action_selected == 0 {
                actions.len() - 1
            } else {
                self.code_action_selected - 1
            };
        }
    }

    /// Get selected code action
    pub fn get_selected_code_action(&self) -> Option<&lsp_types::CodeAction> {
        self.code_actions.as_ref()?.get(self.code_action_selected)
    }

    /// Enter command mode
    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_line.clear();
        self.command_history_index = self.command_history.len();
    }

    /// Handle command line input
    pub fn handle_command_input(&mut self, c: char) -> Result<bool, Box<dyn std::error::Error>> {
        // Returns true if should quit
        match c {
            '\n' | '\r' => {
                // Execute command
                let should_quit = self.execute_command_line()?;
                self.mode = Mode::Normal;
                self.command_line.clear();
                return Ok(should_quit);
            }
            '\x08' | '\x7f' => {
                // Backspace
                self.command_line.pop();
            }
            '\x1b' => {
                // Escape
                self.mode = Mode::Normal;
                self.command_line.clear();
            }
            c if c.is_ascii_graphic() || c == ' ' => {
                self.command_line.push(c);
            }
            _ => {}
        }
        Ok(false) // Don't quit for other inputs
    }

    /// Execute command line
    fn execute_command_line(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // Returns true if should quit
        // Add to history
        self.command_history.push(self.command_line.clone());
        self.command_history_index = self.command_history.len();

        let trimmed = self.command_line.trim();
        if trimmed.is_empty() {
            return Ok(false);
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        match parts[0] {
            "q" | "quit" => {
                // Quit
                Ok(true)
            }
            "x" | "wq" => {
                // Save and quit
                if let Some(path) = self.buffer.file_path.clone() {
                    self.buffer.save_to_file(&path)?;
                }
                Ok(true)
            }
            "w" | "write" => {
                // Save file
                if let Some(path) = self.buffer.file_path.clone() {
                    self.buffer.save_to_file(&path)?;
                } else if parts.len() > 1 {
                    // Save as new file
                    let filename = parts[1].to_string();
                    self.buffer.file_path = Some(filename.clone());
                    self.buffer.save_to_file(&filename)?;
                }
                Ok(false)
            }
            "e" | "edit" if parts.len() > 1 => {
                // Open/edit file
                let filename = parts[1].to_string();
                self.open_file(&filename)?;
                Ok(false)
            }
            "syntax" | "syn" => {
                // Toggle syntax highlighting
                if parts.len() > 1 {
                    match parts[1] {
                        "on" => {
                            // Enable syntax highlighting
                            if let Some(language_id) = self.current_language {
                                let config =
                                    crate::syntax::language::get_language_config(language_id);
                                if let Ok(highlighter) =
                                    crate::syntax::highlighter::SyntaxHighlighter::new(config)
                                {
                                    self.buffer.highlighter = Some(highlighter);
                                }
                            }
                        }
                        "off" => {
                            // Disable syntax highlighting
                            self.buffer.highlighter = None;
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            "lsp" => {
                // LSP commands
                if parts.len() > 1 {
                    match parts[1] {
                        "restart" => {
                            // TODO: Restart LSP servers
                        }
                        "stop" => {
                            // TODO: Stop LSP servers
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            _ => {
                // Unknown command - could show error message
                Ok(false)
            }
        }
    }

    /// Get command line display text
    pub fn get_command_line_display(&self) -> String {
        if self.mode == Mode::Command {
            format!(":{}", self.command_line)
        } else {
            String::new()
        }
    }

    fn open_fuzzy_search(&mut self) {
        let mut fuzzy_state = FuzzySearchState::new();
        fuzzy_state.current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Scan directory and populate items
        fuzzy_state.all_items = crate::fuzzy_search::scan_directory(&fuzzy_state.current_path);
        fuzzy_state.update_filter();

        self.fuzzy_search = Some(fuzzy_state);
        self.mode = Mode::FuzzySearch;
    }
}
