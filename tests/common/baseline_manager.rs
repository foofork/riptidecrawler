/// Baseline Management System for Regression Testing
///
/// Manages storage and comparison of baseline extraction results to detect
/// quality degradations over time.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub url: String,
    pub test_id: String,
    pub method: String,
    pub expected_title: Option<String>,
    pub expected_keywords: Vec<String>,
    pub min_quality_score: Option<f64>,
    pub max_extraction_time_ms: Option<u64>,
    pub min_content_length: Option<usize>,
    pub expected_metadata: HashMap<String, serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub test_id: String,
    pub method: String,
    pub passed: bool,
    pub differences: Vec<Difference>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    pub field: String,
    pub baseline_value: serde_json::Value,
    pub current_value: serde_json::Value,
    pub diff_percentage: Option<f64>,
    pub severity: DifferenceSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifferenceSeverity {
    Critical,  // >50% change
    Major,     // 20-50% change
    Minor,     // 5-20% change
    Negligible, // <5% change
}

pub struct BaselineManager {
    baselines_dir: PathBuf,
}

impl BaselineManager {
    pub fn new(baselines_dir: PathBuf) -> Result<Self> {
        // Ensure baselines directory exists
        fs::create_dir_all(&baselines_dir)
            .context("Failed to create baselines directory")?;
        Ok(Self { baselines_dir })
    }

