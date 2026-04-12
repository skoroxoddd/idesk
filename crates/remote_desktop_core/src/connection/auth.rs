use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug)]
pub struct AuthManager {
    pins: RwLock<HashMap<String, String>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            pins: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_pin(&self, session_id: &str, pin: &str) {
        let hash = simple_hash(pin);
        self.pins.write().unwrap().insert(session_id.to_string(), hash);
    }

    pub fn verify_pin(&self, session_id: &str, pin: &str) -> bool {
        let pins = self.pins.read().unwrap();
        pins.get(session_id)
            .map(|stored| stored == &simple_hash(pin))
            .unwrap_or(false)
    }
}

fn simple_hash(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:x}")
}
