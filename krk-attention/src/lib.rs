//! KRK Attention Thermodynamics Module
//!
//! Maps external attention signals (AI ecosystem trends, market discourse,
//! social velocity) into Kraken physics variables: κ, Φ~, λ_A.
//!
//! This is the real-world forcing function for the causal entropy model.
//! External cognition becomes a control input to the PSC.

use serde::{Deserialize, Serialize};

pub mod ingestion;

// ─── Attention Signal (THE BRIDGE) ───

/// A structured attention vector extracted from external signal sources.
/// Each field maps to a Kraken physics variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionSignal {
    pub timestamp: u64,

    // Raw signal features (domain-specific)
    /// Agent ecosystem dominance (0.0 - 1.0)
    pub agent_density: f64,
    /// Safety/governance discourse pressure (0.0 - 1.0)
    pub safety_pressure: f64,
    /// Open-source diffusion velocity (0.0 - 1.0)
    pub open_source_velocity: f64,
    /// Market consolidation pressure (0.0 - 1.0)
    pub consolidation_pressure: f64,
    /// Physical AI / robotics emergence signal (0.0 - 1.0)
    pub physical_ai_shift: f64,
}

impl AttentionSignal {
    /// Project the attention signal into Kraken physics space.
    /// Returns (κ, Φ~, λ_A) — the three PSC input channels.
    pub fn to_physics(&self) -> PhysicsProjection {
        PhysicsProjection {
            // κ (topology connectivity): driven by agent density + consolidation
            kappa: self.agent_density * 0.6 + self.consolidation_pressure * 0.4,
            // Φ~ (equivalence discoverability): driven by open-source diffusion
            phi: self.open_source_velocity,
            // λ_A (entropy injection rate): driven by agent density + physical AI shift
            lambda_a: self.agent_density * 0.5 + self.physical_ai_shift * 0.3
                + self.safety_pressure * 0.2,
        }
    }
}

/// Physics-space projection of an attention signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsProjection {
    pub kappa: f64,
    pub phi: f64,
    pub lambda_a: f64,
}

impl PhysicsProjection {
    /// Phase ratio R_A = κ × Φ~ / λ_A
    pub fn phase_ratio(&self) -> f64 {
        if self.lambda_a.abs() < 1e-10 {
            return f64::INFINITY;
        }
        (self.kappa * self.phi) / self.lambda_a
    }
}

// ─── Signal Extractor ───

/// Extracts attention signals from raw text content.
/// Uses keyword frequency scoring as a lightweight classifier.
pub struct SignalExtractor;

impl SignalExtractor {
    /// Score keyword frequency in text (normalized to 0.0 - 1.0)
    fn score_keyword(text: &str, keywords: &[&str]) -> f64 {
        let text_lower = text.to_lowercase();
        let total_words = text_lower.split_whitespace().count().max(1) as f64;
        let hits: f64 = keywords.iter()
            .map(|kw| text_lower.matches(&kw.to_lowercase()).count() as f64)
            .sum();
        (hits / total_words * 10.0).min(1.0) // Scale and clamp
    }

    /// Extract an AttentionSignal from raw text content.
    pub fn extract(text: &str, timestamp: u64) -> AttentionSignal {
        AttentionSignal {
            timestamp,
            agent_density: Self::score_keyword(text, &["agent", "autonomous", "agentic", "multi-agent"]),
            safety_pressure: Self::score_keyword(text, &["safety", "alignment", "guardrail", "regulation", "audit"]),
            open_source_velocity: Self::score_keyword(text, &["open-source", "github", "huggingface", "llama", "mistral"]),
            consolidation_pressure: Self::score_keyword(text, &["acquisition", "merger", "openai", "microsoft", "monopoly"]),
            physical_ai_shift: Self::score_keyword(text, &["robot", "embodied", "neuralink", "physical", "hardware"]),
        }
    }
}

// ─── Attention Window (Temporal Smoothing) ───

/// Maintains a sliding window of attention signals for temporal smoothing.
pub struct AttentionWindow {
    signals: Vec<AttentionSignal>,
    max_size: usize,
}

impl AttentionWindow {
    pub fn new(max_size: usize) -> Self {
        Self {
            signals: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, signal: AttentionSignal) {
        if self.signals.len() >= self.max_size {
            self.signals.remove(0);
        }
        self.signals.push(signal);
    }

    /// Compute the averaged attention signal across the window.
    pub fn average(&self) -> Option<AttentionSignal> {
        if self.signals.is_empty() {
            return None;
        }
        let n = self.signals.len() as f64;
        let ts = self.signals.last().map(|s| s.timestamp).unwrap_or(0);

        Some(AttentionSignal {
            timestamp: ts,
            agent_density: self.signals.iter().map(|s| s.agent_density).sum::<f64>() / n,
            safety_pressure: self.signals.iter().map(|s| s.safety_pressure).sum::<f64>() / n,
            open_source_velocity: self.signals.iter().map(|s| s.open_source_velocity).sum::<f64>() / n,
            consolidation_pressure: self.signals.iter().map(|s| s.consolidation_pressure).sum::<f64>() / n,
            physical_ai_shift: self.signals.iter().map(|s| s.physical_ai_shift).sum::<f64>() / n,
        })
    }

    pub fn len(&self) -> usize {
        self.signals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_extraction() {
        let text = "AI agents are becoming autonomous. OpenAI released new agent framework. \
                     Safety concerns about multi-agent systems. GitHub trending with open-source models.";
        let signal = SignalExtractor::extract(text, 1000);

        assert!(signal.agent_density > 0.0, "Should detect agent keywords");
        assert!(signal.safety_pressure > 0.0, "Should detect safety keywords");
        assert!(signal.open_source_velocity > 0.0, "Should detect open-source keywords");

        let physics = signal.to_physics();
        assert!(physics.kappa > 0.0);
        assert!(physics.phi > 0.0);
        assert!(physics.lambda_a > 0.0);

        let r_a = physics.phase_ratio();
        assert!(r_a.is_finite(), "Phase ratio should be finite");
    }

    #[test]
    fn test_attention_window() {
        let mut window = AttentionWindow::new(5);

        for i in 0..10 {
            window.push(AttentionSignal {
                timestamp: i,
                agent_density: 0.1 * i as f64,
                safety_pressure: 0.05,
                open_source_velocity: 0.3,
                consolidation_pressure: 0.0,
                physical_ai_shift: 0.0,
            });
        }

        assert_eq!(window.len(), 5); // Capped at max_size
        let avg = window.average().unwrap();
        assert!(avg.agent_density > 0.0);
        assert_eq!(avg.timestamp, 9);
    }

    #[test]
    fn test_physics_projection() {
        let signal = AttentionSignal {
            timestamp: 0,
            agent_density: 0.8,
            safety_pressure: 0.3,
            open_source_velocity: 0.6,
            consolidation_pressure: 0.4,
            physical_ai_shift: 0.2,
        };

        let physics = signal.to_physics();
        let r_a = physics.phase_ratio();

        println!("κ={:.3}, Φ~={:.3}, λ_A={:.3}, R_A={:.3}",
            physics.kappa, physics.phi, physics.lambda_a, r_a);

        assert!(r_a > 0.0);
    }
}
