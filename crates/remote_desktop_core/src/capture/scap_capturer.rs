use crate::capture::capturer::Capturer;
use crate::capture::frame::CaptureFrame;
use crate::error::{AppError, Result};

/// Screen capturer using scap (macOS + Windows)
pub struct ScapCapturer {
    capturer: Option<scap::Capturer>,
    width: u32,
    height: u32,
}

impl ScapCapturer {
    pub fn new() -> Result<Self> {
        let targets = scap::get_all_targets()
            .map_err(|e| AppError::Capture(format!("Failed to get targets: {e}")))?;

        let display = targets.first()
            .ok_or_else(|| AppError::Capture("No displays found".to_string()))?;

        let capturer = scap::Capturer::new(
            scap::CapturerConfig {
                target: Some(display.clone()),
                fps: 30,
                show_cursor: true,
                ..Default::default()
            },
        );

        let bounds = display.bounds();
        Ok(Self {
            capturer: Some(capturer),
            width: bounds.width as u32,
            height: bounds.height as u32,
        })
    }
}

impl Capturer for ScapCapturer {
    fn start(&mut self) -> Result<()> {
        if let Some(ref mut capturer) = self.capturer {
            capturer.start_capture();
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(ref mut capturer) = self.capturer {
            capturer.stop_capture();
        }
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<CaptureFrame> {
        let frame = self.capturer.as_mut()
            .ok_or_else(|| AppError::Capture("Capturer not initialized".to_string()))?
            .get_frame()
            .map_err(|e| AppError::Capture(format!("Failed to capture frame: {e}")))?;

        Ok(CaptureFrame {
            data: frame.data,
            width: frame.width as u32,
            height: frame.height as u32,
            stride: frame.width as u32 * 4,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        })
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}
