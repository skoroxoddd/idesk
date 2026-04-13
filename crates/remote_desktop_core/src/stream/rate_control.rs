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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_controller_decrease() {
        let mut rc = RateController::new(500_000, 5_000_000, 2_000_000);
        rc.decrease();
        assert_eq!(rc.bitrate(), 1_800_000); // 2M * 0.9
    }

    #[test]
    fn rate_controller_increase() {
        let mut rc = RateController::new(500_000, 5_000_000, 2_000_000);
        rc.increase();
        assert_eq!(rc.bitrate(), 2_200_000); // 2M * 1.1
    }

    #[test]
    fn rate_controller_respects_bounds() {
        let mut rc = RateController::new(500_000, 5_000_000, 2_000_000);

        // Decrease below minimum
        for _ in 0..20 {
            rc.decrease();
        }
        assert_eq!(rc.bitrate(), 500_000);

        // Increase above maximum
        let mut rc2 = RateController::new(500_000, 5_000_000, 4_500_000);
        for _ in 0..20 {
            rc2.increase();
        }
        assert_eq!(rc2.bitrate(), 5_000_000);
    }
}
