//! Scenario C: Branch Cascade
//!
//! Generate admissible conflicting worldlines faster than reconciliation
//! can collapse them. Locates the branch explosion threshold.
//! Goal: Find R_A = 1 critical phase transition.

use crate::ambiguity_injector::AmbiguityInjector;
use crate::branch_pressure_monitor::BranchPressureMonitor;
use crate::collapse_profiler::CollapseProfiler;
use krk_algebra::SubstrateState;

pub fn run(cascade_depth: usize) -> crate::ScenarioResult {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  SCENARIO C: BRANCH CASCADE                              ║");
    println!("║  Locating the R_A = 1 phase transition                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    let injector = AmbiguityInjector;
    let mut pressure = BranchPressureMonitor::new();
    let mut profiler = CollapseProfiler::new();
    let mut state = SubstrateState::new();
    let mut sim_time: f64 = 0.0;
    let mut total_injected: usize = 0;
    let mut total_collapsed: usize = 0;
    let mut r_a_history: Vec<(usize, f64, f64)> = vec![];

    for depth in 1..=cascade_depth {
        sim_time += 10.0;

        // Inject conflicting effects at increasing rates
        let injection_rate = depth * 2;
        let conflicts = injector.generate_conflicting_effects(injection_rate);

        let pre_count = state.projected_effects.len();
        let collapse_start = std::time::Instant::now();

        for e in &conflicts {
            state.reduce(e);
        }

        let collapse_duration = collapse_start.elapsed().as_secs_f64() * 1000.0;
        let post_count = state.projected_effects.len();
        let new_unique = post_count - pre_count;
        let collapsed = injection_rate.saturating_sub(new_unique);

        total_injected += injection_rate;
        total_collapsed += collapsed;

        pressure.record_divergence(new_unique);
        if collapsed > 0 { pressure.record_collapse(collapsed); }
        profiler.profile_collapse(post_count, collapse_duration, new_unique, sim_time);
        pressure.snapshot(sim_time);

        let r_a = if total_injected > 0 { total_collapsed as f64 / total_injected as f64 } else { 0.0 };
        let density = pressure.density();

        r_a_history.push((depth, r_a, density));

        println!("[Depth {:>3}/{}] Injected: {:>4} | New: {:>4} | Collapsed: {:>4} | D_B: {:.3} | R_A: {:.4} | CPU: {:.4}ms",
            depth, cascade_depth, injection_rate, new_unique, collapsed, density, r_a, collapse_duration);
    }

    let final_r_a = r_a_history.last().map(|(_, r, _)| *r).unwrap_or(0.0);
    let final_density = r_a_history.last().map(|(_, _, d)| *d).unwrap_or(0.0);

    // Find the critical transition point where density starts spiking
    let critical_depth = r_a_history.iter()
        .find(|(_, _, d)| *d > 5.0)
        .map(|(depth, _, _)| *depth)
        .unwrap_or(cascade_depth);

    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ SCENARIO C RESULTS                                       │");
    println!("├────────────────────────────────────────────────────────────┤");
    println!("│ Final R_A:                  {:<28.4} │", final_r_a);
    println!("│ Final Branch Density:       {:<28.4} │", final_density);
    println!("│ Total Injected:             {:<28} │", total_injected);
    println!("│ Total Unique Effects:       {:<28} │", state.projected_effects.len());
    println!("│ Critical Depth (D_B > 5):   {:<28} │", critical_depth);
    println!("│ Peak Worldlines:            {:<28} │", pressure.peak_worldlines);
    println!("│ Peak Collapse Spike:        {:<25.4}ms │", profiler.peak_collapse_spike_ms);
    println!("│ Collapse Amplification:     {:<28.4} │", profiler.amplification_factor());
    println!("└────────────────────────────────────────────────────────────┘");

    crate::ScenarioResult {
        name: "C: Branch Cascade".to_string(),
        r_a: final_r_a,
        v_h: 0.0,
        total_injected,
        total_collapsed,
        peak_worldlines: pressure.peak_worldlines,
        peak_rfc: pressure.peak_rfc,
        collapse_amplification: profiler.amplification_factor(),
        peak_collapse_spike_ms: profiler.peak_collapse_spike_ms,
        verdict: if final_density < 2.0 { "BOUNDED" } else if final_density < 10.0 { "CRITICAL" } else { "EXPLODED" }.to_string(),
    }
}
