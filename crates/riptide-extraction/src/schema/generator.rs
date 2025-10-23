//! Schema generation and learning

use super::types::{
    ExtractionSchema, FieldSchema, SchemaAnalysis, SchemaLearnRequest, SchemaLearnResponse,
    SelectorRule,
};
use anyhow::Result;
use scraper::Html;

/// Generates extraction schemas from HTML analysis
pub struct SchemaGenerator {
    #[allow(dead_code)] // Used for future confidence filtering
    confidence_threshold: f64,
}

impl SchemaGenerator {
    /// Create a new schema generator
    pub fn new(confidence_threshold: f64) -> Self {
        Self {
            confidence_threshold,
        }
    }

    /// Learn a schema from HTML content
    pub fn learn_from_html(
        &self,
        html: &str,
        _url: &str,
        request: &SchemaLearnRequest,
    ) -> Result<SchemaLearnResponse> {
        let document = Html::parse_document(html);

        let mut schema = ExtractionSchema::new(
            format!("{}-schema", request.goal),
            "1.0.0".to_string(),
            request.goal.clone(),
        );

        let mut analysis = SchemaAnalysis {
            confidence: 0.0,
            fields_detected: 0,
            selectors_generated: 0,
            patterns_found: Vec::new(),
            warnings: Vec::new(),
        };

        let mut suggestions = Vec::new();

        // Learn based on goal type
        match request.goal.as_str() {
            "article" => {
                self.learn_article_schema(&document, &mut schema, &mut analysis)?;
            }
            "product" => {
                self.learn_product_schema(&document, &mut schema, &mut analysis)?;
            }
            "listing" => {
                self.learn_listing_schema(&document, &mut schema, &mut analysis)?;
            }
            _ => {
                self.learn_generic_schema(&document, &mut schema, &mut analysis)?;
            }
        }

        // Learn specific fields if requested
        if let Some(fields) = &request.fields {
            self.learn_specific_fields(&document, fields, &mut schema, &mut analysis)?;
        }

        // Filter selectors by confidence threshold
        self.filter_low_confidence_selectors(&mut schema, request.confidence_threshold);

        // Update analysis statistics
        analysis.fields_detected = schema.fields.len() as u32;
        analysis.selectors_generated = schema
            .selectors
            .values()
            .map(|rules| rules.len())
            .sum::<usize>() as u32;

        // Calculate overall confidence
        analysis.confidence = self.calculate_overall_confidence(&schema);

        // Generate suggestions
        suggestions.extend(self.generate_suggestions(&schema, &analysis));

        Ok(SchemaLearnResponse {
            schema,
            analysis,
            suggestions,
        })
    }

    /// Learn schema for article content
    fn learn_article_schema(
        &self,
        _document: &Html,
        schema: &mut ExtractionSchema,
        analysis: &mut SchemaAnalysis,
    ) -> Result<()> {
        // Title selectors
        let title_selectors = vec![
            ("h1", 10, 0.9),
            ("article h1", 9, 0.95),
            (".article-title", 8, 0.85),
            ("meta[property='og:title']", 7, 0.8),
        ];

        self.add_field_with_selectors(
            schema,
            "title",
            FieldSchema::required("string").with_description("Article title"),
            &title_selectors,
        )?;

        // Content selectors
        let content_selectors = vec![
            ("article", 10, 0.9),
            (".article-content", 9, 0.85),
            ("main", 8, 0.8),
            (".content", 7, 0.75),
        ];

        self.add_field_with_selectors(
            schema,
            "content",
            FieldSchema::required("string").with_description("Article content"),
            &content_selectors,
        )?;

        // Author selectors
        let author_selectors = vec![
            (".author", 10, 0.9),
            ("[rel='author']", 9, 0.85),
            ("meta[name='author']", 8, 0.8),
        ];

        self.add_field_with_selectors(
            schema,
            "author",
            FieldSchema::optional("string").with_description("Article author"),
            &author_selectors,
        )?;

        // Date selectors
        let date_selectors = vec![
            ("time[datetime]", 10, 0.95),
            (".publish-date", 9, 0.85),
            ("meta[property='article:published_time']", 8, 0.9),
        ];

        self.add_field_with_selectors(
            schema,
            "published_date",
            FieldSchema::optional("string").with_description("Publication date"),
            &date_selectors,
        )?;

        analysis.patterns_found.push("article".to_string());

        Ok(())
    }

    /// Learn schema for product pages
    fn learn_product_schema(
        &self,
        _document: &Html,
        schema: &mut ExtractionSchema,
        analysis: &mut SchemaAnalysis,
    ) -> Result<()> {
        // Product name
        let name_selectors = vec![
            (".product-name", 10, 0.9),
            ("h1.product", 9, 0.85),
            ("[itemprop='name']", 8, 0.95),
        ];

        self.add_field_with_selectors(
            schema,
            "name",
            FieldSchema::required("string").with_description("Product name"),
            &name_selectors,
        )?;

        // Price
        let price_selectors = vec![
            (".price", 10, 0.9),
            ("[itemprop='price']", 9, 0.95),
            (".product-price", 8, 0.85),
        ];

        self.add_field_with_selectors(
            schema,
            "price",
            FieldSchema::required("string").with_description("Product price"),
            &price_selectors,
        )?;

        // Description
        let desc_selectors = vec![
            (".product-description", 10, 0.9),
            ("[itemprop='description']", 9, 0.95),
            (".description", 8, 0.8),
        ];

        self.add_field_with_selectors(
            schema,
            "description",
            FieldSchema::optional("string").with_description("Product description"),
            &desc_selectors,
        )?;

        analysis.patterns_found.push("product".to_string());

        Ok(())
    }

