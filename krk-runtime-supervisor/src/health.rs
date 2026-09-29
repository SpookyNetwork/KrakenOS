use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(u32), // Number of consecutive failed checks
    Critical,      // Requires restart
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub status: HealthStatus,
    pub uptime_secs: u64,
    pub memory_pressure: f64, // 0.0 to 1.0
    pub active_agents: usize,
}

impl NodeHealth {
    pub fn is_operational(&self) -> bool {
        self.status != HealthStatus::Critical && self.memory_pressure < 0.95
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub current_restarts: u32,
    pub backoff_ms: u64,
}

impl RestartPolicy {
    pub fn new(max_restarts: u32) -> Self {
        Self {
            max_restarts,
            current_restarts: 0,
            backoff_ms: 1000,
        }
    }

    pub fn can_restart(&self) -> bool {
        self.current_restarts < self.max_restarts
    }

    pub fn record_restart(&mut self) {
        self.current_restarts += 1;
        self.backoff_ms *= 2; // Exponential backoff
    }
}
