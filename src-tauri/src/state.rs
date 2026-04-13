use remote_desktop_core::connection::id::SessionId;
use remote_desktop_core::connection::ConnectionManager;
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub session_id: SessionId,
    pub fps: AtomicU32,
    pub quality: AtomicU32,
    pub connected: AtomicBool,
    pub connection_manager: Arc<Mutex<ConnectionManager>>,
}

impl Default for AppState {
    fn default() -> Self {
        let signaling_url = std::env::var("SIGNALING_URL")
            .unwrap_or_else(|_| "ws://localhost:8080/ws".to_string());

        Self {
            session_id: SessionId::random(),
            fps: AtomicU32::new(30),
            quality: AtomicU32::new(1),
            connected: AtomicBool::new(false),
            connection_manager: Arc::new(Mutex::new(ConnectionManager::new(signaling_url))),
        }
    }
}
