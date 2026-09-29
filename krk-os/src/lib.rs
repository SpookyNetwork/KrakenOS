//! KRK-OS: Closed-Loop Adaptive Control System over a Stochastic Causal DAG
//!
//! KRK-OS = (X, U, F, Y) where:
//!   X = State (Event DAG, equivalence classes, RFC, horizons)
//!   U = Control (skills → control signals via the Control Compiler)
//!   F = Dynamics (Kraken physics: κ-Φ coupling, collapse, partition noise)
//!   Y = Observation (Observability Manifold: κ, Φ~, λ_A, R_A)
//!
//! The system is a nonlinear adaptive controller over a quotient-valued
//! stochastic dynamical system. It regulates the conditions under which
//! truth becomes discoverable.

pub mod state;
pub mod control;
pub mod system;
