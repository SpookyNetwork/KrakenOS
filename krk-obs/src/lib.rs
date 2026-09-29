//! Kraken Observability Manifold (OM)
//!
//! Collects live DAG telemetry and produces TelemetrySnapshot values
//! for the Phase-Stabilizing Controller. All metrics are computed from
//! PRE-transition state to enforce the strict pre-commit control sequencing.
//!
//! Three telemetry channels:
//! - Topology Distortion Rate (|Δκ_t|): connectivity change across the local horizon
//! - Reconstruction Error (|ΔΦ_t|): delayed equivalence discovery frequency
//! - Partition Entropy: variance in message delay and edge churn

use std::time::Instant;

/// Tracks the rate of topology connectivity change across the local horizon.
/// Measures how fast the graph structure is deforming (|Δκ_t|).
pub struct TopologyTracker {
    /// Number of active peers at last measurement
    last_peer_count: usize,
    /// Number of reachable nodes at last measurement
    last_reachable_count: usize,
    /// Timestamp of last measurement
    last_measurement: Instant,
}

impl TopologyTracker {
    pub fn new(initial_peers: usize, initial_reachable: usize) -> Self {
        Self {
            last_peer_count: initial_peers,
            last_reachable_count: initial_reachable,
            last_measurement: Instant::now(),
        }
    }

    /// Update with current peer/reachability counts.
    /// Returns |Δκ_t|: the normalized rate of topology distortion.
    pub fn update(&mut self, current_peers: usize, current_reachable: usize) -> f64 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_measurement).as_secs_f64().max(1e-6);

        // Topology distortion = normalized change in connectivity
        let peer_delta = (current_peers as f64 - self.last_peer_count as f64).abs();
        let reach_delta = (current_reachable as f64 - self.last_reachable_count as f64).abs();

        // Normalize by previous counts to make dimensionless
        let peer_norm = self.last_peer_count.max(1) as f64;
        let reach_norm = self.last_reachable_count.max(1) as f64;

        let delta_kappa = (peer_delta / peer_norm + reach_delta / reach_norm) / dt;

        self.last_peer_count = current_peers;
        self.last_reachable_count = current_reachable;
        self.last_measurement = now;

        delta_kappa
    }
}

/// Tracks the rate of observability reconstruction error.
/// Measures how frequently delayed equivalence discovery and divergence resolution occur.
pub struct ReconstructionTracker {
    /// Number of equivalence classes discovered at last measurement
    last_equiv_classes: usize,
    /// Number of unresolved divergences at last measurement
    last_divergences: usize,
    /// Timestamp of last measurement
    last_measurement: Instant,
}

impl ReconstructionTracker {
    pub fn new() -> Self {
        Self {
            last_equiv_classes: 0,
            last_divergences: 0,
            last_measurement: Instant::now(),
        }
    }

    /// Update with current equivalence class and divergence counts.
    /// Returns |ΔΦ_t|: the rate of observability reconstruction error.
    pub fn update(&mut self, equiv_classes: usize, divergences: usize) -> f64 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_measurement).as_secs_f64().max(1e-6);

        // Reconstruction error = rate of new equivalence discovery + divergence change
        let equiv_delta = (equiv_classes as f64 - self.last_equiv_classes as f64).abs();
        let div_delta = (divergences as f64 - self.last_divergences as f64).abs();

        let delta_phi = (equiv_delta + div_delta) / dt;

        self.last_equiv_classes = equiv_classes;
        self.last_divergences = divergences;
        self.last_measurement = now;

        delta_phi
    }
}

/// Estimates partition entropy from message delay variance and edge churn.
/// This is the stochastic noise proxy that feeds the PSC gain calculation.
pub struct PartitionEntropyEstimator {
    /// Circular buffer of recent message delays (ms)
    delay_samples: Vec<f64>,
    /// Write position in the circular buffer
    write_pos: usize,
    /// Number of edge drops observed in the current window
    edge_churn_count: u64,
    /// Window size
    window_size: usize,
}

impl PartitionEntropyEstimator {
    pub fn new(window_size: usize) -> Self {
        Self {
            delay_samples: vec![0.0; window_size],
            write_pos: 0,
            edge_churn_count: 0,
            window_size,
        }
    }

    /// Record a message delay observation.
    pub fn record_delay(&mut self, delay_ms: f64) {
        self.delay_samples[self.write_pos] = delay_ms;
        self.write_pos = (self.write_pos + 1) % self.window_size;
    }

    /// Record an edge churn event (peer drop/add).
    pub fn record_edge_churn(&mut self) {
        self.edge_churn_count += 1;
    }

    /// Compute the current partition entropy estimate.
    /// Returns: variance(delays) + normalized edge churn rate.
    pub fn estimate(&mut self) -> f64 {
        // Compute variance of delay samples
        let n = self.delay_samples.len() as f64;
        let mean = self.delay_samples.iter().sum::<f64>() / n;
        let variance = self.delay_samples
            .iter()
            .map(|d| (d - mean).powi(2))
            .sum::<f64>()
            / n;

        // Normalize edge churn by window size
        let churn_rate = self.edge_churn_count as f64 / self.window_size as f64;

        // Reset churn counter for next window
        self.edge_churn_count = 0;

        variance + churn_rate
    }
}

/// The unified Observability Manifold collector.
/// Combines all three telemetry channels into a single interface.
pub struct ObservabilityManifold {
    pub topology: TopologyTracker,
    pub reconstruction: ReconstructionTracker,
    pub partition_entropy: PartitionEntropyEstimator,
    /// Timestamp of the last snapshot collection
    last_snapshot_time: Instant,
}

impl ObservabilityManifold {
    pub fn new(initial_peers: usize, initial_reachable: usize, entropy_window: usize) -> Self {
        Self {
            topology: TopologyTracker::new(initial_peers, initial_reachable),
            reconstruction: ReconstructionTracker::new(),
            partition_entropy: PartitionEntropyEstimator::new(entropy_window),
            last_snapshot_time: Instant::now(),
        }
    }

    /// Collect a full telemetry snapshot from the current DAG state.
    /// Returns (age_ms, delta_kappa, delta_phi, partition_entropy).
    pub fn collect(
        &mut self,
        current_peers: usize,
        current_reachable: usize,
        equiv_classes: usize,
        divergences: usize,
    ) -> (f64, f64, f64, f64) {
        let now = Instant::now();
        let age_ms = now.duration_since(self.last_snapshot_time).as_secs_f64() * 1000.0;
        self.last_snapshot_time = now;

        let delta_kappa = self.topology.update(current_peers, current_reachable);
        let delta_phi = self.reconstruction.update(equiv_classes, divergences);
        let entropy = self.partition_entropy.estimate();

        (age_ms, delta_kappa, delta_phi, entropy)
    }
}
