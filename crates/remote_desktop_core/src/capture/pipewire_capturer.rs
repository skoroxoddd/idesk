use crate::capture::capturer::Capturer;
use crate::capture::frame::CaptureFrame;
use crate::error::Result;

pub struct PipeWireCapturer {
    width: u32,
    height: u32,
}

impl PipeWireCapturer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            width: 1280,
            height: 720,
        })
    }
}

impl Capturer for PipeWireCapturer {
    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<CaptureFrame> {
        Err(crate::error::AppError::Capture("PipeWire capturer not yet implemented".to_string()))
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}
