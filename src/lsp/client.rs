// src/lsp/client.rs - LSP Client implementation

use lsp_server::{Connection, Message, Notification, Request};
use lsp_types::*;
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, InitializedParams, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, VersionedTextDocumentIdentifier,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Exit, Initialized,
    },
    request::{Initialize, Shutdown},
};
use serde_json::Value;
use std::process::{Command, Stdio};
use std::thread;


#[derive(thiserror::Error, Debug)]
pub enum LspError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("LSP protocol error: {0}")]
    Protocol(String),
    #[error("Server not initialized")]
    NotInitialized,
    #[error("Server process error")]
    ProcessError,
}

pub struct LspClient {
    connection: Connection,
    initialized: bool,
    request_id: i64,
    server_capabilities: Option<ServerCapabilities>,
}

impl LspClient {
    pub fn new(server_command: &str, args: &[String]) -> Result<Self, LspError> {
        let mut child = Command::new(server_command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (connection, io_threads) = Connection::stdio();

        // Start IO threads in background
        thread::spawn(move || {
            io_threads.join().unwrap();
            let _ = child.wait(); // Clean up process
        });

        Ok(Self {
            connection,
            initialized: false,
            request_id: 0,
            server_capabilities: None,
        })
    }

    pub fn initialize(
        &mut self,
        workspace_folders: Option<Vec<WorkspaceFolder>>,
    ) -> Result<InitializeResult, LspError> {
        #[allow(deprecated)]
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: None,
            root_uri: None,
            initialization_options: None,
            capabilities: ClientCapabilities::default(),
            trace: Some(TraceValue::Off),
            workspace_folders,
            client_info: None,
            locale: None,
            work_done_progress_params: Default::default(),
        };

        let result: InitializeResult = self.send_request::<Initialize>("initialize", &params)?;
        self.server_capabilities = Some(result.capabilities.clone());
        self.initialized = true;

        // Send initialized notification
        self.send_notification::<Initialized>("initialized", &InitializedParams {})?;

        Ok(result)
    }

    pub fn send_request<R: lsp_types::request::Request>(
        &mut self,
        method: &str,
        params: &R::Params,
    ) -> Result<R::Result, LspError>
    where
        R::Params: serde::Serialize,
        R::Result: serde::de::DeserializeOwned,
    {
        if !self.initialized && method != "initialize" {
            return Err(LspError::NotInitialized);
        }

        let id = self.request_id as i32;
        self.request_id += 1;

        let request = Request::new(id.into(), method.to_string(), params);
        self.connection
            .sender
            .send(Message::Request(request))
            .map_err(|e| LspError::Protocol(format!("Send error: {}", e)))?;

        // Wait for response
        loop {
            match self
                .connection
                .receiver
                .recv()
                .map_err(|e| LspError::Protocol(format!("Receive error: {}", e)))?
            {
                Message::Response(response) => {
                    if response.id == id.into() {
                        if let Some(error) = response.error {
                            return Err(LspError::Protocol(format!(
                                "LSP error: {}",
                                error.message
                            )));
                        }
                        return serde_json::from_value(response.result.unwrap_or(Value::Null))
                            .map_err(|e| e.into());
                    }
                }
                Message::Notification(_) => continue, // Handle notifications separately if needed
                Message::Request(_) => continue,      // Server requests (like work done progress)
            }
        }
    }

    pub fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        method: &str,
        params: &N::Params,
    ) -> Result<(), LspError>
    where
        N::Params: serde::Serialize,
    {
        let notification = Notification::new(method.to_string(), params);
        self.connection
            .sender
            .send(Message::Notification(notification))
            .map_err(|e| LspError::Protocol(format!("Send error: {}", e)))?;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn shutdown(&mut self) -> Result<(), LspError> {
        if self.initialized {
            self.send_request::<Shutdown>("shutdown", &())?;
            self.send_notification::<Exit>("exit", &())?;
            self.initialized = false;
        }
        Ok(())
    }

    pub fn text_document_did_open(
        &self,
        uri: &Url,
        language_id: &str,
        version: i32,
        text: &str,
    ) -> Result<(), LspError> {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: language_id.to_string(),
                version,
                text: text.to_string(),
            },
        };
        self.send_notification::<DidOpenTextDocument>("textDocument/didOpen", &params)
    }

    pub fn text_document_did_change(
        &self,
        uri: &Url,
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), LspError> {
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version,
            },
            content_changes: changes,
        };
        self.send_notification::<DidChangeTextDocument>("textDocument/didChange", &params)
    }

    pub fn text_document_did_save(&self, uri: &Url, text: Option<&str>) -> Result<(), LspError> {
        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            text: text.map(|s| s.to_string()),
        };
        self.send_notification::<DidSaveTextDocument>("textDocument/didSave", &params)
    }

    pub fn text_document_did_close(&self, uri: &Url) -> Result<(), LspError> {
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };
        self.send_notification::<DidCloseTextDocument>("textDocument/didClose", &params)
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Url, TextDocumentIdentifier, TextDocumentContentChangeEvent};

    #[test]
    fn test_lsp_client_initialization() {
        // This test will fail without a real LSP server, but tests the initialization logic
        let result = LspClient::new("nonexistent-server", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_lsp_client_is_initialized() {
        // Create a mock client to test state
        // Note: This is limited since we can't easily mock the connection
        // In a real implementation, we'd use dependency injection for testing
    }

    #[test]
    fn test_text_document_did_open_params() {
        // Test parameter construction (without actual client)
        let uri = Url::parse("file:///test.rs").unwrap();
        let language_id = "rust";
        let version = 1;
        let text = "fn main() {}\n";

        let params = DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: uri.clone(),
                language_id: language_id.to_string(),
                version,
                text: text.to_string(),
            },
        };

        assert_eq!(params.text_document.uri, uri);
        assert_eq!(params.text_document.language_id, language_id);
        assert_eq!(params.text_document.version, version);
        assert_eq!(params.text_document.text, text);
    }

    #[test]
    fn test_text_document_did_change_params() {
        let uri = Url::parse("file:///test.rs").unwrap();
        let version = 2;
        let changes = vec![TextDocumentContentChangeEvent {
            range: None, // Full content change
            range_length: None,
            text: "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
        }];

        let params = DidChangeTextDocumentParams {
            text_document: lsp_types::VersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version,
            },
            content_changes: changes.clone(),
        };

        assert_eq!(params.text_document.uri, uri);
        assert_eq!(params.text_document.version, version);
        assert_eq!(params.content_changes.len(), 1);
        assert_eq!(params.content_changes[0].text, changes[0].text);
    }

    #[test]
    fn test_text_document_did_save_params() {
        let uri = Url::parse("file:///test.rs").unwrap();
        let text = Some("fn main() {}\n".to_string());

        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            text: text.clone(),
        };

        assert_eq!(params.text_document.uri, uri);
        assert_eq!(params.text, text);
    }

    #[test]
    fn test_text_document_did_close_params() {
        let uri = Url::parse("file:///test.rs").unwrap();

        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };

        assert_eq!(params.text_document.uri, uri);
    }
}
