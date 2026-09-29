//! Kraken Phase-Stabilizing Controller (PSC)
//!
//! Implements the runtime-safe stabilization loop for the \kappa-\Phi cocycle.
//! Forces the system to maintain \lambda_{max} < 0 under adversarial partition noise.
//!
//! HARDENING (Wind Tunnel V2 findings):
//! - Minimum entropy floor: prevents EMA from absorbing slow-ramp adversarial growth
//! - Monotonic trend detector: detects sustained growth in the EMA itself
//! - Latency-compensated estimator: boosts sensitivity when causal damping is active
//!
//! HARDENING v3 (Observer-Controller Mismatch Fix):
//! - Dual-timescale observer: fast channel (burst detection) + slow channel (trend estimation)
//! - Innovation residual test: detects adversarial regime switching via prediction error

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemMode {
    /// \lambda_{max} < 0: Full propagation enabled.
    Safe,
    /// \lambda_{max} \approx 0: Throttled gossip, delayed \Phi updates.
    Degraded,
    /// \lambda_{max} > 0: Freeze partition merging, limit edge expansion, force \kappa contraction.
    CollapsePrevention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityAttestation {
    pub lambda_hat: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PSCConfig {
    /// Sliding window size (k)
    pub window_size: usize,
    /// \alpha: weight of partition entropy in gain proxy
    pub alpha: f64,
    /// \epsilon: regularization for log and expectation
    pub epsilon: f64,
    /// \delta: stability margin (e.g., 0.05)
    pub delta: f64,
    /// k1: \kappa-channel control gain
    pub k1: f64,
    /// k2: \Phi-channel control gain
    pub k2: f64,
    /// Expected base gain (for dimensionless normalization)
    pub expected_g: f64,
    /// Maximum allowed telemetry age before control actions become causally invalid
    pub max_control_latency_ms: f64,
    /// HARDENING: Maximum allowed ratio of running_e_g to initial expected_g.
    /// Prevents the EMA from absorbing slow adversarial ramps.
    pub max_ema_inflation: f64,
    /// HARDENING: Number of consecutive ticks of EMA growth before triggering alert.
    pub trend_alert_window: usize,
    /// v3: Fast channel EMA coefficient (higher = less smoothing, faster response)
    pub fast_ema_alpha: f64,
    /// v3: Slow channel EMA coefficient (lower = more smoothing, trend detection)
    pub slow_ema_alpha: f64,
    /// v3: Innovation residual spike threshold (multiples of residual stddev)
    pub innovation_spike_threshold: f64,
    /// v3: Innovation autocorrelation window for drift detection
    pub innovation_window: usize,
}

impl Default for PSCConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            alpha: 1.0,
            epsilon: 1e-6,
            delta: 0.05,
            k1: 0.1,
            k2: 0.1,
            expected_g: 1.0,
            max_control_latency_ms: 100.0,
            max_ema_inflation: 3.0,
            trend_alert_window: 20,
            fast_ema_alpha: 0.3,      // Responsive: reacts in ~3 ticks
            slow_ema_alpha: 0.02,     // Stable: reacts in ~50 ticks
            innovation_spike_threshold: 3.0,  // 3-sigma spike detection
            innovation_window: 20,    // Residual autocorrelation window
        }
    }
}

/// Dual-timescale observer for adversarial-robust estimation.
/// Fast channel detects burst entropy (no smoothing, high variance tolerated).
/// Slow channel detects structural trends (low noise bias).
/// Fused estimate: λ̂ = w_f · λ_f + w_s · λ_s
#[derive(Debug)]
struct DualTimescaleObserver {
    /// Fast channel: high-frequency, burst-sensitive
    fast_estimate: f64,
    fast_alpha: f64,
    /// Slow channel: low-frequency, trend-sensitive
    slow_estimate: f64,
    slow_alpha: f64,
    /// Fusion weights (dynamic, shift toward fast when residual spikes)
    fast_weight: f64,
}

impl DualTimescaleObserver {
    fn new(fast_alpha: f64, slow_alpha: f64) -> Self {
        Self {
            fast_estimate: 0.0,
            fast_alpha,
            slow_estimate: 0.0,
            slow_alpha,
            fast_weight: 0.5,  // Equal weight initially
        }
    }

