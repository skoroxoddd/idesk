use remote_desktop_core::connection::id::SessionId;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct AppState {
    pub session_id: SessionId,
    pub fps: AtomicU32,
    pub quality: AtomicU32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session_id: SessionId::random(),
            fps: AtomicU32::new(30),
            quality: AtomicU32::new(1),
        }
    }
}
