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
