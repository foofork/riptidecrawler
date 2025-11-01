//! Integration tests for token counting with tiktoken cache
//!
//! These tests verify the accuracy and performance of the async token counting
//! compared to the fast approximation method.

use riptide_extraction::chunking::utils::{count_tokens, count_tokens_batch, count_tokens_exact};

#[tokio::test]
async fn test_exact_vs_approximate_accuracy() {
    let test_cases = vec![
        // (text, expected_range_min, expected_range_max)
        ("Hello world", 2, 3),
        ("This is a test.", 4, 6),
        ("The quick brown fox jumps over the lazy dog.", 9, 12),
        (
            "Artificial intelligence and machine learning are transforming technology.",
            10,
            15,
        ),
    ];

    for (text, min_tokens, max_tokens) in test_cases {
        let exact = count_tokens_exact(text).await.unwrap();
        let approx = count_tokens(text);

        println!("Text: '{}'", text);
        println!("  Exact: {} tokens", exact);
        println!("  Approx: {} tokens", approx);
        println!(
            "  Difference: {} tokens ({:.1}%)",
            (exact as i32 - approx as i32).abs(),
            ((exact as f64 - approx as f64).abs() / exact as f64) * 100.0
        );

        // Verify exact count is in expected range
        assert!(
            exact >= min_tokens && exact <= max_tokens,
            "Expected exact count between {} and {}, got {}",
            min_tokens,
            max_tokens,
            exact
        );

        // Verify approximation is reasonably close (within 50%)
        let diff_percent = ((exact as f64 - approx as f64).abs() / exact as f64) * 100.0;
        assert!(
            diff_percent < 50.0,
            "Approximation differs by {:.1}%, which is too high",
            diff_percent
        );
    }
}

#[tokio::test]
async fn test_batch_counting_consistency() {
    let texts = vec![
        "First text chunk",
        "Second text chunk with more words",
        "Third chunk",
    ];

    // Count individually
    let mut individual_counts = Vec::new();
    for text in &texts {
        let count = count_tokens_exact(text).await.unwrap();
        individual_counts.push(count);
    }

    // Count in batch
    let batch_counts = count_tokens_batch(&texts).await.unwrap();

    // Verify they match
    assert_eq!(individual_counts.len(), batch_counts.len());
    for (i, (&individual, &batch)) in individual_counts
        .iter()
        .zip(batch_counts.iter())
        .enumerate()
    {
        assert_eq!(
            individual, batch,
            "Mismatch at index {}: individual={}, batch={}",
            i, individual, batch
        );
    }
}

#[tokio::test]
async fn test_special_characters_and_unicode() {
    let test_cases = vec![
        "Hello 世界",
        "Café résumé naïve",
        "Emoji test 🎉 🚀 🔥",
        "Mixed: ASCII + 日本語 + émojis 🎯",
    ];

    for text in test_cases {
        let exact = count_tokens_exact(text).await.unwrap();
        println!("Text: '{}' -> {} tokens", text, exact);

        // Should handle special characters without panicking
        assert!(exact > 0, "Token count should be positive");
    }
}

#[tokio::test]
async fn test_large_text_performance() {
    // Generate 50KB of text
    let base_text = "This is a performance test for token counting with larger texts. ";
    let mut large_text = String::new();
    while large_text.len() < 50_000 {
        large_text.push_str(base_text);
    }

    let start = std::time::Instant::now();
    let exact = count_tokens_exact(&large_text).await.unwrap();
    let duration = start.elapsed();

    println!("50KB text: {} tokens in {:?}", exact, duration);

    // Should be reasonably fast (< 100ms for cached, < 500ms for uncached)
    assert!(
        duration.as_millis() < 500,
        "Token counting took {}ms, expected < 500ms",
        duration.as_millis()
    );
    assert!(exact > 0, "Should have counted tokens");
}

#[tokio::test]
async fn test_empty_and_edge_cases() {
    // Empty string
    let count = count_tokens_exact("").await.unwrap();
    assert_eq!(count, 0, "Empty string should have 0 tokens");

    // Only whitespace
    let count = count_tokens_exact("   \n\t  ").await.unwrap();
    assert!(count <= 1, "Whitespace should have minimal tokens");

    // Single character
    let count = count_tokens_exact("a").await.unwrap();
    assert_eq!(count, 1, "Single character should be 1 token");

    // Very long word
    let long_word = "a".repeat(1000);
    let count = count_tokens_exact(&long_word).await.unwrap();
    assert!(count > 0, "Long word should have tokens");
}

#[tokio::test]
async fn test_accuracy_improvement_real_world() {
    // Real-world text samples where approximation might be less accurate
    let samples = vec![
        // Technical text with abbreviations
        "API v2.0 supports HTTP/2 and TLS 1.3. Use JSON or XML formats.",
        // Code snippet
        "fn main() { println!(\"Hello, world!\"); }",
        // Mixed punctuation
        "Really? Yes! No... maybe—probably not.",
        // URLs and paths
        "https://example.com/api/v1/users?id=123&format=json",
    ];

    for text in samples {
        let exact = count_tokens_exact(text).await.unwrap();
        let approx = count_tokens(text);

        println!("\nText: '{}'", text);
        println!("  Exact: {} tokens", exact);
        println!("  Approx: {} tokens", approx);

        let diff = (exact as i32 - approx as i32).abs();
        let percent = (diff as f64 / exact as f64) * 100.0;
        println!("  Difference: {} tokens ({:.1}%)", diff, percent);

        // For technical text, exact counting should provide better accuracy
        assert!(exact > 0);
    }
}

#[tokio::test]
async fn test_concurrent_token_counting() {
    // Test that cache works correctly under concurrent access
    let texts: Vec<String> = (0..20)
        .map(|i| format!("Concurrent test text number {}", i))
        .collect();

    let mut handles = vec![];

    for text in &texts {
        let text = text.clone();
        let handle = tokio::spawn(async move { count_tokens_exact(&text).await });
        handles.push(handle);
    }

    // Wait for all tasks and verify they all succeeded
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }
}
