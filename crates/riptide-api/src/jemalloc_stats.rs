//! jemalloc memory statistics collection
//!
//! Provides detailed memory metrics using tikv-jemalloc-ctl for monitoring
//! allocator behavior, memory usage, and fragmentation.
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use tikv_jemalloc_ctl::{epoch, stats};

/// Memory statistics collected from jemalloc
#[derive(Debug, Clone, Default)]
pub struct JemallocStats {
    /// Total number of bytes allocated by the application
    pub allocated: usize,
    /// Total number of bytes in active pages allocated by the application
    pub active: usize,
    /// Maximum number of bytes in physically resident data pages mapped
    pub resident: usize,
    /// Total number of bytes dedicated to jemalloc metadata
    pub metadata: usize,
    /// Total number of bytes in chunks mapped on behalf of the application
    pub mapped: usize,
    /// Total number of bytes retained for future allocations
    pub retained: usize,
}

impl JemallocStats {
    /// Collect current memory statistics from jemalloc
    ///
    /// Returns None if jemalloc is not enabled or stats collection fails
    #[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
    pub fn collect() -> Option<Self> {
        // Refresh jemalloc's internal statistics
        if let Err(e) = epoch::mib() {
            tracing::warn!("Failed to get jemalloc epoch MIB: {}", e);
            return None;
        }

        let epoch_mib = match epoch::mib() {
            Ok(mib) => mib,
            Err(e) => {
                tracing::warn!("Failed to create jemalloc epoch MIB: {}", e);
                return None;
            }
        };

        if let Err(e) = epoch_mib.advance() {
            tracing::warn!("Failed to advance jemalloc epoch: {}", e);
            return None;
        }

        // Collect statistics
        let allocated = stats::allocated::read().ok()?;
        let active = stats::active::read().ok()?;
        let resident = stats::resident::read().ok()?;
        let metadata = stats::metadata::read().ok()?;
        let mapped = stats::mapped::read().ok()?;
        let retained = stats::retained::read().ok()?;

        Some(Self {
            allocated,
            active,
            resident,
            metadata,
            mapped,
            retained,
        })
    }

    /// Collect current memory statistics from jemalloc (fallback for non-jemalloc builds)
    #[cfg(not(all(feature = "jemalloc", not(target_env = "msvc"))))]
    pub fn collect() -> Option<Self> {
        None
    }

    /// Calculate memory fragmentation ratio
    ///
    /// Returns the ratio of active to allocated memory. A higher ratio indicates
    /// more fragmentation. Ideal ratio is close to 1.0.
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.allocated == 0 {
            return 0.0;
        }
        self.active as f64 / self.allocated as f64
    }

    /// Calculate metadata overhead ratio
    ///
    /// Returns the ratio of metadata to allocated memory. Lower is better.
    pub fn metadata_overhead_ratio(&self) -> f64 {
        if self.allocated == 0 {
            return 0.0;
        }
        self.metadata as f64 / self.allocated as f64
    }

    /// Calculate resident to mapped ratio
    ///
    /// Returns the ratio of resident to mapped memory. Indicates how much
    /// of the mapped memory is actually resident in physical RAM.
    #[allow(dead_code)]
    pub fn resident_ratio(&self) -> f64 {
        if self.mapped == 0 {
            return 0.0;
        }
        self.resident as f64 / self.mapped as f64
    }

    /// Get allocated memory in megabytes
    pub fn allocated_mb(&self) -> f64 {
        self.allocated as f64 / (1024.0 * 1024.0)
    }

    /// Get resident memory in megabytes
    pub fn resident_mb(&self) -> f64 {
        self.resident as f64 / (1024.0 * 1024.0)
    }

    /// Get metadata memory in megabytes
    pub fn metadata_mb(&self) -> f64 {
        self.metadata as f64 / (1024.0 * 1024.0)
    }

    /// Get mapped memory in megabytes (available for future memory monitoring features)
    #[allow(dead_code)]
    pub fn mapped_mb(&self) -> f64 {
        self.mapped as f64 / (1024.0 * 1024.0)
    }

    /// Get retained memory in megabytes (available for future memory monitoring features)
    #[allow(dead_code)]
    pub fn retained_mb(&self) -> f64 {
        self.retained as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragmentation_ratio() {
        let stats = JemallocStats {
            allocated: 1000,
            active: 1200,
            ..Default::default()
        };
        assert!((stats.fragmentation_ratio() - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_metadata_overhead_ratio() {
        let stats = JemallocStats {
            allocated: 1000,
            metadata: 50,
            ..Default::default()
        };
        assert!((stats.metadata_overhead_ratio() - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_resident_ratio() {
        let stats = JemallocStats {
            resident: 800,
            mapped: 1000,
            ..Default::default()
        };
        assert!((stats.resident_ratio() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_megabyte_conversions() {
        let stats = JemallocStats {
            allocated: 1024 * 1024,
            resident: 2 * 1024 * 1024,
            metadata: 512 * 1024,
            ..Default::default()
        };
        assert!((stats.allocated_mb() - 1.0).abs() < 0.001);
        assert!((stats.resident_mb() - 2.0).abs() < 0.001);
        assert!((stats.metadata_mb() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_zero_division_safety() {
        let stats = JemallocStats::default();
        assert_eq!(stats.fragmentation_ratio(), 0.0);
        assert_eq!(stats.metadata_overhead_ratio(), 0.0);
        assert_eq!(stats.resident_ratio(), 0.0);
    }
}
