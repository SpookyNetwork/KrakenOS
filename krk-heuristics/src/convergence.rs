//! Admissibility Convergence Prediction
//!
//! Forecasts instability before a violation occurs by predicting
//! admissibility collapse trajectories.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceForecast {
    pub stable_probability: f64,
    pub divergence_probability: f64,
    pub byzantine_probability: f64,
    pub halt_probability: f64,
}

impl ConvergenceForecast {
    /// Determines if a trajectory is approaching an admissibility collapse.
    pub fn is_risky(&self) -> bool {
        self.halt_probability > 0.5 || self.byzantine_probability > 0.3
    }
}
