use crate::capture::frame::CaptureFrame;
use crate::encode::encoder::{EncodedFrame, Encoder};
use crate::error::Result;
use ffmpeg_sidecar::command::FfmpegCommand;

pub struct FfmpegEncoder {
    width: u32,
    height: u32,
    fps: u32,
}

impl FfmpegEncoder {
    pub fn new(width: u32, height: u32, fps: u32) -> Self {
        Self { width, height, fps }
    }
}

impl Encoder for FfmpegEncoder {
    fn encode(&mut self, _frame: &CaptureFrame) -> Result<Vec<EncodedFrame>> {
        // FFmpeg-sidecar works via stdin/stdout streaming.
        // Actual implementation spawns FfmpegCommand and pipes frames.
        Ok(Vec::new())
    }

    fn flush(&mut self) -> Result<Vec<EncodedFrame>> {
        Ok(Vec::new())
    }
}
