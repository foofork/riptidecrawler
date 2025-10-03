//! Comprehensive tests for CSS-004: Merge policy css_wins with conflict audit trail
//!
//! Testing merge policy functionality including:
//! - CSS wins policy (default)
//! - Other wins policy
//! - Merge policy (combine results)
//! - First valid policy
//! - Conflict audit trail generation
//! - Field-specific merge policies

use anyhow::Result;
use riptide_html::css_extraction::*;
use std::collections::HashMap;

/// Test CSS-004: Basic CSS wins merge policy
#[tokio::test]
async fn test_css_wins_merge_policy() -> Result<()> {
    let _html = r#"
        <html>
            <head>
                <title>CSS Title</title>
                <meta name="description" content="CSS Description">
            </head>
            <body>
                <h1>HTML Title</h1>
                <p class="summary">HTML Description</p>
                <article>Main content here</article>
            </body>
        </html>
    "#;

    let mut css_selectors = HashMap::new();
    css_selectors.insert(
        "title".to_string(),
        CssSelectorConfig {
            selector: "title".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    css_selectors.insert(
        "description".to_string(),
        CssSelectorConfig {
            selector: "[name='description']".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    // Simulate CSS extraction results
    let mut css_results = HashMap::new();
    css_results.insert("title".to_string(), vec!["CSS Title".to_string()]);
    css_results.insert(
        "description".to_string(),
        vec!["CSS Description".to_string()],
    );

    // Simulate other extraction method results
    let mut other_results = HashMap::new();
    other_results.insert("title".to_string(), vec!["HTML Title".to_string()]);
    other_results.insert(
        "description".to_string(),
        vec!["HTML Description".to_string()],
    );

    // Test merge with CSS wins policy
    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // CSS should win
    assert_eq!(merged.get("title").unwrap()[0], "CSS Title");
    assert_eq!(merged.get("description").unwrap()[0], "CSS Description");

    // Should generate conflict audit
    assert!(!conflicts.is_empty());
    assert!(conflicts.iter().any(|c| c.field == "title"));
    assert!(conflicts
        .iter()
        .any(|c| matches!(c.policy_used, MergePolicy::CssWins)));

    Ok(())
}

/// Test CSS-004: Other wins merge policy
#[tokio::test]
async fn test_other_wins_merge_policy() -> Result<()> {
    let mut css_selectors = HashMap::new();
    css_selectors.insert(
        "content".to_string(),
        CssSelectorConfig {
            selector: "article".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::OtherWins), // Other method wins
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    let mut css_results = HashMap::new();
    css_results.insert(
        "content".to_string(),
        vec!["CSS extracted content".to_string()],
    );

    let mut other_results = HashMap::new();
    other_results.insert(
        "content".to_string(),
        vec!["LLM extracted content".to_string()],
    );

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Other method should win
    assert_eq!(merged.get("content").unwrap()[0], "LLM extracted content");

    // Should generate conflict audit
    let content_conflict = conflicts.iter().find(|c| c.field == "content").unwrap();
    assert!(matches!(
        content_conflict.policy_used,
        MergePolicy::OtherWins
    ));
    assert_eq!(
        content_conflict.css_value,
        Some("CSS extracted content".to_string())
    );
    assert_eq!(
        content_conflict.other_value,
        Some("LLM extracted content".to_string())
    );

    Ok(())
}

/// Test CSS-004: Merge policy (combine results)
#[tokio::test]
async fn test_merge_combine_policy() -> Result<()> {
    let mut css_selectors = HashMap::new();
    css_selectors.insert(
        "tags".to_string(),
        CssSelectorConfig {
            selector: ".tag".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge), // Combine both results
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    let mut css_results = HashMap::new();
    css_results.insert(
        "tags".to_string(),
        vec!["css-tag".to_string(), "html-tag".to_string()],
    );

    let mut other_results = HashMap::new();
    other_results.insert(
        "tags".to_string(),
        vec!["llm-tag".to_string(), "ai-tag".to_string()],
    );

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Should combine all tags
    let merged_tags = merged.get("tags").unwrap();
    assert_eq!(merged_tags.len(), 4);
    assert!(merged_tags.contains(&"css-tag".to_string()));
    assert!(merged_tags.contains(&"html-tag".to_string()));
    assert!(merged_tags.contains(&"llm-tag".to_string()));
    assert!(merged_tags.contains(&"ai-tag".to_string()));

    // Should generate conflict audit for merge
    let tags_conflict = conflicts.iter().find(|c| c.field == "tags").unwrap();
    assert!(matches!(tags_conflict.policy_used, MergePolicy::Merge));
    assert!(tags_conflict.resolution.contains("Merged"));

    Ok(())
}

/// Test CSS-004: First valid policy
#[tokio::test]
async fn test_first_valid_policy() -> Result<()> {
    let mut css_selectors = HashMap::new();
    css_selectors.insert(
        "author".to_string(),
        CssSelectorConfig {
            selector: ".author".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::FirstValid), // First non-empty wins
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    let mut css_results = HashMap::new();
    css_results.insert("author".to_string(), vec!["CSS Author".to_string()]);

    let mut other_results = HashMap::new();
    other_results.insert("author".to_string(), vec!["Other Author".to_string()]);

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // CSS should win as it's first valid
    assert_eq!(merged.get("author").unwrap()[0], "CSS Author");

    let author_conflict = conflicts.iter().find(|c| c.field == "author").unwrap();
    assert!(matches!(
        author_conflict.policy_used,
        MergePolicy::FirstValid
    ));
    assert!(author_conflict.resolution.contains("First valid"));

    Ok(())
}

/// Test CSS-004: Global vs field-specific merge policies
#[tokio::test]
async fn test_global_vs_field_specific_policies() -> Result<()> {
    let mut css_selectors = HashMap::new();

    // Field with specific policy
    css_selectors.insert(
        "title".to_string(),
        CssSelectorConfig {
            selector: "title".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: true,
            merge_policy: Some(MergePolicy::OtherWins), // Field-specific policy
        },
    );

    // Field without specific policy (will use global)
    css_selectors.insert(
        "content".to_string(),
        CssSelectorConfig {
            selector: "article".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: None, // No field-specific policy
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors).with_merge_policy(MergePolicy::CssWins); // Global policy

    let mut css_results = HashMap::new();
    css_results.insert("title".to_string(), vec!["CSS Title".to_string()]);
    css_results.insert("content".to_string(), vec!["CSS Content".to_string()]);

    let mut other_results = HashMap::new();
    other_results.insert("title".to_string(), vec!["Other Title".to_string()]);
    other_results.insert("content".to_string(), vec!["Other Content".to_string()]);

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Title should use field-specific policy (OtherWins)
    assert_eq!(merged.get("title").unwrap()[0], "Other Title");

    // Content should use global policy (CssWins)
    assert_eq!(merged.get("content").unwrap()[0], "CSS Content");

    // Verify conflict audit reflects correct policies
    let title_conflict = conflicts.iter().find(|c| c.field == "title").unwrap();
    assert!(matches!(title_conflict.policy_used, MergePolicy::OtherWins));

    let content_conflict = conflicts.iter().find(|c| c.field == "content").unwrap();
    assert!(matches!(content_conflict.policy_used, MergePolicy::CssWins));

    Ok(())
}

/// Test CSS-004: No conflicts when values are identical
#[tokio::test]
async fn test_no_conflicts_identical_values() -> Result<()> {
    let mut css_selectors = HashMap::new();
    css_selectors.insert(
        "title".to_string(),
        CssSelectorConfig {
            selector: "title".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: true,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    let mut css_results = HashMap::new();
    css_results.insert("title".to_string(), vec!["Same Title".to_string()]);

    let mut other_results = HashMap::new();
    other_results.insert("title".to_string(), vec!["Same Title".to_string()]);

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Should merge without conflict
    assert_eq!(merged.get("title").unwrap()[0], "Same Title");

    // No conflicts should be generated for identical values
    assert!(conflicts
        .iter()
        .all(|c| c.field != "title" || c.resolution.contains("No conflict")));

    Ok(())
}

/// Test CSS-004: Handling missing values from one method
#[tokio::test]
async fn test_missing_values_handling() -> Result<()> {
    let mut css_selectors = HashMap::new();
    css_selectors.insert(
        "css_only".to_string(),
        CssSelectorConfig {
            selector: ".css-only".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    css_selectors.insert(
        "other_only".to_string(),
        CssSelectorConfig {
            selector: ".other-only".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    let mut css_results = HashMap::new();
    css_results.insert("css_only".to_string(), vec!["CSS value".to_string()]);
    // css_only field missing from other_results

    let mut other_results = HashMap::new();
    other_results.insert("other_only".to_string(), vec!["Other value".to_string()]);
    // other_only field missing from css_results

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Should include values from both methods when no conflicts
    assert_eq!(merged.get("css_only").unwrap()[0], "CSS value");
    assert_eq!(merged.get("other_only").unwrap()[0], "Other value");

    // No conflicts for missing values
    assert!(conflicts.iter().all(|c| !c.field.contains("only")));

    Ok(())
}

/// Test CSS-004: Complex conflict audit trail
#[tokio::test]
async fn test_detailed_conflict_audit() -> Result<()> {
    let mut css_selectors = HashMap::new();

    css_selectors.insert(
        "field1".to_string(),
        CssSelectorConfig {
            selector: ".field1".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    css_selectors.insert(
        "field2".to_string(),
        CssSelectorConfig {
            selector: ".field2".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::OtherWins),
        },
    );

    css_selectors.insert(
        "field3".to_string(),
        CssSelectorConfig {
            selector: ".field3".to_string(),
            transformers: vec!["trim".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::Merge),
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    let mut css_results = HashMap::new();
    css_results.insert("field1".to_string(), vec!["CSS1".to_string()]);
    css_results.insert("field2".to_string(), vec!["CSS2".to_string()]);
    css_results.insert("field3".to_string(), vec!["CSS3".to_string()]);

    let mut other_results = HashMap::new();
    other_results.insert("field1".to_string(), vec!["Other1".to_string()]);
    other_results.insert("field2".to_string(), vec!["Other2".to_string()]);
    other_results.insert("field3".to_string(), vec!["Other3".to_string()]);

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Verify merge results
    assert_eq!(merged.get("field1").unwrap()[0], "CSS1"); // CSS wins
    assert_eq!(merged.get("field2").unwrap()[0], "Other2"); // Other wins
    assert_eq!(merged.get("field3").unwrap().len(), 2); // Merged

    // Verify audit trail completeness
    assert_eq!(conflicts.len(), 3);

    for conflict in &conflicts {
        match conflict.field.as_str() {
            "field1" => {
                assert!(matches!(conflict.policy_used, MergePolicy::CssWins));
                assert_eq!(conflict.css_value, Some("CSS1".to_string()));
                assert_eq!(conflict.other_value, Some("Other1".to_string()));
                assert!(conflict.resolution.contains("CSS wins"));
            }
            "field2" => {
                assert!(matches!(conflict.policy_used, MergePolicy::OtherWins));
                assert_eq!(conflict.css_value, Some("CSS2".to_string()));
                assert_eq!(conflict.other_value, Some("Other2".to_string()));
                assert!(conflict.resolution.contains("Other wins"));
            }
            "field3" => {
                assert!(matches!(conflict.policy_used, MergePolicy::Merge));
                assert_eq!(conflict.css_value, Some("CSS3".to_string()));
                assert_eq!(conflict.other_value, Some("Other3".to_string()));
                assert!(conflict.resolution.contains("Merged"));
            }
            _ => panic!("Unexpected field in conflict audit: {}", conflict.field),
        }
    }

    Ok(())
}

/// Test CSS-004: Conflict audit with transformers
#[tokio::test]
async fn test_conflict_audit_with_transformers() -> Result<()> {
    let mut css_selectors = HashMap::new();
    css_selectors.insert(
        "price".to_string(),
        CssSelectorConfig {
            selector: ".price".to_string(),
            transformers: vec!["trim".to_string(), "currency".to_string()],
            has_text_filter: None,
            fallbacks: vec![],
            required: false,
            merge_policy: Some(MergePolicy::CssWins),
        },
    );

    let extractor = CssJsonExtractor::new(css_selectors);

    // Values that would be different after transformation
    let mut css_results = HashMap::new();
    css_results.insert("price".to_string(), vec!["1299.99".to_string()]); // After transformation

    let mut other_results = HashMap::new();
    other_results.insert("price".to_string(), vec!["$1,299.99".to_string()]); // Raw value

    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);

    // Should detect conflict even with different formats
    assert!(!conflicts.is_empty());

    let price_conflict = conflicts.iter().find(|c| c.field == "price");
    if let Some(conflict) = price_conflict {
        assert_eq!(conflict.field, "price");
        assert!(matches!(conflict.policy_used, MergePolicy::CssWins));
    }

    Ok(())
}

/// Test CSS-004: Performance with many conflicts
#[tokio::test]
async fn test_merge_performance_many_conflicts() -> Result<()> {
    let mut css_selectors = HashMap::new();

    // Create many fields with different policies
    for i in 0..50 {
        let field_name = format!("field_{}", i);
        let policy = match i % 4 {
            0 => MergePolicy::CssWins,
            1 => MergePolicy::OtherWins,
            2 => MergePolicy::Merge,
            _ => MergePolicy::FirstValid,
        };

        css_selectors.insert(
            field_name,
            CssSelectorConfig {
                selector: format!(".field-{}", i),
                transformers: vec!["trim".to_string()],
                has_text_filter: None,
                fallbacks: vec![],
                required: false,
                merge_policy: Some(policy),
            },
        );
    }

    let extractor = CssJsonExtractor::new(css_selectors);

    let mut css_results = HashMap::new();
    let mut other_results = HashMap::new();

    for i in 0..50 {
        let field_name = format!("field_{}", i);
        css_results.insert(field_name.clone(), vec![format!("CSS_{}", i)]);
        other_results.insert(field_name, vec![format!("Other_{}", i)]);
    }

    let start = std::time::Instant::now();
    let (merged, conflicts) = extractor.merge_with_other(&css_results, &other_results);
    let duration = start.elapsed();

    // Should complete quickly even with many conflicts
    assert!(duration < std::time::Duration::from_millis(100));

    // Should handle all fields
    assert_eq!(merged.len(), 50);
    assert_eq!(conflicts.len(), 50);

    Ok(())
}
