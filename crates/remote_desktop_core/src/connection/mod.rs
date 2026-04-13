pub mod id;
pub mod auth;

use crate::capture::capturer::Capturer;
use crate::encode::encoder::Encoder;
use crate::input::injector::InputInjector;
use crate::network::webrtc_peer::WebRtcPeer;
use crate::network::webrtc_peer::WebRtcSession;
use crate::network::signaling::SignalingClient;
use crate::network::ice::IceServer;
use crate::stream::pipeline::StreamPipeline;
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

/// Active remote desktop session
pub struct RemoteSession {
    pub session: WebRtcSession,
    pub pipeline_handle: Option<tokio::task::JoinHandle<()>>,
    pub input_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Connection manager — orchestrates signaling + WebRTC + pipeline
pub struct ConnectionManager {
    signaling_url: String,
    signaling: Option<SignalingClient>,
    webrtc: WebRtcPeer,
    pub capturer: Option<Box<dyn Capturer>>,
    pub encoder: Option<Box<dyn Encoder>>,
    pub injector: Option<Box<dyn InputInjector>>,
    fps: u32,
    active_session: Option<Arc<Mutex<RemoteSession>>>,
}

impl ConnectionManager {
    pub fn new(signaling_url: String) -> Self {
        Self {
            signaling_url: signaling_url.clone(),
            signaling: None,
            webrtc: WebRtcPeer::new(signaling_url, IceServer::default_stun()),
            capturer: None,
            encoder: None,
            injector: None,
            fps: 30,
            active_session: None,
        }
    }

    pub fn set_capturer(&mut self, capturer: Box<dyn Capturer>) {
        self.capturer = Some(capturer);
    }

    pub fn set_encoder(&mut self, encoder: Box<dyn Encoder>) {
        self.encoder = Some(encoder);
    }

    pub fn set_injector(&mut self, injector: Box<dyn InputInjector>) {
        self.injector = Some(injector);
    }

    pub fn set_fps(&mut self, fps: u32) {
        self.fps = fps.clamp(10, 60);
    }

    /// Connect to signaling server
    pub async fn connect_signaling(&mut self, session_id: String) -> Result<()> {
        let mut client = SignalingClient::new(self.signaling_url.clone());
        client.set_session_id(session_id);
        client.connect().await?;
        self.signaling = Some(client);
        Ok(())
    }

    /// Start streaming as the controlled side
    /// 1. Create WebRTC offer
    /// 2. Send via signaling
    /// 3. Wait for answer
    /// 4. Start capture→encode→send pipeline
    pub async fn start_streaming(&mut self, peer_id: String) -> Result<()> {
        let signaling = self.signaling.as_ref()
            .ok_or_else(|| crate::error::AppError::Connection("Not connected to signaling".to_string()))?;

        let capturer = self.capturer.take()
            .ok_or_else(|| crate::error::AppError::Connection("No capturer set".to_string()))?;

        let encoder = self.encoder.take()
            .ok_or_else(|| crate::error::AppError::Connection("No encoder set".to_string()))?;

        // Create WebRTC offer
        let (offer_sdp, session) = self.webrtc.create_offer().await?;
        info!("Created WebRTC offer, sending to peer {}", peer_id);

        // Send offer via signaling
        signaling.send_to(&peer_id, "offer", serde_json::json!({ "sdp": offer_sdp })).await?;

        // Wait for answer
        info!("Waiting for answer from {}", peer_id);
        let answer_msg = signaling.wait_for_message("answer", &peer_id).await?;
        let answer_sdp = answer_msg.payload
            .and_then(|p| p.get("sdp").and_then(|v| v.as_str().map(String::from)))
            .ok_or_else(|| crate::error::AppError::Connection("No SDP in answer".to_string()))?;

        info!("Received answer, setting remote description");

        // Start capture pipeline
        let width = capturer.width();
        let height = capturer.height();
        let fps = self.fps;

        let (mut pipeline, mut frame_rx) = StreamPipeline::new(capturer, encoder, fps);

        // Forward frames to WebRTC
        let session_clone = session.clone();
        let pipeline_handle = tokio::spawn(async move {
            while let Some(frame_output) = frame_rx.recv().await {
                if let Err(e) = session_clone.send_frame(&frame_output).await {
                    error!("Failed to send frame: {e}");
                }
            }
        });

        // Run pipeline in background
        let pipeline_run_handle = tokio::spawn(async move {
            if let Err(e) = pipeline.run().await {
                error!("Pipeline error: {e}");
            }
        });

        // Forward input events to injector
        let injector = self.injector.take();
        let input_handle = tokio::spawn(async move {
            if let Some(mut injector) = injector {
                info!("Input event receiver started");
                // Input events come from the data channel
                // They are handled by the WebRTC session
                let _ = injector;
            }
        });

        self.active_session = Some(Arc::new(Mutex::new(RemoteSession {
            session,
            pipeline_handle: Some(pipeline_handle),
            input_handle: Some(input_handle),
        })));

        info!("Streaming started to peer {}", peer_id);
        Ok(())
    }

    /// Connect as the controller side
    /// 1. Wait for offer from peer via signaling
    /// 2. Create answer
    /// 3. Send answer via signaling
    pub async fn connect_to_peer(&mut self, peer_id: String) -> Result<()> {
        let signaling = self.signaling.as_ref()
            .ok_or_else(|| crate::error::AppError::Connection("Not connected to signaling".to_string()))?;

        // Wait for offer from peer
        info!("Waiting for offer from {}", peer_id);
        let offer_msg = signaling.wait_for_message("offer", &peer_id).await?;
        let offer_sdp = offer_msg.payload
            .and_then(|p| p.get("sdp").and_then(|v| v.as_str().map(String::from)))
            .ok_or_else(|| crate::error::AppError::Connection("No SDP in offer".to_string()))?;

        info!("Received offer, creating answer");

        // Create answer
        let (answer_sdp, session) = self.webrtc.answer_offer(offer_sdp).await?;

        // Send answer
        signaling.send_to(&peer_id, "answer", serde_json::json!({ "sdp": answer_sdp })).await?;
        info!("Answer sent to {}", peer_id);

        self.active_session = Some(Arc::new(Mutex::new(RemoteSession {
            session,
            pipeline_handle: None,
            input_handle: None,
        })));

        info!("Connected to peer {}", peer_id);
        Ok(())
    }

    /// Disconnect active session
    pub async fn disconnect(&mut self) {
        if let Some(session) = self.active_session.take() {
            let mut s = session.lock().await;
            if let Some(h) = s.pipeline_handle.take() {
                h.abort();
            }
            if let Some(h) = s.input_handle.take() {
                h.abort();
            }
            let _ = s.session.peer_connection().close().await;
            info!("Session disconnected");
        }
    }

    pub fn is_connected(&self) -> bool {
        self.active_session.is_some()
    }
}
