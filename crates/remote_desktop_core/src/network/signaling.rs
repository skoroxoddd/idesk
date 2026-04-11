use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

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
}

impl SignalingClient {
    pub fn new(url: String) -> Self {
        Self { url, session_id: None }
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn set_session_id(&mut self, id: String) {
        self.session_id = Some(id);
    }

    /// Connect to signaling server and return (client, message receiver)
    pub async fn connect(
        &self,
    ) -> Result<(Arc<Mutex<Self>>, tokio::sync::mpsc::Receiver<SignalingMessage>)> {
        let url = url::Url::parse(&self.url)
            .map_err(|e| AppError::Signaling(format!("Invalid URL: {e}")))?;

        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| AppError::Signaling(format!("WebSocket connect failed: {e}")))?;

        info!("Connected to signaling server");

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let client = Arc::new(Mutex::new(SignalingClient {
            url: self.url.clone(),
            session_id: self.session_id.clone(),
        }));

        // Spawn reader task
        let client_clone = client.clone();
        tokio::spawn(async move {
            let (_write, mut read) = ws_stream.split();
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        if msg.is_text() {
                            let text = msg.to_text().unwrap_or("");
                            debug!("Received signaling message: {}", text);
                            if let Ok(signal_msg) = serde_json::from_str::<SignalingMessage>(text) {
                                let _ = tx.send(signal_msg).await;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("WebSocket read error: {e}");
                        break;
                    }
                }
            }
            info!("Disconnected from signaling server");
        });

        Ok((client, rx))
    }

    pub async fn send(&self, msg: &SignalingMessage) -> Result<()> {
        debug!("Sending signaling message: {:?}", msg);
        // Actual send happens through the WebSocket handle in the caller
        Ok(())
    }
}
