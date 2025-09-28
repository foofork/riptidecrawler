//! Allocation pattern analysis and top allocator tracking

use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info};

use super::AllocationInfo;

/// Analyzer for allocation patterns and top allocators
pub struct AllocationAnalyzer {
    allocator_stats: HashMap<String, AllocatorStats>,
    operation_stats: HashMap<String, OperationStats>,
    size_distribution: SizeDistribution,
}

/// Statistics for a specific allocator/component
#[derive(Debug, Clone)]
struct AllocatorStats {
    total_allocations: u64,
    total_bytes: u64,
    peak_bytes: u64,
    average_size: f64,
    largest_allocation: usize,
    recent_allocations: Vec<AllocationInfo>,
}

/// Statistics for allocation operations
#[derive(Debug, Clone)]
struct OperationStats {
    count: u64,
    total_bytes: u64,
    average_size: f64,
    components: HashMap<String, u64>,
}

/// Size distribution analysis
#[derive(Debug, Clone)]
struct SizeDistribution {
    tiny: u64,      // <1KB
    small: u64,     // 1KB-64KB
    medium: u64,    // 64KB-1MB
    large: u64,     // 1MB-16MB
    huge: u64,      // >16MB
}

impl AllocationAnalyzer {
    /// Create a new allocation analyzer
    pub fn new() -> Result<Self> {
        debug!("Created allocation analyzer");

        Ok(Self {
            allocator_stats: HashMap::new(),
            operation_stats: HashMap::new(),
            size_distribution: SizeDistribution {
                tiny: 0,
                small: 0,
                medium: 0,
                large: 0,
                huge: 0,
            },
        })
    }

    /// Record an allocation for analysis
    pub async fn record_allocation(&mut self, allocation: AllocationInfo) -> Result<()> {
        let component = &allocation.component;
        let operation = &allocation.operation;
        let size = allocation.size;

        // Update allocator stats
        let allocator_stats = self.allocator_stats.entry(component.clone())
            .or_insert_with(|| AllocatorStats {
                total_allocations: 0,
                total_bytes: 0,
                peak_bytes: 0,
                average_size: 0.0,
                largest_allocation: 0,
                recent_allocations: Vec::new(),
            });

        allocator_stats.total_allocations += 1;
        allocator_stats.total_bytes += size as u64;
        allocator_stats.largest_allocation = allocator_stats.largest_allocation.max(size);
        allocator_stats.average_size = allocator_stats.total_bytes as f64 / allocator_stats.total_allocations as f64;

        // Keep recent allocations for pattern analysis
        allocator_stats.recent_allocations.push(allocation.clone());
        if allocator_stats.recent_allocations.len() > 50 {
            allocator_stats.recent_allocations.drain(0..25);
        }

        // Update operation stats
        let operation_stats = self.operation_stats.entry(operation.clone())
            .or_insert_with(|| OperationStats {
                count: 0,
                total_bytes: 0,
                average_size: 0.0,
                components: HashMap::new(),
            });

        operation_stats.count += 1;
        operation_stats.total_bytes += size as u64;
        operation_stats.average_size = operation_stats.total_bytes as f64 / operation_stats.count as f64;
        *operation_stats.components.entry(component.clone()).or_insert(0) += 1;

        // Update size distribution
        match size {
            0..=1024 => self.size_distribution.tiny += 1,
            1025..=65536 => self.size_distribution.small += 1,
            65537..=1048576 => self.size_distribution.medium += 1,
            1048577..=16777216 => self.size_distribution.large += 1,
            _ => self.size_distribution.huge += 1,
        }

        debug!(
            component = component,
            operation = operation,
            size = size,
            "Recorded allocation for analysis"
        );

        Ok(())
    }

    /// Get top allocators by total bytes allocated
    pub async fn get_top_allocators(&self) -> Result<Vec<(String, u64)>> {
        let mut allocators: Vec<(String, u64)> = self.allocator_stats
            .iter()
            .map(|(name, stats)| (name.clone(), stats.total_bytes))
            .collect();

        allocators.sort_by(|a, b| b.1.cmp(&a.1));
        allocators.truncate(20); // Top 20

        info!(
            count = allocators.len(),
            "Retrieved top allocators"
        );

        Ok(allocators)
    }

