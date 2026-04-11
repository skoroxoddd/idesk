use crate::capture::capturer::Capturer;
use crate::capture::frame::CaptureFrame;
use crate::error::Result;

/// Mock capturer for testing — generates colored frames
pub struct MockCapturer {
    width: u32,
    height: u32,
    frame_count: u64,
}

impl MockCapturer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            frame_count: 0,
        }
    }
}

impl Capturer for MockCapturer {
    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn capture_frame(&mut self) -> Result<CaptureFrame> {
        self.frame_count += 1;
        let size = (self.width * self.height * 4) as usize;
        let mut data = vec![0u8; size];

        // Generate a simple gradient that changes per frame
        let hue = (self.frame_count * 15) % 360;
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = ((y * self.width + x) * 4) as usize;
                let r = ((hue as u32 + x) % 256) as u8;
                let g = ((hue as u32 + y) % 256) as u8;
                let b = (hue as u8).wrapping_add((x / 2) as u8);
                data[idx] = b;     // B
                data[idx + 1] = g; // G
                data[idx + 2] = r; // R
                data[idx + 3] = 255; // A
            }
        }

        Ok(CaptureFrame {
            data,
            width: self.width,
            height: self.height,
            stride: self.width * 4,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_capturer_produces_frames() {
        let mut capturer = MockCapturer::new(640, 480);
        capturer.start().unwrap();
        let frame = capturer.capture_frame().unwrap();
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
        assert_eq!(frame.data.len(), 640 * 480 * 4);
    }
}
