//! Performance benchmarks for P1-B6 stealth features
//!
//! Measures the overhead of various stealth operations

use riptide_stealth::{
    CdpStealthIntegrator, EnhancedFingerprintGenerator, StealthController, StealthLevel,
    StealthLevelConfig, StealthPreset,
};
use std::time::Instant;

fn benchmark_operation<F>(name: &str, iterations: usize, mut op: F) -> f64
where
    F: FnMut(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        op();
    }
    let duration = start.elapsed();
    let avg_micros = duration.as_micros() as f64 / iterations as f64;

    #[cfg(feature = "benchmark-debug")]
    println!(
        "{}: {:.2} μs/op ({} iterations)",
        name, avg_micros, iterations
    );
    avg_micros
}

fn main() {
    #[cfg(feature = "benchmark-debug")]
    println!("=== P1-B6 Stealth Performance Benchmarks ===\n");

    // Benchmark: Fingerprint generation
    #[cfg(feature = "benchmark-debug")]
    println!("## Fingerprint Generation");
    let mut generator = EnhancedFingerprintGenerator::with_default_config();

    benchmark_operation("Basic fingerprint generation", 1000, || {
        use riptide_stealth::FingerprintGenerator;
        let _ = FingerprintGenerator::generate();
    });

    benchmark_operation("Context-aware fingerprint generation", 1000, || {
        let _ = generator.generate_contextual(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            None,
        );
    });

    benchmark_operation("Session-cached fingerprint", 1000, || {
        let _ = generator.generate_contextual(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            Some("session1"),
        );
    });

    // Benchmark: CDP integration
    #[cfg(feature = "benchmark-debug")]
    println!("\n## CDP Integration");
    let mut integrator = CdpStealthIntegrator::new();

    benchmark_operation("CDP commands generation", 1000, || {
        let _ = integrator.generate_stealth_commands(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            Some("session1"),
        );
    });

    benchmark_operation("Batch headers (10 requests)", 100, || {
        let _ = integrator.generate_batch_headers(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            Some("session1"),
            10,
        );
    });

    // Benchmark: Stealth levels
    #[cfg(feature = "benchmark-debug")]
    println!("\n## Stealth Level Configuration");

    for level in [
        StealthLevel::None,
        StealthLevel::Low,
        StealthLevel::Medium,
        StealthLevel::High,
    ] {
        let name = format!("Config creation: {:?}", level);
        benchmark_operation(&name, 10000, || {
            let _ = StealthLevelConfig::from_level(level);
        });
    }

    // Benchmark: JavaScript generation
    #[cfg(feature = "benchmark-debug")]
    println!("\n## JavaScript Generation");

    for preset in [
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ] {
        let mut controller = StealthController::from_preset(preset.clone());
        let name = format!("JS generation: {:?}", preset);

        benchmark_operation(&name, 100, || {
            let _ = controller.get_stealth_js();
        });
    }

    // Benchmark: Header generation
    #[cfg(feature = "benchmark-debug")]
    println!("\n## Header Generation");

    let controller = StealthController::from_preset(StealthPreset::High);

    benchmark_operation("Random headers", 1000, || {
        let _ = controller.generate_headers();
    });

    // Memory overhead test
    #[cfg(feature = "benchmark-debug")]
    println!("\n## Memory Overhead");

    let mut generator = EnhancedFingerprintGenerator::with_default_config();
    let start_memory = std::mem::size_of_val(&generator);

    // Generate 100 cached sessions
    for i in 0..100 {
        let session_id = format!("session{}", i);
        generator.generate_contextual("ua", Some(&session_id));
    }

    let cache_size = generator.cache_size();
    #[cfg(feature = "benchmark-debug")]
    println!("Cache size: {} sessions", cache_size);
    #[cfg(feature = "benchmark-debug")]
    println!("Base generator size: {} bytes", start_memory);

    // Comparative overhead
    #[cfg(feature = "benchmark-debug")]
    println!("\n## Relative Overhead Analysis");

    let none_time = {
        let mut controller = StealthController::from_preset(StealthPreset::None);
        benchmark_operation("Stealth None (baseline)", 100, || {
            let _ = controller.get_stealth_js();
            let _ = controller.generate_headers();
        })
    };

    let _low_time = {
        let mut controller = StealthController::from_preset(StealthPreset::Low);
        benchmark_operation("Stealth Low", 100, || {
            let _ = controller.get_stealth_js();
            let _ = controller.generate_headers();
        })
    };

    let _medium_time = {
        let mut controller = StealthController::from_preset(StealthPreset::Medium);
        benchmark_operation("Stealth Medium", 100, || {
            let _ = controller.get_stealth_js();
            let _ = controller.generate_headers();
        })
    };

    let _high_time = {
        let mut controller = StealthController::from_preset(StealthPreset::High);
        benchmark_operation("Stealth High", 100, || {
            let _ = controller.get_stealth_js();
            let _ = controller.generate_headers();
        })
    };

    #[cfg(feature = "benchmark-debug")]
    {
        println!("\n## Overhead Summary");
        println!("None (baseline): {:.2} μs", none_time);
        println!(
            "Low:    {:.2} μs ({:.1}% overhead)",
            low_time,
            (low_time / none_time.max(1.0) - 1.0) * 100.0
        );
        println!(
            "Medium: {:.2} μs ({:.1}% overhead)",
            medium_time,
            (medium_time / none_time.max(1.0) - 1.0) * 100.0
        );
        println!(
            "High:   {:.2} μs ({:.1}% overhead)",
            high_time,
            (high_time / none_time.max(1.0) - 1.0) * 100.0
        );

        println!("\n=== Benchmark Complete ===");
    }
}
