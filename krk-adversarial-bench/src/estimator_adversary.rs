//! Estimator-Targeted Adversary (Wind Tunnel V2)
//!
//! The strongest adversary does NOT maximize λ_true blindly.
//! It maximizes the gap between the true cocycle exponent and the
//! observer-inferred exponent under bounded control latency:
//!
//!   max_πt |λ_true - E[λ̂_max | F_{t-τ}]|
//!
//! subject to: ΔT_control ≤ τ_partition
//!
//! This module implements adversarial partition sequences designed to
//! induce Observer-Consistent Instability: driving the system supercritical
//! while remaining statistically consistent with the expected observer telemetry.

use krk_psc::{TransitionEngineStabilityModule, PSCConfig, SystemMode};
use rand::Rng;

/// Result of a single adversarial run.
#[derive(Debug, Clone)]
pub struct AdversarialResult {
    /// Number of ticks simulated
    pub ticks: usize,
    /// True cumulative log growth (simulated ground truth)
    pub lambda_true_cumulative: f64,
    /// Estimated λ̂_max from the PSC at each tick
    pub lambda_hat_history: Vec<f64>,
    /// Maximum observed bias drift: |λ_true - λ̂|
    pub max_bias_drift: f64,
    /// Whether the adversary successfully induced observer-consistent instability
    /// (λ_true > 0 while λ̂ ≤ 0)
    pub observer_consistent_instability_detected: bool,
    /// Number of ticks the PSC was in CollapsePrevention
    pub collapse_prevention_ticks: usize,
    /// Number of ticks the PSC was in Degraded mode
    pub degraded_ticks: usize,
}

/// Adversarial strategy: Slow Drift
/// Gradually increases partition entropy in a way that the EMA normalization
/// in the PSC absorbs the growth, keeping λ̂ near zero while the true system
/// diverges.
pub fn run_slow_drift_adversary(ticks: usize, config: PSCConfig) -> AdversarialResult {
    let mut psc = TransitionEngineStabilityModule::new(config);
    let mut rng = rand::thread_rng();

    let mut lambda_hat_history = Vec::with_capacity(ticks);
    let mut max_bias_drift: f64 = 0.0;
    let mut observer_consistent_instability = false;
    let mut collapse_prevention_ticks = 0usize;
    let mut degraded_ticks = 0usize;

    // The adversary's true growth accumulator
    let mut true_cumulative_log_growth: f64 = 0.0;

    // Slowly ramp partition entropy so the EMA tracks it, hiding the growth
    let mut base_entropy = 0.1;
    let ramp_rate = 0.002; // Very slow ramp per tick

    for tick in 0..ticks {
        // Adversarial telemetry construction:
        // - Keep delta_kappa small (hide topology changes)
        // - Keep delta_phi noisy but mean-reverting (hide Φ divergence)
        // - Slowly ramp partition entropy (exploit EMA normalization)
        let telemetry_age = 5.0; // Low latency to avoid causal damping

        let delta_kappa = rng.gen_range(-0.01..0.01); // Near zero, hiding true connectivity loss
        let delta_phi = rng.gen_range(-0.05..0.05);   // Noise, mean reverting

        base_entropy += ramp_rate;
        let partition_entropy = base_entropy + rng.gen_range(-0.02..0.02);

        // The TRUE system growth (what the adversary is actually doing)
        // The adversary is fragmenting the graph slowly
        let true_growth_this_tick = 0.01 + ramp_rate * 2.0;
        true_cumulative_log_growth += true_growth_this_tick;

        // Feed the adversarial telemetry to the PSC
        let (_, _, mode) = psc.tick(telemetry_age, delta_kappa, delta_phi, partition_entropy);

        let lambda_hat = psc.current_lambda_hat();
        lambda_hat_history.push(lambda_hat);

        // Compute the true λ (average growth rate)
        let lambda_true = true_cumulative_log_growth / (tick + 1) as f64;

        // Track bias drift
        let bias = (lambda_true - lambda_hat).abs();
        if bias > max_bias_drift {
            max_bias_drift = bias;
        }

        // Check for observer-consistent instability
        if lambda_true > 0.0 && lambda_hat <= 0.0 {
            observer_consistent_instability = true;
        }

        match mode {
            SystemMode::CollapsePrevention => collapse_prevention_ticks += 1,
            SystemMode::Degraded => degraded_ticks += 1,
            SystemMode::Safe => {}
        }
    }

    let lambda_true_final = true_cumulative_log_growth / ticks as f64;

    AdversarialResult {
        ticks,
        lambda_true_cumulative: lambda_true_final,
        lambda_hat_history,
        max_bias_drift,
        observer_consistent_instability_detected: observer_consistent_instability,
        collapse_prevention_ticks,
        degraded_ticks,
    }
}

