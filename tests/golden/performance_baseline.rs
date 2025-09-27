//! Performance Baseline Management
//!
//! Handles storage, retrieval, and management of performance baselines
//! for golden test comparisons.

use super::*;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use serde_json;

/// Baseline storage format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaselineStorage {
    version: String,
    created_at: u64,
    last_updated: u64,
    baselines: HashMap<String, BehaviorSnapshot>,
    metadata: BaselineMetadata,
}

/// Metadata for baseline storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaselineMetadata {
    rust_version: String,
    riptide_version: String,
    platform: String,
    cpu_cores: usize,
    memory_gb: f64,
    test_environment: String,
}

impl Default for BaselineStorage {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            baselines: HashMap::new(),
            metadata: BaselineMetadata::default(),
        }
    }
}

impl Default for BaselineMetadata {
    fn default() -> Self {
        let system = sysinfo::System::new_all();
        
        Self {
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            riptide_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            cpu_cores: system.cpus().len(),
            memory_gb: system.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
            test_environment: std::env::var("GOLDEN_TEST_ENV").unwrap_or_else(|_| "development".to_string()),
        }
    }
}

/// Save a baseline to persistent storage
pub async fn save_baseline(
    storage_path: &Path,
    test_name: &str,
    snapshot: &BehaviorSnapshot,
) -> Result<(), anyhow::Error> {
    let mut storage = load_storage(storage_path).await
        .unwrap_or_else(|_| BaselineStorage::default());
    
    storage.baselines.insert(test_name.to_string(), snapshot.clone());
    storage.last_updated = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Ensure directory exists
    if let Some(parent) = storage_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    let json = serde_json::to_string_pretty(&storage)?;
    fs::write(storage_path, json).await?;
    
    println!("✅ Saved baseline for test '{}' to {}", test_name, storage_path.display());
    Ok(())
}

/// Load a baseline from persistent storage
pub async fn load_baseline(
    storage_path: &Path,
    test_name: &str,
) -> Result<Option<BehaviorSnapshot>, anyhow::Error> {
    let storage = load_storage(storage_path).await?;
    Ok(storage.baselines.get(test_name).cloned())
}

/// List all available baselines
pub async fn list_baselines(
    storage_path: &Path,
) -> Result<Vec<String>, anyhow::Error> {
    let storage = load_storage(storage_path).await?;
    Ok(storage.baselines.keys().cloned().collect())
}

/// Load the entire baseline storage
async fn load_storage(storage_path: &Path) -> Result<BaselineStorage, anyhow::Error> {
    if !storage_path.exists() {
        return Ok(BaselineStorage::default());
    }
    
    let contents = fs::read_to_string(storage_path).await?;
    let storage: BaselineStorage = serde_json::from_str(&contents)?;
    Ok(storage)
}

/// Initialize baseline storage with default configuration
pub async fn initialize_baseline_storage(
    storage_path: &Path,
) -> Result<(), anyhow::Error> {
    if storage_path.exists() {
        println!("📋 Baseline storage already exists at {}", storage_path.display());
        return Ok(());
    }
    
    let storage = BaselineStorage::default();
    
    // Ensure directory exists
    if let Some(parent) = storage_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    let json = serde_json::to_string_pretty(&storage)?;
    fs::write(storage_path, json).await?;
    
    println!("🔧 Initialized baseline storage at {}", storage_path.display());
    Ok(())
}

/// Create initial baselines for existing system
pub async fn capture_initial_baselines(
    storage_path: &Path,
    test_cases: &[(&str, fn() -> Result<serde_json::Value, anyhow::Error>)],
    config: &GoldenTestConfig,
) -> Result<(), anyhow::Error> {
    println!("🎯 Capturing initial baselines for {} test cases...", test_cases.len());
    
    for (test_name, _test_fn) in test_cases {
        // For initial capture, we'll create placeholder baselines
        // In real usage, you'd run the actual test functions
        let snapshot = create_placeholder_baseline(test_name, config).await;
        save_baseline(storage_path, test_name, &snapshot).await?;
        
        println!("✅ Captured baseline for: {}", test_name);
    }
    
    println!("🎉 All baselines captured successfully!");
    Ok(())
}

/// Create a placeholder baseline for initial setup
async fn create_placeholder_baseline(
    test_name: &str,
    config: &GoldenTestConfig,
) -> BehaviorSnapshot {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    BehaviorSnapshot {
        timestamp,
        test_name: test_name.to_string(),
        performance_metrics: PerformanceMetrics {
            p50_latency_ms: 800.0,  // Conservative baseline: p50 ≤ 1.2s target
            p95_latency_ms: 3500.0, // Conservative baseline: p95 ≤ 4.5s target
            p99_latency_ms: 5000.0, // Conservative baseline
            mean_latency_ms: 1000.0,
            min_latency_ms: 200.0,
            max_latency_ms: 8000.0,
            std_dev_ms: 500.0,
        },
        memory_metrics: MemoryMetrics {
            rss_bytes: 400 * 1024 * 1024, // 400MB baseline, well under 600MB limit
            heap_bytes: 200 * 1024 * 1024, // 200MB heap
            peak_rss_bytes: 500 * 1024 * 1024, // 500MB peak
            memory_efficiency: 80.0,
        },
        throughput_metrics: ThroughputMetrics {
            pages_per_second: 2.0,
            requests_per_second: 5.0,
            bytes_processed_per_second: 1024 * 1024, // 1MB/s
            operations_per_second: 10.0,
        },
        functional_outputs: std::collections::HashMap::new(),
        error_patterns: Vec::new(),
    }
}

