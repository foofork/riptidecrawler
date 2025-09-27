//! Memory Monitor Module
//!
//! Dedicated memory usage tracking and monitoring for golden tests.
//! Ensures RSS memory stays within the 600MB constraint.

use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, ProcessExt};

/// Advanced memory monitoring with real-time tracking
pub struct MemoryMonitor {
    system: System,
    tracking: Arc<AtomicBool>,
    peak_rss: Arc<AtomicU64>,
    current_rss: Arc<AtomicU64>,
    samples: Arc<std::sync::Mutex<Vec<MemorySample>>>,
    limit_bytes: u64,
}

/// Memory usage sample at a point in time
#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub rss_bytes: u64,
    pub heap_bytes: u64,
    pub cpu_usage: f32,
}

/// Memory monitoring report
#[derive(Debug, Clone)]
pub struct MemoryReport {
    pub duration: Duration,
    pub samples_count: usize,
    pub initial_rss: u64,
    pub peak_rss: u64,
    pub final_rss: u64,
    pub average_rss: u64,
    pub memory_efficiency: f64,
    pub limit_exceeded: bool,
    pub violations: Vec<MemoryViolation>,
}

/// Memory constraint violation
#[derive(Debug, Clone)]
pub struct MemoryViolation {
    pub timestamp: Instant,
    pub violation_type: MemoryViolationType,
    pub measured_bytes: u64,
    pub limit_bytes: u64,
    pub severity: ViolationSeverity,
}

/// Types of memory violations
#[derive(Debug, Clone)]
pub enum MemoryViolationType {
    RssLimitExceeded,
    SuddenSpike,
    SustainedHigh,
    MemoryLeak,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl MemoryMonitor {
    /// Create new memory monitor with specified limit
    pub fn new(limit_bytes: u64) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let current_process = system.process(sysinfo::get_current_pid().unwrap()).unwrap();
        let initial_rss = current_process.memory() * 1024; // Convert to bytes
        
        Self {
            system,
            tracking: Arc::new(AtomicBool::new(false)),
            peak_rss: Arc::new(AtomicU64::new(initial_rss)),
            current_rss: Arc::new(AtomicU64::new(initial_rss)),
            samples: Arc::new(std::sync::Mutex::new(Vec::new())),
            limit_bytes,
        }
    }
    
    /// Start real-time memory monitoring
    pub fn start_monitoring(&self, sample_interval_ms: u64) {
        self.tracking.store(true, Ordering::SeqCst);
        
        let tracking = self.tracking.clone();
        let peak_rss = self.peak_rss.clone();
        let current_rss = self.current_rss.clone();
        let samples = self.samples.clone();
        let limit_bytes = self.limit_bytes;
        
        // Spawn background monitoring task
        tokio::spawn(async move {
            let mut system = System::new_all();
            let start_time = Instant::now();
            
            while tracking.load(Ordering::SeqCst) {
                system.refresh_all();
                
                if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
                    let memory_bytes = process.memory() * 1024;
                    let cpu_usage = process.cpu_usage();
                    
                    current_rss.store(memory_bytes, Ordering::SeqCst);
                    
                    // Update peak
                    let current_peak = peak_rss.load(Ordering::SeqCst);
                    if memory_bytes > current_peak {
                        peak_rss.store(memory_bytes, Ordering::SeqCst);
                    }
                    
                    // Record sample
                    let sample = MemorySample {
                        timestamp: start_time.elapsed().into(),
                        rss_bytes: memory_bytes,
                        heap_bytes: 0, // TODO: Get heap info if available
                        cpu_usage,
                    };
                    
                    if let Ok(mut samples_guard) = samples.lock() {
                        samples_guard.push(sample);
                        
                        // Limit sample history to prevent memory growth
                        if samples_guard.len() > 10000 {
                            samples_guard.drain(0..1000);
                        }
                    }
                    
                    // Check for immediate violations
                    if memory_bytes > limit_bytes {
                        eprintln!(
                            "⚠️  MEMORY LIMIT EXCEEDED: {:.1}MB > {:.1}MB",
                            memory_bytes as f64 / (1024.0 * 1024.0),
                            limit_bytes as f64 / (1024.0 * 1024.0)
                        );
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(sample_interval_ms)).await;
            }
        });
    }
    
