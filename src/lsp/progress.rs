// src/lsp/progress.rs - LSP progress reporting

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ProgressItem {
    pub token: String,
    pub title: String,
    pub message: Option<String>,
    pub percentage: Option<u32>,
    pub cancellable: bool,
}

#[derive(Debug, Clone)]
pub struct ProgressManager {
    items: Arc<Mutex<HashMap<String, ProgressItem>>>,
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressManager {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_progress(&self, token: String, title: String, cancellable: bool) {
        let mut items = self.items.lock().await;
        items.insert(
            token.clone(),
            ProgressItem {
                token,
                title,
                message: None,
                percentage: None,
                cancellable,
            },
        );
    }

    pub async fn update_progress(
        &self,
        token: String,
        message: Option<String>,
        percentage: Option<u32>,
    ) {
        let mut items = self.items.lock().await;
        if let Some(item) = items.get_mut(&token) {
            item.message = message;
            item.percentage = percentage;
        }
    }

    pub async fn end_progress(&self, token: String) {
        let mut items = self.items.lock().await;
        items.remove(&token);
    }

    pub async fn get_all_progress(&self) -> Vec<ProgressItem> {
        let items = self.items.lock().await;
        items.values().cloned().collect()
    }

    pub async fn has_active_progress(&self) -> bool {
        let items = self.items.lock().await;
        !items.is_empty()
    }
}
