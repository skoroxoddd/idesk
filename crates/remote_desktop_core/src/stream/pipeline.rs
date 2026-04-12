use crate::capture::capturer::Capturer;
use crate::encode::encoder::Encoder;
use crate::encode::encoder::EncodedFrame;
use crate::error::Result;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Output from the stream pipeline
pub struct PipelineOutput {
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub timestamp_ms: u64,
}

/// Capture -> Encode -> Send pipeline
pub struct StreamPipeline {
    capturer: Box<dyn Capturer>,
    encoder: Box<dyn Encoder>,
    sender: mpsc::Sender<PipelineOutput>,
    fps: u32,
    running: bool,
    /// SPS/PPS data for decoder initialization (sent with keyframes)
    sps_pps: Option<Vec<u8>>,
    /// Adaptive bitrate state
    target_bitrate: u32,
    frame_count: u64,
    encode_time_ms: f64,
}

impl StreamPipeline {
    pub fn new(
        capturer: Box<dyn Capturer>,
        encoder: Box<dyn Encoder>,
        fps: u32,
    ) -> (Self, mpsc::Receiver<PipelineOutput>) {
        let (tx, rx) = mpsc::channel(256);
        (
            Self {
                capturer,
                encoder,
                sender: tx,
                fps,
                running: false,
                sps_pps: None,
                target_bitrate: 2_000_000, // 2 Mbps default
                frame_count: 0,
                encode_time_ms: 0.0,
            },
            rx,
        )
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running = true;
        let interval = std::time::Duration::from_millis(1000 / self.fps as u64);

        info!(
            "Stream pipeline started: {}x{} @ {}fps, bitrate={}bps",
            self.capturer.width(),
            self.capturer.height(),
            self.fps,
            self.target_bitrate,
        );

        while self.running {
            let frame_start = std::time::Instant::now();

            match self.capturer.capture_frame() {
                Ok(frame) => {
                    let encode_start = std::time::Instant::now();
                    match self.encoder.encode(&frame) {
                        Ok(encoded_frames) => {
                            self.encode_time_ms = encode_start.elapsed().as_secs_f64() * 1000.0;
                            self.frame_count += 1;

                            for ef in encoded_frames {
                                // On keyframes, prepend SPS/PPS if available
                                let data = if ef.is_keyframe {
                                    if let Some(ref sps_pps_data) = self.sps_pps {
                                        let mut combined = sps_pps_data.clone();
                                        combined.extend_from_slice(&ef.data);
                                        combined
                                    } else {
                                        ef.data
                                    }
                                } else {
                                    ef.data
                                };

                                if self.sender.send(PipelineOutput {
                                    data,
                                    is_keyframe: ef.is_keyframe,
                                    timestamp_ms: ef.timestamp_ms,
                                }).await.is_err() {
                                    info!("Receiver dropped, stopping pipeline");
                                    self.running = false;
                                    break;
                                }
                            }
                        }
                        Err(e) => error!("Encode error: {e}"),
                    }

                    // Adaptive bitrate: if encoding takes too long, reduce quality
                    if self.encode_time_ms > (1000.0 / self.fps as f64) * 0.8 {
                        self.target_bitrate = (self.target_bitrate as f64 * 0.9) as u32;
                        warn!(
                            "Encoding too slow: {:.1}ms, reducing bitrate to {}",
                            self.encode_time_ms, self.target_bitrate
                        );
                    }
                }
                Err(e) => error!("Capture error: {e}"),
            }

            // Rate limiting — sleep remaining frame time
            let elapsed = frame_start.elapsed();
            if elapsed < interval {
                tokio::time::sleep(interval - elapsed).await;
            }
        }

        info!("Stream pipeline stopped after {} frames", self.frame_count);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn set_bitrate(&mut self, bitrate: u32) {
        self.target_bitrate = bitrate;
    }

    pub fn set_sps_pps(&mut self, data: Vec<u8>) {
        self.sps_pps = Some(data);
    }
}
