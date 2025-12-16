// src/lsp/client.rs - LSP Client implementation (Helix-style async architecture)

use super::transport::{Transport, TransportError};
use lsp_types::*;
use lsp_types::{
    ClientCapabilities,
    ClientInfo,
    DidChangeTextDocumentParams,
    DidCloseTextDocumentParams,
    DidOpenTextDocumentParams,
    DidSaveTextDocumentParams,
    InitializedParams,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument, Exit,
    },
    request::Shutdown,
    // Additional capabilities types
};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(thiserror::Error, Debug)]
pub enum LspError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("LSP protocol error: {0}")]
    Protocol(String),
    #[error("Server not initialized")]
    NotInitialized,
    #[error("Server process error")]
    ProcessError,
}

#[derive(Clone)]
pub struct LspClient {
    transport: Arc<Mutex<Option<Transport>>>,
    initialized: bool,
    server_capabilities: Option<ServerCapabilities>,
    process_handle: Arc<Mutex<Option<std::process::Child>>>,
    server_command: String,
    server_args: Vec<String>,
    connection_attempts: Arc<Mutex<u32>>,
}

impl LspClient {
    pub async fn new(server_command: &str, args: &[String]) -> Result<Self, LspError> {
        let child = Command::new(server_command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Redirect stderr to /dev/null to suppress LSP server output
            .spawn()?;

        let (connection, _io_threads) = lsp_server::Connection::stdio();

        let transport = Transport::new(connection);

        Ok(Self {
            transport: Arc::new(Mutex::new(Some(transport))),
            initialized: false,
            server_capabilities: None,
            process_handle: Arc::new(Mutex::new(Some(child))),
            server_command: server_command.to_string(),
            server_args: args.to_vec(),
            connection_attempts: Arc::new(Mutex::new(1)),
        })
    }

    pub async fn initialize(
        &mut self,
        workspace_folders: Option<Vec<WorkspaceFolder>>,
        root_uri: Option<Url>,
    ) -> Result<InitializeResult, LspError> {
        let transport = self.transport.lock().await;
        let transport = transport.as_ref().ok_or(LspError::NotInitialized)?;

        // Client capabilities - currently using defaults (empty capabilities object)
        // TODO: Add proper LSP client capabilities declaration to inform servers what features we support
        // This affects what the server sends us (e.g., completion, diagnostics, etc.)
        let capabilities = ClientCapabilities::default();

        #[allow(deprecated)]
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: root_uri
                .as_ref()
                .and_then(|u| u.to_file_path().ok())
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            root_uri,
            initialization_options: None,
            capabilities,
            trace: Some(TraceValue::Off),
            workspace_folders,
            client_info: Some(ClientInfo {
                name: "texty".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            locale: None,
            work_done_progress_params: Default::default(),
        };

        let params_value = serde_json::to_value(params)?;
        let result_value = transport
            .send_request("initialize".to_string(), params_value)
            .await?;
        let result: InitializeResult = serde_json::from_value(result_value)?;

        self.server_capabilities = Some(result.capabilities.clone());
        self.initialized = true;

        // Send initialized notification
        let initialized_params = serde_json::to_value(InitializedParams {})?;
        transport.send_notification("initialized".to_string(), initialized_params)?;

        Ok(result)
    }

    pub async fn send_request<R: lsp_types::request::Request>(
        &self,
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

        let transport = self.transport.lock().await;
        let transport = transport.as_ref().ok_or(LspError::NotInitialized)?;

        let params_value = serde_json::to_value(params)?;
        let result_value = transport
            .send_request(method.to_string(), params_value)
            .await?;
        serde_json::from_value(result_value).map_err(|e| e.into())
    }

    pub async fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        method: &str,
        params: &N::Params,
    ) -> Result<(), LspError>
    where
        N::Params: serde::Serialize,
    {
        let transport = self.transport.lock().await;
        let transport = transport.as_ref().ok_or(LspError::NotInitialized)?;

        let params_value = serde_json::to_value(params)?;
        transport.send_notification(method.to_string(), params_value)?;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub async fn shutdown(&mut self) -> Result<(), LspError> {
        if self.initialized {
            self.send_request::<Shutdown>("shutdown", &()).await?;
            self.send_notification::<Exit>("exit", &()).await?;
            self.initialized = false;
        }
        Ok(())
    }

    pub async fn text_document_did_open(
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
            .await
    }

    pub async fn text_document_did_change(
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
            .await
    }

    pub async fn text_document_did_save(
        &self,
        uri: &Url,
        text: Option<&str>,
    ) -> Result<(), LspError> {
        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            text: text.map(|s| s.to_string()),
        };
        self.send_notification::<DidSaveTextDocument>("textDocument/didSave", &params)
            .await
    }

    pub async fn text_document_did_close(&self, uri: &Url) -> Result<(), LspError> {
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };
        self.send_notification::<DidCloseTextDocument>("textDocument/didClose", &params)
            .await
    }

