//! Memory leak detection and analysis

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

use super::{LeakAnalysis, LeakInfo, AllocationInfo};

/// Memory leak detector that tracks allocation patterns
pub struct LeakDetector {
    allocations: HashMap<String, ComponentAllocations>,
    start_time: Instant,
    #[allow(dead_code)]
    last_analysis: Option<Instant>,
}

/// Allocation tracking for a specific component
#[derive(Debug, Clone)]
struct ComponentAllocations {
    total_allocations: u64,
    total_size: u64,
    peak_size: u64,
    recent_allocations: Vec<AllocationInfo>,
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen: chrono::DateTime<chrono::Utc>,
}

impl LeakDetector {
    /// Create a new leak detector
    pub fn new() -> Result<Self> {
        debug!("Created memory leak detector");

        Ok(Self {
            allocations: HashMap::new(),
            start_time: Instant::now(),
            last_analysis: None,
        })
    }

    /// Record an allocation
    pub async fn record_allocation(&mut self, allocation: AllocationInfo) -> Result<()> {
        let component = allocation.component.clone();
        let size = allocation.size as u64;

        let entry = self.allocations.entry(component.clone())
            .or_insert_with(|| ComponentAllocations {
                total_allocations: 0,
                total_size: 0,
                peak_size: 0,
                recent_allocations: Vec::new(),
                first_seen: allocation.timestamp,
                last_seen: allocation.timestamp,
            });

        entry.total_allocations += 1;
        entry.total_size += size;
        entry.peak_size = entry.peak_size.max(entry.total_size);
        entry.last_seen = allocation.timestamp;

        // Keep only recent allocations to avoid memory growth
        entry.recent_allocations.push(allocation);
        if entry.recent_allocations.len() > 100 {
            entry.recent_allocations.drain(0..50);
        }

        debug!(
            component = component,
            size = size,
            total_allocations = entry.total_allocations,
            "Recorded allocation"
        );

        Ok(())
    }

    /// Record a deallocation
    pub async fn record_deallocation(&mut self, component: &str, size: u64) -> Result<()> {
        if let Some(entry) = self.allocations.get_mut(component) {
            entry.total_size = entry.total_size.saturating_sub(size);
            entry.last_seen = chrono::Utc::now();

            debug!(
                component = component,
                size = size,
                remaining_size = entry.total_size,
                "Recorded deallocation"
            );
        }

        Ok(())
    }

    /// Analyze potential memory leaks
    pub async fn analyze_leaks(&self) -> Result<LeakAnalysis> {
        let analysis_time = Instant::now();
        let elapsed = analysis_time.duration_since(self.start_time);

        info!("Starting memory leak analysis");

        let mut potential_leaks = Vec::new();
        let mut total_growth = 0u64;
        let mut largest_allocations = Vec::new();
        let mut suspicious_patterns = Vec::new();

        for (component, allocations) in &self.allocations {
            // Calculate growth rate
            let duration_hours = elapsed.as_secs_f64() / 3600.0;
            let growth_rate = if duration_hours > 0.0 {
                allocations.total_size as f64 / duration_hours
            } else {
                0.0
            };

            // Check for potential leaks based on various criteria
            let is_potential_leak = self.is_potential_leak(allocations, growth_rate);

            if is_potential_leak {
                let leak_info = LeakInfo {
                    component: component.clone(),
                    allocation_count: allocations.total_allocations,
                    total_size_bytes: allocations.total_size,
                    average_size_bytes: if allocations.total_allocations > 0 {
                        allocations.total_size as f64 / allocations.total_allocations as f64
                    } else {
                        0.0
                    },
                    growth_rate,
                    first_seen: allocations.first_seen,
                    last_seen: allocations.last_seen,
                };

                potential_leaks.push(leak_info);
                total_growth += allocations.total_size;
            }

            // Collect largest allocations
            for allocation in &allocations.recent_allocations {
                largest_allocations.push(allocation.clone());
            }

            // Detect suspicious patterns
            if let Some(pattern) = self.detect_suspicious_pattern(component, allocations) {
                suspicious_patterns.push(pattern);
            }
        }

        // Sort by severity
        potential_leaks.sort_by(|a, b| {
            b.total_size_bytes.cmp(&a.total_size_bytes)
                .then(b.growth_rate.partial_cmp(&a.growth_rate).unwrap_or(std::cmp::Ordering::Equal))
        });

        // Sort largest allocations by size
        largest_allocations.sort_by(|a, b| b.size.cmp(&a.size));
        largest_allocations.truncate(20); // Keep top 20

        let growth_rate_mb_per_hour = if elapsed.as_secs_f64() > 0.0 {
            (total_growth as f64 / 1024.0 / 1024.0) / (elapsed.as_secs_f64() / 3600.0)
        } else {
            0.0
        };

        let analysis = LeakAnalysis {
            potential_leaks,
            growth_rate_mb_per_hour,
            largest_allocations,
            suspicious_patterns,
        };

        info!(
            potential_leaks = analysis.potential_leaks.len(),
            growth_rate_mb_h = growth_rate_mb_per_hour,
            "Memory leak analysis completed"
        );

        Ok(analysis)
    }

