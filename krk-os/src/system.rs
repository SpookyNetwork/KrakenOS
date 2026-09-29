//! KRK-OS System Core
//!
//! The closed-loop adaptive control system.
//! Glues State (X), Control (U), Dynamics (F), and Observation (Y)
//! into a single executable update loop.
//!
//! Main loop per tick:
//!   1. Ingest events into the DAG
//!   2. Observe: compute metrics from current state
//!   3. Control: compile skills → control signal, apply policy filter
//!   4. Feed PSC: run the Phase-Stabilizing Controller
//!   5. Actuate: apply control to state
//!   6. Collapse: resolve equivalence classes

use crate::control::{
    ControlCompiler, ControlIR, ControlSignal, Metrics, PolicyFilter, SkillLibrary,
};
use crate::state::{Event, SubstrateState};
use krk_psc::{PSCConfig, SystemMode, TransitionEngineStabilityModule};

/// The KRK-OS kernel: a closed-loop adaptive control system
/// over a stochastic causal DAG with quotient-based state reduction.
pub struct KrkOS {
    /// X: The physical substrate state
    pub state: SubstrateState,
    /// U: The control program (compiled skill tree)
    control_program: ControlIR,
    /// Policy filter (governance constraints)
    policy: PolicyFilter,
    /// PSC: Phase-Stabilizing Controller (spectral stability enforcement)
    psc: TransitionEngineStabilityModule,
    /// Current PSC mode
    current_mode: SystemMode,
    /// Tick counter
    tick: u64,
    /// Telemetry history for observability
    metrics_history: Vec<Metrics>,
    /// Maximum telemetry history length
    max_history: usize,
}

