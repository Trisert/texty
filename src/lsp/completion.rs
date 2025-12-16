// src/lsp/completion.rs - LSP completion handling

use super::client::{LspClient, LspError};
use lsp_types::{
    CompletionItem, CompletionParams, CompletionResponse, Position, TextDocumentIdentifier, Url,
};

#[derive(Debug)]
pub struct CompletionManager {
    pub items: Vec<CompletionItem>,
    pub current_index: usize,
}

impl Default for CompletionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionManager {
    pub fn get_current_item(&self) -> Option<&lsp_types::CompletionItem> {
        if self.items.is_empty() {
            None
        } else {
            Some(&self.items[self.current_index])
        }
    }

    pub fn set_items(&mut self, items: Vec<lsp_types::CompletionItem>) {
        self.items = items;
        self.current_index = 0;
    }
    pub fn new() -> Self {
        Self {
            items: vec![],
            current_index: 0,
        }
    }

    pub async fn request_completion(
        &mut self,
        client: &LspClient,
        uri: &Url,
        position: Position,
    ) -> Result<&[CompletionItem], LspError> {
        let params = CompletionParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let response: Option<CompletionResponse> = client
            .send_request::<lsp_types::request::Completion>("textDocument/completion", &params)
            .await?;
        let response = response.ok_or(LspError::Protocol("No completion response".to_string()))?;
        self.items = match response {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        };
        self.current_index = 0;
        Ok(self.items.as_slice())
    }

    pub fn next_item(&mut self) {
        if !self.items.is_empty() {
            self.current_index = (self.current_index + 1) % self.items.len();
        }
    }

    pub fn prev_item(&mut self) {
        if !self.items.is_empty() {
            self.current_index = if self.current_index == 0 {
                self.items.len() - 1
            } else {
                self.current_index - 1
            };
        }
    }

    pub fn current_item(&self) -> Option<&CompletionItem> {
        self.items.get(self.current_index)
    }

    pub fn accept_completion(&self) -> Option<&str> {
        self.current_item()
            .and_then(|item| item.insert_text.as_deref().or(item.label.as_str().into()))
    }
}
