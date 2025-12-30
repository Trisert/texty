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
use crate::motion::Position;
use crate::registers::Registers;
use crate::syntax::{LanguageId, LanguageRegistry, load_languages_config};
use crate::ui::widgets::completion::CompletionPopup;
use crate::vim_parser::VimParser;
use crate::viewport::Viewport;
use lsp_types::{Diagnostic, Url};
use std::collections::HashMap;
use std::path::PathBuf;
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
    pub command_line: String,           // Current command line input
    pub command_history: Vec<String>,   // Command history
    pub status_message: Option<String>, // Temporary status message
    pub command_history_index: usize,   // Current position in history
    // Vim-specific state
    pub vim_parser: VimParser,
    pub registers: Registers,
    pub visual_start: Option<Position>,
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
            status_message: None,
            vim_parser: VimParser::new(),
            registers: Registers::new(),
            visual_start: None,
        }
    }

    pub fn execute_command(&mut self, cmd: Command) -> bool {
        // Clear status message on new commands (except for commands that just show status)
        if !matches!(cmd, Command::FormatBuffer) {
            self.status_message = None;
        }

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
                if self.cursor.line < self.buffer.line_count().saturating_sub(1) {
                    self.cursor.line += 1;
                }
            }
            Command::InsertChar(c) => {
                if self.mode == Mode::Insert {
                    let _ = self
                        .buffer
                        .insert_char(c, self.cursor.line, self.cursor.col);

                    // Handle cursor positioning based on character type
                    if c == '\n' {
                        // Move to beginning of next line after newline
                        self.cursor.line += 1;
                        self.cursor.col = 0;
                    } else {
                        // Normal character: advance column
                        self.cursor.col += 1;
                    }

                    self.notify_text_change();
                } else if (self.mode == Mode::Normal || self.mode == Mode::FuzzySearch)
                    && self.fuzzy_search.is_some()
                {
                    // Handle typing in fuzzy search
                    if let Some(fuzzy) = &mut self.fuzzy_search {
                        let mut new_query = fuzzy.query.clone();
                        new_query.push(c);
                        fuzzy.update_query(new_query);
                    }
                }
            }
            Command::DeleteChar => {
                if self.mode == Mode::Insert {
                    if self.cursor.col > 0 {
                        // Normal backspace: delete previous character in current line
                        let _ = self
                            .buffer
                            .delete_char(self.cursor.line, self.cursor.col - 1);
                        self.cursor.col -= 1;
                    } else if self.cursor.col == 0 && self.cursor.line > 0 {
                        // Backspace at line start: delete newline and join with previous line
                        let prev_line_len = self.buffer.line_len(self.cursor.line - 1);
                        let _ = self.buffer.delete_char(self.cursor.line, 0);
                        self.cursor.line -= 1;
                        self.cursor.col = prev_line_len;
                    }
                    // If at (0, 0), do nothing (already at beginning of file)
                } else if self.mode == Mode::Normal {
                    if self.fuzzy_search.is_some() {
                        // Handle backspace in fuzzy search
                        if let Some(fuzzy) = &mut self.fuzzy_search {
                            let mut new_query = fuzzy.query.clone();
                            new_query.pop();
                            fuzzy.update_query(new_query);
                        }
                    } else {
                        // Backspace in normal mode: delete previous character
                        if self.cursor.col > 0 {
                            let _ = self
                                .buffer
                                .delete_char(self.cursor.line, self.cursor.col - 1);
                            self.cursor.col -= 1;
                        } else if self.cursor.col == 0 && self.cursor.line > 0 {
                            // Backspace at line start in normal mode
                            let prev_line_len = self.buffer.line_len(self.cursor.line - 1);
                            let _ = self.buffer.delete_char(self.cursor.line, 0);
                            self.cursor.line -= 1;
                            self.cursor.col = prev_line_len;
                        }
                    }
                } else if self.mode == Mode::FuzzySearch && self.fuzzy_search.is_some() {
                    // Handle backspace in fuzzy search mode
                    if let Some(fuzzy) = &mut self.fuzzy_search {
                        fuzzy.query.pop();
                        fuzzy.update_filter();
                    }
                }
            }
            Command::OpenFuzzySearch => {
                self.open_fuzzy_search();
            }
            Command::FuzzySearchUp => {
                if let Some(fuzzy) = &mut self.fuzzy_search
                    && let Some(item) = fuzzy.select_prev()
                {
                    // Auto-open file when navigating with arrow keys
                    if !item.is_dir {
                        self.open_file(&item.path.to_string_lossy()).ok();
                    }
                }
            }
            Command::FuzzySearchDown => {
                if let Some(fuzzy) = &mut self.fuzzy_search
                    && let Some(item) = fuzzy.select_next()
                {
                    // Auto-open file when navigating with arrow keys
                    if !item.is_dir {
                        self.open_file(&item.path.to_string_lossy()).ok();
                    }
                }
            }
            Command::FuzzySearchSelect => {
                // Extract selected item info first to avoid borrow conflicts
                let selected_item = self
                    .fuzzy_search
                    .as_ref()
                    .and_then(|f| f.get_selected_item())
                    .cloned();

                if let Some(item) = selected_item {
                    if item.is_dir {
                        // Navigate to directory
                        if let Some(fuzzy) = &mut self.fuzzy_search {
                            fuzzy.navigate_to_directory(item.path);
                        }
                    } else {
                        // Open file full-screen and close fuzzy search (Enter key behavior)
                        self.open_file(&item.path.to_string_lossy()).ok();
                        self.fuzzy_search = None; // Close fuzzy search
                        self.mode = Mode::Normal; // Return to normal mode
                    }
                }
            }
            Command::FuzzySearchCancel => {
                self.fuzzy_search = None;
                self.mode = Mode::Normal;
            }
            Command::FuzzySearchToggleRecursive => {
                if let Some(fuzzy) = &mut self.fuzzy_search {
                    fuzzy.toggle_recursive();
                    let mode_text = if fuzzy.recursive_search {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    self.status_message = Some(format!("Recursive search {}", mode_text));
                }
            }
            Command::FuzzySearchToggleGitignore => {
                if let Some(fuzzy) = &mut self.fuzzy_search {
                    fuzzy.toggle_gitignore();
                    let mode_text = if fuzzy.follow_gitignore {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    self.status_message = Some(format!("Gitignore filtering {}", mode_text));
                }
            }
            Command::FuzzySearchLoadMore => {
                if let Some(fuzzy) = &mut self.fuzzy_search {
                    fuzzy.load_more_results();
                }
            }
            Command::InsertMode => self.mode = Mode::Insert,
            Command::NormalMode => self.mode = Mode::Normal,

            Command::FormatBuffer => {
                if let Some(formatter) = &self.formatter {
                    let cursor_line = self.cursor.line;
                    let cursor_col = self.cursor.col;
                    match self
                        .buffer
                        .format_buffer(formatter, cursor_line, cursor_col)
                    {
                        Ok((new_line, new_col)) => {
                            self.cursor.line = new_line;
                            self.cursor.col = new_col;
                            self.status_message = Some("Formatted".to_string());
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Format failed: {}", e));
                        }
                    }
                } else {
                    self.status_message =
                        Some("No formatter available for this file type".to_string());
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

            // ===== Vim-style motion commands =====
            Command::MoveWordForward(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut new_pos = pos;
                for _ in 0..count {
                    new_pos = motion::word_forward(&self.buffer, new_pos);
                    // Clamp to buffer bounds
                    new_pos.line = new_pos.line.min(self.buffer.line_count().saturating_sub(1));
                    new_pos.col = new_pos.col.min(self.buffer.line_len(new_pos.line));
                }
                self.cursor.line = new_pos.line;
                self.cursor.col = new_pos.col;
            }
            Command::MoveWordBackward(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut new_pos = pos;
                for _ in 0..count {
                    new_pos = motion::word_backward(&self.buffer, new_pos);
                }
                self.cursor.line = new_pos.line;
                self.cursor.col = new_pos.col;
            }
            Command::MoveWordEnd(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut new_pos = pos;
                for _ in 0..count {
                    new_pos = motion::word_end(&self.buffer, new_pos);
                    new_pos.line = new_pos.line.min(self.buffer.line_count().saturating_sub(1));
                    new_pos.col = new_pos.col.min(self.buffer.line_len(new_pos.line).saturating_sub(1));
                }
                self.cursor.line = new_pos.line;
                self.cursor.col = new_pos.col;
            }
            Command::MoveLineStart => {
                self.cursor.col = 0;
            }
            Command::MoveLineEnd(count) => {
                let target_line = (self.cursor.line + count - 1).min(self.buffer.line_count().saturating_sub(1));
                self.cursor.line = target_line;
                self.cursor.col = self.buffer.line_len(target_line).saturating_sub(1);
            }
            Command::MoveFirstNonBlank => {
                use crate::motion;
                let pos = motion::Position::new(self.cursor.line, self.cursor.col);
                let new_pos = motion::first_non_blank(&self.buffer, pos);
                self.cursor.col = new_pos.col;
            }
            Command::MoveFileStart => {
                self.cursor.line = 0;
                self.cursor.col = 0;
            }
            Command::MoveFileEnd => {
                self.cursor.line = self.buffer.line_count().saturating_sub(1);
                self.cursor.col = 0;
            }
            Command::MoveScreenTop => {
                self.cursor.line = self.viewport.offset_line;
            }
            Command::MoveScreenMiddle => {
                self.cursor.line = self.viewport.offset_line + self.viewport.rows / 2;
            }
            Command::MoveScreenBottom => {
                self.cursor.line = (self.viewport.offset_line + self.viewport.rows).min(self.buffer.line_count().saturating_sub(1));
            }

            // ===== Vim-style delete commands =====
            Command::DeleteCharForward(count) => {
                if let Ok(_deleted) = self.buffer.delete_char_forward(self.cursor.line, self.cursor.col, count) {
                    self.notify_text_change();
                }
            }
            Command::ReplaceChar(ch) => {
                if self.buffer.replace_char(self.cursor.line, self.cursor.col, ch).is_ok() {
                    self.notify_text_change();
                }
            }
            Command::DeleteLine => {
                if let Ok(_deleted) = self.buffer.delete_line(self.cursor.line) {
                    // Yank to registers
                    // TODO: self.registers.yank(deleted, '"');
                    self.notify_text_change();
                }
            }
            Command::DeleteWord(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                if let Ok(_deleted) = self.buffer.delete_range(pos, end_pos) {
                    self.notify_text_change();
                }
            }
            Command::DeleteToEnd => {
                use crate::motion::{self, Position};
                let start = Position::new(self.cursor.line, self.cursor.col);
                let end = motion::line_end(&self.buffer, start);
                if let Ok(_deleted) = self.buffer.delete_range(start, end) {
                    self.notify_text_change();
                }
            }
            Command::DeleteToStart => {
                use crate::motion::Position;
                let start = Position::new(self.cursor.line, 0);
                let end = Position::new(self.cursor.line, self.cursor.col);
                if let Ok(_deleted) = self.buffer.delete_range(start, end) {
                    self.cursor.col = 0;
                    self.notify_text_change();
                }
            }
            Command::JoinLines(count) => {
                for _ in 0..count {
                    if self.buffer.join_lines(self.cursor.line).is_ok() {
                        self.notify_text_change();
                    }
                }
            }

            // ===== Yank commands =====
            Command::YankLine => {
                let text = self.buffer.get_line_content(self.cursor.line);
                // TODO: self.registers.yank(text, '"');
                self.status_message = Some(format!("Yanked line ({} chars)", text.len()));
            }
            Command::YankToEnd => {
                use crate::motion::{self, Position};
                let start = Position::new(self.cursor.line, self.cursor.col);
                let end = motion::line_end(&self.buffer, start);
                let text = self.buffer.get_range(start, end);
                // TODO: self.registers.yank(text, '"');
                self.status_message = Some(format!("Yanked to end ({} chars)", text.len()));
            }

            // ===== Paste commands =====
            Command::PasteAfter => {
                // TODO: if let Some(text) = self.registers.get('"') {
                //     let text = text.to_string();
                //     if self.buffer.insert_text(&text, self.cursor.line, self.cursor.col + 1).is_ok() {
                //         self.cursor.col += text.len();
                //         self.notify_text_change();
                //     }
                // }
            }
            Command::PasteBefore => {
                // TODO: if let Some(text) = self.registers.get('"') {
                //     let text = text.to_string();
                //     if self.buffer.insert_text(&text, self.cursor.line, self.cursor.col).is_ok() {
                //         self.cursor.col += text.len();
                //         self.notify_text_change();
                //     }
                // }
            }

            // ===== Change commands =====
            Command::SubstituteChar => {
                if self.buffer.delete_char(self.cursor.line, self.cursor.col).is_ok() {
                    self.mode = Mode::Insert;
                }
            }
            Command::SubstituteLine => {
                self.cursor.col = 0;
                if self.buffer.delete_range(
                    crate::motion::Position::new(self.cursor.line, 0),
                    crate::motion::Position::new(self.cursor.line, self.buffer.line_len(self.cursor.line)),
                ).is_ok() {
                    self.mode = Mode::Insert;
                    self.notify_text_change();
                }
            }
            Command::ChangeLine => {
                self.cursor.col = 0;
                if self.buffer.delete_line(self.cursor.line).is_ok() {
                    self.mode = Mode::Insert;
                    self.notify_text_change();
                }
            }
            Command::ChangeWord(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                if let Ok(_deleted) = self.buffer.delete_range(pos, end_pos) {
                    self.mode = Mode::Insert;
                    self.notify_text_change();
                }
            }
            Command::ChangeToEnd => {
                use crate::motion::{self, Position};
                let start = Position::new(self.cursor.line, self.cursor.col);
                let end = motion::line_end(&self.buffer, start);
                if let Ok(_deleted) = self.buffer.delete_range(start, end) {
                    self.mode = Mode::Insert;
                    self.notify_text_change();
                }
            }

            // ===== Visual mode =====
            Command::VisualChar => {
                self.mode = Mode::Visual;
                self.status_message = Some("-- VISUAL --".to_string());
            }
            Command::VisualLine => {
                self.mode = Mode::Visual;
                self.status_message = Some("-- VISUAL LINE --".to_string());
            }

            // ===== Undo/Redo =====
            Command::Undo => {
                // TODO: implement undo
                self.status_message = Some("Undo not yet implemented".to_string());
            }
            Command::Redo => {
                // TODO: implement redo
                self.status_message = Some("Redo not yet implemented".to_string());
            }

            Command::DeleteToStartWord(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut start_pos = pos;
                for _ in 0..count {
                    start_pos = motion::word_backward(&self.buffer, start_pos);
                }
                if let Ok(_deleted) = self.buffer.delete_range(start_pos, pos) {
                    self.cursor.line = start_pos.line;
                    self.cursor.col = start_pos.col;
                    self.notify_text_change();
                }
            }
            Command::DeleteToEndWord(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_end(&self.buffer, end_pos);
                }
                if let Ok(_deleted) = self.buffer.delete_range(pos, end_pos) {
                    self.notify_text_change();
                }
            }
            Command::DeleteInnerWord(count) => {
                // Inner word - similar to DeleteWord but more precise
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                // Remove trailing whitespace
                while end_pos.col > 0 && self.cursor.line == end_pos.line {
                    let line = self.buffer.line(end_pos.line).unwrap_or_default();
                    let chars: Vec<char> = line.chars().collect();
                    if end_pos.col < chars.len() && chars[end_pos.col].is_whitespace() {
                        end_pos.col -= 1;
                    } else {
                        break;
                    }
                }
                if let Ok(_deleted) = self.buffer.delete_range(pos, end_pos) {
                    self.notify_text_change();
                }
            }
            Command::DeleteAWord(count) => {
                // A word - includes trailing space (same as DeleteWord for now)
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                if let Ok(_deleted) = self.buffer.delete_range(pos, end_pos) {
                    self.notify_text_change();
                }
            }
            Command::DeleteToEndOfFile => {
                use crate::motion::Position;
                let start = Position::new(self.cursor.line, self.cursor.col);
                let end = Position::new(self.buffer.line_count().saturating_sub(1), 0);
                if let Ok(_deleted) = self.buffer.delete_range(start, end) {
                    self.notify_text_change();
                }
            }
            Command::DeleteToStartOfFile => {
                use crate::motion::Position;
                let start = Position::new(0, 0);
                let end = Position::new(self.cursor.line, self.cursor.col);
                if let Ok(_deleted) = self.buffer.delete_range(start, end) {
                    self.cursor.line = 0;
                    self.cursor.col = 0;
                    self.notify_text_change();
                }
            }
            Command::DeleteLineIntoRegister(_reg) => {
                if let Ok(_deleted) = self.buffer.delete_line(self.cursor.line) {
                    // TODO: self.registers.yank(deleted, reg);
                    self.notify_text_change();
                }
            }
            Command::YankWord(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                let text = self.buffer.get_range(pos, end_pos);
                // TODO: self.registers.yank(text, '"');
                self.status_message = Some(format!("Yanked word ({} chars)", text.len()));
            }
            Command::YankToStart => {
                use crate::motion::Position;
                let start = Position::new(self.cursor.line, 0);
                let end = Position::new(self.cursor.line, self.cursor.col);
                let text = self.buffer.get_range(start, end);
                // TODO: self.registers.yank(text, '"');
                self.status_message = Some(format!("Yanked to start ({} chars)", text.len()));
            }
            Command::YankInnerWord(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                // Remove trailing whitespace
                while end_pos.col > 0 && self.cursor.line == end_pos.line {
                    let line = self.buffer.line(end_pos.line).unwrap_or_default();
                    let chars: Vec<char> = line.chars().collect();
                    if end_pos.col < chars.len() && chars[end_pos.col].is_whitespace() {
                        end_pos.col -= 1;
                    } else {
                        break;
                    }
                }
                let text = self.buffer.get_range(pos, end_pos);
                // TODO: self.registers.yank(text, '"');
                self.status_message = Some(format!("Yanked inner word ({} chars)", text.len()));
            }
            Command::YankAWord(count) => {
                // Same as YankWord for now
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                let text = self.buffer.get_range(pos, end_pos);
                // TODO: self.registers.yank(text, '"');
                self.status_message = Some(format!("Yanked word ({} chars)", text.len()));
            }
            Command::ChangeToStart => {
                use crate::motion::Position;
                let start = Position::new(self.cursor.line, 0);
                let end = Position::new(self.cursor.line, self.cursor.col);
                if let Ok(_deleted) = self.buffer.delete_range(start, end) {
                    self.cursor.col = 0;
                    self.mode = Mode::Insert;
                    self.notify_text_change();
                }
            }
            Command::ChangeInnerWord(count) => {
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                if let Ok(_deleted) = self.buffer.delete_range(pos, end_pos) {
                    self.mode = Mode::Insert;
                    self.notify_text_change();
                }
            }
            Command::ChangeAWord(count) => {
                // Same as ChangeWord for now
                use crate::motion::{self, Position};
                let pos = Position::new(self.cursor.line, self.cursor.col);
                let mut end_pos = pos;
                for _ in 0..count {
                    end_pos = motion::word_forward(&self.buffer, end_pos);
                }
                if let Ok(_deleted) = self.buffer.delete_range(pos, end_pos) {
                    self.mode = Mode::Insert;
                    self.notify_text_change();
                }
            }
            Command::IndentLine(count) => {
                if self.buffer.indent_range(self.cursor.line, self.cursor.line + count - 1, 4).is_ok() {
                    self.notify_text_change();
                }
            }
            Command::UnindentLine(count) => {
                if self.buffer.unindent_range(self.cursor.line, self.cursor.line + count - 1, 4).is_ok() {
                    self.notify_text_change();
                }
            }

            _ => {
                // Unknown command
                self.status_message = Some(format!("Unknown command: {:?}", cmd));
            }
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

        // Reset viewport and cursor to ensure clean rendering state
        self.viewport.offset_line = 0;
        self.viewport.offset_col = 0;
        self.cursor.line = 0;
        self.cursor.col = 0;

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
        fuzzy_state.rescan_current_directory();

        self.fuzzy_search = Some(fuzzy_state);
        self.mode = Mode::FuzzySearch;
    }

    /// Start fuzzy search in a specific directory
    pub fn start_fuzzy_search_in_dir(&mut self, dir_path: &std::path::Path) {
        let mut fuzzy_state = FuzzySearchState::new_in_directory(dir_path);

        // Scan directory and populate items
        fuzzy_state.rescan_current_directory();

        self.fuzzy_search = Some(fuzzy_state);
        self.mode = Mode::FuzzySearch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_down_in_empty_buffer() {
        let mut editor = Editor::new();
        // Clear buffer to make it truly empty
        editor.buffer.rope = ropey::Rope::from("");

        // Try to move down - should not crash
        editor.execute_command(Command::MoveDown);
        assert_eq!(editor.cursor.line, 0);
        assert_eq!(editor.cursor.col, 0);
    }

    #[test]
    fn test_move_up_from_top() {
        let mut editor = Editor::new();
        editor.execute_command(Command::MoveUp);
        assert_eq!(editor.cursor.line, 0);
    }

    #[test]
    fn test_move_left_from_start() {
        let mut editor = Editor::new();
        editor.execute_command(Command::MoveLeft);
        assert_eq!(editor.cursor.col, 0);
    }

    #[test]
    fn test_move_right_at_end_of_line() {
        let mut editor = Editor::new();
        editor.buffer.insert_char('a', 0, 0).unwrap();
        editor.buffer.insert_char('b', 0, 1).unwrap();
        editor.cursor.col = 2; // Move to end

        editor.execute_command(Command::MoveRight);
        assert_eq!(editor.cursor.col, 2);
    }

    #[test]
    fn test_move_down_to_last_line() {
        let mut editor = Editor::new();
        editor.buffer.insert_char('a', 0, 0).unwrap();
        editor.buffer.insert_char('\n', 0, 1).unwrap();
        editor.buffer.insert_char('b', 1, 0).unwrap();

        editor.cursor.line = 0;
        editor.execute_command(Command::MoveDown);
        assert_eq!(editor.cursor.line, 1);
    }

    #[test]
    fn test_delete_char_from_start() {
        let mut editor = Editor::new();
        editor.execute_command(Command::DeleteChar);
        // Should not crash when deleting at col 0
        assert_eq!(editor.cursor.col, 0);
    }

    #[test]
    fn test_insert_mode_movement() {
        let mut editor = Editor::new();
        editor.mode = Mode::Insert;
        editor.buffer.insert_char('a', 0, 0).unwrap();
        editor.buffer.insert_char('b', 0, 1).unwrap();

        editor.cursor.col = 0;
        editor.execute_command(Command::MoveRight);
        assert_eq!(editor.cursor.col, 1);

        editor.execute_command(Command::MoveLeft);
        assert_eq!(editor.cursor.col, 0);
    }

    #[test]
    fn test_viewport_scrolling_on_move() {
        let mut editor = Editor::new();
        // Add many lines
        for i in 0..30 {
            if i > 0 {
                editor.buffer.insert_char('\n', 0, 0).unwrap();
            }
            editor.buffer.insert_char('a', 0, 0).unwrap();
        }

        // Set viewport smaller than content
        editor.viewport.rows = 10;
        editor.viewport.cols = 80;

        // Move cursor to line 25
        editor.cursor.line = 25;
        editor
            .viewport
            .scroll_to_cursor(editor.cursor.line, editor.cursor.col);

        // Offset should be adjusted to keep cursor visible
        assert!(editor.cursor.line >= editor.viewport.offset_line);
        assert!(editor.cursor.line < editor.viewport.offset_line + editor.viewport.rows);
    }

    #[test]
    fn test_cursor_col_adjustment_after_text_change() {
        let mut editor = Editor::new();
        editor.buffer.insert_char('a', 0, 0).unwrap();
        editor.buffer.insert_char('b', 0, 1).unwrap();
        editor.buffer.insert_char('c', 0, 2).unwrap();

        editor.cursor.col = 3; // Beyond line length

        // Text change should clamp cursor
        editor.notify_text_change();
        let line_len = editor.buffer.line_len(0);
        assert!(editor.cursor.col <= line_len);
    }

    #[test]
    fn test_multiple_rapid_movements() {
        let mut editor = Editor::new();
        editor.buffer.insert_char('a', 0, 0).unwrap();
        editor.buffer.insert_char('b', 0, 1).unwrap();
        editor.buffer.insert_char('c', 0, 2).unwrap();
        editor.buffer.insert_char('d', 0, 3).unwrap();

        // Rapid movements should not overflow
        for _ in 0..100 {
            editor.execute_command(Command::MoveRight);
        }
        assert_eq!(editor.cursor.col, 4); // Should be clamped to line length

        for _ in 0..100 {
            editor.execute_command(Command::MoveLeft);
        }
        assert_eq!(editor.cursor.col, 0); // Should be clamped to 0
    }

    #[test]
    fn test_multiline_navigation() {
        let mut editor = Editor::new();
        // Create 20 lines
        for i in 0..20 {
            if i > 0 {
                editor.buffer.insert_char('\n', 0, 0).unwrap();
            }
            editor.buffer.insert_char('a', 0, 0).unwrap();
        }

        editor.cursor.line = 0;

        // Move down through all lines
        for i in 0..19 {
            editor.execute_command(Command::MoveDown);
            assert_eq!(editor.cursor.line, i + 1);
        }

        // Try to move down past end
        editor.execute_command(Command::MoveDown);
        assert_eq!(editor.cursor.line, 19);

        // Move back up
        for i in (1..=19).rev() {
            editor.execute_command(Command::MoveUp);
            assert_eq!(editor.cursor.line, i - 1);
        }
    }
}
