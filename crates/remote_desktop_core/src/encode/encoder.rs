use crate::capture::frame::CaptureFrame;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub timestamp_ms: u64,
}

pub trait Encoder: Send + Sync {
    fn encode(&mut self, frame: &CaptureFrame) -> Result<Vec<EncodedFrame>>;
    fn flush(&mut self) -> Result<Vec<EncodedFrame>>;
}
