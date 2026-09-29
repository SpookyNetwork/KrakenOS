//! KRK-OS Control Layer
//!
//! The Control Compiler and Control Law Generator.
//! Converts skills/agents/policies into control vectors u(t) = (Δκ, ΔΦ, Δλ, Δσ).
//!
//! This is the missing bridge between "cognition" and "physics."
//! Skills are NOT procedures — they are control signal generators.

use serde::{Deserialize, Serialize};

// ─── Control Signal (THE CORE PRIMITIVE) ───

/// The unified control vector.
/// Every skill, agent, and policy ultimately compiles down to this.
///
/// u(t) → (Δκ, ΔΦ~, Δλ_A, Δσ_C)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ControlSignal {
    /// Change in topology connectivity (positive = strengthen gossip, negative = contract)
    pub delta_kappa: f64,
    /// Change in equivalence discoverability (positive = accelerate collapse)
    pub delta_phi: f64,
    /// Change in entropy injection rate (negative = reduce ambiguity pressure)
    pub delta_lambda: f64,
    /// Change in collapse cost (positive = more aggressive collapse)
    pub delta_sigma: f64,
}

impl ControlSignal {
    pub fn zero() -> Self {
        Self {
            delta_kappa: 0.0,
            delta_phi: 0.0,
            delta_lambda: 0.0,
            delta_sigma: 0.0,
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            delta_kappa: self.delta_kappa * factor,
            delta_phi: self.delta_phi * factor,
            delta_lambda: self.delta_lambda * factor,
            delta_sigma: self.delta_sigma * factor,
        }
    }

    pub fn add(&self, other: &ControlSignal) -> Self {
        Self {
            delta_kappa: self.delta_kappa + other.delta_kappa,
            delta_phi: self.delta_phi + other.delta_phi,
            delta_lambda: self.delta_lambda + other.delta_lambda,
            delta_sigma: self.delta_sigma + other.delta_sigma,
        }
    }

    /// Magnitude of the control vector
    pub fn magnitude(&self) -> f64 {
        (self.delta_kappa.powi(2)
            + self.delta_phi.powi(2)
            + self.delta_lambda.powi(2)
            + self.delta_sigma.powi(2))
        .sqrt()
    }
}

// ─── Observability Metrics ───

/// The observation vector Y = h(X)
/// What the controller sees of the plant state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Metrics {
    /// κ: graph connectivity (peer count / reachability)
    pub kappa: f64,
    /// Φ~: equivalence discoverability (fraction of resolved classes)
    pub phi: f64,
    /// λ_A: entropy injection rate (events/sec without equivalence)
    pub lambda_a: f64,
    /// C⊕: collapse cost (compute time per collapse operation)
    pub c_collapse: f64,
    /// R_A: phase ratio (κ × Φ~ / λ_A)
    pub r_a: f64,
}

impl Metrics {
    /// Compute the phase ratio from the raw metrics
    pub fn compute_phase_ratio(kappa: f64, phi: f64, lambda_a: f64) -> f64 {
        if lambda_a.abs() < 1e-10 {
            return f64::INFINITY;
        }
        (kappa * phi) / lambda_a
    }
}

// ─── Skill IR (Intermediate Representation) ───

/// A skill compiled into control IR.
/// Skills are not procedures — they are weighted control influences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillNode {
    pub name: String,
    pub effects: ControlSignal,
    pub preconditions: Vec<Precondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Precondition {
    /// R_A must be above this threshold
    PhaseRatioAbove(f64),
    /// R_A must be below this threshold
    PhaseRatioBelow(f64),
    /// κ must be above this threshold
    ConnectivityAbove(f64),
    /// λ_A must be below this threshold
    EntropyBelow(f64),
}

impl Precondition {
    pub fn satisfied(&self, metrics: &Metrics) -> bool {
        match self {
            Precondition::PhaseRatioAbove(t) => metrics.r_a > *t,
            Precondition::PhaseRatioBelow(t) => metrics.r_a < *t,
            Precondition::ConnectivityAbove(t) => metrics.kappa > *t,
            Precondition::EntropyBelow(t) => metrics.lambda_a < *t,
        }
    }
}

// ─── Control IR (The Compiler Chain) ───

/// Intermediate representation for compiled control programs.
/// Skill → IR → Optimization → ControlSignal → Actuator
#[derive(Debug, Clone)]
pub enum ControlIR {
    /// A single skill node
    Skill(SkillNode),
    /// Sequential composition of IR nodes
    Sequence(Vec<ControlIR>),
    /// Conditional branch based on metrics
    Branch {
        condition: Precondition,
        then_branch: Box<ControlIR>,
        else_branch: Box<ControlIR>,
    },
    /// Direct control signal injection
    Direct(ControlSignal),
}

