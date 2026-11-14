/// Shown on the prediction screen. This is an estimate, not a diagnosis.
pub const WELLNESS_DISCLAIMER: &str =
    "This is an estimate from your own logs. Ouroboros is not a medical device.";

/// How the tracker answered a prediction prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feedback {
    /// The period started on the predicted day.
    Started,
    /// The period has not started yet.
    NotYet,
    /// The period started before the window.
    StartedEarlier,
}

/// On-device mean and variance for cycle length and period length.
#[derive(Debug, Clone, PartialEq)]
pub struct CycleModel {
    /// Mean cycle length in days.
    pub cycle_mean: f64,
    /// Variance of cycle length. The prediction width never treats this as zero.
    pub cycle_var: f64,
    /// Mean period length in days.
    pub period_mean: f64,
    /// Variance of period length.
    pub period_var: f64,
    /// Number of observed cycles.
    pub samples: u32,
}

impl Default for CycleModel {
    fn default() -> Self {
        Self::new()
    }
}

impl CycleModel {
    /// Prior centered on a 28-day cycle and a 5-day period.
    pub fn new() -> Self {
        Self {
            cycle_mean: 28.0,
            cycle_var: 4.0,
            period_mean: 5.0,
            period_var: 1.0,
            samples: 0,
        }
    }

    /// Fold one observed cycle length into the mean and variance.
    pub fn observe_cycle(&mut self, length_days: f64) {
        self.samples += 1;
        let n = f64::from(self.samples);
        let delta = length_days - self.cycle_mean;
        self.cycle_mean += delta / n;
        let delta2 = length_days - self.cycle_mean;
        self.cycle_var = ((self.cycle_var * (n - 1.0)) + delta * delta2) / n;
        if self.cycle_var < 1.0 {
            self.cycle_var = 1.0;
        }
    }

    /// Apply a prompt answer. `NotYet` moves the mean later and does not shrink variance to zero.
    pub fn feedback(&mut self, kind: Feedback) {
        match kind {
            Feedback::Started => self.observe_cycle(self.cycle_mean),
            Feedback::NotYet => {
                self.cycle_mean += 1.0;
                self.cycle_var = (self.cycle_var + 1.0).max(1.0);
            }
            Feedback::StartedEarlier => {
                self.cycle_mean = (self.cycle_mean - 1.0).max(1.0);
                self.cycle_var = (self.cycle_var + 0.5).max(1.0);
            }
        }
    }

    /// Inclusive day window around the mean. The width is always greater than zero.
    pub fn next_window(&self) -> (f64, f64) {
        let width = self.cycle_var.sqrt().max(1.0);
        (self.cycle_mean - width, self.cycle_mean + width)
    }

    /// True when today is inside the predicted start window.
    pub fn prompt_due(&self, day: f64) -> bool {
        let (low, high) = self.next_window();
        day >= low && day <= high
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_history_stays_near_day_28() {
        let mut model = CycleModel::new();
        for _ in 0..6 {
            model.observe_cycle(28.0);
        }
        let (low, high) = model.next_window();
        assert!((model.cycle_mean - 28.0).abs() < 1.0);
        assert!(low < 28.0 && high > 28.0);
        assert!(high - low > 0.0);
    }

    #[test]
    fn not_yet_does_not_shrink_variance_to_zero() {
        let mut model = CycleModel::new();
        model.observe_cycle(28.0);
        let before = model.cycle_var;
        model.feedback(Feedback::NotYet);
        assert!(model.cycle_var >= before);
        assert!(model.cycle_var > 0.0);
        let (low, high) = model.next_window();
        assert!(high - low > 0.0);
    }

    #[test]
    fn single_cycle_has_non_zero_width() {
        let mut model = CycleModel::new();
        model.observe_cycle(30.0);
        let (low, high) = model.next_window();
        assert!(high - low > 0.0);
    }
}