    fn update(&mut self, observation: f64, residual_spike: bool) {
        // Fast channel: high alpha = less smoothing
        self.fast_estimate = self.fast_alpha * observation
            + (1.0 - self.fast_alpha) * self.fast_estimate;

        // Slow channel: low alpha = more smoothing
        self.slow_estimate = self.slow_alpha * observation
            + (1.0 - self.slow_alpha) * self.slow_estimate;

        // Dynamic fusion: shift toward fast channel when adversarial regime detected
        if residual_spike {
            self.fast_weight = (self.fast_weight + 0.1).min(0.9);
        } else {
            self.fast_weight = (self.fast_weight - 0.02).max(0.3);
        }
    }

    fn fused_estimate(&self) -> f64 {
        let slow_weight = 1.0 - self.fast_weight;
        self.fast_weight * self.fast_estimate + slow_weight * self.slow_estimate
    }
}

/// Innovation residual test (Kalman-style).
/// Monitors r(t) = λ_observed(t) - λ_predicted(t | model).
/// If |r(t)| spikes → adversarial regime.
/// If residual autocorrelates → drift attack.
#[derive(Debug)]
struct InnovationResidualTest {
    residuals: VecDeque<f64>,
    window_size: usize,
    spike_threshold: f64,
    running_variance: f64,
}

impl InnovationResidualTest {
    fn new(window_size: usize, spike_threshold: f64) -> Self {
        Self {
            residuals: VecDeque::with_capacity(window_size),
            window_size,
            spike_threshold,
            running_variance: 0.01,  // Initial variance estimate
        }
    }

    /// Update with new residual and return (is_spike, is_autocorrelated)
    fn update(&mut self, observed: f64, predicted: f64) -> (bool, bool) {
        let residual = observed - predicted;
        self.residuals.push_back(residual);
        if self.residuals.len() > self.window_size {
            self.residuals.pop_front();
        }

        // Update running variance estimate
        let mean: f64 = self.residuals.iter().sum::<f64>()
            / self.residuals.len().max(1) as f64;
        let variance: f64 = self.residuals.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / self.residuals.len().max(1) as f64;
        self.running_variance = 0.9 * self.running_variance + 0.1 * variance;

        // Spike detection: |r(t)| > threshold × σ
        let stddev = self.running_variance.sqrt().max(0.001);
        let is_spike = residual.abs() > self.spike_threshold * stddev;

        // Autocorrelation detection: if residuals are consistently same-sign,
        // the adversary is injecting a drift attack
        let is_autocorrelated = if self.residuals.len() >= 5 {
            let recent: Vec<_> = self.residuals.iter().rev().take(5).collect();
            let same_sign = recent.iter().all(|&&r| r > 0.0)
                || recent.iter().all(|&&r| r < 0.0);
            same_sign
        } else {
            false
        };

        (is_spike, is_autocorrelated)
    }

    /// Get the current innovation residual for diagnostics
    fn current_residual(&self) -> f64 {
        self.residuals.back().copied().unwrap_or(0.0)
    }
}

pub struct TransitionEngineStabilityModule {
    config: PSCConfig,
    log_energy_window: VecDeque<f64>,
    /// Running estimation of \mathbb{E}[g_t]
    running_e_g: f64,
    /// Initial expected_g (for floor/ceiling enforcement)
    initial_e_g: f64,
    current_lambda_hat: f64,
    /// HARDENING: Consecutive ticks where EMA has been growing
    ema_growth_streak: usize,
    /// HARDENING: Previous EMA value for trend detection
    prev_running_e_g: f64,
    /// HARDENING: Bias correction term (accumulated latency-induced estimator lag)
    bias_correction: f64,
    /// v3: Dual-timescale observer
    dual_observer: DualTimescaleObserver,
    /// v3: Innovation residual test
    innovation_test: InnovationResidualTest,
    /// v3: Previous raw λ estimate (for innovation prediction)
    prev_raw_lambda: f64,
    /// v3: Whether adversarial regime has been detected
    adversarial_regime_detected: bool,
}