impl KrkOS {
    pub fn new(rfc_capacity: usize, psc_config: PSCConfig) -> Self {
        Self {
            state: SubstrateState::new(rfc_capacity),
            control_program: SkillLibrary::default_program(),
            policy: PolicyFilter::default_policy(),
            psc: TransitionEngineStabilityModule::new(psc_config),
            current_mode: SystemMode::Safe,
            tick: 0,
            metrics_history: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn with_control_program(mut self, program: ControlIR) -> Self {
        self.control_program = program;
        self
    }

    pub fn with_policy(mut self, policy: PolicyFilter) -> Self {
        self.policy = policy;
        self
    }

    /// Execute one tick of the KRK-OS control loop.
    ///
    /// This is the SINGLE point where everything converges:
    /// Ingest → Observe → Control → PSC → Actuate → Collapse
    pub fn step(&mut self, events: Vec<Event>) -> StepResult {
        self.tick += 1;

        // 1. INGEST: Add events to the DAG
        self.state.ingest_batch(events);

        // 2. OBSERVE: Compute metrics from current (pre-transition) state
        let metrics = self.observe();

        // 3. CONTROL: Compile skills → control signal
        let raw_signal = ControlCompiler::lower(&self.control_program, &metrics);

        // 4. POLICY FILTER: Apply governance constraints
        let filtered_signal = self.policy.filter(raw_signal, &metrics);

        // 5. PSC: Feed the Phase-Stabilizing Controller
        //    The PSC observes the CONTROL SIGNAL as telemetry
        //    (how much the controller is trying to change the system)
        let telemetry_age = 1.0; // 1ms within same process (minimal latency)
        let (u_gossip, u_collapse, mode) = self.psc.tick(
            telemetry_age,
            filtered_signal.delta_kappa,
            filtered_signal.delta_phi,
            filtered_signal.delta_lambda.abs(),
        );
        self.current_mode = mode;

        // 6. ACTUATE: Apply control to state (PSC-gated)
        let applied_signal = match mode {
            SystemMode::CollapsePrevention => {
                // PSC override: zero out all control, force collapse
                ControlSignal::zero()
            }
            SystemMode::Degraded => {
                // PSC throttle: scale down control
                filtered_signal.scale(0.5)
            }
            SystemMode::Safe => {
                // Full control authority
                filtered_signal
            }
        };

        // 7. COLLAPSE: Resolve equivalence classes and RFC buffer
        self.state.resolve_rfc();
        self.state.equivalence.rebuild(&self.state.dag);

        // 8. RECORD: Store metrics for history
        if self.metrics_history.len() >= self.max_history {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(metrics);

        StepResult {
            tick: self.tick,
            metrics,
            raw_signal,
            applied_signal,
            mode,
            u_gossip,
            u_collapse,
            lambda_hat: self.psc.current_lambda_hat(),
            dag_size: self.state.dag.len(),
            equiv_classes: self.state.equivalence.class_count(),
            divergences: self.state.equivalence.divergence_count(),
            rfc_backlog: self.state.rfc.len(),
        }
    }

    /// Compute observability metrics from the current state
    fn observe(&self) -> Metrics {
        let dag_size = self.state.dag.len().max(1) as f64;
        let equiv_classes = self.state.equivalence.class_count() as f64;
        let divergences = self.state.equivalence.divergence_count() as f64;
        let horizon_size = self.state.horizon.size() as f64;
        let rfc_size = self.state.rfc.len() as f64;

        // κ: connectivity = horizon coverage / dag size
        let kappa = horizon_size / dag_size;

        // Φ~: equivalence discoverability = 1 - (divergences / equiv_classes)
        let phi = if equiv_classes > 0.0 {
            1.0 - (divergences / equiv_classes)
        } else {
            1.0
        };

        // λ_A: entropy injection rate = RFC backlog / dag size
        let lambda_a = rfc_size / dag_size;

        // C⊕: collapse cost (proportional to equivalence class count)
        let c_collapse = equiv_classes / dag_size;

        // R_A: phase ratio
        let r_a = Metrics::compute_phase_ratio(kappa, phi, lambda_a.max(0.001));

        Metrics {
            kappa,
            phi,
            lambda_a,
            c_collapse,
            r_a,
        }
    }

    /// Get the current system mode
    pub fn mode(&self) -> SystemMode {
        self.current_mode
    }

    /// Get the current λ̂_max estimate
    pub fn lambda_hat(&self) -> f64 {
        self.psc.current_lambda_hat()
    }

    /// Get the current tick count
    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    /// Get metrics history
    pub fn metrics_history(&self) -> &[Metrics] {
        &self.metrics_history
    }
}

/// Result of a single KRK-OS step
#[derive(Debug, Clone)]
pub struct StepResult {
    pub tick: u64,
    pub metrics: Metrics,
    pub raw_signal: ControlSignal,
    pub applied_signal: ControlSignal,
    pub mode: SystemMode,
    pub u_gossip: f64,
    pub u_collapse: f64,
    pub lambda_hat: f64,
    pub dag_size: usize,
    pub equiv_classes: usize,
    pub divergences: usize,
    pub rfc_backlog: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Event;

    fn make_event(payload: &[u8], deps: Vec<u64>) -> Event {
        let mut hash = [0u8; 32];
        for (i, b) in payload.iter().enumerate() {
            if i < 32 {
                hash[i] = *b;
            }
        }
        Event {
            id: 0, // Will be assigned by DAG
            payload_hash: hash,
            dependencies: deps,
            timestamp: 0,
        }
    }

    #[test]
    fn test_krk_os_basic_loop() {
        let mut os = KrkOS::new(1000, PSCConfig::default());

        // Ingest some unique events
        for i in 0..10 {
            let result = os.step(vec![make_event(&[i], vec![])]);
            assert_eq!(result.dag_size, (i as usize) + 1);
        }

        // System should be in Safe mode with low entropy
        assert_eq!(os.mode(), SystemMode::Safe);
        println!("After 10 unique events: mode={:?}, λ̂={:.6}", os.mode(), os.lambda_hat());
    }

    #[test]
    fn test_krk_os_divergence_detection() {
        let mut os = KrkOS::new(1000, PSCConfig::default());

        // Ingest events with SAME payload (creates divergences)
        let same_payload = [42u8; 1];
        for _ in 0..20 {
            let result = os.step(vec![make_event(&same_payload, vec![])]);
            println!(
                "tick={}, equiv_classes={}, divergences={}, R_A={:.4}, mode={:?}",
                result.tick, result.equiv_classes, result.divergences,
                result.metrics.r_a, result.mode,
            );
        }
    }

    #[test]
    fn test_krk_os_full_closed_loop() {
        let mut os = KrkOS::new(1000, PSCConfig::default());

        // Phase 1: Normal operation (diverse events)
        println!("=== Phase 1: Normal Operation ===");
        for i in 0..50 {
            let result = os.step(vec![make_event(&[i as u8], vec![])]);
            if i % 10 == 0 {
                println!(
                    "tick={}, κ={:.3}, Φ={:.3}, λ_A={:.3}, R_A={:.3}, mode={:?}",
                    result.tick, result.metrics.kappa, result.metrics.phi,
                    result.metrics.lambda_a, result.metrics.r_a, result.mode,
                );
            }
        }

        // Phase 2: Stress (many duplicate events - ambiguity spike)
        println!("\n=== Phase 2: Ambiguity Spike ===");
        for i in 0..50 {
            // Inject events with only 3 distinct payloads (massive divergence)
            let payload = [(i % 3) as u8];
            let result = os.step(vec![make_event(&payload, vec![])]);
            if i % 10 == 0 {
                println!(
                    "tick={}, κ={:.3}, Φ={:.3}, λ_A={:.3}, R_A={:.3}, mode={:?}, λ̂={:.6}",
                    result.tick, result.metrics.kappa, result.metrics.phi,
                    result.metrics.lambda_a, result.metrics.r_a, result.mode,
                    result.lambda_hat,
                );
            }
        }

        println!("\nFinal state: mode={:?}, λ̂={:.6}", os.mode(), os.lambda_hat());
    }
}
