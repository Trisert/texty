// src/lsp/transport.rs - Async transport layer for LSP communication (Helix-style)

// use futures::channel::mpsc;  // Future: for async message handling
// use futures::StreamExt;  // Future: for async message handling
use lsp_server::{Connection, ErrorCode, Message, RequestId, Response};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

// Future: Transport event types for async message handling
// #[derive(Debug, Clone)]
// pub enum TransportEvent {
//     Request { id: RequestId, method: String },
//     Response { id: RequestId, result: serde_json::Value },
//     Notification { method: String, params: serde_json::Value },
//     Progress { token: serde_json::Value, value: serde_json::Value },
// }
//
// pub struct TransportHandle {
//     pub sender: mpsc::UnboundedSender<Message>,
//     pub event_receiver: mpsc::UnboundedReceiver<TransportEvent>,
// }

#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Channel error: {0}")]
    Channel(String),
    #[error("Connection closed")]
    ConnectionClosed,
}

pub struct Transport {
    connection: Connection,
    pending_requests: Arc<Mutex<HashMap<RequestId, oneshot::Sender<Response>>>>,
    next_id: Arc<Mutex<i64>>,
}

impl Transport {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    // Future: Async message processing
    // async fn process_messages(
    //     connection: Connection,
    //     mut message_receiver: mpsc::UnboundedReceiver<Message>,
    //     _pending_requests: Arc<Mutex<HashMap<RequestId, oneshot::Sender<Response>>>>,
    //     _event_sender: mpsc::UnboundedSender<TransportEvent>,
    // ) {
    //     // Handle incoming messages asynchronously
    //     while let Some(message) = message_receiver.next().await {
    //         if connection.sender.send(message).is_err() {
    //             break; // Connection closed
    //         }
    //     }
    // }

    pub async fn send_request(
        &self,
        method: String,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, TransportError> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            *next_id += 1;
            RequestId::from((*next_id - 1) as i32)
        };

        let request = lsp_server::Request::new(id.clone(), method, params);
        let (response_sender, response_receiver) = oneshot::channel();

        // Register pending request
        self.pending_requests
            .lock()
            .unwrap()
            .insert(id.clone(), response_sender);

        // Send request
        self.connection
            .sender
            .send(Message::Request(request))
            .map_err(|_| TransportError::ConnectionClosed)?;

        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), response_receiver).await {
            Ok(Ok(response)) => {
                if let Some(error) = response.error {
                    return Err(TransportError::Channel(format!(
                        "LSP error: {}",
                        error.message
                    )));
                }
                Ok(response.result.unwrap_or(serde_json::Value::Null))
            }
            _ => Err(TransportError::Channel(
                "Request timeout or connection closed".to_string(),
            )),
        }
    }

    pub fn send_notification(
        &self,
        method: String,
        params: serde_json::Value,
    ) -> Result<(), TransportError> {
        let notification = lsp_server::Notification::new(method, params);
        self.connection
            .sender
            .send(Message::Notification(notification))
            .map_err(|_| TransportError::ConnectionClosed)
    }

    pub fn is_connected(&self) -> bool {
        // TODO: Implement connection health check
        true
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        // Clean up pending requests
        let mut pending = self.pending_requests.lock().unwrap();
        for (_, sender) in pending.drain() {
            let _ = sender.send(Response::new_err(
                RequestId::from(-1),
                ErrorCode::InternalError as i32,
                "Transport shutting down".to_string(),
            ));
        }
    }
}
