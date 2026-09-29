pub mod transition;
pub mod lattice;

pub use transition::{TransitionEngine, CandidateState, TelemetrySnapshot, PSCTickResult};
pub use lattice::LatticeReducer;

// Re-export PSC types for downstream consumers
pub use krk_psc::{SystemMode, PSCConfig, StabilityAttestation};
