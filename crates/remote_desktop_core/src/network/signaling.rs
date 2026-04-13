use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn, error};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub payload: Option<serde_json::Value>,
}

pub struct SignalingClient {
    url: String,
    session_id: Option<String>,
    ws: Arc<Mutex<Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>>,
}

impl SignalingClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            session_id: None,
            ws: Arc::new(Mutex::new(None)),
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn set_session_id(&mut self, id: String) {
        self.session_id = Some(id);
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to signaling server at {}", self.url);

        let (ws_stream, _) = connect_async(&self.url).await
            .map_err(|e| AppError::Network(format!("WebSocket connection failed: {e}")))?;

        let mut ws = self.ws.lock().await;
        *ws = Some(ws_stream);
        drop(ws);

        // Register this session
        if let Some(ref id) = self.session_id {
            let msg = SignalingMessage {
                msg_type: "register".to_string(),
                from: Some(id.clone()),
                to: None,
                payload: None,
            };
            self.send(&msg).await?;
            info!("Registered with signaling server as {}", id);
        }

        Ok(())
    }

    pub async fn send(&self, msg: &SignalingMessage) -> Result<()> {
        let json = serde_json::to_string(msg)
            .map_err(|e| AppError::Network(format!("Failed to serialize: {e}")))?;

        let mut ws = self.ws.lock().await;
        if let Some(ref mut ws_stream) = *ws {
            ws_stream.send(Message::Text(json.into()))
                .await
                .map_err(|e| AppError::Network(format!("Failed to send: {e}")))?;
        } else {
            return Err(AppError::Network("Not connected".to_string()));
        }
        Ok(())
    }

    pub async fn send_to(&self, target: &str, msg_type: &str, payload: serde_json::Value) -> Result<()> {
        let msg = SignalingMessage {
            msg_type: msg_type.to_string(),
            from: self.session_id.clone(),
            to: Some(target.to_string()),
            payload: Some(payload),
        };
        self.send(&msg).await
    }

    /// Wait for a specific message type from a specific peer
    pub async fn wait_for_message(&self, expected_type: &str, from_peer: &str) -> Result<SignalingMessage> {
        let mut ws = self.ws.lock().await;
        let ws_stream = ws.as_mut()
            .ok_or_else(|| AppError::Network("Not connected".to_string()))?;

        loop {
            let msg = ws_stream.next().await
                .ok_or_else(|| AppError::Network("Connection closed".to_string()))?
                .map_err(|e| AppError::Network(format!("WebSocket error: {e}")))?;

            if let Message::Text(text) = msg {
                match serde_json::from_str::<SignalingMessage>(&text) {
                    Ok(signal_msg) => {
                        debug!("Received signaling message: {}", signal_msg.msg_type);
                        if signal_msg.msg_type == expected_type && signal_msg.from.as_deref() == Some(from_peer) {
                            return Ok(signal_msg);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse signaling message: {e}");
                    }
                }
            }
        }
    }

    /// Receive the next message (any type)
    pub async fn recv(&self) -> Result<SignalingMessage> {
        let mut ws = self.ws.lock().await;
        let ws_stream = ws.as_mut()
            .ok_or_else(|| AppError::Network("Not connected".to_string()))?;

        loop {
            let msg = ws_stream.next().await
                .ok_or_else(|| AppError::Network("Connection closed".to_string()))?
                .map_err(|e| AppError::Network(format!("WebSocket error: {e}")))?;

            if let Message::Text(text) = msg {
                match serde_json::from_str::<SignalingMessage>(&text) {
                    Ok(signal_msg) => {
                        debug!("Received signaling message: {}", signal_msg.msg_type);
                        return Ok(signal_msg);
                    }
                    Err(e) => {
                        warn!("Failed to parse signaling message: {e}");
                    }
                }
            }
        }
    }
}
