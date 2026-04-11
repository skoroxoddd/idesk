use dashmap::DashMap;
use tracing::info;

/// Maps session IDs to WebSocket sender channels
pub struct PeerRegistry {
    peers: DashMap<String, actix_ws::SinkWriter>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: DashMap::new(),
        }
    }

    pub fn register(&self, session_id: String, writer: actix_ws::SinkWriter) {
        info!("Registering peer: {session_id}");
        self.peers.insert(session_id, writer);
    }

    pub fn unregister(&self, session_id: &str) {
        info!("Unregistering peer: {session_id}");
        self.peers.remove(session_id);
    }

    pub fn get_sender(&self, session_id: &str) -> Option<actix_ws::SinkWriter> {
        self.peers.get(session_id).map(|entry| entry.value().clone())
    }

    pub fn is_online(&self, session_id: &str) -> bool {
        self.peers.contains_key(session_id)
    }
}
