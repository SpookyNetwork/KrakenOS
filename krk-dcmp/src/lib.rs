//! Distributed Constitutional Messaging Protocol (DCMP)
//!
//! QUIC-based constraint lattice reduction protocol replacing probabilistic consensus.
//!
//! ACTUATOR LAYER:
//! The GossipTopologyActuator and CollapseRegulationActuator implement the physical
//! mechanisms that the PSC's control signals (u_gossip, u_collapse) drive.

use serde::{Deserialize, Serialize};

pub type Hash256 = [u8; 32];
pub type NodeId = Hash256;
pub type BlsSignature = Vec<u8>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageType {
    Boot,
    Attestation,
    StateDelta,
    Snapshot,
    PressureVector,
    GovernanceUpdate,
    InvariantViolation,
    Reconciliation,
    Halt,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DcmpEnvelope {
    pub version: u16,
    pub epoch: u64,
    pub hlc: u128, // Hybrid Logical Clock

    pub sender: NodeId,
    pub message_type: MessageType,

    pub payload_hash: Hash256,
    pub payload: Vec<u8>,

    pub signature: BlsSignature,
}

pub struct Proposal {
    pub delta: Vec<u8>,
    pub epoch_id: u64,
}

/// Abstract QUIC-backed mesh adapter. Every edge requires SAT-validation
/// before synchronization is allowed.
pub struct DcmpMeshAdapter {
    // Under the hood, this will hold quinn::Endpoint and manage connections.
}

impl DcmpMeshAdapter {
    pub fn new() -> Self {
        Self {}
    }

    /// Verifies the structural and cryptographic validity of an incoming DCMP envelope.
    pub fn verify_envelope(&self, _env: &DcmpEnvelope) -> bool {
        // Must perform: SAT(attestation ∧ governance_hash ∧ protocol_version ∧ epoch)
        true
    }
}

// ─── PSC ACTUATORS ───

/// The Gossip Topology Actuator: implements u_gossip from the PSC.
/// Controls peer connections, edge dropping, and κ manipulation.
///
/// When u_gossip < 0: contract topology (drop low-priority peers, increase stiffness)
/// When u_gossip ≈ 0: maintain current topology
/// When u_gossip > 0: expand topology (accept new peers, reduce barriers)
pub struct GossipTopologyActuator {
    /// Current number of active peer connections
    pub active_peers: usize,
    /// Maximum allowed peers (hard cap)
    pub max_peers: usize,
    /// Minimum allowed peers (safety floor)
    pub min_peers: usize,
    /// Current gossip fan-out (how many peers receive each message)
    pub fan_out: usize,
    /// Current gossip interval (ms between rounds)
    pub gossip_interval_ms: u64,
}

impl GossipTopologyActuator {
    pub fn new(initial_peers: usize) -> Self {
        Self {
            active_peers: initial_peers,
            max_peers: 128,
            min_peers: 3,
            fan_out: 4,
            gossip_interval_ms: 100,
        }
    }

    /// Apply the PSC's u_gossip control signal to the topology.
    /// Returns the new peer target count.
    pub fn apply_control(&mut self, u_gossip: f64) -> TopologyAction {
        if u_gossip < -0.3 {
            // Heavy contraction: drop peers, reduce fan-out, slow gossip
            let drop_count = ((-u_gossip * 5.0) as usize).min(self.active_peers.saturating_sub(self.min_peers));
            self.fan_out = self.fan_out.saturating_sub(1).max(1);
            self.gossip_interval_ms = (self.gossip_interval_ms + 50).min(1000);
            TopologyAction::Contract { drop_peers: drop_count }
        } else if u_gossip < -0.05 {
            // Mild contraction: slow gossip slightly
            self.gossip_interval_ms = (self.gossip_interval_ms + 10).min(500);
            TopologyAction::Throttle
        } else if u_gossip > 0.1 {
            // Expansion: accept new peers, increase fan-out
            let add_count = ((u_gossip * 3.0) as usize).min(self.max_peers - self.active_peers);
            self.fan_out = (self.fan_out + 1).min(8);
            self.gossip_interval_ms = self.gossip_interval_ms.saturating_sub(10).max(50);
            TopologyAction::Expand { add_peers: add_count }
        } else {
            TopologyAction::Maintain
        }
    }
}

#[derive(Debug, Clone)]
pub enum TopologyAction {
    /// Drop peers to contract the gossip mesh
    Contract { drop_peers: usize },
    /// Slow down gossip without dropping peers
    Throttle,
    /// Maintain current topology
    Maintain,
    /// Accept new peers to expand the mesh
    Expand { add_peers: usize },
}

/// The Collapse Regulation Actuator: implements u_collapse from the PSC.
/// Controls the rate and aggressiveness of quotient projection (equivalence collapse).
///
/// When u_collapse > 0: accelerate collapse (batch more aggressively, force projection)
/// When u_collapse ≈ 0: maintain normal collapse rate
/// When u_collapse < 0: defer collapse (accumulate more evidence before projecting)
pub struct CollapseRegulationActuator {
    /// Current collapse batch size
    pub batch_size: usize,
    /// Maximum collapse batch size
    pub max_batch_size: usize,
    /// Minimum collapse batch size
    pub min_batch_size: usize,
    /// Current collapse delay (ms between collapse rounds)
    pub collapse_delay_ms: u64,
    /// Whether forced immediate collapse is active
    pub force_immediate: bool,
}

impl CollapseRegulationActuator {
    pub fn new() -> Self {
        Self {
            batch_size: 10,
            max_batch_size: 1000,
            min_batch_size: 1,
            collapse_delay_ms: 100,
            force_immediate: false,
        }
    }

    /// Apply the PSC's u_collapse control signal.
    pub fn apply_control(&mut self, u_collapse: f64) -> CollapseAction {
        if u_collapse > 0.5 {
            // Aggressive collapse: force immediate, max batch
            self.force_immediate = true;
            self.batch_size = self.max_batch_size;
            self.collapse_delay_ms = 0;
            CollapseAction::ForceImmediate
        } else if u_collapse > 0.1 {
            // Accelerated collapse: increase batch, reduce delay
            self.batch_size = (self.batch_size * 2).min(self.max_batch_size);
            self.collapse_delay_ms = self.collapse_delay_ms.saturating_sub(20).max(10);
            self.force_immediate = false;
            CollapseAction::Accelerate { batch_size: self.batch_size }
        } else if u_collapse < -0.1 {
            // Deferred collapse: shrink batch, increase delay
            self.batch_size = (self.batch_size / 2).max(self.min_batch_size);
            self.collapse_delay_ms = (self.collapse_delay_ms + 50).min(5000);
            self.force_immediate = false;
            CollapseAction::Defer { delay_ms: self.collapse_delay_ms }
        } else {
            self.force_immediate = false;
            CollapseAction::Normal
        }
    }
}

#[derive(Debug, Clone)]
pub enum CollapseAction {
    /// Force immediate collapse of all pending equivalence classes
    ForceImmediate,
    /// Increase collapse throughput
    Accelerate { batch_size: usize },
    /// Normal collapse rate
    Normal,
    /// Defer collapse to accumulate more evidence
    Defer { delay_ms: u64 },
}
