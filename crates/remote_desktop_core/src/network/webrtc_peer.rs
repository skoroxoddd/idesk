use crate::error::{AppError, Result};
use crate::network::ice::IceServer;
use crate::stream::pipeline::PipelineOutput;
use crate::input::events::InputEvent;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use bytes::Bytes;

/// Represents a connected WebRTC peer session
#[derive(Clone)]
pub struct WebRtcSession {
    peer_connection: Arc<webrtc::peer_connection::RTCPeerConnection>,
    data_channel: Arc<webrtc::data_channel::RTCDataChannel>,
}

impl WebRtcSession {
    /// Send encoded frame over data channel
    pub async fn send_frame(&self, output: &PipelineOutput) -> Result<()> {
        let mut msg = Vec::with_capacity(9 + output.data.len());
        msg.push(if output.is_keyframe { 1u8 } else { 0u8 });
        msg.extend_from_slice(&output.timestamp_ms.to_le_bytes());
        msg.extend_from_slice(&output.data);
        self.data_channel.send(&Bytes::from(msg)).await
            .map_err(|e| AppError::Network(format!("Failed to send frame: {e}")))?;
        Ok(())
    }

    /// Send input event over data channel
    pub async fn send_input(&self, event: &InputEvent) -> Result<()> {
        let data = event.to_bytes();
        self.data_channel.send(&Bytes::from(data)).await
            .map_err(|e| AppError::Network(format!("Failed to send input: {e}")))?;
        Ok(())
    }

    pub fn peer_connection(&self) -> &webrtc::peer_connection::RTCPeerConnection {
        &self.peer_connection
    }
}

/// WebRTC peer connection manager
pub struct WebRtcPeer {
    signaling_url: String,
    ice_servers: Vec<IceServer>,
}

impl WebRtcPeer {
    pub fn new(signaling_url: String, ice_servers: Vec<IceServer>) -> Self {
        Self {
            signaling_url,
            ice_servers,
        }
    }

    /// Create a new peer connection with ICE servers
    async fn create_connection(&self) -> Result<(
        Arc<webrtc::peer_connection::RTCPeerConnection>,
        Arc<webrtc::data_channel::RTCDataChannel>,
    )> {
        let api = APIBuilder::new().build();

        let ice_servers: Vec<RTCIceServer> = self.ice_servers
            .iter()
            .map(|s| s.to_webrtc())
            .collect();

        let config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        let peer_connection = Arc::new(api.new_peer_connection(config).await
            .map_err(|e| AppError::Network(format!("Failed to create peer connection: {e}")))?);

        let ordered = true;
        let data_channel = peer_connection
            .create_data_channel("remote-desktop", Some(RTCDataChannelInit {
                ordered: Some(ordered),
                ..Default::default()
            }))
            .await
            .map_err(|e| AppError::Network(format!("Failed to create data channel: {e}")))?;

        info!("WebRTC data channel created: {}", data_channel.label());

        Ok((peer_connection, data_channel))
    }

    /// Create an SDP offer for the controlled side
    pub async fn create_offer(&self) -> Result<(String, WebRtcSession)> {
        let (peer_connection, data_channel) = self.create_connection().await?;

        let offer = peer_connection.create_offer(None).await
            .map_err(|e| AppError::Network(format!("Failed to create offer: {e}")))?;

        peer_connection.set_local_description(offer).await
            .map_err(|e| AppError::Network(format!("Failed to set local description: {e}")))?;

        // Wait for ICE gathering
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let sdp = peer_connection.local_description().await
            .ok_or_else(|| AppError::Network("No local description".to_string()))?;

        let session = WebRtcSession {
            peer_connection,
            data_channel,
        };

        Ok((sdp.sdp, session))
    }

    /// Answer an SDP offer from the controlled side
    pub async fn answer_offer(&self, offer_sdp: String) -> Result<(String, WebRtcSession)> {
        let api = APIBuilder::new().build();

        let ice_servers: Vec<RTCIceServer> = self.ice_servers
            .iter()
            .map(|s| s.to_webrtc())
            .collect();

        let config = RTCConfiguration {
            ice_servers,
            ..Default::default()
        };

        let peer_connection = Arc::new(api.new_peer_connection(config).await
            .map_err(|e| AppError::Network(format!("Failed to create peer connection: {e}")))?);

        // Wait for data channel from remote
        let (data_channel_tx, data_channel_rx) = tokio::sync::oneshot::channel();
        let data_channel_tx = Arc::new(Mutex::new(Some(data_channel_tx)));

        {
            let dc_tx = data_channel_tx.clone();
            peer_connection.on_data_channel(Box::new(move |dc| {
                let dc_tx = dc_tx.clone();
                Box::pin(async move {
                    info!("Received data channel: {}", dc.label());
                    if let Some(tx) = dc_tx.lock().await.take() {
                        let _ = tx.send(dc);
                    }
                })
            }));
        }

        // Set remote description
        let offer = RTCSessionDescription::offer(offer_sdp)
            .map_err(|e| AppError::Network(format!("Invalid offer SDP: {e}")))?;
        peer_connection.set_remote_description(offer).await
            .map_err(|e| AppError::Network(format!("Failed to set remote description: {e}")))?;

        // Create answer
        let answer = peer_connection.create_answer(None).await
            .map_err(|e| AppError::Network(format!("Failed to create answer: {e}")))?;

        peer_connection.set_local_description(answer).await
            .map_err(|e| AppError::Network(format!("Failed to set local description: {e}")))?;

        // Wait for ICE gathering and data channel
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let sdp = peer_connection.local_description().await
            .ok_or_else(|| AppError::Network("No local description".to_string()))?;

        let data_channel = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            data_channel_rx
        ).await
            .map_err(|_| AppError::Network("Timeout waiting for data channel".to_string()))?
            .map_err(|_| AppError::Network("Data channel sender dropped".to_string()))?;

        let session = WebRtcSession {
            peer_connection,
            data_channel,
        };

        Ok((sdp.sdp, session))
    }
}