    /// Generate a baseline from extraction result
    pub fn generate_baseline(
        &self,
        test_id: &str,
        method: &str,
        url: &str,
        content: &str,
        metadata: &HashMap<String, serde_json::Value>,
        duration_ms: u64,
    ) -> Result<Baseline> {
        let baseline = Baseline {
            url: url.to_string(),
            test_id: test_id.to_string(),
            method: method.to_string(),
            expected_title: Self::extract_title(content, metadata),
            expected_keywords: Self::extract_keywords(content),
            min_quality_score: metadata.get("quality_score").and_then(|v| v.as_f64()),
            max_extraction_time_ms: Some(duration_ms + (duration_ms / 2)), // 150% buffer
            min_content_length: Some((content.len() as f64 * 0.8) as usize), // 80% minimum
            expected_metadata: Self::extract_expected_metadata(metadata),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(baseline)
    }

    /// Save a baseline to disk
    pub fn save_baseline(&self, baseline: &Baseline) -> Result<PathBuf> {
        let filename = format!("{}_{}.json", baseline.test_id, baseline.method);
        let path = self.baselines_dir.join(filename);

        let json = serde_json::to_string_pretty(baseline)
            .context("Failed to serialize baseline")?;
        fs::write(&path, json)
            .context("Failed to write baseline file")?;

        Ok(path)
    }

    /// Load a baseline from disk
    pub fn load_baseline(&self, test_id: &str, method: &str) -> Result<Baseline> {
        let filename = format!("{}_{}.json", test_id, method);
        let path = self.baselines_dir.join(filename);

        let content = fs::read_to_string(&path)
            .context(format!("Failed to read baseline file: {}", path.display()))?;
        let baseline: Baseline = serde_json::from_str(&content)
            .context("Failed to parse baseline JSON")?;

        Ok(baseline)
    }

    /// Check if a baseline exists
    pub fn baseline_exists(&self, test_id: &str, method: &str) -> bool {
        let filename = format!("{}_{}.json", test_id, method);
        self.baselines_dir.join(filename).exists()
    }

    /// Compare current result against baseline
    pub fn compare_against_baseline(
        &self,
        baseline: &Baseline,
        content: &str,
        metadata: &HashMap<String, serde_json::Value>,
        duration_ms: u64,
    ) -> ComparisonResult {
        let mut differences = Vec::new();

        // Compare content length
        if let Some(min_length) = baseline.min_content_length {
            let current_length = content.len();
            let diff_pct = Self::calculate_percentage_diff(min_length, current_length);

            if current_length < min_length {
                differences.push(Difference {
                    field: "content_length".to_string(),
                    baseline_value: serde_json::json!(min_length),
                    current_value: serde_json::json!(current_length),
                    diff_percentage: Some(diff_pct),
                    severity: Self::classify_severity(diff_pct),
                });
            }
        }

        // Compare quality score
        if let Some(min_quality) = baseline.min_quality_score {
            if let Some(current_quality) = metadata.get("quality_score").and_then(|v| v.as_f64()) {
                let diff_pct = Self::calculate_percentage_diff(
                    (min_quality * 100.0) as usize,
                    (current_quality * 100.0) as usize,
                );

                if current_quality < min_quality {
                    differences.push(Difference {
                        field: "quality_score".to_string(),
                        baseline_value: serde_json::json!(min_quality),
                        current_value: serde_json::json!(current_quality),
                        diff_percentage: Some(diff_pct),
                        severity: Self::classify_severity(diff_pct),
                    });
                }
            }
        }

        // Compare extraction time
        if let Some(max_time) = baseline.max_extraction_time_ms {
            if duration_ms > max_time {
                let diff_pct = Self::calculate_percentage_diff(max_time as usize, duration_ms as usize);

                differences.push(Difference {
                    field: "extraction_time_ms".to_string(),
                    baseline_value: serde_json::json!(max_time),
                    current_value: serde_json::json!(duration_ms),
                    diff_percentage: Some(diff_pct),
                    severity: Self::classify_severity(diff_pct),
                });
            }
        }

        // Compare title
        if let Some(expected_title) = &baseline.expected_title {
            let current_title = Self::extract_title(content, metadata);
            if current_title.as_ref() != Some(expected_title) {
                differences.push(Difference {
                    field: "title".to_string(),
                    baseline_value: serde_json::json!(expected_title),
                    current_value: serde_json::json!(current_title),
                    diff_percentage: None,
                    severity: DifferenceSeverity::Major,
                });
            }
        }

        // Compare keywords
        let current_keywords = Self::extract_keywords(content);
        let keyword_match_rate = Self::calculate_keyword_match_rate(
            &baseline.expected_keywords,
            &current_keywords,
        );

        if keyword_match_rate < 0.8 {
            // Less than 80% match
            let diff_pct = (1.0 - keyword_match_rate) * 100.0;
            differences.push(Difference {
                field: "keywords".to_string(),
                baseline_value: serde_json::json!(baseline.expected_keywords),
                current_value: serde_json::json!(current_keywords),
                diff_percentage: Some(diff_pct),
                severity: Self::classify_severity(diff_pct),
            });
        }

        // Determine if comparison passed
        let critical_diffs = differences.iter()
            .filter(|d| d.severity == DifferenceSeverity::Critical)
            .count();
        let major_diffs = differences.iter()
            .filter(|d| d.severity == DifferenceSeverity::Major)
            .count();

        let passed = critical_diffs == 0 && major_diffs < 2;

        let summary = if passed {
            format!("✅ Passed: {} differences (all minor/negligible)", differences.len())
        } else {
            format!(
                "❌ Failed: {} critical, {} major, {} total differences",
                critical_diffs, major_diffs, differences.len()
            )
        };

        ComparisonResult {
            test_id: baseline.test_id.clone(),
            method: baseline.method.clone(),
            passed,
            differences,
            summary,
        }
    }

    /// List all baselines
    pub fn list_baselines(&self) -> Result<Vec<(String, String)>> {
        let mut baselines = Vec::new();

        for entry in fs::read_dir(&self.baselines_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    // Parse filename format: "test_id_method.json"
                    if let Some(last_underscore) = filename.rfind('_') {
                        let test_id = filename[..last_underscore].to_string();
                        let method = filename[last_underscore + 1..].to_string();
                        baselines.push((test_id, method));
                    }
                }
            }
        }

