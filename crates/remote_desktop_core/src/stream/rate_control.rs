/// Adaptive bitrate controller
pub struct RateController {
    min_bitrate: u32,
    max_bitrate: u32,
    current_bitrate: u32,
    step_factor: f64,
}

impl RateController {
    pub fn new(min_bitrate: u32, max_bitrate: u32, initial_bitrate: u32) -> Self {
        Self {
            min_bitrate,
            max_bitrate,
            current_bitrate: initial_bitrate,
            step_factor: 0.1,
        }
    }

    /// Called when encoding is too slow — reduce bitrate
    pub fn decrease(&mut self) {
        self.current_bitrate = (self.current_bitrate as f64 * (1.0 - self.step_factor)) as u32;
        self.current_bitrate = self.current_bitrate.max(self.min_bitrate);
    }

    /// Called when encoding is fast enough — increase bitrate
    pub fn increase(&mut self) {
        self.current_bitrate = (self.current_bitrate as f64 * (1.0 + self.step_factor)) as u32;
        self.current_bitrate = self.current_bitrate.min(self.max_bitrate);
    }

    pub fn bitrate(&self) -> u32 {
        self.current_bitrate
    }
}