impl TransitionEngineStabilityModule {
    pub fn new(config: PSCConfig) -> Self {
        let expected_g = config.expected_g;
        let dual_observer = DualTimescaleObserver::new(
            config.fast_ema_alpha,
            config.slow_ema_alpha,
        );
        let innovation_test = InnovationResidualTest::new(
            config.innovation_window,
            config.innovation_spike_threshold,
        );
        Self {
            config,
            log_energy_window: VecDeque::new(),
            running_e_g: expected_g,
            initial_e_g: expected_g,
            current_lambda_hat: 0.0,
            ema_growth_streak: 0,
            prev_running_e_g: expected_g,
            bias_correction: 0.0,
            dual_observer,
            innovation_test,
            prev_raw_lambda: 0.0,
            adversarial_regime_detected: false,
        }
    }

    /// Primary execution loop per tick
    pub fn tick(
        &mut self,
        telemetry_age_ms: f64,
        delta_kappa: f64,
        delta_phi: f64,
        partition_entropy: f64,
    ) -> (f64, f64, SystemMode) {
        // CAUSAL STABILITY DAMPING (Soft Attenuation Layer)
        let latency_ratio = telemetry_age_ms / self.config.max_control_latency_ms;
        let causal_damping = (-(latency_ratio.powi(2)) * 2.0).exp();

        // 1. ONLINE ESTIMATOR
        // Raw gain proxy
        let g_t = delta_kappa.abs()
            + delta_phi.abs()
            + (self.config.alpha * partition_entropy);

        // Update running expectation (simple exponential moving average)
        self.prev_running_e_g = self.running_e_g;
        self.running_e_g = 0.9 * self.running_e_g + 0.1 * g_t;

        // HARDENING #1: Minimum Entropy Floor / Ceiling
        let ema_ceiling = self.initial_e_g * self.config.max_ema_inflation;
        if self.running_e_g > ema_ceiling {
            self.running_e_g = ema_ceiling;
        }
        let ema_floor = self.initial_e_g * 0.1;
        if self.running_e_g < ema_floor {
            self.running_e_g = ema_floor;
        }

        // HARDENING #2: Monotonic Trend Detector
        if self.running_e_g > self.prev_running_e_g + self.config.epsilon {
            self.ema_growth_streak += 1;
        } else {
            self.ema_growth_streak = 0;
        }

        let trend_bias = if self.ema_growth_streak >= self.config.trend_alert_window {
            let streak_factor = (self.ema_growth_streak - self.config.trend_alert_window) as f64;
            (streak_factor * 0.005).min(0.1)
        } else {
            0.0
        };

        // HARDENING #3: Latency-Compensated Estimator
        let latency_bias = if causal_damping < 0.5 {
            (1.0 - causal_damping) * 0.02
        } else {
            0.0
        };

        // Accumulate bias correction (exponentially decaying)
        self.bias_correction = 0.95 * self.bias_correction + trend_bias + latency_bias;

        // Dimensionless normalization
        let g_tilde = g_t / (self.running_e_g + self.config.epsilon);
        let log_energy_t = (g_tilde + self.config.epsilon).ln();

        self.log_energy_window.push_back(log_energy_t);
        if self.log_energy_window.len() > self.config.window_size {
            self.log_energy_window.pop_front();
        }

        // Calculate raw \hat{\lambda}_{max}
        let sum: f64 = self.log_energy_window.iter().sum();
        let raw_lambda = sum / self.log_energy_window.len().max(1) as f64;

        // ──── v3 FIX 1: DUAL-TIMESCALE OBSERVER ────
        // Feed raw_lambda into both fast and slow channels.
        // Fast channel catches burst attacks; slow channel catches drift.
        let (is_spike, is_autocorrelated) = self.innovation_test.update(
            raw_lambda,
            self.prev_raw_lambda,
        );

        self.dual_observer.update(raw_lambda, is_spike);
        let fused_lambda = self.dual_observer.fused_estimate();

        // ──── v3 FIX 2: INNOVATION RESIDUAL TEST ────
        // Detect adversarial regime switching via prediction error statistics.
        let adversarial_bias = if is_spike {
            // Burst attack detected: inject pessimistic correction
            0.05
        } else if is_autocorrelated {
            // Drift attack detected: inject moderate correction
            0.03
        } else {
            0.0
        };

        self.adversarial_regime_detected = is_spike || is_autocorrelated;

        // Store for next tick's prediction
        self.prev_raw_lambda = raw_lambda;

        // Final λ̂: fuse dual-timescale estimate with all bias corrections
        self.current_lambda_hat = fused_lambda
            + self.bias_correction
            + adversarial_bias;

        // 2. CONTROL LAW SYNTHESIS
        let e_t = self.current_lambda_hat + self.config.delta;
        let activation = if e_t > 0.0 { e_t } else { 0.0 };

        // Apply causal damping to the control vectors
        let u_gossip = -self.config.k1 * activation * causal_damping;
        let u_collapse = self.config.k2 * activation * causal_damping;

        // 3. STABILITY GATE
        let mode = if causal_damping < 0.2 {
            SystemMode::Degraded
        } else if self.adversarial_regime_detected && self.current_lambda_hat > 0.0 {
            // v3: Adversarial regime + positive λ̂ → immediate collapse prevention
            SystemMode::CollapsePrevention
        } else if self.current_lambda_hat > self.config.delta {
            SystemMode::CollapsePrevention
        } else if self.current_lambda_hat > -self.config.delta {
            SystemMode::Degraded
        } else {
            SystemMode::Safe
        };

        (u_gossip, u_collapse, mode)
    }