    /// Get top operations by frequency
    pub async fn get_top_operations(&self) -> Result<Vec<(String, u64, f64)>> {
        let mut operations: Vec<(String, u64, f64)> = self.operation_stats
            .iter()
            .map(|(name, stats)| (name.clone(), stats.count, stats.average_size))
            .collect();

        operations.sort_by(|a, b| b.1.cmp(&a.1));
        operations.truncate(20); // Top 20

        info!(
            count = operations.len(),
            "Retrieved top operations"
        );

        Ok(operations)
    }

    /// Get allocation size distribution
    pub async fn get_size_distribution(&self) -> Result<HashMap<String, u64>> {
        let mut distribution = HashMap::new();

        distribution.insert("tiny (<1KB)".to_string(), self.size_distribution.tiny);
        distribution.insert("small (1KB-64KB)".to_string(), self.size_distribution.small);
        distribution.insert("medium (64KB-1MB)".to_string(), self.size_distribution.medium);
        distribution.insert("large (1MB-16MB)".to_string(), self.size_distribution.large);
        distribution.insert("huge (>16MB)".to_string(), self.size_distribution.huge);

        Ok(distribution)
    }

    /// Analyze allocation patterns for optimization opportunities
    pub async fn analyze_patterns(&self) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Analyze size distribution for pooling opportunities
        let total_allocations = self.size_distribution.tiny +
            self.size_distribution.small +
            self.size_distribution.medium +
            self.size_distribution.large +
            self.size_distribution.huge;

        if total_allocations > 0 {
            let small_percentage = (self.size_distribution.tiny + self.size_distribution.small) as f64 / total_allocations as f64 * 100.0;

            if small_percentage > 80.0 {
                recommendations.push("High percentage of small allocations detected. Consider implementing object pools for frequently allocated small objects.".to_string());
            }

            let huge_percentage = self.size_distribution.huge as f64 / total_allocations as f64 * 100.0;

            if huge_percentage > 5.0 {
                recommendations.push("Significant number of huge allocations (>16MB) detected. Consider streaming or chunking for large data processing.".to_string());
            }
        }

        // Analyze top allocators for optimization
        let top_allocators = self.get_top_allocators().await?;

        for (component, bytes) in &top_allocators[..3.min(top_allocators.len())] {
            if *bytes > 100 * 1024 * 1024 { // >100MB
                recommendations.push(format!("Component '{}' has high memory allocation ({}MB). Consider optimizing memory usage or implementing cleanup routines.", component, bytes / 1024 / 1024));
            }
        }

        // Analyze operation patterns
        let top_operations = self.get_top_operations().await?;

        for (operation, count, avg_size) in &top_operations[..3.min(top_operations.len())] {
            if *count > 10000 {
                recommendations.push(format!("Operation '{}' called very frequently ({} times, avg {}bytes). Consider batching or caching.", operation, count, *avg_size as u64));
            }
        }

        // Analyze allocation frequency per component
        for (component, stats) in &self.allocator_stats {
            if stats.total_allocations > 5000 && stats.average_size < 1024.0 {
                recommendations.push(format!("Component '{}' makes many small allocations (avg {:.0}bytes). Consider using a memory pool or pre-allocation.", component, stats.average_size));
            }
        }

        info!(
            recommendations = recommendations.len(),
            "Generated allocation pattern analysis"
        );