        Ok(baselines)
    }

    // Helper methods

    fn extract_title(
        content: &str,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Option<String> {
        // Try metadata first
        if let Some(title) = metadata.get("title").and_then(|v| v.as_str()) {
            return Some(title.to_string());
        }

        // Try HTML title tag
        if let Some(start) = content.find("<title>") {
            if let Some(end) = content[start..].find("</title>") {
                let title = &content[start + 7..start + end];
                return Some(title.trim().to_string());
            }
        }

        // Try Markdown H1
        for line in content.lines() {
            if line.starts_with("# ") {
                return Some(line[2..].trim().to_string());
            }
        }

        None
    }

    fn extract_keywords(content: &str) -> Vec<String> {
        // Simple keyword extraction - first 10 words longer than 5 chars
        content
            .split_whitespace()
            .filter(|w| w.len() > 5)
            .map(|w| w.to_lowercase())
            .take(10)
            .collect()
    }

    fn extract_expected_metadata(
        metadata: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        // Extract key metadata fields to track
        let mut expected = HashMap::new();

        for key in &["word_count", "has_images", "has_links", "language"] {
            if let Some(value) = metadata.get(*key) {
                expected.insert(key.to_string(), value.clone());
            }
        }

        expected
    }

    fn calculate_percentage_diff(baseline: usize, current: usize) -> f64 {
        if baseline == 0 {
            return 0.0;
        }
        ((baseline as f64 - current as f64).abs() / baseline as f64) * 100.0
    }

    fn calculate_keyword_match_rate(expected: &[String], current: &[String]) -> f64 {
        if expected.is_empty() {
            return 1.0;
        }

        let matches = expected
            .iter()
            .filter(|k| current.contains(k))
            .count();

        matches as f64 / expected.len() as f64
    }

    fn classify_severity(diff_percentage: f64) -> DifferenceSeverity {
        if diff_percentage >= 50.0 {
            DifferenceSeverity::Critical
        } else if diff_percentage >= 20.0 {
            DifferenceSeverity::Major
        } else if diff_percentage >= 5.0 {
            DifferenceSeverity::Minor
        } else {
            DifferenceSeverity::Negligible
        }
    }

    /// Delete a baseline
    pub fn delete_baseline(&self, test_id: &str, method: &str) -> Result<()> {
        let filename = format!("{}_{}.json", test_id, method);
        let path = self.baselines_dir.join(filename);

        if path.exists() {
            fs::remove_file(path).context("Failed to delete baseline file")?;
        }

        Ok(())
    }

    /// Update an existing baseline
    pub fn update_baseline(&self, mut baseline: Baseline) -> Result<PathBuf> {
        baseline.updated_at = chrono::Utc::now().to_rfc3339();
        self.save_baseline(&baseline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_baseline_creation_and_storage() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BaselineManager::new(temp_dir.path().to_path_buf()).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("quality_score".to_string(), serde_json::json!(0.85));

        let baseline = manager
            .generate_baseline(
                "test-1",
                "trek",
                "https://example.com",
                "Test content with enough length",
                &metadata,
                100,
            )
            .unwrap();

        let path = manager.save_baseline(&baseline).unwrap();
        assert!(path.exists());

        let loaded = manager.load_baseline("test-1", "trek").unwrap();
        assert_eq!(loaded.test_id, "test-1");
        assert_eq!(loaded.method, "trek");
    }

    #[test]
    fn test_baseline_comparison() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BaselineManager::new(temp_dir.path().to_path_buf()).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("quality_score".to_string(), serde_json::json!(0.9));

        let baseline = manager
            .generate_baseline(
                "test-1",
                "trek",
                "https://example.com",
                "Original test content with sufficient length",
                &metadata,
                100,
            )
            .unwrap();

        // Compare with similar content (should pass)
        let mut current_metadata = HashMap::new();
        current_metadata.insert("quality_score".to_string(), serde_json::json!(0.88));

        let result = manager.compare_against_baseline(
            &baseline,
            "Current test content with sufficient length",
            &current_metadata,
            110,
        );

        assert!(result.passed, "Should pass with minor differences");
    }

    #[test]
    fn test_severity_classification() {
        assert_eq!(
            BaselineManager::classify_severity(60.0),
            DifferenceSeverity::Critical
        );
        assert_eq!(
            BaselineManager::classify_severity(30.0),
            DifferenceSeverity::Major
        );
        assert_eq!(
            BaselineManager::classify_severity(10.0),
            DifferenceSeverity::Minor
        );
        assert_eq!(
            BaselineManager::classify_severity(2.0),
            DifferenceSeverity::Negligible
        );
    }
}
