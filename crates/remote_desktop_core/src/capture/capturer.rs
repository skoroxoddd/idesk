use crate::capture::frame::CaptureFrame;
use crate::error::Result;

pub trait Capturer: Send + Sync {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn capture_frame(&mut self) -> Result<CaptureFrame>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
