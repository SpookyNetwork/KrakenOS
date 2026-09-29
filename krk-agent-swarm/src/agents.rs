//! Concrete Agent Implementations
//!
//! Five specialized agents forming the cognitive execution stack:
//! - Planner: produces execution graph
//! - Executor: carries out actions
//! - Critic: adversarially evaluates proposed actions
//! - MemoryAgent: compresses and stores observations
//! - RiskAgent: blocks unsafe execution paths

use crate::*;

// ─── PLANNER ───

pub struct PlannerAgent {
    last_plan: Option<String>,
    confidence: f64,
}

impl PlannerAgent {
    pub fn new() -> Self {
        Self { last_plan: None, confidence: 0.8 }
    }
}

impl Agent for PlannerAgent {
    fn id(&self) -> &str { "planner" }
    fn perceive(&mut self, world: &WorldState) {
        // In degraded mode, reduce planning confidence.
        self.confidence = if world.system_mode == SystemMode::Degraded { 0.5 } else { 0.8 };
    }
    fn act(&mut self, env: &mut ExecutionEnvelope) -> AgentAction {
        let _ = env.consume_tokens(50); // Planning overhead
        let plan = format!("plan_tick_{}", 0);
        self.last_plan = Some(plan.clone());
        AgentAction {
            agent_id: "planner".to_string(),
            action_type: ActionType::Plan,
            confidence: self.confidence,
            payload: plan,
        }
    }
    fn score(&self) -> f64 { self.confidence }
}

// ─── EXECUTOR ───

pub struct ExecutorAgent {
    confidence: f64,
}

impl ExecutorAgent {
    pub fn new() -> Self {
        Self { confidence: 0.9 }
    }
}

impl Agent for ExecutorAgent {
    fn id(&self) -> &str { "executor" }
    fn perceive(&mut self, world: &WorldState) {
        self.confidence = if world.adversarial_detected { 0.2 } else { 0.9 };
    }
    fn act(&mut self, env: &mut ExecutionEnvelope) -> AgentAction {
        let _ = env.consume_tokens(150); // High execution cost
        AgentAction {
            agent_id: "executor".to_string(),
            action_type: ActionType::Execute,
            confidence: self.confidence,
            payload: "execute_pending".to_string(),
        }
    }
    fn score(&self) -> f64 { self.confidence }
}

// ─── CRITIC ───

pub struct CriticAgent {
    rejection_rate: f64,
}

impl CriticAgent {
    pub fn new() -> Self {
        Self { rejection_rate: 0.0 }
    }
}

impl Agent for CriticAgent {
    fn id(&self) -> &str { "critic" }
    fn perceive(&mut self, world: &WorldState) {
        // Higher entropy → more critical.
        self.rejection_rate = world.entropy_level;
    }
    fn act(&mut self, env: &mut ExecutionEnvelope) -> AgentAction {
        let _ = env.consume_tokens(20); // Cheap critique
        let action_type = if self.rejection_rate > 0.5 {
            ActionType::Block
        } else {
            ActionType::Critique
        };
        AgentAction {
            agent_id: "critic".to_string(),
            action_type,
            confidence: 1.0 - self.rejection_rate,
            payload: format!("critique_rejection_rate_{:.2}", self.rejection_rate),
        }
    }
    fn score(&self) -> f64 { 0.9 } // Critic always has high base confidence
}

// ─── MEMORY AGENT ───

pub struct MemoryAgent {
    buffer_size: usize,
}

impl MemoryAgent {
    pub fn new() -> Self {
        Self { buffer_size: 0 }
    }
}

impl Agent for MemoryAgent {
    fn id(&self) -> &str { "memory" }
    fn perceive(&mut self, _world: &WorldState) {
        self.buffer_size += 1;
    }
    fn act(&mut self, env: &mut ExecutionEnvelope) -> AgentAction {
        let _ = env.consume_tokens(10); // Negligible memory check
        let action_type = if self.buffer_size > 100 {
            self.buffer_size = 0;
            ActionType::Memorize // Trigger compression
        } else {
            ActionType::Noop
        };
        AgentAction {
            agent_id: "memory".to_string(),
            action_type,
            confidence: 1.0,
            payload: format!("buffer_size_{}", self.buffer_size),
        }
    }
    fn score(&self) -> f64 { 1.0 }
}

// ─── RISK AGENT ───

pub struct RiskAgent {
    risk_level: f64,
}

impl RiskAgent {
    pub fn new() -> Self {
        Self { risk_level: 0.0 }
    }
}

impl Agent for RiskAgent {
    fn id(&self) -> &str { "risk" }
    fn perceive(&mut self, world: &WorldState) {
        self.risk_level = world.entropy_level;
        if world.adversarial_detected {
            self.risk_level = 1.0;
        }
    }
    fn act(&mut self, env: &mut ExecutionEnvelope) -> AgentAction {
        let _ = env.consume_tokens(5); // Cheap risk calc
        let action_type = if self.risk_level > 0.7 {
            ActionType::Block
        } else {
            ActionType::Noop
        };
        AgentAction {
            agent_id: "risk".to_string(),
            action_type,
            confidence: 1.0 - self.risk_level * 0.5,
            payload: format!("risk_{:.2}", self.risk_level),
        }
    }
    fn score(&self) -> f64 { 0.95 }
}
