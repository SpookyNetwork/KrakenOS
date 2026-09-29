use crate::health::{HealthStatus, NodeHealth, RestartPolicy};
use krk_psc::SystemMode;
use krk_agent_swarm::SwarmController;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub node_id: String,
    pub tick_interval_ms: u64,
    pub health_check_interval_ms: u64,
    pub restart_policy: RestartPolicy,
}

/// The top-level process orchestrator.
/// Wraps the SwarmController, PSC, and enforces health limits.
pub struct SupervisorNode {
    pub config: SupervisorConfig,
    health: Arc<Mutex<NodeHealth>>,
    swarm: Arc<Mutex<SwarmController>>,
    // In a real implementation, PSC and MemoryPlane would also be managed here.
}

impl SupervisorNode {
    pub fn new(config: SupervisorConfig, swarm: SwarmController) -> Self {
        Self {
            config,
            health: Arc::new(Mutex::new(NodeHealth {
                status: HealthStatus::Healthy,
                uptime_secs: 0,
                memory_pressure: 0.0,
                active_agents: swarm.agent_count(),
            })),
            swarm: Arc::new(Mutex::new(swarm)),
        }
    }

    /// The main asynchronous execution loop.
    pub async fn run(&self) {
        tracing::info!("Supervisor {} starting main loop...", self.config.node_id);

        let health_clone = Arc::clone(&self.health);
        let swarm_clone = Arc::clone(&self.swarm);
        let tick_ms = self.config.tick_interval_ms;

        // Task 1: The Execution Loop
        let execution_task = tokio::spawn(async move {
            let mut tick_count = 0;
            loop {
                // Check health before ticking
                let is_op = {
                    let h = health_clone.lock().unwrap();
                    h.is_operational()
                };

                if !is_op {
                    tracing::warn!("Node health critical. Halting execution loop.");
                    break; // Trigger supervisor restart sequence
                }

                {
                    // Execute one cognitive tick
                    let mut swarm = swarm_clone.lock().unwrap();
                    // Mock world state for supervisor context
                    let world = krk_agent_swarm::WorldState {
                        tick: tick_count,
                        system_mode: SystemMode::Safe,
                        entropy_level: 0.1,
                        state_hash: [0u8; 32],
                        adversarial_detected: false,
                    };
                    let _actions = swarm.tick(&world);
                    tracing::debug!("Swarm tick {} completed.", tick_count);
                }

                tick_count += 1;
                sleep(Duration::from_millis(tick_ms)).await;
            }
        });

        // Task 2: The Health Check Daemon
        let health_checker = Arc::clone(&self.health);
        let check_ms = self.config.health_check_interval_ms;
        let health_task = tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(check_ms)).await;
                let mut h = health_checker.lock().unwrap();
                h.uptime_secs += check_ms / 1000;
                
                // Simulated memory pressure increase over time (simulating entropy accumulation)
                h.memory_pressure += 0.01; 
                
                if h.memory_pressure > 0.90 {
                    tracing::warn!("Memory pressure reaching critical bounds: {:.2}", h.memory_pressure);
                    h.status = HealthStatus::Degraded(1);
                }
                if h.memory_pressure >= 1.0 {
                    tracing::error!("MEMORY COLLAPSE. Triggering node critical state.");
                    h.status = HealthStatus::Critical;
                    break;
                }
            }
        });

        // Await either task failing/completing
        let _ = tokio::select! {
            _ = execution_task => tracing::error!("Execution task died."),
            _ = health_task => tracing::error!("Health daemon died."),
        };

        // Here we would implement the RestartPolicy recovery sequence
        tracing::info!("Supervisor {} halting.", self.config.node_id);
    }
}
