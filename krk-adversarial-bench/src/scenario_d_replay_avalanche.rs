//! Scenario D: Replay Avalanche
//! Validates Σ ⊕ e ⊕ e ⊕ ... ⊕ e = Σ ⊕ e under extreme replay pressure.

use crate::ambiguity_injector::AmbiguityInjector;
use crate::collapse_profiler::CollapseProfiler;
use krk_algebra::SubstrateState;

pub fn run(replay_factor: usize) -> crate::ScenarioResult {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║  SCENARIO D: REPLAY AVALANCHE                    ║");
    println!("╚════════════════════════════════════════════════════╝");

    let injector = AmbiguityInjector;
    let mut profiler = CollapseProfiler::new();
    let mut state = SubstrateState::new();

    let canonical = injector.generate_conflicting_effects(20);
    for e in &canonical { state.reduce(e); }
    let baseline = state.projected_effects.len();
    println!("[Baseline] {} unique effects", baseline);

    let mut total_replays = 0usize;
    let mut max_cpu = 0.0f64;

    for wave in 0..replay_factor {
        let t = std::time::Instant::now();
        for e in &canonical { state.reduce(e); total_replays += 1; }
        let cpu = t.elapsed().as_secs_f64() * 1000.0;
        max_cpu = max_cpu.max(cpu);
        profiler.profile_collapse(baseline, cpu, 0, wave as f64 * 10.0);
        if wave % 50 == 0 || wave == replay_factor - 1 {
            println!("  [Wave {:>4}/{}] Size: {} | CPU: {:.4}ms | {}",
                wave+1, replay_factor, state.projected_effects.len(), cpu,
                if state.projected_effects.len() == baseline { "✓" } else { "✗" });
        }
    }

    let ok = state.projected_effects.len() == baseline;
    println!("│ Idempotent: {} | Replays: {} | Peak CPU: {:.4}ms │", ok, total_replays, max_cpu);

    crate::ScenarioResult {
        name: "D: Replay Avalanche".into(),
        r_a: if ok { 1.0 } else { 0.0 }, v_h: 0.0,
        total_injected: total_replays, total_collapsed: total_replays,
        peak_worldlines: baseline, peak_rfc: 0,
        collapse_amplification: profiler.amplification_factor(),
        peak_collapse_spike_ms: max_cpu,
        verdict: if ok { "IDEMPOTENT ✓".into() } else { "VIOLATION ✗".into() },
    }
}
