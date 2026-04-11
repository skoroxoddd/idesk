use crate::capture::frame::CaptureFrame;
use crate::encode::encoder::{EncodedFrame, Encoder};
use crate::error::{AppError, Result};
use ffmpeg_sidecar::{command::FfmpegCommand, iteration::VideoFrame};

pub struct FfmpegEncoder {
    width: u32,
    height: u32,
    fps: u32,
    child: Option<std::process::Child>,
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<std::process::ChildStdout>,
}

impl FfmpegEncoder {
    pub fn new(width: u32, height: u32, fps: u32) -> Self {
        Self {
            width,
            height,
            fps,
            child: None,
            stdin: None,
            stdout: None,
        }
    }

    fn spawn_ffmpeg(&mut self) -> Result<()> {
        let child = FfmpegCommand::new()
            .format("rawvideo")
            .pixel_format("bgra")
            .size(&format!("{}x{}", self.width, self.height))
            .frame_rate(self.fps)
            .input("-")
            .codec_video("libx264")
            .preset("ultrafast")
            .tune("zerolatency")
            .pixel_format("yuv420p")
            .format("h264")
            .output("-")
            .spawn()
            .map_err(|e| AppError::Encode(format!("Failed to spawn ffmpeg: {e}")))?;

        self.child = Some(child);
        Ok(())
    }
}

impl Encoder for FfmpegEncoder {
    fn encode(&mut self, _frame: &CaptureFrame) -> Result<Vec<EncodedFrame>> {
        // FFmpeg-sidecar works via stdin/stdout streaming, so encoding
        // is handled by writing to stdin and reading from stdout.
        // This is a placeholder — actual implementation uses async I/O.
        Ok(Vec::new())
    }

    fn flush(&mut self) -> Result<Vec<EncodedFrame>> {
        Ok(Vec::new())
    }
}

impl Drop for FfmpegEncoder {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}
