use webrtc::ice::ICETransportPolicy;

#[derive(Debug, Clone)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

impl IceServer {
    pub fn default_stun() -> Vec<Self> {
        vec![
            IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            },
            IceServer {
                urls: vec!["stun:stun1.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            },
        ]
    }

    pub fn to_webrtc(&self) -> webrtc::api::setting_engine::IceServer {
        let mut server = webrtc::api::setting_engine::IceServer {
            urls: self.urls.clone(),
            username: self.username.clone().unwrap_or_default(),
            credential: self.credential.clone().unwrap_or_default(),
            ..Default::default()
        };
        server
    }
}
