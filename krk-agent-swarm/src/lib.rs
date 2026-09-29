//! KRK Agent Swarm — Multi-Agent Cognitive Execution System
//!
//! Implements a distributed cognition engine where specialized agents
//! (planner, executor, critic, memory, risk) operate as independent
//! control-theoretic actuators gated by the control-plane policy stack.
//!
//! Core invariants:
//! - No shared mutable state between agents (message passing only)
//! - Every agent action passes through control-plane policy evaluation
//! - Byzantine tolerance: deviating agents are quarantined
//! - PSC override: global kill switch for all agent execution

pub mod agents;
pub mod coordination;

use serde::{Deserialize, Serialize};
use krk_psc::SystemMode;
use krk_sandbox::{ExecutionEnvelope, SandboxManager};

// ─── CORE DOMAIN TYPES ───

/// The observable world state injected into agents each tick.
#[derive(Debug, Clone)]
pub struct WorldState {
    pub tick: u64,
    pub system_mode: SystemMode,
    pub entropy_level: f64,
    pub state_hash: [u8; 32],
    pub adversarial_detected: bool,
}

/// An action proposed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub agent_id: String,
    pub action_type: ActionType,
    pub confidence: f64,
    pub payload: String,
}

/// Typed action categories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Plan,
    Execute,
    Critique,
    Memorize,
    Block,
    Noop,
}

// ─── CORE TRAIT ───

/// The fundamental agent interface — perceive/act/score.
pub trait Agent: Send + Sync {
    /// Unique agent identifier.
    fn id(&self) -> &str;

    /// Ingest world state (read-only observation).
    fn perceive(&mut self, world: &WorldState);

    /// Propose an action based on current perception, bounded by the sandbox envelope.
    fn act(&mut self, env: &mut ExecutionEnvelope) -> AgentAction;

    /// Self-assessed confidence score (0.0 = no confidence, 1.0 = certain).
    fn score(&self) -> f64;
}

// ─── SWARM CONTROLLER ───

/// The swarm controller orchestrates all agents in a single tick.
/// It enforces:
/// - Isolation: agents never see each other's state
/// - Byzantine tolerance: agents with low scores are quarantined
/// - Kill switch: respects PSC mode restrictions
pub struct SwarmController {
    agents: Vec<Box<dyn Agent>>,
    /// Minimum confidence score to allow action execution.
    pub min_confidence: f64,
    /// Maximum allowed agent deviation before quarantine.
    pub quarantine_threshold: f64,
}

impl SwarmController {
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            min_confidence: 0.3,
            quarantine_threshold: 0.1,
        }
    }

    /// Register an agent in the swarm.
    pub fn add_agent(&mut self, agent: Box<dyn Agent>) {
        self.agents.push(agent);
    }

    /// Execute one tick across all agents.
    /// Returns only actions that pass confidence and mode filters.
    pub fn tick(&mut self, world: &WorldState) -> Vec<AgentAction> {
        // PSC Kill Switch: In CollapsePrevention, no agent acts.
        if world.system_mode == SystemMode::CollapsePrevention {
            return Vec::new();
        }

        let mut actions = Vec::new();

        for agent in &mut self.agents {
            // Phase 1: Perceive
            agent.perceive(world);

            // Phase 2: Confidence gate
            if agent.score() < self.min_confidence {
                continue; // Quarantine low-confidence agent
            }

            // Phase 3: Act within a provisioned sandbox envelope
            let mut envelope = SandboxManager::provision_envelope(agent.id(), 1000, 5000);
            let action = agent.act(&mut envelope);

            // Phase 4: Byzantine deviation check
            if action.confidence < self.quarantine_threshold {
                eprintln!("SWARM | QUARANTINE: Agent {} below deviation threshold", agent.id());
                continue;
            }

            actions.push(action);
        }

        actions
    }

    /// Number of registered agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAgent {
        name: &'static str,
        confidence: f64,
    }

    impl Agent for TestAgent {
        fn id(&self) -> &str { self.name }
        fn perceive(&mut self, _world: &WorldState) {}
        fn act(&mut self, _env: &mut ExecutionEnvelope) -> AgentAction {
            AgentAction {
                agent_id: self.name.to_string(),
                action_type: ActionType::Execute,
                confidence: self.confidence,
                payload: "test_action".to_string(),
            }
        }
        fn score(&self) -> f64 { self.confidence }
    }

    fn test_world(mode: SystemMode) -> WorldState {
        WorldState {
            tick: 1,
            system_mode: mode,
            entropy_level: 0.1,
            state_hash: [0u8; 32],
            adversarial_detected: false,
        }
    }

    #[test]
    fn test_swarm_normal_tick() {
        let mut swarm = SwarmController::new();
        swarm.add_agent(Box::new(TestAgent { name: "planner", confidence: 0.9 }));
        swarm.add_agent(Box::new(TestAgent { name: "executor", confidence: 0.8 }));

        let actions = swarm.tick(&test_world(SystemMode::Safe));
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_swarm_collapse_prevention_kills_all() {
        let mut swarm = SwarmController::new();
        swarm.add_agent(Box::new(TestAgent { name: "planner", confidence: 0.9 }));

        let actions = swarm.tick(&test_world(SystemMode::CollapsePrevention));
        assert_eq!(actions.len(), 0, "No agents should act in CollapsePrevention");
    }

    #[test]
    fn test_swarm_quarantines_low_confidence() {
        let mut swarm = SwarmController::new();
        swarm.add_agent(Box::new(TestAgent { name: "good", confidence: 0.9 }));
        swarm.add_agent(Box::new(TestAgent { name: "bad", confidence: 0.05 }));

        let actions = swarm.tick(&test_world(SystemMode::Safe));
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].agent_id, "good");
    }
}
