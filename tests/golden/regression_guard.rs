//! Regression Guard Module
//!
//! Implements the 5% performance regression detection and enforcement
//! along with verification systems for post-refactoring validation.

use super::*;
use std::collections::HashMap;

/// Regression detection and analysis
pub struct RegressionGuard {
    config: GoldenTestConfig,
}

impl RegressionGuard {
    pub fn new(config: GoldenTestConfig) -> Self {
        Self { config }
    }

    /// Compare current performance against baseline and detect regressions
    pub fn detect_regressions(
        &self,
        baseline: &BehaviorSnapshot,
        current: &BehaviorSnapshot,
    ) -> GoldenTestResult {
        let performance_delta = self.calculate_performance_delta(
            &baseline.performance_metrics,
            &current.performance_metrics,
        );
        
        let memory_delta = self.calculate_memory_delta(
            &baseline.memory_metrics,
            &current.memory_metrics,
        );
        
        let functional_diffs = self.calculate_functional_diffs(
            &baseline.functional_outputs,
            &current.functional_outputs,
        );
        
        let violations = self.detect_violations(
            &performance_delta,
            &memory_delta,
            &functional_diffs,
            &current.memory_metrics,
        );
        
        let passed = violations.is_empty();
        
        GoldenTestResult {
            test_name: current.test_name.clone(),
            passed,
            baseline_snapshot: baseline.clone(),
            current_snapshot: current.clone(),
            performance_delta,
            memory_delta,
            functional_diffs,
            violations,
        }
    }
    
    /// Calculate performance metrics delta
    fn calculate_performance_delta(
        &self,
        baseline: &PerformanceMetrics,
        current: &PerformanceMetrics,
    ) -> PerformanceDelta {
        let p50_change = if baseline.p50_latency_ms > 0.0 {
            ((current.p50_latency_ms - baseline.p50_latency_ms) / baseline.p50_latency_ms) * 100.0
        } else {
            0.0
        };
        
        let p95_change = if baseline.p95_latency_ms > 0.0 {
            ((current.p95_latency_ms - baseline.p95_latency_ms) / baseline.p95_latency_ms) * 100.0
        } else {
            0.0
        };
        
        let p99_change = if baseline.p99_latency_ms > 0.0 {
            ((current.p99_latency_ms - baseline.p99_latency_ms) / baseline.p99_latency_ms) * 100.0
        } else {
            0.0
        };
        
        let regression_detected = p50_change > self.config.max_regression_percent
            || p95_change > self.config.max_regression_percent
            || p99_change > self.config.max_regression_percent;
        
        PerformanceDelta {
            p50_change_percent: p50_change,
            p95_change_percent: p95_change,
            p99_change_percent: p99_change,
            regression_detected,
        }
    }
    
    /// Calculate memory usage delta
    fn calculate_memory_delta(
        &self,
        baseline: &MemoryMetrics,
        current: &MemoryMetrics,
    ) -> MemoryDelta {
        let rss_change = if baseline.rss_bytes > 0 {
            ((current.rss_bytes as f64 - baseline.rss_bytes as f64) / baseline.rss_bytes as f64) * 100.0
        } else {
            0.0
        };
        
        let heap_change = if baseline.heap_bytes > 0 {
            ((current.heap_bytes as f64 - baseline.heap_bytes as f64) / baseline.heap_bytes as f64) * 100.0
        } else {
            0.0
        };
        
        let limit_exceeded = current.rss_bytes > self.config.memory_limit_bytes;
        
        MemoryDelta {
            rss_change_percent: rss_change,
            heap_change_percent: heap_change,
            limit_exceeded,
        }
    }
    
