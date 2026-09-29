//! Kraken Adversarial Benchmark Runner
//! Executes all 5 causal physics scenarios in sequence.

use krk_adversarial_bench::*;

fn main() {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║          KRAKEN CAUSAL PHYSICS WIND TUNNEL v0.1             ║");
    println!("║  Measuring ambiguity economics under adversarial geometry   ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    let mut results: Vec<ScenarioResult> = vec![];

    // A: Delayed Equivalence Storm (10 waves)
    results.push(scenario_a_delayed_equivalence::run(10));
    println!();

    // B: Horizon Starvation Mesh (8 nodes)
    results.push(scenario_b_horizon_starvation::run(8));
    println!();

    // C: Branch Cascade (50 depth levels)
    results.push(scenario_c_branch_cascade::run(50));
    println!();

    // D: Replay Avalanche (200x replay)
    results.push(scenario_d_replay_avalanche::run(200));
    println!();

    // E: Percolation Failure (10 isolated regions)
    results.push(scenario_e_percolation_failure::run(10));
    println!();

    // Final Summary
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              WIND TUNNEL FINAL REPORT                       ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    for r in &results {
        println!("║ {:<30} │ {:<12} │ R_A: {:<6.3} ║",
            r.name, r.verdict, r.r_a);
    }
    println!("╚═══════════════════════════════════════════════════════════════╝");

    // Write JSON report
    let json = serde_json::to_string_pretty(&results).unwrap_or_default();
    let report_path = "wind_tunnel_report.json";
    std::fs::write(report_path, &json).ok();
    println!("\n[Report written to {}]", report_path);
}