    /// Check if allocations indicate a potential memory leak
    fn is_potential_leak(&self, allocations: &ComponentAllocations, growth_rate: f64) -> bool {
        // Criteria for potential memory leak:

        // 1. High growth rate (>10MB/hour)
        if growth_rate > 10.0 * 1024.0 * 1024.0 {
            return true;
        }

        // 2. Large total size (>50MB) with recent activity
        if allocations.total_size > 50 * 1024 * 1024 {
            let recent_activity = chrono::Utc::now().timestamp() - allocations.last_seen.timestamp() < 60;
            if recent_activity {
                return true;
            }
        }

        // 3. Many small allocations without corresponding deallocations
        if allocations.total_allocations > 1000 && allocations.total_size > 1024 * 1024 {
            return true;
        }

        // 4. Steadily growing peak size
        if allocations.peak_size > allocations.total_size * 2 {
            return true;
        }

        false
    }

    /// Detect suspicious allocation patterns
    fn detect_suspicious_pattern(&self, component: &str, allocations: &ComponentAllocations) -> Option<String> {
        // Pattern 1: Exponential growth
        if allocations.recent_allocations.len() >= 10 {
            let recent = &allocations.recent_allocations[allocations.recent_allocations.len() - 10..];
            let sizes: Vec<usize> = recent.iter().map(|a| a.size).collect();

            if self.is_exponential_growth(&sizes) {
                return Some(format!("{}: Exponential allocation growth detected", component));
            }
        }

        // Pattern 2: Regular large allocations
        let large_allocations = allocations.recent_allocations
            .iter()
            .filter(|a| a.size > 1024 * 1024) // >1MB
            .count();

        if large_allocations > 5 {
            return Some(format!("{}: Frequent large allocations ({})", component, large_allocations));
        }

        // Pattern 3: Identical stack traces (potential loop leak)
        if allocations.recent_allocations.len() >= 5 {
            let mut stack_counts = HashMap::new();
            for allocation in &allocations.recent_allocations {
                *stack_counts.entry(&allocation.stack_trace).or_insert(0) += 1;
            }

            for (_stack, count) in stack_counts {
                if count >= 5 {
                    return Some(format!("{}: Repeated allocation pattern detected ({} times)", component, count));
                }
            }
        }

        None
    }

    /// Check if a sequence of numbers shows exponential growth
    fn is_exponential_growth(&self, sizes: &[usize]) -> bool {
        if sizes.len() < 5 {
            return false;
        }

        // Check if each element is roughly double the previous
        let mut exponential_count = 0;
        for i in 1..sizes.len() {
            if sizes[i] > sizes[i-1] * 2 {
                exponential_count += 1;
            }
        }

        exponential_count >= sizes.len() / 2
    }

    /// Get memory pressure score (0.0 = low, 1.0 = high)
    pub async fn get_memory_pressure(&self) -> Result<f64> {
        let total_size: u64 = self.allocations.values()
            .map(|a| a.total_size)
            .sum();

        let total_allocations: u64 = self.allocations.values()
            .map(|a| a.total_allocations)
            .sum();

        // Calculate pressure based on total memory and allocation rate
        let size_pressure = (total_size as f64 / (500.0 * 1024.0 * 1024.0)).min(1.0); // 500MB = high pressure
        let allocation_pressure = (total_allocations as f64 / 10000.0).min(1.0); // 10k allocations = high pressure

        Ok((size_pressure + allocation_pressure) / 2.0)
    }

    /// Clear old allocation data to prevent memory growth
    pub async fn cleanup_old_data(&mut self, retention_period: Duration) -> Result<()> {
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(retention_period)?;

        for allocations in self.allocations.values_mut() {
            allocations.recent_allocations.retain(|a| a.timestamp >= cutoff);
        }

        // Remove components with no recent allocations
        self.allocations.retain(|_, allocations| {
            !allocations.recent_allocations.is_empty() || allocations.last_seen >= cutoff
        });

        debug!("Cleaned up old allocation data");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_leak_detector_creation() {
        let detector = LeakDetector::new().unwrap();
        assert!(detector.allocations.is_empty());
    }

    #[tokio::test]
    async fn test_allocation_recording() {
        let mut detector = LeakDetector::new().unwrap();

        let allocation = AllocationInfo {
            timestamp: chrono::Utc::now(),
            size: 1024,
            alignment: 8,
            stack_trace: vec!["test_function".to_string()],
            component: "test_component".to_string(),
            operation: "test_operation".to_string(),
        };

        detector.record_allocation(allocation).await.unwrap();

        assert_eq!(detector.allocations.len(), 1);
        assert!(detector.allocations.contains_key("test_component"));
    }

    #[tokio::test]
    async fn test_leak_analysis() {
        let mut detector = LeakDetector::new().unwrap();

        // Add some test allocations
        for i in 0..10 {
            let allocation = AllocationInfo {
                timestamp: chrono::Utc::now(),
                size: 1024 * 1024, // 1MB each
                alignment: 8,
                stack_trace: vec![format!("function_{}", i)],
                component: "test_component".to_string(),
                operation: "test_operation".to_string(),
            };

            detector.record_allocation(allocation).await.unwrap();
        }

        let analysis = detector.analyze_leaks().await.unwrap();

        // Should detect potential leak due to large allocations
        assert!(!analysis.potential_leaks.is_empty());
    }

    #[tokio::test]
    async fn test_memory_pressure() {
        let detector = LeakDetector::new().unwrap();
        let pressure = detector.get_memory_pressure().await.unwrap();

        // Should be low for empty detector
        assert!(pressure < 0.1);
    }
}