    pub fn current_lambda_hat(&self) -> f64 {
        self.current_lambda_hat
    }

    /// Get the current bias correction being applied
    pub fn bias_correction(&self) -> f64 {
        self.bias_correction
    }

    /// Get the current EMA growth streak
    pub fn ema_growth_streak(&self) -> usize {
        self.ema_growth_streak
    }

    /// v3: Whether an adversarial regime has been detected this tick
    pub fn adversarial_regime_detected(&self) -> bool {
        self.adversarial_regime_detected
    }

    /// v3: Current innovation residual
    pub fn innovation_residual(&self) -> f64 {
        self.innovation_test.current_residual()
    }

    /// v3: Current dual-observer fusion weights
    pub fn observer_fast_weight(&self) -> f64 {
        self.dual_observer.fast_weight
    }

    /// Sign the current stability state (Placeholder for TEE)
    pub fn attest_stability(&self, timestamp: u64) -> StabilityAttestation {
        StabilityAttestation {
            lambda_hat: self.current_lambda_hat,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_observer_burst_detection() {
        let mut psc = TransitionEngineStabilityModule::new(PSCConfig::default());
        // Extended warm-up to establish stable baseline
        // (first ticks will naturally have same-sign residuals during transition)
        for _ in 0..100 {
            psc.tick(10.0, 0.01, 0.01, 0.1);
        }
        // After 100 ticks of stable operation, regime should be calm
        // (autocorrelation clears as variance stabilizes)

        // Sudden burst: 100x normal amplitude
        psc.tick(10.0, 1.0, 1.0, 5.0);
        psc.tick(10.0, 1.0, 1.0, 5.0);
        psc.tick(10.0, 1.0, 1.0, 5.0);
        // After sustained burst, adversarial detection should trigger
        assert!(psc.adversarial_regime_detected(),
            "Burst attack should be detected by innovation residual or autocorrelation");
    }

    #[test]
    fn test_dual_observer_drift_detection() {
        let mut psc = TransitionEngineStabilityModule::new(PSCConfig::default());
        // Normal baseline
        for _ in 0..50 {
            psc.tick(10.0, 0.01, 0.01, 0.1);
        }
        // Slow consistent ramp (all residuals same sign)
        for i in 0..30 {
            let ramp = 0.1 + (i as f64 * 0.02);
            psc.tick(10.0, ramp, ramp, ramp);
        }
        // At some point during the ramp, autocorrelation should trigger
        assert!(psc.adversarial_regime_detected(),
            "Drift attack should be detected by innovation autocorrelation");
    }

    #[test]
    fn test_safe_under_normal_operation() {
        let mut psc = TransitionEngineStabilityModule::new(PSCConfig::default());
        // Stable low-entropy operation
        for _ in 0..100 {
            let (_, _, mode) = psc.tick(10.0, 0.01, 0.01, 0.05);
            assert_ne!(mode, SystemMode::CollapsePrevention,
                "Normal operation should never trigger collapse prevention");
        }
    }
}