    /// Learn schema for listing pages
    fn learn_listing_schema(
        &self,
        _document: &Html,
        schema: &mut ExtractionSchema,
        analysis: &mut SchemaAnalysis,
    ) -> Result<()> {
        // Items container
        let items_selectors = vec![
            (".listing-item", 10, 0.9),
            (".item", 9, 0.8),
            ("article", 8, 0.75),
        ];

        self.add_field_with_selectors(
            schema,
            "items",
            FieldSchema::required("array").with_description("List items"),
            &items_selectors,
        )?;

        analysis.patterns_found.push("listing".to_string());

        Ok(())
    }

    /// Learn generic schema
    fn learn_generic_schema(
        &self,
        _document: &Html,
        schema: &mut ExtractionSchema,
        analysis: &mut SchemaAnalysis,
    ) -> Result<()> {
        // Basic fields for any page
        let title_selectors = vec![("title", 10, 1.0), ("h1", 9, 0.9)];

        self.add_field_with_selectors(
            schema,
            "title",
            FieldSchema::required("string").with_description("Page title"),
            &title_selectors,
        )?;

        analysis.patterns_found.push("generic".to_string());

        Ok(())
    }

    /// Learn specific fields requested by user
    fn learn_specific_fields(
        &self,
        _document: &Html,
        fields: &[String],
        schema: &mut ExtractionSchema,
        _analysis: &mut SchemaAnalysis,
    ) -> Result<()> {
        for field_name in fields {
            // Generate selectors based on field name
            let selectors = vec![
                (format!(".{}", field_name), 10, 0.8),
                (format!("[name='{}']", field_name), 9, 0.75),
                (format!("#{}", field_name), 8, 0.7),
            ];

            let selector_refs: Vec<(&str, u32, f64)> = selectors
                .iter()
                .map(|(s, p, c)| (s.as_str(), *p, *c))
                .collect();

            self.add_field_with_selectors(
                schema,
                field_name,
                FieldSchema::optional("string"),
                &selector_refs,
            )?;
        }

        Ok(())
    }

    /// Add a field with its selectors to the schema
    fn add_field_with_selectors(
        &self,
        schema: &mut ExtractionSchema,
        field_name: &str,
        field_schema: FieldSchema,
        selectors: &[(&str, u32, f64)],
    ) -> Result<()> {
        schema.add_field(field_name.to_string(), field_schema);

        for (selector, priority, confidence) in selectors {
            schema.add_selector(
                field_name.to_string(),
                SelectorRule::css(selector.to_string(), *priority, *confidence),
            );
        }

        Ok(())
    }

    /// Filter out selectors below confidence threshold
    fn filter_low_confidence_selectors(&self, schema: &mut ExtractionSchema, threshold: f64) {
        for rules in schema.selectors.values_mut() {
            rules.retain(|rule| rule.confidence >= threshold);
        }

        // Remove fields with no selectors
        let fields_to_remove: Vec<String> = schema
            .selectors
            .iter()
            .filter(|(_, rules)| rules.is_empty())
            .map(|(field, _)| field.clone())
            .collect();

        for field in fields_to_remove {
            schema.selectors.remove(&field);
            schema.fields.remove(&field);
        }
    }

    /// Calculate overall confidence score
    fn calculate_overall_confidence(&self, schema: &ExtractionSchema) -> f64 {
        if schema.selectors.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = schema
            .selectors
            .values()
            .flat_map(|rules| rules.iter().map(|r| r.confidence))
            .sum();

        let count = schema
            .selectors
            .values()
            .flat_map(|rules| rules.iter())
            .count() as f64;

        if count > 0.0 {
            total_confidence / count
        } else {
            0.0
        }
    }

    /// Generate suggestions for improving the schema
    fn generate_suggestions(
        &self,
        schema: &ExtractionSchema,
        analysis: &SchemaAnalysis,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        if analysis.confidence < 0.7 {
            suggestions.push(
                "Consider lowering confidence threshold or testing with more representative URLs"
                    .to_string(),
            );
        }

        if schema.fields.len() < 3 {
            suggestions
                .push("Schema has few fields, consider adding more specific selectors".to_string());
        }

        for (field, rules) in &schema.selectors {
            if rules.len() == 1 {
                suggestions.push(format!(
                    "Field '{}' has only one selector, consider adding fallbacks",
                    field
                ));
            }
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = SchemaGenerator::new(0.7);
        assert_eq!(generator.confidence_threshold, 0.7);
    }

    #[test]
    fn test_learn_article_schema() {
        let generator = SchemaGenerator::new(0.7);
        let html = r#"
            <html>
                <head><title>Test Article</title></head>
                <body>
                    <article>
                        <h1>Article Title</h1>
                        <div class="author">John Doe</div>
                        <time datetime="2024-01-01">January 1, 2024</time>
                        <div class="article-content">Article content here</div>
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

        assert!(response.schema.fields.len() >= 2); // At least title and content
        assert!(response.analysis.confidence > 0.0);
        assert!(!response.analysis.patterns_found.is_empty());
    }

    #[test]
    fn test_learn_product_schema() {
        let generator = SchemaGenerator::new(0.7);
        let html = r#"
            <html>
                <body>
                    <div class="product">
                        <h1 class="product-name">Product Name</h1>
                        <span class="price">$19.99</span>
                        <div class="product-description">Description here</div>
                    </div>
                </body>
            </html>
        "#;

        let request = SchemaLearnRequest {
            url: "http://example.com".to_string(),
            goal: "product".to_string(),
            confidence_threshold: 0.7,
            fields: None,
            verbose: false,
        };

        let response = generator
            .learn_from_html(html, "http://example.com", &request)
            .unwrap();

        assert!(response.schema.fields.contains_key("name"));
        assert!(response.schema.fields.contains_key("price"));
    }
}