    /// Stop monitoring and generate report
    pub fn stop_monitoring(&self) -> MemoryReport {
        self.tracking.store(false, Ordering::SeqCst);
        
        // Wait a moment for final samples
        std::thread::sleep(Duration::from_millis(100));
        
        let samples = self.samples.lock().unwrap().clone();
        let peak_rss = self.peak_rss.load(Ordering::SeqCst);
        let current_rss = self.current_rss.load(Ordering::SeqCst);
        
        self.generate_report(&samples, peak_rss, current_rss)
    }
    
    /// Get current memory usage
    pub fn current_usage(&mut self) -> u64 {
        self.system.refresh_all();
        if let Some(process) = self.system.process(sysinfo::get_current_pid().unwrap()) {
            let memory_bytes = process.memory() * 1024;
            self.current_rss.store(memory_bytes, Ordering::SeqCst);
            memory_bytes
        } else {
            0
        }
    }
    
    /// Check if memory limit is currently exceeded
    pub fn is_limit_exceeded(&mut self) -> bool {
        self.current_usage() > self.limit_bytes
    }
    
    /// Generate comprehensive memory report
    fn generate_report(&self, samples: &[MemorySample], peak_rss: u64, current_rss: u64) -> MemoryReport {
        if samples.is_empty() {
            return MemoryReport {
                duration: Duration::from_secs(0),
                samples_count: 0,
                initial_rss: current_rss,
                peak_rss,
                final_rss: current_rss,
                average_rss: current_rss,
                memory_efficiency: 100.0,
                limit_exceeded: current_rss > self.limit_bytes,
                violations: Vec::new(),
            };
        }
        
        let duration = samples.last().unwrap().timestamp - samples[0].timestamp;
        let initial_rss = samples[0].rss_bytes;
        let final_rss = samples.last().unwrap().rss_bytes;
        
        let average_rss = samples.iter().map(|s| s.rss_bytes).sum::<u64>() / samples.len() as u64;
        
        let memory_efficiency = if peak_rss > 0 {
            ((peak_rss - final_rss) as f64 / peak_rss as f64) * 100.0
        } else {
            100.0
        };
        
        let violations = self.detect_violations(samples);
        let limit_exceeded = peak_rss > self.limit_bytes;
        
        MemoryReport {
            duration,
            samples_count: samples.len(),
            initial_rss,
            peak_rss,
            final_rss,
            average_rss,
            memory_efficiency,
            limit_exceeded,
            violations,
        }
    }
    
    /// Detect memory usage violations from samples
    fn detect_violations(&self, samples: &[MemorySample]) -> Vec<MemoryViolation> {
        let mut violations = Vec::new();
        
        for (i, sample) in samples.iter().enumerate() {
            // Check RSS limit violations
            if sample.rss_bytes > self.limit_bytes {
                violations.push(MemoryViolation {
                    timestamp: sample.timestamp,
                    violation_type: MemoryViolationType::RssLimitExceeded,
                    measured_bytes: sample.rss_bytes,
                    limit_bytes: self.limit_bytes,
                    severity: ViolationSeverity::Critical,
                });
            }
            
            // Check for sudden spikes (50% increase in short time)
            if i > 0 {
                let prev_sample = &samples[i - 1];
                let growth_rate = (sample.rss_bytes as f64 - prev_sample.rss_bytes as f64) / prev_sample.rss_bytes as f64;
                
                if growth_rate > 0.5 {
                    violations.push(MemoryViolation {
                        timestamp: sample.timestamp,
                        violation_type: MemoryViolationType::SuddenSpike,
                        measured_bytes: sample.rss_bytes,
                        limit_bytes: prev_sample.rss_bytes,
                        severity: ViolationSeverity::High,
                    });
                }
            }
            
            // Check for sustained high usage (>90% of limit for >10 samples)
            if sample.rss_bytes > (self.limit_bytes as f64 * 0.9) as u64 {
                let high_usage_count = samples[i.saturating_sub(10)..=i]
                    .iter()
                    .filter(|s| s.rss_bytes > (self.limit_bytes as f64 * 0.9) as u64)
                    .count();
                
                if high_usage_count >= 10 {
                    violations.push(MemoryViolation {
                        timestamp: sample.timestamp,
                        violation_type: MemoryViolationType::SustainedHigh,
                        measured_bytes: sample.rss_bytes,
                        limit_bytes: self.limit_bytes,
                        severity: ViolationSeverity::Medium,
                    });
                }
            }
        }
        
        // Detect potential memory leaks (continuous growth)
        if samples.len() > 20 {
            let early_avg = samples[0..10].iter().map(|s| s.rss_bytes).sum::<u64>() / 10;
            let late_avg = samples[samples.len()-10..].iter().map(|s| s.rss_bytes).sum::<u64>() / 10;
            
            let growth_rate = (late_avg as f64 - early_avg as f64) / early_avg as f64;
            
            if growth_rate > 0.2 { // 20% growth might indicate a leak
                violations.push(MemoryViolation {
                    timestamp: samples.last().unwrap().timestamp,
                    violation_type: MemoryViolationType::MemoryLeak,
                    measured_bytes: late_avg,
                    limit_bytes: early_avg,
                    severity: ViolationSeverity::High,
                });
            }
        }
        
        violations
    }
}

