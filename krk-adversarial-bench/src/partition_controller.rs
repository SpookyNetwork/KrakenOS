//! Partition Controller
//!
//! Simulates network partitions, asymmetric visibility, delayed gossip,
//! and selective horizon starvation. Models causal geometry, not just latency.

use rand::Rng;
use std::collections::{HashMap, HashSet};

/// Represents a simulated node in the causal network
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct NodeId(pub String);

/// Defines the visibility relationship between nodes
#[derive(Debug, Clone)]
pub struct PartitionController {
    /// Which nodes can each node currently see?
    visibility_matrix: HashMap<String, HashSet<String>>,
    /// Artificial delivery delays per edge (ms)
    delivery_delays: HashMap<(String, String), f64>,
    /// Whether the bridge between clusters is active
    bridge_active: bool,
}

impl PartitionController {
    pub fn new() -> Self {
        Self {
            visibility_matrix: HashMap::new(),
            delivery_delays: HashMap::new(),
            bridge_active: false,
        }
    }

    /// Create a two-cluster partition topology
    pub fn create_partition(&mut self, cluster_a: &[&str], cluster_b: &[&str]) {
        // Nodes within a cluster can see each other
        for &a in cluster_a {
            let mut visible = HashSet::new();
            for &peer in cluster_a {
                visible.insert(peer.to_string());
            }
            self.visibility_matrix.insert(a.to_string(), visible);
        }
        for &b in cluster_b {
            let mut visible = HashSet::new();
            for &peer in cluster_b {
                visible.insert(peer.to_string());
            }
            self.visibility_matrix.insert(b.to_string(), visible);
        }
        self.bridge_active = false;
    }

    /// Restore bridge between clusters with optional delay
    pub fn restore_bridge(&mut self, cluster_a: &[&str], cluster_b: &[&str], delay_ms: f64) {
        for &a in cluster_a {
            for &b in cluster_b {
                self.visibility_matrix.entry(a.to_string()).or_default().insert(b.to_string());
                self.visibility_matrix.entry(b.to_string()).or_default().insert(a.to_string());
                self.delivery_delays.insert((a.to_string(), b.to_string()), delay_ms);
                self.delivery_delays.insert((b.to_string(), a.to_string()), delay_ms);
            }
        }
        self.bridge_active = true;
    }

    /// Create a sparse gossip mesh with limited connectivity
    pub fn create_sparse_mesh(&mut self, nodes: &[&str], connectivity: f64) {
        let mut rng = rand::thread_rng();
        for &a in nodes {
            let mut visible = HashSet::new();
            visible.insert(a.to_string());
            for &b in nodes {
                if a != b && rng.gen::<f64>() < connectivity {
                    visible.insert(b.to_string());
                }
            }
            self.visibility_matrix.insert(a.to_string(), visible);
        }
    }

    /// Selectively withhold event visibility from a node
    pub fn selectively_withhold(&mut self, node_id: &str, target: &str) {
        if let Some(visible) = self.visibility_matrix.get_mut(node_id) {
            visible.remove(target);
        }
    }

    /// Get the visible peers for a given node
    pub fn visible_peers(&self, node_id: &str) -> HashSet<String> {
        self.visibility_matrix.get(node_id).cloned().unwrap_or_default()
    }

    /// Check if bridge is active
    pub fn is_bridge_active(&self) -> bool {
        self.bridge_active
    }

    /// Get delivery delay between two nodes
    pub fn delivery_delay(&self, from: &str, to: &str) -> f64 {
        self.delivery_delays.get(&(from.to_string(), to.to_string())).copied().unwrap_or(0.0)
    }
}