        Ok(recommendations)
    }

    /// Get memory fragmentation analysis
    pub async fn analyze_fragmentation(&self) -> Result<HashMap<String, f64>> {
        let mut fragmentation = HashMap::new();

        for (component, stats) in &self.allocator_stats {
            // Simple fragmentation metric: ratio of largest to average allocation
            if stats.average_size > 0.0 {
                let fragmentation_ratio = stats.largest_allocation as f64 / stats.average_size;
                fragmentation.insert(component.clone(), fragmentation_ratio);
            }
        }

        Ok(fragmentation)
    }

    /// Get allocation timeline for trending
    pub async fn get_allocation_timeline(&self, component: &str) -> Result<Vec<(chrono::DateTime<chrono::Utc>, usize)>> {
        if let Some(stats) = self.allocator_stats.get(component) {
            let timeline: Vec<(chrono::DateTime<chrono::Utc>, usize)> = stats.recent_allocations
                .iter()
                .map(|allocation| (allocation.timestamp, allocation.size))
                .collect();

            Ok(timeline)
        } else {
            Ok(Vec::new())
        }
    }

    /// Generate allocation efficiency score (0.0 = poor, 1.0 = excellent)
    pub async fn calculate_efficiency_score(&self) -> Result<f64> {
        let total_allocations = self.allocator_stats.values()
            .map(|stats| stats.total_allocations)
            .sum::<u64>();

        if total_allocations == 0 {
            return Ok(1.0);
        }

        let total_size = self.size_distribution.tiny +
            self.size_distribution.small +
            self.size_distribution.medium +
            self.size_distribution.large +
            self.size_distribution.huge;

        // Efficiency factors
        let size_efficiency = if total_size > 0 {
            // Prefer medium-sized allocations (more efficient than many tiny or few huge)
            let medium_ratio = self.size_distribution.medium as f64 / total_size as f64;
            (medium_ratio * 2.0).min(1.0)
        } else {
            1.0
        };

        // Component concentration (prefer fewer components doing more work)
        let component_count = self.allocator_stats.len() as f64;
        let concentration_efficiency = if component_count > 0.0 {
            (20.0 / component_count).min(1.0)
        } else {
            1.0
        };

        // Overall efficiency score
        let efficiency = (size_efficiency + concentration_efficiency) / 2.0;

        Ok(efficiency)
    }

    /// Clear old data to prevent memory growth
    pub async fn cleanup_old_data(&mut self, max_age_hours: f64) -> Result<()> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        for stats in self.allocator_stats.values_mut() {
            stats.recent_allocations.retain(|allocation| allocation.timestamp >= cutoff);
        }

        debug!("Cleaned up old allocation analysis data");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allocation_analyzer_creation() {
        let analyzer = AllocationAnalyzer::new().unwrap();
        assert!(analyzer.allocator_stats.is_empty());
    }

    #[tokio::test]
    async fn test_allocation_recording() {
        let mut analyzer = AllocationAnalyzer::new().unwrap();

        let allocation = AllocationInfo {
            timestamp: chrono::Utc::now(),
            size: 1024,
            alignment: 8,
            stack_trace: vec!["test_function".to_string()],
            component: "test_component".to_string(),
            operation: "test_operation".to_string(),
        };

        analyzer.record_allocation(allocation).await.unwrap();

        assert_eq!(analyzer.allocator_stats.len(), 1);
        assert_eq!(analyzer.operation_stats.len(), 1);
    }

    #[tokio::test]
    async fn test_top_allocators() {
        let mut analyzer = AllocationAnalyzer::new().unwrap();

        // Add some test allocations
        for i in 0..5 {
            let allocation = AllocationInfo {
                timestamp: chrono::Utc::now(),
                size: (i + 1) * 1024,
                alignment: 8,
                stack_trace: vec![format!("function_{}", i)],
                component: format!("component_{}", i),
                operation: "test_operation".to_string(),
            };

            analyzer.record_allocation(allocation).await.unwrap();
        }

        let top_allocators = analyzer.get_top_allocators().await.unwrap();
        assert_eq!(top_allocators.len(), 5);

        // Should be sorted by bytes (largest first)
        assert!(top_allocators[0].1 >= top_allocators[1].1);
    }

    #[tokio::test]
    async fn test_size_distribution() {
        let mut analyzer = AllocationAnalyzer::new().unwrap();

        // Add allocations of different sizes
        let sizes = vec![100, 2048, 100000, 2000000, 20000000];

        for (i, size) in sizes.iter().enumerate() {
            let allocation = AllocationInfo {
                timestamp: chrono::Utc::now(),
                size: *size,
                alignment: 8,
                stack_trace: vec![format!("function_{}", i)],
                component: "test_component".to_string(),
                operation: "test_operation".to_string(),
            };

            analyzer.record_allocation(allocation).await.unwrap();
        }

        let distribution = analyzer.get_size_distribution().await.unwrap();

        assert_eq!(distribution["tiny (<1KB)"], 1);
        assert_eq!(distribution["small (1KB-64KB)"], 1);
        assert_eq!(distribution["medium (64KB-1MB)"], 1);
        assert_eq!(distribution["large (1MB-16MB)"], 1);
        assert_eq!(distribution["huge (>16MB)"], 1);
    }

    #[tokio::test]
    async fn test_efficiency_score() {
        let analyzer = AllocationAnalyzer::new().unwrap();
        let score = analyzer.calculate_efficiency_score().await.unwrap();

        // Empty analyzer should have perfect efficiency
        assert_eq!(score, 1.0);
    }
}