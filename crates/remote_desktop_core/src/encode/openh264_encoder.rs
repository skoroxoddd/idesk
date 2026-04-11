use crate::capture::frame::CaptureFrame;
use crate::encode::encoder::{EncodedFrame, Encoder};
use crate::error::{AppError, Result};

pub struct OpenH264Encoder {
    width: u32,
    height: u32,
    fps: u32,
    bitrate: u32,
    encoder: Option<openh264::encoder::Encoder>,
    config: OpenH264Config,
}

#[derive(Debug, Clone)]
pub struct OpenH264Config {
    pub fps: u32,
    pub bitrate: u32,
    pub max_nal_size: u32,
}

impl Default for OpenH264Config {
    fn default() -> Self {
        Self {
            fps: 30,
            bitrate: 2_000_000, // 2 Mbps
            max_nal_size: 0,
        }
    }
}

impl OpenH264Encoder {
    pub fn new(width: u32, height: u32, config: OpenH264Config) -> Result<Self> {
        let encoder = Self::create_encoder(width, height, &config)?;
        Ok(Self {
            width,
            height,
            fps: config.fps,
            bitrate: config.bitrate,
            encoder: Some(encoder),
            config,
        })
    }

    fn create_encoder(width: u32, height: u32, config: &OpenH264Config) -> Result<openh264::encoder::Encoder> {
        use openh264::encoder::EncoderConfig;

        let encoder_config = EncoderConfig::with_width_height(width as i32, height as i32)
            .map_err(|e| AppError::Encode(format!("Failed to create encoder config: {e}")))?
            .frame_rate(config.fps as f32)
            .max_bitrate(config.bitrate as i32)
            .target_bitrate(config.bitrate as i32)
            .enable_skip_frame(false)
            .enable_denoise(false)
            .max_nal_size(config.max_nal_size);

        let encoder = openh264::encoder::Encoder::with_config(encoder_config)
            .map_err(|e| AppError::Encode(format!("Failed to create encoder: {e}")))?;

        Ok(encoder)
    }
}

impl Encoder for OpenH264Encoder {
    fn encode(&mut self, frame: &CaptureFrame) -> Result<Vec<EncodedFrame>> {
        let encoder = self.encoder.as_mut()
            .ok_or_else(|| AppError::Encode("Encoder not initialized".to_string()))?;

        // Convert BGRA to I420 (YUV420) for openh264
        let yuv = bgra_to_i420(&frame.data, frame.width, frame.height);

        let src = openh264::YUVSource::with_i420(
            frame.width as i32,
            frame.height as i32,
            &yuv,
        ).map_err(|e| AppError::Encode(format!("Failed to create YUV source: {e}")))?;

        let nals = encoder.encode(&src)
            .map_err(|e| AppError::Encode(format!("Encoding failed: {e}")))?;

        let mut frames = Vec::new();
        for nal in nals {
            frames.push(EncodedFrame {
                data: nal.to_vec(),
                is_keyframe: nal.nal_type() == openh264::NalType::NalUnitCodedSliceIdr,
                timestamp_ms: frame.timestamp_ms,
            });
        }

        Ok(frames)
    }

    fn flush(&mut self) -> Result<Vec<EncodedFrame>> {
        Ok(Vec::new())
    }
}

fn bgra_to_i420(bgra: &[u8], width: u32, height: u32) -> Vec<u8> {
    let y_size = (width * height) as usize;
    let uv_size = (width * height / 4) as usize;
    let mut i420 = vec![0u8; y_size + 2 * uv_size];

    let (y_plane, rest) = i420.split_at_mut(y_size);
    let (u_plane, v_plane) = rest.split_at_mut(uv_size);

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let b = bgra[idx] as f32;
            let g = bgra[idx + 1] as f32;
            let r = bgra[idx + 2] as f32;

            let yi = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 255.0) as u8;
            let ui = (128.0 - 0.169 * r - 0.331 * g + 0.5 * b).clamp(0.0, 255.0) as u8;
            let vi = (128.0 + 0.5 * r - 0.419 * g - 0.081 * b).clamp(0.0, 255.0) as u8;

            y_plane[(y * width + x) as usize] = yi;

            if y % 2 == 0 && x % 2 == 0 {
                let uv_idx = ((y / 2) * (width / 2) + (x / 2)) as usize;
                u_plane[uv_idx] = ui;
                v_plane[uv_idx] = vi;
            }
        }
    }

    i420
}