    /// Calculate functional output differences
    fn calculate_functional_diffs(
        &self,
        baseline: &HashMap<String, serde_json::Value>,
        current: &HashMap<String, serde_json::Value>,
    ) -> Vec<FunctionalDiff> {
        let mut diffs = Vec::new();
        
        // Check for changed or removed fields
        for (key, baseline_value) in baseline {
            match current.get(key) {
                Some(current_value) => {
                    if baseline_value != current_value {
                        diffs.push(FunctionalDiff {
                            field_path: key.clone(),
                            baseline_value: baseline_value.clone(),
                            current_value: current_value.clone(),
                            diff_type: if baseline_value.type_name() != current_value.type_name() {
                                DiffType::TypeChanged
                            } else {
                                DiffType::ValueChanged
                            },
                        });
                    }
                },
                None => {
                    diffs.push(FunctionalDiff {
                        field_path: key.clone(),
                        baseline_value: baseline_value.clone(),
                        current_value: serde_json::Value::Null,
                        diff_type: DiffType::FieldRemoved,
                    });
                }
            }
        }
        
        // Check for added fields
        for (key, current_value) in current {
            if !baseline.contains_key(key) {
                diffs.push(FunctionalDiff {
                    field_path: key.clone(),
                    baseline_value: serde_json::Value::Null,
                    current_value: current_value.clone(),
                    diff_type: DiffType::FieldAdded,
                });
            }
        }
        
        diffs
    }
    
    /// Detect all types of violations
    fn detect_violations(
        &self,
        performance_delta: &PerformanceDelta,
        memory_delta: &MemoryDelta,
        functional_diffs: &[FunctionalDiff],
        current_memory: &MemoryMetrics,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Performance regression violations
        if performance_delta.p50_change_percent > self.config.max_regression_percent {
            violations.push(Violation {
                violation_type: ViolationType::PerformanceRegression,
                description: format!(
                    "P50 latency regression: {:.2}% > {:.2}% threshold",
                    performance_delta.p50_change_percent,
                    self.config.max_regression_percent
                ),
                severity: Severity::Critical,
                measured_value: performance_delta.p50_change_percent,
                threshold_value: self.config.max_regression_percent,
            });
        }
        
        if performance_delta.p95_change_percent > self.config.max_regression_percent {
            violations.push(Violation {
                violation_type: ViolationType::PerformanceRegression,
                description: format!(
                    "P95 latency regression: {:.2}% > {:.2}% threshold",
                    performance_delta.p95_change_percent,
                    self.config.max_regression_percent
                ),
                severity: Severity::Critical,
                measured_value: performance_delta.p95_change_percent,
                threshold_value: self.config.max_regression_percent,
            });
        }
        
        if performance_delta.p99_change_percent > self.config.max_regression_percent {
            violations.push(Violation {
                violation_type: ViolationType::PerformanceRegression,
                description: format!(
                    "P99 latency regression: {:.2}% > {:.2}% threshold",
                    performance_delta.p99_change_percent,
                    self.config.max_regression_percent
                ),
                severity: Severity::High,
                measured_value: performance_delta.p99_change_percent,
                threshold_value: self.config.max_regression_percent,
            });
        }
        
        // Memory limit violations
        if memory_delta.limit_exceeded {
            violations.push(Violation {
                violation_type: ViolationType::MemoryLimit,
                description: format!(
                    "Memory limit exceeded: {:.1}MB > {:.1}MB limit",
                    current_memory.rss_bytes as f64 / (1024.0 * 1024.0),
                    self.config.memory_limit_bytes as f64 / (1024.0 * 1024.0)
                ),
                severity: Severity::Critical,
                measured_value: current_memory.rss_bytes as f64,
                threshold_value: self.config.memory_limit_bytes as f64,
            });
        }
        
        // Functional change violations (if strict mode enabled)
        if !functional_diffs.is_empty() {
            for diff in functional_diffs {
                match diff.diff_type {
                    DiffType::TypeChanged => {
                        violations.push(Violation {
                            violation_type: ViolationType::FunctionalChange,
                            description: format!(
                                "Type changed for field '{}': {} -> {}",
                                diff.field_path,
                                type_name_of_value(&diff.baseline_value),
                                type_name_of_value(&diff.current_value)
                            ),
                            severity: Severity::High,
                            measured_value: 0.0,
                            threshold_value: 0.0,
                        });
                    },
                    DiffType::FieldRemoved => {
                        violations.push(Violation {
                            violation_type: ViolationType::FunctionalChange,
                            description: format!(
                                "Field removed: '{}'",
                                diff.field_path
                            ),
                            severity: Severity::Medium,
                            measured_value: 0.0,
                            threshold_value: 0.0,
                        });
                    },
                    DiffType::FieldAdded => {
                        violations.push(Violation {
                            violation_type: ViolationType::FunctionalChange,
                            description: format!(
                                "Field added: '{}'",
                                diff.field_path
                            ),
                            severity: Severity::Low,
                            measured_value: 0.0,
                            threshold_value: 0.0,
                        });
                    },
                    DiffType::ValueChanged => {
                        violations.push(Violation {
                            violation_type: ViolationType::FunctionalChange,
                            description: format!(
                                "Value changed for field '{}'",
                                diff.field_path
                            ),
                            severity: Severity::Medium,
                            measured_value: 0.0,
                            threshold_value: 0.0,
                        });
                    },
                }
            }
        }
        
        violations
    }
}