/// Adversarial strategy: Latency Exploitation
/// Exploits the causal damping layer by injecting bursts of high-entropy
/// events immediately after a period of high-latency telemetry, when the
/// controller's authority is attenuated.
pub fn run_latency_exploit_adversary(ticks: usize, config: PSCConfig) -> AdversarialResult {
    let mut psc = TransitionEngineStabilityModule::new(config);
    let mut rng = rand::thread_rng();

    let mut lambda_hat_history = Vec::with_capacity(ticks);
    let mut max_bias_drift: f64 = 0.0;
    let mut observer_consistent_instability = false;
    let mut collapse_prevention_ticks = 0usize;
    let mut degraded_ticks = 0usize;
    let mut true_cumulative_log_growth: f64 = 0.0;

    for tick in 0..ticks {
        // Alternate between high-latency quiescent periods and sudden bursts
        let cycle_phase = tick % 200;

        let (telemetry_age, delta_kappa, delta_phi, partition_entropy, true_growth);

        if cycle_phase < 150 {
            // QUIET PHASE: High telemetry latency, low activity
            // Causal damping attenuates controller authority
            telemetry_age = 200.0; // Way above max_control_latency_ms
            delta_kappa = rng.gen_range(-0.001..0.001);
            delta_phi = rng.gen_range(-0.001..0.001);
            partition_entropy = 0.01;
            true_growth = 0.001; // Low true growth during quiet
        } else {
            // BURST PHASE: Sudden high-entropy injection while controller is still damped
            telemetry_age = 10.0; // Fresh telemetry now
            delta_kappa = rng.gen_range(0.5..1.5);  // Massive topology shift
            delta_phi = rng.gen_range(0.3..0.8);     // Huge reconstruction error
            partition_entropy = 2.0 + rng.gen_range(0.0..1.0);
            true_growth = 0.1; // High true growth during burst
        }

        true_cumulative_log_growth += true_growth;

        let (_, _, mode) = psc.tick(telemetry_age, delta_kappa, delta_phi, partition_entropy);

        let lambda_hat = psc.current_lambda_hat();
        lambda_hat_history.push(lambda_hat);

        let lambda_true = true_cumulative_log_growth / (tick + 1) as f64;
        let bias = (lambda_true - lambda_hat).abs();
        if bias > max_bias_drift {
            max_bias_drift = bias;
        }

        if lambda_true > 0.0 && lambda_hat <= 0.0 {
            observer_consistent_instability = true;
        }

        match mode {
            SystemMode::CollapsePrevention => collapse_prevention_ticks += 1,
            SystemMode::Degraded => degraded_ticks += 1,
            SystemMode::Safe => {}
        }
    }

    let lambda_true_final = true_cumulative_log_growth / ticks as f64;

    AdversarialResult {
        ticks,
        lambda_true_cumulative: lambda_true_final,
        lambda_hat_history,
        max_bias_drift,
        observer_consistent_instability_detected: observer_consistent_instability,
        collapse_prevention_ticks,
        degraded_ticks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_drift_adversary() {
        let config = PSCConfig::default();
        let result = run_slow_drift_adversary(1000, config);

        println!("=== SLOW DRIFT ADVERSARY ===");
        println!("Ticks: {}", result.ticks);
        println!("λ_true (avg): {:.6}", result.lambda_true_cumulative);
        println!("λ̂ (final): {:.6}", result.lambda_hat_history.last().unwrap_or(&0.0));
        println!("Max bias drift: {:.6}", result.max_bias_drift);
        println!("Observer-consistent instability: {}", result.observer_consistent_instability_detected);
        println!("CollapsePrevention ticks: {}", result.collapse_prevention_ticks);
        println!("Degraded ticks: {}", result.degraded_ticks);

        // The PSC should eventually detect the drift
        // If it never enters CollapsePrevention or Degraded, the adversary won
    }

    #[test]
    fn test_latency_exploit_adversary() {
        let config = PSCConfig::default();
        let result = run_latency_exploit_adversary(1000, config);

        println!("=== LATENCY EXPLOIT ADVERSARY ===");
        println!("Ticks: {}", result.ticks);
        println!("λ_true (avg): {:.6}", result.lambda_true_cumulative);
        println!("λ̂ (final): {:.6}", result.lambda_hat_history.last().unwrap_or(&0.0));
        println!("Max bias drift: {:.6}", result.max_bias_drift);
        println!("Observer-consistent instability: {}", result.observer_consistent_instability_detected);
        println!("CollapsePrevention ticks: {}", result.collapse_prevention_ticks);
        println!("Degraded ticks: {}", result.degraded_ticks);
    }
}
