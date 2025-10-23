//! Integration tests for schema extraction module

use riptide_extraction::schema::{
    ExtractionSchema, FieldSchema, SchemaComparator, SchemaExtractor, SchemaGenerator,
    SchemaLearnRequest, SchemaMetadata, SchemaRegistry, SchemaValidator, SelectorRule,
    ValidationRules,
};
use std::collections::HashMap;

#[test]
fn test_schema_creation_and_modification() {
    let mut schema = ExtractionSchema::new(
        "test-schema".to_string(),
        "1.0.0".to_string(),
        "article".to_string(),
    );

    assert_eq!(schema.name, "test-schema");
    assert_eq!(schema.version, "1.0.0");
    assert_eq!(schema.goal, "article");
    assert!(schema.fields.is_empty());

    // Add field
    schema.add_field("title".to_string(), FieldSchema::required("string"));
    assert_eq!(schema.fields.len(), 1);

    // Add selector
    schema.add_selector("title".to_string(), SelectorRule::css("h1", 10, 0.9));
    assert_eq!(schema.selectors.len(), 1);
}

#[test]
fn test_field_schema_builders() {
    let required_field = FieldSchema::required("string")
        .with_description("Test description")
        .with_default(serde_json::json!("default value"));

    assert!(required_field.required);
    assert_eq!(required_field.field_type, "string");
    assert_eq!(
        required_field.description,
        Some("Test description".to_string())
    );
    assert!(required_field.default.is_some());

    let optional_field = FieldSchema::optional("number");
    assert!(!optional_field.required);
    assert_eq!(optional_field.field_type, "number");
}

#[test]
fn test_selector_rule_builders() {
    let css_rule = SelectorRule::css("div.content", 5, 0.85).with_fallback("div");
    assert_eq!(css_rule.selector_type, "css");
    assert_eq!(css_rule.selector, "div.content");
    assert_eq!(css_rule.priority, 5);
    assert_eq!(css_rule.confidence, 0.85);
    assert_eq!(css_rule.fallback, Some("div".to_string()));

    let xpath_rule = SelectorRule::xpath("//div[@class='content']", 8, 0.9);
    assert_eq!(xpath_rule.selector_type, "xpath");
}

#[test]
fn test_schema_extractor_creation() {
    let schema = create_test_schema();
    let extractor = SchemaExtractor::new(schema.clone());
    assert_eq!(extractor.schema().name, "test-article");
}

#[test]
fn test_schema_extraction_basic() {
    let schema = create_test_schema();
    let extractor = SchemaExtractor::new(schema);

    let html = r#"
        <html>
            <head><title>Page Title</title></head>
            <body>
                <h1>Article Title</h1>
                <article>
                    <p>Article content goes here.</p>
                </article>
            </body>
        </html>
    "#;

    let result = extractor.extract(html, "http://example.com").unwrap();
    assert!(result.contains_key("title"));
    assert_eq!(result["title"].as_str().unwrap(), "Article Title");
}

#[test]
fn test_schema_extraction_with_fallback() {
    let mut schema = ExtractionSchema::new(
        "fallback-test".to_string(),
        "1.0.0".to_string(),
        "article".to_string(),
    );

    schema.add_field("title".to_string(), FieldSchema::required("string"));
    schema.add_selector(
        "title".to_string(),
        SelectorRule::css("h1.article-title", 10, 0.9).with_fallback("h1"),
    );

    let extractor = SchemaExtractor::new(schema);

    let html = r#"
        <html>
            <body>
                <h1>Fallback Title</h1>
            </body>
        </html>
    "#;

    let result = extractor.extract(html, "http://example.com").unwrap();
    assert!(result.contains_key("title"));
    assert_eq!(result["title"].as_str().unwrap(), "Fallback Title");
}

#[test]
fn test_schema_generator() {
    let generator = SchemaGenerator::new(0.7);
    let html = r#"
        <html>
            <head><title>Test Article</title></head>
            <body>
                <article>
                    <h1>Article Title</h1>
                    <div class="article-content">Content here</div>
                </article>
            </body>
        </html>
    "#;

    let request = SchemaLearnRequest {
        url: "http://example.com".to_string(),
        goal: "article".to_string(),
        confidence_threshold: 0.7,
        fields: None,
        verbose: false,
    };

    let response = generator
        .learn_from_html(html, "http://example.com", &request)
        .unwrap();

    assert!(!response.schema.fields.is_empty());
    assert!(!response.schema.selectors.is_empty());
    assert!(response.analysis.confidence > 0.0);
    assert!(response.analysis.fields_detected > 0);
}

#[test]
fn test_schema_comparator_identical() {
    let schema1 = create_test_schema();
    let schema2 = create_test_schema();

    let comparator = SchemaComparator::new();
    let comparison = comparator.compare(&schema1, &schema2).unwrap();

    assert_eq!(comparison.added_fields.len(), 0);
    assert_eq!(comparison.removed_fields.len(), 0);
    assert!(comparison.common_fields.len() > 0);
}

#[test]
fn test_schema_comparator_differences() {
    let schema1 = create_test_schema();
    let mut schema2 = create_test_schema();

    // Add extra field to schema2
    schema2.add_field("author".to_string(), FieldSchema::optional("string"));

    let comparator = SchemaComparator::new();
    let comparison = comparator.compare(&schema1, &schema2).unwrap();

    assert_eq!(comparison.added_fields.len(), 1);
    assert!(comparison.added_fields.contains(&"author".to_string()));
}

