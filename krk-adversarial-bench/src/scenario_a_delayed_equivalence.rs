//! Scenario A: Delayed Equivalence Storm
//!
//! Inject equivalent events with distinct provenance into isolated partitions,
//! then restore the bridge and measure equivalence discovery half-life (T_~).

use crate::partition_controller::PartitionController;
use crate::ambiguity_injector::AmbiguityInjector;
use crate::horizon_tracker::HorizonTracker;
use crate::branch_pressure_monitor::BranchPressureMonitor;
use crate::collapse_profiler::CollapseProfiler;
use krk_algebra::SubstrateState;

pub fn run(wave_count: usize) -> crate::ScenarioResult {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  SCENARIO A: DELAYED EQUIVALENCE STORM                   ║");
    println!("║  Measuring T_~ (equivalence discovery half-life)         ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    let mut pc = PartitionController::new();
    let injector = AmbiguityInjector;
    let mut tracker = HorizonTracker::new(&["n1", "n2", "n3", "n4"]);
    let mut pressure = BranchPressureMonitor::new();
    let mut profiler = CollapseProfiler::new();

    // Phase 0: Establish partition geometry
    pc.create_partition(&["n1", "n2"], &["n3", "n4"]);
    println!("[Geometry] Cluster A: (n1, n2) | Cluster B: (n3, n4) | Bridge: DOWN");

    let mut state_a = SubstrateState::new();
    let mut state_b = SubstrateState::new();
    let mut total_injected: usize = 0;
    let mut total_collapsed: usize = 0;
    let mut sim_time: f64 = 0.0;

    for wave in 0..wave_count {
        sim_time += 50.0;

        // Phase 1: Divergence Growth — inject equivalent effects into isolated clusters
        let storm_a = injector.generate_equivalence_storm("order_fulfill", 5);
        let storm_b = injector.generate_equivalence_storm("order_fulfill", 5);

        for e in &storm_a {
            state_a.reduce(e);
            let id = e.causal_id();
            tracker.horizons[0].observe(id.clone(), sim_time);
            tracker.horizons[1].observe(id, sim_time);
        }
        for e in &storm_b {
            state_b.reduce(e);
            let id = e.causal_id();
            tracker.horizons[2].observe(id.clone(), sim_time);
            tracker.horizons[3].observe(id, sim_time);
        }

        total_injected += storm_a.len() + storm_b.len();
        pressure.record_divergence(storm_a.len() + storm_b.len());
        pressure.snapshot(sim_time);

        println!("[Wave {}/{}] Injected {} effects | Worldlines: {} | RFC: {} | D_B: {:.3}",
            wave + 1, wave_count, storm_a.len() + storm_b.len(),
            pressure.active_worldlines, pressure.rfc_occupancy, pressure.density());

        // Phase 2: Restore bridge and force horizon intersection
        sim_time += 100.0;
        pc.restore_bridge(&["n1", "n2"], &["n3", "n4"], 50.0);

        // Simulate cross-cluster horizon propagation
        let mut merged_state = SubstrateState::new();
        let collapse_start = std::time::Instant::now();

        for e in &storm_a { merged_state.reduce(e); }
        for e in &storm_b { merged_state.reduce(e); }

        let collapse_duration = collapse_start.elapsed().as_secs_f64() * 1000.0;
        let unique_after = merged_state.projected_effects.len();
        let collapsed_count = (storm_a.len() + storm_b.len()).saturating_sub(unique_after);

        total_collapsed += collapsed_count;
        profiler.profile_collapse(unique_after, collapse_duration, collapsed_count, sim_time);
        pressure.record_collapse(collapsed_count);

        for e in &storm_a {
            let id = e.causal_id();
            tracker.horizons[2].observe(id.clone(), sim_time);
            tracker.horizons[3].observe(id, sim_time);
        }
        for e in &storm_b {
            let id = e.causal_id();
            tracker.horizons[0].observe(id.clone(), sim_time);
            tracker.horizons[1].observe(id, sim_time);
        }

        tracker.record_equivalence_discovery(sim_time);
        pressure.snapshot(sim_time);

        println!("  [Collapse] Unique effects: {} | Collapsed: {} | CPU: {:.4}ms",
            unique_after, collapsed_count, collapse_duration);

        // Re-partition for next wave
        pc.create_partition(&["n1", "n2"], &["n3", "n4"]);
    }

    // Final metrics
    let v_h = tracker.measure_velocity(sim_time);
    let r_a = if total_injected > 0 {
        total_collapsed as f64 / total_injected as f64
    } else { 0.0 };

    let asymmetry = tracker.horizon_asymmetry();
    let intersection = tracker.horizon_intersection(0, 2);

    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ SCENARIO A RESULTS                                       │");
    println!("├────────────────────────────────────────────────────────────┤");
    println!("│ R_A (Viability Ratio):      {:<28.4} │", r_a);
    println!("│ V_H (Horizon Velocity):     {:<28.4} │", v_h);
    println!("│ Total Injected:             {:<28} │", total_injected);
    println!("│ Total Collapsed:            {:<28} │", total_collapsed);
    println!("│ Horizon Asymmetry:          {:<28} │", asymmetry);
    println!("│ Cross-Cluster Intersection: {:<28} │", intersection);
    println!("│ Peak Worldlines:            {:<28} │", pressure.peak_worldlines);
    println!("│ Peak RFC Occupancy:         {:<28} │", pressure.peak_rfc);
    println!("│ Collapse Amplification:     {:<28.4} │", profiler.amplification_factor());
    println!("│ Peak Collapse Spike:        {:<25.4}ms │", profiler.peak_collapse_spike_ms);
    println!("└────────────────────────────────────────────────────────────┘");

    crate::ScenarioResult {
        name: "A: Delayed Equivalence Storm".to_string(),
        r_a,
        v_h,
        total_injected,
        total_collapsed,
        peak_worldlines: pressure.peak_worldlines,
        peak_rfc: pressure.peak_rfc,
        collapse_amplification: profiler.amplification_factor(),
        peak_collapse_spike_ms: profiler.peak_collapse_spike_ms,
        verdict: if r_a > 0.5 { "STABLE" } else if r_a > 0.0 { "METASTABLE" } else { "SATURATED" }.to_string(),
    }
}
