use crate::capture::capturer::Capturer;
use crate::capture::frame::CaptureFrame;
use crate::error::Result;

pub struct ScapCapturer {
    width: u32,
    height: u32,
}

impl ScapCapturer {
    pub fn new() -> Result<Self> {
        // scap initialization goes here
        Ok(Self {
            width: 1280,
            height: 720,
        })
    }
}

impl Capturer for ScapCapturer {
    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<CaptureFrame> {
        // scap capture integration
        Err(crate::error::AppError::Capture("scap not yet implemented".to_string()))
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}
