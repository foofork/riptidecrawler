//! Phase 4 Performance Validation CLI
//!
//! Runs comprehensive performance benchmarks and generates validation report.

use riptide_performance::phase4_validation::benchmarks::{
    export_results_to_json, print_results, Phase4BenchmarkSuite,
};
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let iterations = args
        .iter()
        .position(|a| a == "--iterations")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let output = args
        .iter()
        .position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/workspaces/eventmesh/phase4-results.json".to_string());

    println!("🚀 Phase 4 Performance Validator");
    println!("Iterations: {}", iterations);
    println!("Output: {}", output);
    println!();

    let suite = Phase4BenchmarkSuite::new(iterations);
    let report = suite.run_full_validation().await;

    // Print results to console
    print_results(&report);

    // Export to JSON
    if let Err(e) = export_results_to_json(&report, &output) {
        eprintln!("❌ Failed to export results: {}", e);
        std::process::exit(1);
    }

    println!("\n✅ Results exported to: {}", output);

    // Exit with appropriate code
    if report.overall_verdict.all_passed {
        println!("\n🎉 ALL VALIDATION CHECKS PASSED!");
        std::process::exit(0);
    } else {
        println!("\n❌ SOME VALIDATION CHECKS FAILED!");
        std::process::exit(1);
    }
}