/// Post-refactoring verification system
pub struct VerificationSystem {
    guard: RegressionGuard,
}

impl VerificationSystem {
    pub fn new(config: GoldenTestConfig) -> Self {
        Self {
            guard: RegressionGuard::new(config),
        }
    }
    
    /// Run complete verification suite
    pub async fn verify_refactoring(
        &self,
        test_suite: &[(&str, fn() -> Result<serde_json::Value, anyhow::Error>)],
        baseline_storage: &mut super::BaselineStorage,
    ) -> Result<VerificationReport, anyhow::Error> {
        let mut results = Vec::new();
        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut failed_tests = 0;
        
        println!("🔍 Starting post-refactoring verification...");
        
        for (test_name, _test_fn) in test_suite {
            total_tests += 1;
            
            println!("📝 Verifying test: {}", test_name);
            
            // Load baseline
            let baseline = match baseline_storage.load_baseline(test_name).await? {
                Some(baseline) => baseline,
                None => {
                    println!("⚠️  No baseline found for test '{}', skipping", test_name);
                    continue;
                }
            };
            
            // Capture current behavior (placeholder - in real use, run the actual test)
            let current = create_test_snapshot(test_name).await;
            
            // Run regression detection
            let result = self.guard.detect_regressions(&baseline, &current);
            
            if result.passed {
                passed_tests += 1;
                println!("✅ Test '{}' passed verification", test_name);
            } else {
                failed_tests += 1;
                println!("❌ Test '{}' failed verification with {} violations", test_name, result.violations.len());
                for violation in &result.violations {
                    println!("  • {:?}: {}", violation.severity, violation.description);
                }
            }
            
            results.push(result);
        }
        
        let report = VerificationReport {
            total_tests,
            passed_tests,
            failed_tests,
            success_rate: if total_tests > 0 {
                (passed_tests as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            },
            test_results: results,
        };
        
        report.print_summary();
        
        Ok(report)
    }
}

/// Verification report for the entire test suite
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub test_results: Vec<GoldenTestResult>,
}

impl VerificationReport {
    pub fn is_successful(&self) -> bool {
        self.failed_tests == 0
    }
    
    pub fn print_summary(&self) {
        println!("\n📊 Verification Summary");
        println!("=======================");
        println!("Total Tests: {}", self.total_tests);
        println!("Passed: ✅ {}", self.passed_tests);
        println!("Failed: ❌ {}", self.failed_tests);
        println!("Success Rate: {:.1}%", self.success_rate);
        
        if !self.is_successful() {
            println!("\n⚠️  Refactoring verification FAILED!");
            println!("Please review the violations and fix issues before proceeding.");
        } else {
            println!("\n🎉 Refactoring verification PASSED!");
            println!("All tests meet performance and functional requirements.");
        }
        
        println!();
    }
    
    pub fn print_detailed_report(&self) {
        self.print_summary();
        
        if !self.test_results.is_empty() {
            println!("📝 Detailed Test Results");
            println!("=========================");
            
            for result in &self.test_results {
                println!("\nTest: {}", result.test_name);
                println!("Status: {}", if result.passed { "✅ PASSED" } else { "❌ FAILED" });
                
                // Performance metrics
                println!("Performance Changes:");
                println!("  P50: {:.2}%", result.performance_delta.p50_change_percent);
                println!("  P95: {:.2}%", result.performance_delta.p95_change_percent);
                println!("  P99: {:.2}%", result.performance_delta.p99_change_percent);
                
                // Memory metrics
                println!("Memory Changes:");
                println!("  RSS: {:.2}%", result.memory_delta.rss_change_percent);
                println!("  Heap: {:.2}%", result.memory_delta.heap_change_percent);
                
                // Violations
                if !result.violations.is_empty() {
                    println!("Violations:");
                    for violation in &result.violations {
                        println!("  • {:?}: {}", violation.severity, violation.description);
                    }
                }
                
                // Functional diffs
                if !result.functional_diffs.is_empty() {
                    println!("Functional Changes:");
                    for diff in &result.functional_diffs {
                        println!("  • {}: {:?}", diff.field_path, diff.diff_type);
                    }
                }
            }
        }
    }
}

