//! Scenario B: Horizon Starvation Mesh
//!
//! Create sparse gossip bridges, asymmetric partitions, and selective withholding.
//! Determines minimum connectivity required for convergence.
//! Goal: Find V_H collapse threshold.

use crate::partition_controller::PartitionController;
use crate::ambiguity_injector::AmbiguityInjector;
use crate::horizon_tracker::HorizonTracker;
use crate::branch_pressure_monitor::BranchPressureMonitor;
use crate::collapse_profiler::CollapseProfiler;
use krk_algebra::SubstrateState;

pub fn run(node_count: usize) -> crate::ScenarioResult {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  SCENARIO B: HORIZON STARVATION MESH                     ║");
    println!("║  Measuring minimum connectivity for convergence          ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    let injector = AmbiguityInjector;
    let nodes: Vec<String> = (0..node_count).map(|i| format!("n{}", i)).collect();
    let node_refs: Vec<&str> = nodes.iter().map(|s| s.as_str()).collect();

    // Test decreasing connectivity levels
    let connectivity_levels = [0.8, 0.5, 0.3, 0.15, 0.05];
    let mut results: Vec<(f64, f64, usize)> = vec![];

    for &connectivity in &connectivity_levels {
        let mut pc = PartitionController::new();
        pc.create_sparse_mesh(&node_refs, connectivity);

        let mut tracker = HorizonTracker::new(&node_refs);
        let mut pressure = BranchPressureMonitor::new();
        let mut profiler = CollapseProfiler::new();
        let mut global_state = SubstrateState::new();
        let mut sim_time: f64 = 0.0;

        // Inject effects at each node
        let effects_per_node = 10;
        let mut all_effects = vec![];

        for (i, node) in nodes.iter().enumerate() {
            let effects = injector.generate_equivalence_storm(&format!("payload_{}", i % 3), effects_per_node);
            for e in &effects {
                let id = e.causal_id();
                tracker.horizons[i].observe(id, sim_time);
                global_state.reduce(e);
            }
            pressure.record_divergence(effects.len());
            all_effects.push(effects);
        }

        sim_time += 200.0;

        // Simulate gossip propagation based on visibility
        for (i, node) in nodes.iter().enumerate() {
            let visible = pc.visible_peers(node);
            for (j, peer) in nodes.iter().enumerate() {
                if visible.contains(peer) && i != j {
                    // Propagate node i's effects to node j
                    for e in &all_effects[i] {
                        let id = e.causal_id();
                        tracker.horizons[j].observe(id, sim_time);
                    }
                }
            }
        }

        // Measure convergence
        let total_injected = node_count * effects_per_node;
        let unique_global = global_state.projected_effects.len();

        // Measure horizon coverage: what fraction of global effects each node sees
        let avg_coverage: f64 = tracker.horizons.iter()
            .map(|h| h.size() as f64 / unique_global.max(1) as f64)
            .sum::<f64>() / node_count as f64;

        let collapsed = total_injected.saturating_sub(unique_global);
        pressure.record_collapse(collapsed);
        profiler.profile_collapse(unique_global, 0.01, collapsed, sim_time);

        let v_h = tracker.measure_velocity(sim_time);

        println!("[Connectivity {:.0}%] Coverage: {:.1}% | V_H: {:.4} | Unique: {} | Asymmetry: {}",
            connectivity * 100.0, avg_coverage * 100.0, v_h, unique_global, tracker.horizon_asymmetry());

        results.push((connectivity, avg_coverage, tracker.horizon_asymmetry()));
    }

    // Find the percolation threshold
    let starvation_threshold = results.iter()
        .find(|(_, coverage, _)| *coverage < 0.5)
        .map(|(c, _, _)| *c)
        .unwrap_or(0.0);

    let last = results.last().unwrap_or(&(0.0, 0.0, 0));

    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ SCENARIO B RESULTS                                       │");
    println!("├────────────────────────────────────────────────────────────┤");
    println!("│ Starvation Threshold:       {:<25.0}%   │", starvation_threshold * 100.0);
    println!("│ Final Coverage at 5%:       {:<25.1}%   │", last.1 * 100.0);
    println!("│ Final Asymmetry at 5%:      {:<28} │", last.2);
    for (c, cov, asym) in &results {
        println!("│  [{:>3.0}%] Coverage: {:>5.1}% | Asymmetry: {:<14} │", c * 100.0, cov * 100.0, asym);
    }
    println!("└────────────────────────────────────────────────────────────┘");

    crate::ScenarioResult {
        name: "B: Horizon Starvation Mesh".to_string(),
        r_a: last.1,
        v_h: 0.0,
        total_injected: node_count * 10,
        total_collapsed: 0,
        peak_worldlines: node_count * 10,
        peak_rfc: node_count * 10,
        collapse_amplification: 0.0,
        peak_collapse_spike_ms: 0.0,
        verdict: if starvation_threshold <= 0.15 { "RESILIENT" } else { "FRAGILE" }.to_string(),
    }
}
