use crate::messages::SignalingMessage;
use crate::registry::PeerRegistry;
use actix_web::web;
use actix_ws::{AggregatedMessage, Session};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub async fn ws_handler(
    registry: web::Data<Arc<PeerRegistry>>,
    req: actix_web::HttpRequest,
    stream: web::Payload,
) -> std::result::Result<actix_web::HttpResponse, actix_web::Error> {
    let (res, mut session, msg_stream) = actix_ws::handle(&req, stream)?;
    let mut msg_stream = msg_stream.aggregate_continuations();

    let registry = registry.into_inner();

    actix_web::rt::spawn(async move {
        let mut session_id: Option<String> = None;

        while let Some(msg) = msg_stream.recv().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    match serde_json::from_str::<SignalingMessage>(&text) {
                        Ok(signal_msg) => {
                            match signal_msg.msg_type.as_str() {
                                "register" => {
                                    if let Some(id) = signal_msg.from.clone() {
                                        registry.register(id.clone(), session.clone());
                                        session_id = Some(id);
                                        info!("Peer registered: {}", signal_msg.from.as_ref().unwrap());
                                    }
                                }
                                "offer" | "answer" | "ice-candidate" => {
                                    if let Some(ref target) = signal_msg.to {
                                        if let Some(target_session) = registry.get_session(target) {
                                            if let Ok(json) = serde_json::to_string(&signal_msg) {
                                                let mut sess = target_session.lock().await;
                                                if let Err(e) = sess.text(json).await {
                                                    warn!("Failed to send to {target}: {e}");
                                                }
                                            }
                                        } else {
                                            warn!("Target peer {target} not found");
                                        }
                                    }
                                }
                                "ping" => {
                                    let _ = session.text(r#"{"type":"pong"}"#).await;
                                }
                                _ => {
                                    warn!("Unknown message type: {}", signal_msg.msg_type);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Invalid signaling message: {e}");
                        }
                    }
                }
                Ok(AggregatedMessage::Ping(bytes)) => {
                    let _ = session.pong(&bytes).await;
                }
                Ok(AggregatedMessage::Close(reason)) => {
                    info!("WebSocket closed: {:?}", reason);
                    if let Some(id) = &session_id {
                        registry.unregister(id);
                    }
                    let _ = session.close(None).await;
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {e}");
                    break;
                }
                _ => {}
            }
        }

        // Cleanup on disconnect
        if let Some(id) = &session_id {
            registry.unregister(id);
        }
    });

    Ok(res)
}