    pub async fn goto_definition(
        &self,
        uri: &Url,
        position: lsp_types::Position,
    ) -> Result<Option<lsp_types::GotoDefinitionResponse>, LspError> {
        let params = lsp_types::GotoDefinitionParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response: Option<lsp_types::GotoDefinitionResponse> = self
            .send_request::<lsp_types::request::GotoDefinition>("textDocument/definition", &params)
            .await?;
        Ok(response)
    }

    pub async fn find_references(
        &self,
        uri: &Url,
        position: lsp_types::Position,
        include_declaration: bool,
    ) -> Result<Option<Vec<lsp_types::Location>>, LspError> {
        let params = lsp_types::ReferenceParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: lsp_types::ReferenceContext {
                include_declaration,
            },
        };

        let response: Option<Vec<lsp_types::Location>> = self
            .send_request::<lsp_types::request::References>("textDocument/references", &params)
            .await?;
        Ok(response)
    }

    pub async fn hover(
        &self,
        uri: &Url,
        position: lsp_types::Position,
    ) -> Result<Option<lsp_types::Hover>, LspError> {
        let params = lsp_types::HoverParams {
            text_document_position_params: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            work_done_progress_params: Default::default(),
        };

        let response: Option<lsp_types::Hover> = self
            .send_request::<lsp_types::request::HoverRequest>("textDocument/hover", &params)
            .await?;
        Ok(response)
    }

    pub async fn workspace_symbols(
        &self,
        query: String,
    ) -> Result<Option<lsp_types::WorkspaceSymbolResponse>, LspError> {
        let params = lsp_types::WorkspaceSymbolParams {
            query,
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response: Option<lsp_types::WorkspaceSymbolResponse> = self
            .send_request::<lsp_types::request::WorkspaceSymbolRequest>("workspace/symbol", &params)
            .await?;
        Ok(response)
    }

    pub async fn code_actions(
        &self,
        uri: &Url,
        range: lsp_types::Range,
        context: lsp_types::CodeActionContext,
    ) -> Result<Option<lsp_types::CodeActionResponse>, LspError> {
        let params = lsp_types::CodeActionParams {
            text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
            range,
            context,
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response: Option<lsp_types::CodeActionResponse> = self
            .send_request::<lsp_types::request::CodeActionRequest>(
                "textDocument/codeAction",
                &params,
            )
            .await?;
        Ok(response)
    }

    pub async fn is_healthy(&self) -> bool {
        let transport = self.transport.lock().await;
        transport
            .as_ref()
            .map(|t| t.is_connected())
            .unwrap_or(false)
    }

    pub async fn restart_if_needed(&mut self) -> Result<(), LspError> {
        if !self.is_healthy().await {
            let attempts = {
                let mut attempts = self.connection_attempts.lock().await;
                *attempts += 1;
                *attempts
            };

            if attempts <= 3 {
                // Max 3 restart attempts
                eprintln!(
                    "LSP server unhealthy, attempting restart (attempt {})",
                    attempts
                );

                // Clean up existing process
                if let Some(mut child) = self.process_handle.lock().await.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }

                // Create new connection
                let child = Command::new(&self.server_command)
                    .args(&self.server_args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;

                let (connection, _io_threads) = lsp_server::Connection::stdio();
                let transport = Transport::new(connection);

                *self.transport.lock().await = Some(transport);
                *self.process_handle.lock().await = Some(child);
                self.initialized = false;

                // Re-initialize
                let workspace_folders = None;
                let root_uri = None;
                self.initialize(workspace_folders, root_uri).await?;

                Ok(())
            } else {
                Err(LspError::Protocol(
                    "Max restart attempts exceeded".to_string(),
                ))
            }
        } else {
            // Reset attempts on successful connection
            *self.connection_attempts.lock().await = 0;
            Ok(())
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Note: shutdown() is async, so we can't call it here
        // The transport will be dropped automatically
        if let Ok(mut guard) = self.process_handle.try_lock()
            && let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{TextDocumentContentChangeEvent, TextDocumentIdentifier, Url};

    // TODO: Add proper LSP client tests when we have a test LSP server

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
