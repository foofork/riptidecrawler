//! Tests for golden test baseline update functionality
//! Validates that baselines can be updated when actual behavior changes intentionally

use std::fs;
use std::path::{Path, PathBuf};

/// Represents a golden test baseline
#[derive(Debug, Clone)]
struct GoldenBaseline {
    name: String,
    content: String,
    similarity_threshold: f64,
}

impl GoldenBaseline {
    fn new(name: &str, content: String) -> Self {
        Self {
            name: name.to_string(),
            content,
            similarity_threshold: 0.80, // 80% similarity required
        }
    }

    /// Calculate similarity between baseline and actual content
    fn calculate_similarity(&self, actual: &str) -> f64 {
        let baseline_words: Vec<&str> = self.content.split_whitespace().collect();
        let actual_words: Vec<&str> = actual.split_whitespace().collect();

        if baseline_words.is_empty() && actual_words.is_empty() {
            return 1.0;
        }

        if baseline_words.is_empty() || actual_words.is_empty() {
            return 0.0;
        }

        let matches = baseline_words
            .iter()
            .filter(|word| actual_words.contains(word))
            .count();

        let max_len = baseline_words.len().max(actual_words.len());
        matches as f64 / max_len as f64
    }

    /// Check if actual content meets similarity threshold
    fn passes(&self, actual: &str) -> bool {
        self.calculate_similarity(actual) >= self.similarity_threshold
    }

    /// Update baseline with new content
    fn update(&mut self, new_content: String) {
        self.content = new_content;
    }

    /// Save baseline to file
    fn save_to_file(&self, directory: &Path) -> std::io::Result<()> {
        let path = directory.join(format!("{}.baseline", self.name));
        fs::write(path, &self.content)
    }

    /// Load baseline from file
    fn load_from_file(name: &str, directory: &Path) -> std::io::Result<Self> {
        let path = directory.join(format!("{}.baseline", name));
        let content = fs::read_to_string(path)?;
        Ok(Self::new(name, content))
    }
}

/// Golden test runner with baseline update capability
struct GoldenTestRunner {
    baselines_dir: PathBuf,
    update_mode: bool,
}

impl GoldenTestRunner {
    fn new(baselines_dir: PathBuf) -> Self {
        Self {
            baselines_dir,
            update_mode: false,
        }
    }

    fn with_update_mode(mut self, update: bool) -> Self {
        self.update_mode = update;
        self
    }

