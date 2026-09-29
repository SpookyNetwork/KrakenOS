//! Kraken Adversarial Benchmark Harness
//! A distributed systems wind tunnel for causal entropy measurement.

pub mod partition_controller;
pub mod ambiguity_injector;
pub mod horizon_tracker;
pub mod branch_pressure_monitor;
pub mod collapse_profiler;
pub mod scenario_a_delayed_equivalence;
pub mod scenario_b_horizon_starvation;
pub mod scenario_c_branch_cascade;
pub mod scenario_d_replay_avalanche;
pub mod scenario_e_percolation_failure;
pub mod estimator_adversary;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScenarioResult {
    pub name: String,
    pub r_a: f64,
    pub v_h: f64,
    pub total_injected: usize,
    pub total_collapsed: usize,
    pub peak_worldlines: usize,
    pub peak_rfc: usize,
    pub collapse_amplification: f64,
    pub peak_collapse_spike_ms: f64,
    pub verdict: String,
}