#[test]
fn test_schema_registry_operations() {
    let mut registry = SchemaRegistry::new();

    // Register schema
    let schema = create_test_schema();
    registry.register(schema).unwrap();
    assert_eq!(registry.count(), 1);

    // Get schema
    let retrieved = registry.get("test-article", Some("1.0.0")).unwrap();
    assert_eq!(retrieved.name, "test-article");

    // Increment usage
    registry.increment_usage("test-article", "1.0.0").unwrap();
    let updated = registry.get("test-article", Some("1.0.0")).unwrap();
    assert_eq!(updated.metadata.usage_count, 1);

    // Remove schema
    registry.remove("test-article", Some("1.0.0")).unwrap();
    assert_eq!(registry.count(), 0);
}

#[test]
fn test_schema_registry_list() {
    use riptide_extraction::schema::registry::ListRequest;

    let mut registry = SchemaRegistry::new();

    // Register multiple schemas
    let mut schema1 = create_test_schema();
    schema1.name = "schema1".to_string();
    schema1.metadata.is_public = true;
    registry.register(schema1).unwrap();

    let mut schema2 = create_test_schema();
    schema2.name = "schema2".to_string();
    schema2.metadata.is_public = false;
    registry.register(schema2).unwrap();

    // List all schemas
    let request = ListRequest {
        tag: None,
        goal: None,
        public_only: false,
        limit: 10,
    };
    let response = registry.list(&request).unwrap();
    assert_eq!(response.total, 2);

    // List only public schemas
    let public_request = ListRequest {
        tag: None,
        goal: None,
        public_only: true,
        limit: 10,
    };
    let public_response = registry.list(&public_request).unwrap();
    assert_eq!(public_response.total, 1);
}

#[test]
fn test_schema_validator_structure() {
    let validator = SchemaValidator::new();
    let schema = create_test_schema();

    let warnings = validator.validate_schema_structure(&schema).unwrap();
    assert!(warnings.is_empty(), "Valid schema should have no warnings");
}

#[test]
fn test_schema_validator_empty_schema() {
    let validator = SchemaValidator::new();
    let schema =
        ExtractionSchema::new("empty".to_string(), "1.0.0".to_string(), "test".to_string());

    let warnings = validator.validate_schema_structure(&schema).unwrap();
    assert!(!warnings.is_empty());
    assert!(warnings[0].contains("no fields"));
}

#[test]
fn test_schema_validator_low_confidence() {
    let validator = SchemaValidator::new();
    let mut schema = create_test_schema();

    // Set low confidence
    if let Some(rules) = schema.selectors.get_mut("title") {
        rules[0].confidence = 0.3;
    }

    let warnings = validator.validate_schema_structure(&schema).unwrap();
    assert!(warnings.iter().any(|w| w.contains("low confidence")));
}

#[tokio::test]
async fn test_schema_extraction_test_result() {
    let schema = create_test_schema();
    let extractor = SchemaExtractor::new(schema);

    let html = r#"
        <html>
            <body>
                <h1>Test Title</h1>
                <article>Content</article>
            </body>
        </html>
    "#;

    let result = extractor
        .test_extraction(html, "http://example.com")
        .await
        .unwrap();

    assert!(result.success);
    assert!(result.confidence > 0.0);
    assert_eq!(result.fields_extracted, 2);
    assert!(result.missing_fields.is_empty());
}

#[test]
fn test_validation_rules() {
    let mut schema = create_test_schema();
    schema.set_validation(ValidationRules {
        min_fields: Some(2),
        required_fields: Some(vec!["title".to_string()]),
        min_confidence: Some(0.7),
        custom_rules: None,
    });

    assert!(schema.validation.is_some());
    let validation = schema.validation.unwrap();
    assert_eq!(validation.min_fields, Some(2));
    assert_eq!(validation.min_confidence, Some(0.7));
}

#[test]
fn test_schema_metadata_defaults() {
    let metadata = SchemaMetadata::default();
    assert!(!metadata.is_public);
    assert_eq!(metadata.usage_count, 0);
    assert!(metadata.success_rate.is_none());
    assert!(metadata.tags.is_empty());
}

// Helper function to create a test schema
fn create_test_schema() -> ExtractionSchema {
    let mut schema = ExtractionSchema {
        name: "test-article".to_string(),
        version: "1.0.0".to_string(),
        goal: "article".to_string(),
        description: Some("Test schema for articles".to_string()),
        fields: HashMap::new(),
        selectors: HashMap::new(),
        validation: None,
        metadata: SchemaMetadata::default(),
    };

    // Add title field
    schema.add_field(
        "title".to_string(),
        FieldSchema::required("string").with_description("Article title"),
    );
    schema.add_selector("title".to_string(), SelectorRule::css("h1", 10, 0.9));

    // Add content field
    schema.add_field(
        "content".to_string(),
        FieldSchema::required("string").with_description("Article content"),
    );
    schema.add_selector(
        "content".to_string(),
        SelectorRule::css("article", 10, 0.85),
    );

    schema
}
