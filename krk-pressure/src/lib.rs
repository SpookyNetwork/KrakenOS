//! KRK Pressure Monitor
//!
//! Tracks partition pressure, branch pressure, and entropy injection rate.
//! Feeds the PSC telemetry pipeline as a partition entropy proxy.

use serde::{Deserialize, Serialize};

/// Pressure metrics collected from the DAG and network layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureVector {
    /// Number of unresolved branches in the DAG
    pub active_branches: usize,
    /// Rate of new branch creation (branches/tick)
    pub branch_creation_rate: f64,
    /// Number of pending RFC items
    pub rfc_backlog: usize,
    /// Message queue depth across all peers
    pub gossip_queue_depth: usize,
    /// Average round-trip latency to peers (ms)
    pub avg_peer_latency_ms: f64,
}

/// Pressure state tracker with windowed averaging.
pub struct PressureMonitor {
    window: Vec<PressureVector>,
    window_size: usize,
}

impl PressureMonitor {
    pub fn new(window_size: usize) -> Self {
        Self {
            window: Vec::with_capacity(window_size),
            window_size,
        }
    }

    pub fn record(&mut self, pressure: PressureVector) {
        if self.window.len() >= self.window_size {
            self.window.remove(0);
        }
        self.window.push(pressure);
    }

    /// Compute the partition entropy proxy from the pressure window.
    /// This feeds directly into the PSC telemetry as partition_entropy.
    pub fn partition_entropy_proxy(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }
        let n = self.window.len() as f64;

        // Variance of peer latency (delay variance component)
        let mean_latency = self.window.iter()
            .map(|p| p.avg_peer_latency_ms)
            .sum::<f64>() / n;
        let latency_variance = self.window.iter()
            .map(|p| (p.avg_peer_latency_ms - mean_latency).powi(2))
            .sum::<f64>() / n;

        // Edge churn component (branch creation as topology change)
        let mean_branch_rate = self.window.iter()
            .map(|p| p.branch_creation_rate)
            .sum::<f64>() / n;

        // RFC backlog as entropy accumulation
        let mean_rfc = self.window.iter()
            .map(|p| p.rfc_backlog as f64)
            .sum::<f64>() / n;

        latency_variance + mean_branch_rate + (mean_rfc * 0.01)
    }

    /// Compute the topology distortion rate |Δκ_t| from gossip queue changes.
    pub fn topology_distortion_rate(&self) -> f64 {
        if self.window.len() < 2 {
            return 0.0;
        }
        let last = &self.window[self.window.len() - 1];
        let prev = &self.window[self.window.len() - 2];

        let queue_delta = (last.gossip_queue_depth as f64 - prev.gossip_queue_depth as f64).abs();
        let branch_delta = (last.active_branches as f64 - prev.active_branches as f64).abs();

        queue_delta + branch_delta
    }

    pub fn latest(&self) -> Option<&PressureVector> {
        self.window.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressure_monitor() {
        let mut monitor = PressureMonitor::new(10);

        for i in 0..10 {
            monitor.record(PressureVector {
                active_branches: i * 2,
                branch_creation_rate: 0.1 * i as f64,
                rfc_backlog: i * 5,
                gossip_queue_depth: 100 + i * 10,
                avg_peer_latency_ms: 10.0 + i as f64,
            });
        }

        let entropy = monitor.partition_entropy_proxy();
        assert!(entropy > 0.0, "Entropy should be positive with varying pressure");

        let distortion = monitor.topology_distortion_rate();
        assert!(distortion > 0.0, "Distortion should be positive with changing queues");
    }
}