// ─── Control Compiler ───

/// Compiles skills and policies into executable control signals.
pub struct ControlCompiler;

impl ControlCompiler {
    /// Lower a full IR tree into a single control signal given current metrics.
    pub fn lower(ir: &ControlIR, metrics: &Metrics) -> ControlSignal {
        match ir {
            ControlIR::Skill(skill) => {
                // Check preconditions
                let all_met = skill
                    .preconditions
                    .iter()
                    .all(|p| p.satisfied(metrics));
                if all_met {
                    skill.effects
                } else {
                    ControlSignal::zero()
                }
            }
            ControlIR::Direct(signal) => *signal,
            ControlIR::Sequence(seq) => {
                seq.iter()
                    .map(|node| Self::lower(node, metrics))
                    .fold(ControlSignal::zero(), |acc, s| acc.add(&s))
            }
            ControlIR::Branch {
                condition,
                then_branch,
                else_branch,
            } => {
                if condition.satisfied(metrics) {
                    Self::lower(then_branch, metrics)
                } else {
                    Self::lower(else_branch, metrics)
                }
            }
        }
    }
}

// ─── Policy Filter ───

/// Governance-level constraint filter applied after the control compiler.
/// Ensures control signals respect system safety bounds.
pub struct PolicyFilter {
    /// Maximum allowed control magnitude
    pub max_signal_magnitude: f64,
    /// Minimum R_A before emergency stabilization overrides
    pub emergency_ra_threshold: f64,
}

impl PolicyFilter {
    pub fn default_policy() -> Self {
        Self {
            max_signal_magnitude: 2.0,
            emergency_ra_threshold: 0.5,
        }
    }

    /// Apply governance constraints to a control signal.
    pub fn filter(&self, signal: ControlSignal, metrics: &Metrics) -> ControlSignal {
        // Emergency stabilization override
        if metrics.r_a < self.emergency_ra_threshold {
            return Self::emergency_stabilize();
        }

        // Clamp signal magnitude
        let mag = signal.magnitude();
        if mag > self.max_signal_magnitude {
            signal.scale(self.max_signal_magnitude / mag)
        } else {
            signal
        }
    }

    /// Emergency stabilization: strengthen connectivity, accelerate collapse,
    /// reduce entropy injection.
    fn emergency_stabilize() -> ControlSignal {
        ControlSignal {
            delta_kappa: 0.5,
            delta_phi: 0.4,
            delta_lambda: -0.6,
            delta_sigma: 0.3,
        }
    }
}

// ─── Pre-Built Skills (Standard Library) ───

/// Standard skill library for common control operations.
pub struct SkillLibrary;

impl SkillLibrary {
    /// Reduce ambiguity pressure: strengthen gossip, accelerate collapse
    pub fn reduce_ambiguity() -> SkillNode {
        SkillNode {
            name: "reduce_ambiguity".to_string(),
            effects: ControlSignal {
                delta_kappa: 0.2,
                delta_phi: 0.1,
                delta_lambda: -0.3,
                delta_sigma: 0.4,
            },
            preconditions: vec![Precondition::PhaseRatioBelow(0.8)],
        }
    }

    /// Increase exploration: allow more entropy, relax collapse
    pub fn increase_exploration() -> SkillNode {
        SkillNode {
            name: "increase_exploration".to_string(),
            effects: ControlSignal {
                delta_kappa: 0.0,
                delta_phi: -0.1,
                delta_lambda: 0.2,
                delta_sigma: -0.1,
            },
            preconditions: vec![Precondition::PhaseRatioAbove(1.2)],
        }
    }

    /// Maintain critical regime: minimal correction
    pub fn maintain_critical() -> SkillNode {
        SkillNode {
            name: "maintain_critical".to_string(),
            effects: ControlSignal {
                delta_kappa: 0.01,
                delta_phi: 0.01,
                delta_lambda: -0.01,
                delta_sigma: 0.0,
            },
            preconditions: vec![],
        }
    }

    /// Build the default control program (the standard AIOS policy)
    pub fn default_program() -> ControlIR {
        ControlIR::Branch {
            condition: Precondition::PhaseRatioBelow(0.8),
            then_branch: Box::new(ControlIR::Skill(Self::reduce_ambiguity())),
            else_branch: Box::new(ControlIR::Branch {
                condition: Precondition::PhaseRatioAbove(1.2),
                then_branch: Box::new(ControlIR::Skill(Self::increase_exploration())),
                else_branch: Box::new(ControlIR::Skill(Self::maintain_critical())),
            }),
        }
    }
}
