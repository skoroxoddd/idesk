use crate::capture::frame::CaptureFrame;
use crate::encode::encoder::{EncodedFrame, Encoder};
use crate::error::{AppError, Result};
use openh264::formats::YUVBuffer;
use openh264::encoder::EncoderConfig;

pub struct OpenH264Encoder {
    encoder: openh264::encoder::Encoder,
    width: u32,
    height: u32,
}

impl OpenH264Encoder {
    pub fn new(width: u32, height: u32, bitrate: u32) -> Result<Self> {
        let config = EncoderConfig::new(width, height)
            .set_bitrate_bps(bitrate);

        let encoder = openh264::encoder::Encoder::with_config(config)
            .map_err(|e| AppError::Encode(format!("Failed to create encoder: {e}")))?;

        Ok(Self {
            encoder,
            width,
            height,
        })
    }
}

impl Encoder for OpenH264Encoder {
    fn encode(&mut self, frame: &CaptureFrame) -> Result<Vec<EncodedFrame>> {
        let mut yuv = YUVBuffer::new(self.width as usize, self.height as usize);
        yuv.read_rgb(&frame.data);

        let bitstream = self.encoder.encode(&yuv)
            .map_err(|e| AppError::Encode(format!("Encoding failed: {e}")))?;

        let mut data = Vec::new();
        bitstream.write_vec(&mut data);

        Ok(vec![EncodedFrame {
            data,
            is_keyframe: matches!(bitstream.frame_type(), openh264::encoder::FrameType::IDR | openh264::encoder::FrameType::I),
            timestamp_ms: frame.timestamp_ms,
        }])
    }

    fn flush(&mut self) -> Result<Vec<EncodedFrame>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::capturer::Capturer;

    /// Create a test frame with RGB data (3 bytes per pixel)
    fn make_test_frame(width: u32, height: u32) -> CaptureFrame {
        let size = (width * height * 3) as usize;
        let mut data = vec![0u8; size];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                data[idx] = (x % 256) as u8;       // R
                data[idx + 1] = (y % 256) as u8;   // G
                data[idx + 2] = 128u8;              // B
            }
        }
        CaptureFrame {
            data,
            width,
            height,
            stride: width * 3,
            timestamp_ms: 0,
        }
    }

    #[test]
    fn encode_mock_frames() {
        let mut encoder = OpenH264Encoder::new(320, 240, 500_000).unwrap();

        // Encode first frame — should produce a keyframe with SPS/PPS
        let frame = make_test_frame(320, 240);
        let encoded = encoder.encode(&frame).unwrap();
        assert!(!encoded.is_empty());
        assert!(encoded[0].is_keyframe); // First frame should be IDR
        assert!(!encoded[0].data.is_empty());

        // Encode a few more frames
        for _ in 0..5 {
            let frame = make_test_frame(320, 240);
            let encoded = encoder.encode(&frame).unwrap();
            assert!(!encoded.is_empty());
        }
    }
}
