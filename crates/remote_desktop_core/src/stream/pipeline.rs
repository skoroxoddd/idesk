use crate::capture::capturer::Capturer;
use crate::encode::encoder::Encoder;
use crate::error::Result;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Capture -> Encode -> Send pipeline
pub struct StreamPipeline {
    capturer: Box<dyn Capturer>,
    encoder: Box<dyn Encoder>,
    sender: mpsc::Sender<Vec<u8>>,
    fps: u32,
    running: bool,
}

impl StreamPipeline {
    pub fn new(
        capturer: Box<dyn Capturer>,
        encoder: Box<dyn Encoder>,
        fps: u32,
    ) -> (Self, mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = mpsc::channel(128);
        (
            Self {
                capturer,
                encoder,
                sender: tx,
                fps,
                running: false,
            },
            rx,
        )
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        let interval = std::time::Duration::from_millis(1000 / self.fps as u64);
        let mut last_frame = std::time::Instant::now();

        info!(
            "Stream pipeline started: {}x{} @ {}fps",
            self.capturer.width(),
            self.capturer.height(),
            self.fps
        );

        while self.running {
            let frame_start = std::time::Instant::now();

            match self.capturer.capture_frame() {
                Ok(frame) => {
                    match self.encoder.encode(&frame) {
                        Ok(encoded_frames) => {
                            for ef in encoded_frames {
                                if self.sender.send(ef.data).await.is_err() {
                                    info!("Receiver dropped, stopping pipeline");
                                    self.running = false;
                                    break;
                                }
                            }
                        }
                        Err(e) => error!("Encode error: {e}"),
                    }
                }
                Err(e) => error!("Capture error: {e}"),
            }

            // Rate limiting — sleep remaining frame time
            let elapsed = frame_start.elapsed();
            if elapsed < interval {
                tokio::time::sleep(interval - elapsed).await;
            }

            // Drop late frames to keep latency low
            if last_frame.elapsed() > interval * 3 {
                // We're falling behind — skip frames
                while let Ok(_) = self.capturer.capture_frame() {}
            }
            last_frame = frame_start;
        }

        info!("Stream pipeline stopped");
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}