impl MemoryReport {
    /// Print detailed memory report
    pub fn print_report(&self) {
        println!("\n📊 Memory Usage Report");
        println!("========================");
        println!("Duration: {:.2}s", self.duration.as_secs_f64());
        println!("Samples: {}", self.samples_count);
        println!("Initial RSS: {:.1}MB", self.initial_rss as f64 / (1024.0 * 1024.0));
        println!("Peak RSS: {:.1}MB", self.peak_rss as f64 / (1024.0 * 1024.0));
        println!("Final RSS: {:.1}MB", self.final_rss as f64 / (1024.0 * 1024.0));
        println!("Average RSS: {:.1}MB", self.average_rss as f64 / (1024.0 * 1024.0));
        println!("Memory Efficiency: {:.1}%", self.memory_efficiency);
        
        if self.limit_exceeded {
            println!("❌ Limit Exceeded: YES");
        } else {
            println!("✅ Limit Exceeded: NO");
        }
        
        if !self.violations.is_empty() {
            println!("\n⚠️  Memory Violations ({}):", self.violations.len());
            for violation in &self.violations {
                println!(
                    "  {:?}: {:?} at {:.2}s ({:.1}MB)",
                    violation.severity,
                    violation.violation_type,
                    violation.timestamp.as_secs_f64(),
                    violation.measured_bytes as f64 / (1024.0 * 1024.0)
                );
            }
        }
        
        println!();
    }
    
    /// Check if memory usage is acceptable
    pub fn is_acceptable(&self) -> bool {
        !self.limit_exceeded && 
        self.violations.iter().all(|v| v.severity != ViolationSeverity::Critical)
    }
    
    /// Get memory metrics for golden test integration
    pub fn to_memory_metrics(&self) -> MemoryMetrics {
        MemoryMetrics {
            rss_bytes: self.final_rss,
            heap_bytes: 0, // TODO: Implement heap tracking
            peak_rss_bytes: self.peak_rss,
            memory_efficiency: self.memory_efficiency,
        }
    }
}

/// Memory-aware test wrapper
pub async fn monitor_test_memory<F, T, R>(
    test_fn: F,
    memory_limit_mb: u64,
    sample_interval_ms: u64,
) -> Result<(R, MemoryReport), anyhow::Error>
where
    F: FnOnce() -> T,
    T: std::future::Future<Output = Result<R, anyhow::Error>>,
{
    let monitor = MemoryMonitor::new(memory_limit_mb * 1024 * 1024);
    
    monitor.start_monitoring(sample_interval_ms);
    
    let result = test_fn().await;
    
    let report = monitor.stop_monitoring();
    
    match result {
        Ok(value) => Ok((value, report)),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_monitor() {
        let monitor = MemoryMonitor::new(100 * 1024 * 1024); // 100MB limit
        
        monitor.start_monitoring(10); // 10ms intervals
        
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let report = monitor.stop_monitoring();
        
        assert!(report.samples_count > 0);
        assert!(report.duration > Duration::from_millis(50));
    }
    
    #[tokio::test]
    async fn test_memory_limit_detection() {
        let monitor = MemoryMonitor::new(1); // 1 byte limit (will definitely exceed)
        
        monitor.start_monitoring(10);
        tokio::time::sleep(Duration::from_millis(50)).await;
        let report = monitor.stop_monitoring();
        
        assert!(report.limit_exceeded);
        assert!(!report.violations.is_empty());
    }
    
    #[tokio::test]
    async fn test_monitored_test_execution() {
        let test_fn = || async {
            // Simulate test work
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok("test result")
        };
        
        let (result, report) = monitor_test_memory(test_fn, 600, 10).await.unwrap();
        
        assert_eq!(result, "test result");
        assert!(report.samples_count > 0);
    }
}
