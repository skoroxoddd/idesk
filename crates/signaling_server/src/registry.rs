use actix_ws::Session;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Maps session IDs to WebSocket sessions
pub struct PeerRegistry {
    peers: DashMap<String, Arc<Mutex<Session>>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: DashMap::new(),
        }
    }

    pub fn register(&self, session_id: String, session: Session) {
        info!("Registering peer: {session_id}");
        self.peers.insert(session_id, Arc::new(Mutex::new(session)));
    }

    pub fn unregister(&self, session_id: &str) {
        info!("Unregistering peer: {session_id}");
        self.peers.remove(session_id);
    }

    pub fn get_session(&self, session_id: &str) -> Option<Arc<Mutex<Session>>> {
        self.peers.get(session_id).map(|entry| entry.value().clone())
    }

    pub fn is_online(&self, session_id: &str) -> bool {
        self.peers.contains_key(session_id)
    }
}