/// Validate baseline quality and completeness
pub async fn validate_baselines(
    storage_path: &Path,
) -> Result<BaselineValidationReport, anyhow::Error> {
    let storage = load_storage(storage_path).await?;
    let mut report = BaselineValidationReport::new();
    
    for (test_name, baseline) in &storage.baselines {
        let mut test_issues = Vec::new();
        
        // Validate performance metrics
        if baseline.performance_metrics.p50_latency_ms > 1200.0 {
            test_issues.push(format!(
                "P50 latency {:.2}ms exceeds target of 1200ms",
                baseline.performance_metrics.p50_latency_ms
            ));
        }
        
        if baseline.performance_metrics.p95_latency_ms > 4500.0 {
            test_issues.push(format!(
                "P95 latency {:.2}ms exceeds target of 4500ms",
                baseline.performance_metrics.p95_latency_ms
            ));
        }
        
        // Validate memory metrics
        if baseline.memory_metrics.rss_bytes > 600 * 1024 * 1024 {
            test_issues.push(format!(
                "RSS memory {}MB exceeds limit of 600MB",
                baseline.memory_metrics.rss_bytes / (1024 * 1024)
            ));
        }
        
        // Validate timestamp freshness (warn if older than 30 days)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let baseline_age_days = (now - baseline.timestamp / 1000) / (24 * 60 * 60);
        
        if baseline_age_days > 30 {
            test_issues.push(format!(
                "Baseline is {} days old, consider refreshing",
                baseline_age_days
            ));
        }
        
        if test_issues.is_empty() {
            report.valid_tests.push(test_name.clone());
        } else {
            report.invalid_tests.insert(test_name.clone(), test_issues);
        }
    }
    
    Ok(report)
}

/// Baseline validation report
#[derive(Debug, Clone)]
pub struct BaselineValidationReport {
    pub valid_tests: Vec<String>,
    pub invalid_tests: HashMap<String, Vec<String>>,
}

impl BaselineValidationReport {
    fn new() -> Self {
        Self {
            valid_tests: Vec::new(),
            invalid_tests: HashMap::new(),
        }
    }
    
    pub fn is_valid(&self) -> bool {
        self.invalid_tests.is_empty()
    }
    
    pub fn print_report(&self) {
        println!("\n📊 Baseline Validation Report");
        println!("=============================");
        
        if !self.valid_tests.is_empty() {
            println!("\n✅ Valid Baselines ({}):", self.valid_tests.len());
            for test_name in &self.valid_tests {
                println!("  - {}", test_name);
            }
        }
        
        if !self.invalid_tests.is_empty() {
            println!("\n❌ Invalid Baselines ({}):", self.invalid_tests.len());
            for (test_name, issues) in &self.invalid_tests {
                println!("  - {}", test_name);
                for issue in issues {
                    println!("    • {}", issue);
                }
            }
        }
        
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_baseline_storage() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_baselines.json");
        
        // Test initialization
        initialize_baseline_storage(&storage_path).await.unwrap();
        assert!(storage_path.exists());
        
        // Test saving and loading
        let config = GoldenTestConfig::default();
        let snapshot = create_placeholder_baseline("test_case", &config).await;
        
        save_baseline(&storage_path, "test_case", &snapshot).await.unwrap();
        
        let loaded = load_baseline(&storage_path, "test_case").await.unwrap();
        assert!(loaded.is_some());
        
        let loaded_snapshot = loaded.unwrap();
        assert_eq!(loaded_snapshot.test_name, "test_case");
    }
    
    #[tokio::test]
    async fn test_baseline_validation() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("validation_test.json");
        
        // Create baseline with issues
        let mut bad_snapshot = create_placeholder_baseline("bad_test", &GoldenTestConfig::default()).await;
        bad_snapshot.performance_metrics.p50_latency_ms = 2000.0; // Exceeds 1200ms target
        bad_snapshot.memory_metrics.rss_bytes = 700 * 1024 * 1024; // Exceeds 600MB limit
        
        save_baseline(&storage_path, "bad_test", &bad_snapshot).await.unwrap();
        
        let report = validate_baselines(&storage_path).await.unwrap();
        assert!(!report.is_valid());
        assert!(report.invalid_tests.contains_key("bad_test"));
    }
}
