//! Scenario E: Equivalence Percolation Failure
//! Tests ambiguity delocalization — the terminal failure mode.

use crate::ambiguity_injector::AmbiguityInjector;
use crate::horizon_tracker::HorizonTracker;
use crate::branch_pressure_monitor::BranchPressureMonitor;
use krk_algebra::SubstrateState;

pub fn run(region_count: usize) -> crate::ScenarioResult {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║  SCENARIO E: EQUIVALENCE PERCOLATION FAILURE      ║");
    println!("╚════════════════════════════════════════════════════╝");

    let injector = AmbiguityInjector;
    let nodes: Vec<String> = (0..region_count*2).map(|i| format!("n{}", i)).collect();
    let node_refs: Vec<&str> = nodes.iter().map(|s| s.as_str()).collect();
    let mut tracker = HorizonTracker::new(&node_refs);
    let mut pressure = BranchPressureMonitor::new();

    // Create isolated regions, each with their own equivalent effects
    let mut region_states: Vec<SubstrateState> = vec![];
    let mut all_effects = vec![];

    for r in 0..region_count {
        let mut region_state = SubstrateState::new();
        let effects = injector.generate_equivalence_storm("shared_payload", 10);
        for e in &effects {
            region_state.reduce(e);
            let id = e.causal_id();
            tracker.horizons[r * 2].observe(id.clone(), 0.0);
            tracker.horizons[r * 2 + 1].observe(id, 0.0);
        }
        pressure.record_divergence(effects.len());
        all_effects.push(effects);
        region_states.push(region_state);
    }

    // Measure cross-region intersection (should be near zero = delocalized)
    let mut cross_intersections = vec![];
    for i in 0..region_count {
        for j in (i+1)..region_count {
            let ix = tracker.horizon_intersection(i * 2, j * 2);
            cross_intersections.push(ix);
        }
    }
    let avg_intersection = cross_intersections.iter().sum::<usize>() as f64
        / cross_intersections.len().max(1) as f64;

    // Attempt global reconciliation
    let mut global = SubstrateState::new();
    let t = std::time::Instant::now();
    for region_effects in &all_effects {
        for e in region_effects { global.reduce(e); }
    }
    let collapse_ms = t.elapsed().as_secs_f64() * 1000.0;
    let total_injected = region_count * 10;
    let unique = global.projected_effects.len();
    let collapsed = total_injected.saturating_sub(unique);

    let locality = if total_injected > 0 {
        1.0 - (avg_intersection / unique.max(1) as f64)
    } else { 1.0 };

    println!("│ Regions: {} | Total: {} | Unique: {} | Collapsed: {} │", region_count, total_injected, unique, collapsed);
    println!("│ Avg Cross-Intersection: {:.2} | Locality(L): {:.4} │", avg_intersection, locality);
    println!("│ Reconciliation CPU: {:.4}ms │", collapse_ms);

    let delocalized = avg_intersection < 1.0 && region_count > 3;

    crate::ScenarioResult {
        name: "E: Percolation Failure".into(),
        r_a: collapsed as f64 / total_injected.max(1) as f64, v_h: 0.0,
        total_injected, total_collapsed: collapsed,
        peak_worldlines: pressure.peak_worldlines, peak_rfc: pressure.peak_rfc,
        collapse_amplification: collapse_ms / unique.max(1) as f64,
        peak_collapse_spike_ms: collapse_ms,
        verdict: if delocalized { "DELOCALIZED ⚠" } else { "LOCALIZED ✓" }.into(),
    }
}