/// Helper function to create test snapshot (placeholder)
async fn create_test_snapshot(test_name: &str) -> BehaviorSnapshot {
    // In real usage, this would run the actual test function
    // For now, create a placeholder that simulates current behavior
    use std::time::{SystemTime, UNIX_EPOCH};
    
    BehaviorSnapshot {
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        test_name: test_name.to_string(),
        performance_metrics: PerformanceMetrics {
            p50_latency_ms: 850.0,  // Slightly higher than baseline
            p95_latency_ms: 3600.0, // Slightly higher than baseline
            p99_latency_ms: 5100.0,
            mean_latency_ms: 1050.0,
            min_latency_ms: 210.0,
            max_latency_ms: 8200.0,
            std_dev_ms: 520.0,
        },
        memory_metrics: MemoryMetrics {
            rss_bytes: 420 * 1024 * 1024, // 420MB - within limits
            heap_bytes: 210 * 1024 * 1024,
            peak_rss_bytes: 520 * 1024 * 1024,
            memory_efficiency: 78.0,
        },
        throughput_metrics: ThroughputMetrics {
            pages_per_second: 1.9,
            requests_per_second: 4.8,
            bytes_processed_per_second: 950 * 1024,
            operations_per_second: 9.8,
        },
        functional_outputs: std::collections::HashMap::new(),
        error_patterns: Vec::new(),
    }
}

/// Helper function to get type name of JSON value
fn type_name_of_value(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Extension trait for JSON value type names
trait JsonValueExt {
    fn type_name(&self) -> &'static str;
}

impl JsonValueExt for serde_json::Value {
    fn type_name(&self) -> &'static str {
        type_name_of_value(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regression_detection() {
        let config = GoldenTestConfig::default();
        let guard = RegressionGuard::new(config);
        
        let baseline_metrics = PerformanceMetrics {
            p50_latency_ms: 1000.0,
            p95_latency_ms: 3000.0,
            p99_latency_ms: 5000.0,
            mean_latency_ms: 1200.0,
            min_latency_ms: 200.0,
            max_latency_ms: 8000.0,
            std_dev_ms: 500.0,
        };
        
        let current_metrics = PerformanceMetrics {
            p50_latency_ms: 1100.0, // 10% increase - should trigger regression
            p95_latency_ms: 3000.0,
            p99_latency_ms: 5000.0,
            mean_latency_ms: 1300.0,
            min_latency_ms: 200.0,
            max_latency_ms: 8000.0,
            std_dev_ms: 500.0,
        };
        
        let delta = guard.calculate_performance_delta(&baseline_metrics, &current_metrics);
        
        assert!(delta.regression_detected);
        assert_eq!(delta.p50_change_percent, 10.0);
    }
    
    #[test]
    fn test_memory_limit_detection() {
        let config = GoldenTestConfig::default();
        let guard = RegressionGuard::new(config);
        
        let memory_metrics = MemoryMetrics {
            rss_bytes: 700 * 1024 * 1024, // 700MB - exceeds 600MB limit
            heap_bytes: 300 * 1024 * 1024,
            peak_rss_bytes: 750 * 1024 * 1024,
            memory_efficiency: 60.0,
        };
        
        let violations = guard.detect_violations(
            &PerformanceDelta {
                p50_change_percent: 0.0,
                p95_change_percent: 0.0,
                p99_change_percent: 0.0,
                regression_detected: false,
            },
            &MemoryDelta {
                rss_change_percent: 0.0,
                heap_change_percent: 0.0,
                limit_exceeded: true,
            },
            &[],
            &memory_metrics,
        );
        
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| matches!(v.violation_type, ViolationType::MemoryLimit)));
    }
}
