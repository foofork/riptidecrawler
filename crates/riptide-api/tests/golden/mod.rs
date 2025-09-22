/// Golden tests module
///
/// This module contains golden tests that verify content extraction
/// against known-good baseline outputs.

pub mod fixtures;
pub mod test_extraction;

use std::fs;
use std::path::Path;
use serde_json::Value;
use riptide_core::types::ExtractedDoc;

/// Golden test runner for content extraction verification
///
/// This module provides utilities for running golden tests that verify
/// content extraction results against known-good baseline outputs.
///
/// Golden tests help ensure that:
/// - Content extraction produces consistent results
/// - Changes to extraction logic don't break existing functionality
/// - New content types are handled correctly
/// - Gate decisions are made appropriately for different content patterns
pub struct GoldenTestRunner {
    fixtures_dir: String,
    baseline_dir: String,
}

impl GoldenTestRunner {
    pub fn new(fixtures_dir: &str, baseline_dir: &str) -> Self {
        Self {
            fixtures_dir: fixtures_dir.to_string(),
            baseline_dir: baseline_dir.to_string(),
        }
    }

    /// Load HTML fixture content from file
    pub fn load_fixture(&self, name: &str) -> std::io::Result<String> {
        let path = format!("{}/{}.html", self.fixtures_dir, name);
        fs::read_to_string(path)
    }

    /// Load expected extraction result from baseline
    pub fn load_baseline(&self, name: &str) -> std::io::Result<ExtractedDoc> {
        let path = format!("{}/{}.json", self.baseline_dir, name);
        let content = fs::read_to_string(path)?;
        let doc: ExtractedDoc = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(doc)
    }

    /// Save extraction result as new baseline
    pub fn save_baseline(&self, name: &str, doc: &ExtractedDoc) -> std::io::Result<()> {
        let path = format!("{}/{}.json", self.baseline_dir, name);

        // Ensure directory exists
        if let Some(parent) = Path::new(&path).parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(doc)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }

    /// Compare extracted document with baseline
    pub fn compare_extraction(&self, name: &str, actual: &ExtractedDoc) -> Result<(), String> {
        let expected = self.load_baseline(name)
            .map_err(|e| format!("Failed to load baseline for {}: {}", name, e))?;

        // Compare key fields
        if actual.url != expected.url {
            return Err(format!("URL mismatch: got '{}', expected '{}'", actual.url, expected.url));
        }

        if actual.title != expected.title {
            return Err(format!("Title mismatch: got '{:?}', expected '{:?}'", actual.title, expected.title));
        }

        // For text content, allow some flexibility in whitespace but check core content
        let actual_text = normalize_text(&actual.text);
        let expected_text = normalize_text(&expected.text);

        if actual_text.len() < expected_text.len() * 80 / 100 {
            return Err(format!(
                "Text content too short: got {} chars, expected at least {} chars (80% of {})",
                actual_text.len(),
                expected_text.len() * 80 / 100,
                expected_text.len()
            ));
        }

        // Check that key content phrases are present
        if !actual_text.is_empty() && !expected_text.is_empty() {
            let expected_words: Vec<&str> = expected_text.split_whitespace().collect();
            let actual_words: Vec<&str> = actual_text.split_whitespace().collect();

            // Check that at least 60% of significant words are present
            let significant_words: Vec<&str> = expected_words
                .iter()
                .filter(|word| word.len() > 3 && !is_common_word(word))
                .cloned()
                .collect();

            let found_words = significant_words
                .iter()
                .filter(|word| actual_words.contains(word))
                .count();

            let coverage = if significant_words.is_empty() {
                1.0
            } else {
                found_words as f64 / significant_words.len() as f64
            };

            if coverage < 0.6 {
                return Err(format!(
                    "Content coverage too low: {:.1}% (found {}/{} significant words)",
                    coverage * 100.0,
                    found_words,
                    significant_words.len()
                ));
            }
        }

        Ok(())
    }

    /// Run all golden tests in the fixtures directory
    pub fn run_all_tests<F>(&self, extractor: F) -> Vec<(String, Result<(), String>)>
    where
        F: Fn(&str, &str) -> Result<ExtractedDoc, String>,
    {
        let mut results = Vec::new();

        // Find all HTML fixtures
        if let Ok(entries) = fs::read_dir(&self.fixtures_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".html") {
                        let test_name = file_name.strip_suffix(".html").unwrap();

                        let result = self.run_single_test(test_name, &extractor);
                        results.push((test_name.to_string(), result));
                    }
                }
            }
        }

        results
    }

    /// Run a single golden test
    pub fn run_single_test<F>(&self, name: &str, extractor: F) -> Result<(), String>
    where
        F: Fn(&str, &str) -> Result<ExtractedDoc, String>,
    {
        // Load fixture HTML
        let html = self.load_fixture(name)
            .map_err(|e| format!("Failed to load fixture {}: {}", name, e))?;

        // Extract content
        let url = format!("https://example.com/{}", name);
        let extracted = extractor(&html, &url)?;

        // Compare with baseline
        self.compare_extraction(name, &extracted)
    }
}

/// Normalize text for comparison by removing extra whitespace and normalizing line endings
fn normalize_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase()
}

/// Check if a word is a common word that shouldn't be used for content matching
fn is_common_word(word: &str) -> bool {
    const COMMON_WORDS: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was", "one",
        "our", "out", "day", "get", "has", "him", "his", "how", "man", "new", "now", "old", "see",
        "two", "way", "who", "boy", "did", "its", "let", "put", "say", "she", "too", "use", "with",
        "have", "from", "they", "know", "want", "been", "good", "much", "some", "time", "very",
        "when", "come", "here", "just", "like", "long", "make", "many", "over", "such", "take",
        "than", "them", "well", "were", "will", "would", "your", "about", "after", "again", "could",
        "first", "other", "right", "think", "where", "being", "every", "great", "might", "shall",
        "should", "still", "those", "under", "while", "before", "little", "most", "never", "only",
        "own", "same", "tell", "through", "very", "work", "years", "place", "back", "call", "came",
        "each", "even", "hand", "high", "keep", "last", "left", "life", "live", "look", "made",
        "move", "part", "seem", "show", "side", "also", "another", "any", "around", "because",
        "both", "during", "each", "few", "found", "give", "help", "however", "into", "may", "need",
        "number", "off", "part", "people", "public", "really", "since", "small", "sound", "state",
        "turn", "want", "water", "went", "what", "without", "write"
    ];

    COMMON_WORDS.contains(&word.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let input = "  This   is  \n  some   text  \t with   whitespace  ";
        let expected = "this is some text with whitespace";
        assert_eq!(normalize_text(input), expected);
    }

    #[test]
    fn test_is_common_word() {
        assert!(is_common_word("the"));
        assert!(is_common_word("and"));
        assert!(is_common_word("THE")); // case insensitive
        assert!(!is_common_word("extraction"));
        assert!(!is_common_word("algorithm"));
    }

    #[test]
    fn test_golden_test_runner_creation() {
        let runner = GoldenTestRunner::new("fixtures", "baselines");
        assert_eq!(runner.fixtures_dir, "fixtures");
        assert_eq!(runner.baseline_dir, "baselines");
    }
}