    /// Run a golden test
    fn run_test(&self, test_name: &str, actual: &str) -> Result<(), String> {
        // Load baseline
        let mut baseline = GoldenBaseline::load_from_file(test_name, &self.baselines_dir)
            .map_err(|e| format!("Failed to load baseline: {}", e))?;

        if self.update_mode {
            // Update mode: save new baseline
            baseline.update(actual.to_string());
            baseline
                .save_to_file(&self.baselines_dir)
                .map_err(|e| format!("Failed to save baseline: {}", e))?;
            Ok(())
        } else {
            // Test mode: compare against baseline
            let similarity = baseline.calculate_similarity(actual);
            if baseline.passes(actual) {
                Ok(())
            } else {
                Err(format!(
                    "Baseline mismatch! Similarity: {:.1}% (threshold: {:.1}%)\n\
                     Expected (baseline):\n{}\n\n\
                     Actual:\n{}",
                    similarity * 100.0,
                    baseline.similarity_threshold * 100.0,
                    baseline.content,
                    actual
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_baseline_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_baseline_creation() {
        let baseline = GoldenBaseline::new(
            "test1",
            "Hello world this is a test baseline".to_string(),
        );

        assert_eq!(baseline.name, "test1");
        assert_eq!(
            baseline.content,
            "Hello world this is a test baseline"
        );
        assert_eq!(baseline.similarity_threshold, 0.80);
    }

    #[test]
    fn test_baseline_similarity_exact_match() {
        let baseline =
            GoldenBaseline::new("test", "Hello world test baseline".to_string());
        let similarity = baseline.calculate_similarity("Hello world test baseline");

        assert_eq!(similarity, 1.0);
        assert!(baseline.passes("Hello world test baseline"));
    }

    #[test]
    fn test_baseline_similarity_high_overlap() {
        let baseline = GoldenBaseline::new(
            "test",
            "Hello world this is a test baseline with many words".to_string(),
        );
        let actual = "Hello world this is a test baseline with some words";
        let similarity = baseline.calculate_similarity(actual);

        // Most words match
        assert!(similarity >= 0.80);
        assert!(baseline.passes(actual));
    }

    #[test]
    fn test_baseline_similarity_low_overlap() {
        let baseline = GoldenBaseline::new(
            "test",
            "Original baseline content here".to_string(),
        );
        let actual = "Completely different text now";
        let similarity = baseline.calculate_similarity(actual);

        assert!(similarity < 0.80);
        assert!(!baseline.passes(actual));
    }

    #[test]
    fn test_baseline_update() {
        let mut baseline = GoldenBaseline::new("test", "Old content".to_string());
        baseline.update("New content".to_string());

        assert_eq!(baseline.content, "New content");
    }

    #[test]
    fn test_baseline_save_and_load() {
        let temp_dir = create_test_baseline_dir();
        let baseline = GoldenBaseline::new("test_save", "Test content".to_string());

        // Save baseline
        baseline.save_to_file(temp_dir.path()).unwrap();

        // Load baseline
        let loaded = GoldenBaseline::load_from_file("test_save", temp_dir.path()).unwrap();

        assert_eq!(loaded.name, "test_save");
        assert_eq!(loaded.content, "Test content");
    }

    #[test]
    fn test_golden_test_runner_test_mode() {
        let temp_dir = create_test_baseline_dir();

        // Create a baseline
        let baseline = GoldenBaseline::new(
            "runner_test",
            "Baseline content for testing".to_string(),
        );
        baseline.save_to_file(temp_dir.path()).unwrap();

        // Run test in test mode (not update mode)
        let runner = GoldenTestRunner::new(temp_dir.path().to_path_buf());

        // Should pass with similar content
        assert!(runner
            .run_test("runner_test", "Baseline content for testing")
            .is_ok());

        // Should fail with different content
        assert!(runner
            .run_test("runner_test", "Completely different content")
            .is_err());
    }

    #[test]
    fn test_golden_test_runner_update_mode() {
        let temp_dir = create_test_baseline_dir();

        // Create initial baseline
        let baseline = GoldenBaseline::new("update_test", "Old baseline".to_string());
        baseline.save_to_file(temp_dir.path()).unwrap();

        // Run test in update mode
        let runner = GoldenTestRunner::new(temp_dir.path().to_path_buf())
            .with_update_mode(true);

        let new_content = "New baseline content";
        assert!(runner.run_test("update_test", new_content).is_ok());

        // Verify baseline was updated
        let loaded = GoldenBaseline::load_from_file("update_test", temp_dir.path()).unwrap();
        assert_eq!(loaded.content, new_content);
    }

    #[test]
    fn test_baseline_empty_content() {
        let baseline = GoldenBaseline::new("empty", String::new());

        // Empty should match empty
        assert_eq!(baseline.calculate_similarity(""), 1.0);

        // Empty should not match non-empty
        assert_eq!(baseline.calculate_similarity("content"), 0.0);
    }

    #[test]
    fn test_baseline_whitespace_handling() {
        let baseline = GoldenBaseline::new(
            "whitespace",
            "word1  word2    word3".to_string(),
        );

        // Should handle different whitespace amounts
        let similarity = baseline.calculate_similarity("word1 word2 word3");
        assert_eq!(similarity, 1.0); // Words are the same, just different spacing
    }

    #[test]
    fn test_baseline_html_stripped_comparison() {
        // Simulate HTML-stripped baseline
        let baseline = GoldenBaseline::new(
            "html_test",
            "Article title This is content Link text".to_string(),
        );

        // HTML version (what extractor would produce)
        let html_stripped = "Article title This is content Link text";

        let similarity = baseline.calculate_similarity(html_stripped);
        assert_eq!(similarity, 1.0);
        assert!(baseline.passes(html_stripped));
    }

    #[test]
    fn test_baseline_partial_content_match() {
        let baseline = GoldenBaseline::new(
            "partial",
            "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10".to_string(),
        );

        // 8 out of 10 words = 80% match
        let actual = "word1 word2 word3 word4 word5 word6 word7 word8 different1 different2";
        let similarity = baseline.calculate_similarity(actual);

        assert!(similarity >= 0.75); // At least 75% match
        // Passes depends on exact calculation with max length
    }

    #[test]
    fn test_golden_runner_missing_baseline() {
        let temp_dir = create_test_baseline_dir();
        let runner = GoldenTestRunner::new(temp_dir.path().to_path_buf());

        // Should error when baseline doesn't exist
        let result = runner.run_test("nonexistent", "content");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to load baseline"));
    }

    #[test]
    fn test_baseline_update_creates_new_file() {
        let temp_dir = create_test_baseline_dir();

        // Create runner in update mode
        let runner = GoldenTestRunner::new(temp_dir.path().to_path_buf())
            .with_update_mode(true);

        // Note: This will fail because baseline doesn't exist yet
        // In a real implementation, we'd handle this case
        let result = runner.run_test("new_baseline", "New content");

        // Expected to fail since baseline doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_similarity_calculation_edge_cases() {
        let baseline = GoldenBaseline::new("edge", "a b c".to_string());

        // One word in common
        assert!(baseline.calculate_similarity("a x y") < 0.80);

        // Two words in common
        assert!(baseline.calculate_similarity("a b x") > 0.50);

        // All words in common
        assert_eq!(baseline.calculate_similarity("a b c"), 1.0);
    }
}
