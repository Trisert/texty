// src/editor.rs - Core editor coordinator

use crate::buffer::Buffer;
use crate::command::Command;
use crate::cursor::Cursor;
use crate::formatter::external::{Formatter, get_formatter_config};
use crate::lsp::completion::CompletionManager;
use crate::lsp::diagnostics::DiagnosticManager;
use crate::lsp::manager::LspManager;
use crate::mode::Mode;
use crate::syntax::LanguageId;
use crate::viewport::Viewport;
use lsp_types::{Position, Url};

pub struct Editor {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub mode: Mode,
    pub viewport: Viewport,
    pub formatter: Option<Formatter>,
    pub lsp_manager: LspManager,
    pub completion_manager: CompletionManager,
    pub diagnostic_manager: DiagnosticManager,
    pub current_language: Option<LanguageId>,
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

        Self {
            buffer,
            cursor: Cursor::new(),
            mode: Mode::Normal,
            viewport: Viewport::new(20, 80),
            formatter,
            lsp_manager: LspManager::new(),
            completion_manager: CompletionManager::new(),
            diagnostic_manager: DiagnosticManager::new(),
            current_language: Some(LanguageId::Rust), // Default to Rust for now
        }
    }

    pub fn execute_command(&mut self, cmd: Command) {
        match cmd {
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
                if c == '\n' {
                    self.buffer
                        .insert_char(c, self.cursor.line, self.cursor.col)
                        .ok();
                    self.cursor.line += 1;
                    self.cursor.col = 0;
                } else {
                    self.buffer
                        .insert_char(c, self.cursor.line, self.cursor.col)
                        .ok();
                    self.cursor.col += 1;
                }
                self.notify_text_change();
            }
            Command::DeleteChar => {
                self.buffer
                    .delete_char(self.cursor.line, self.cursor.col)
                    .ok();
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                }
                self.notify_text_change();
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
                if let Some(language) = self.current_language {
                    let uri = self.get_buffer_uri();
                    if let (Some(uri), Ok(client)) =
                        (uri, self.lsp_manager.get_or_start_client(language))
                    {
                        let position = Position {
                            line: self.cursor.line as u32,
                            character: self.cursor.col as u32,
                        };
                        if let Ok(()) = self
                            .completion_manager
                            .request_completion(client, &uri, position)
                        {
                            // Completion items are now available in completion_manager
                            // UI would need to display them
                        }
                    }
                }
            }
            Command::GotoDefinition => {
                // TODO: Implement LSP goto definition
                eprintln!("LSP goto definition not implemented yet");
            }
            Command::SaveFile => {
                let path = self.buffer.file_path.as_ref().cloned();
                if let Some(path) = path
                    && self.buffer.save_to_file(&path).is_ok() {
                        // Notify LSP server
                        if let Some(language) = self.current_language {
                            let uri = self.get_buffer_uri();
                            if let (Some(uri), Ok(client)) =
                                (uri, self.lsp_manager.get_or_start_client(language))
                            {
                                client
                                    .text_document_did_save(
                                        &uri,
                                        Some(&self.buffer.rope.to_string()),
                                    )
                                    .ok();
                            }
                        }
                    }
            }
            _ => {}
        }
        // Update desired_col
        self.cursor.desired_col = self.cursor.col;
        // Scroll to keep cursor visible
        self.viewport
            .scroll_to_cursor(self.cursor.line, self.cursor.col);
    }

    pub fn handle_resize(&mut self, rows: u16, cols: u16) {
        self.viewport.rows = rows as usize - 1; // Leave room for status bar
        self.viewport.cols = cols as usize;
    }

    fn get_buffer_uri(&self) -> Option<Url> {
        self.buffer
            .file_path
            .as_ref()
            .and_then(|path| Url::from_file_path(path).ok())
    }

    pub fn open_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer.load_from_file(path)?;
        self.buffer.file_path = Some(path.to_string());

        // Update language based on file extension
        self.current_language = crate::syntax::language::get_language_config_by_extension(
            std::path::Path::new(path)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or(""),
        )
        .map(|config| config.id);

        // Notify LSP server
        if let Some(language) = self.current_language {
            let uri = self.get_buffer_uri();
            if let (Some(uri), Ok(client)) = (uri, self.lsp_manager.get_or_start_client(language)) {
                let language_id = match language {
                    LanguageId::Rust => "rust",
                    LanguageId::Python => "python",
                    LanguageId::JavaScript => "javascript",
                    LanguageId::TypeScript => "typescript",
                };
                client.text_document_did_open(
                    &uri,
                    language_id,
                    1,
                    &self.buffer.rope.to_string(),
                )?;
            }
        }

        Ok(())
    }

    pub fn notify_text_change(&mut self) {
        if let Some(language) = self.current_language {
            let uri = self.get_buffer_uri();
            if let (Some(uri), Ok(client)) = (uri, self.lsp_manager.get_or_start_client(language)) {
                let changes = vec![lsp_types::TextDocumentContentChangeEvent {
                    range: None, // Full content change
                    range_length: None,
                    text: self.buffer.rope.to_string(),
                }];
                client.text_document_did_change(&uri, 1, changes).ok(); // Ignore errors for now
            }
        }
    }
}
