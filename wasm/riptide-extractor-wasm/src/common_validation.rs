//! Common validation functions for WASM extractors.
//!
//! This module provides validation utilities for WASM components.
//! Note: Cannot use riptide-core directly due to WASM compilation constraints.

use crate::ExtractionError;

/// Validate HTML and URL input using common validation patterns
pub fn validate_extraction_input(html: &str, url: &str) -> Result<(), ExtractionError> {
    // Validate HTML is not empty
    if html.trim().is_empty() {
        return Err(ExtractionError::InvalidHtml(
            "Empty HTML content".to_string(),
        ));
    }

    // Validate URL format using common pattern
    if let Err(e) = validate_url_format(url) {
        return Err(ExtractionError::InvalidHtml(format!(
            "Invalid URL format: {}",
            e
        )));
    }

    // Validate HTML structure
    validate_html_structure(html)?;

    Ok(())
}

/// Validate URL format (follows same patterns as riptide-core common validation)
pub fn validate_url_format(url: &str) -> Result<(), String> {
    // Basic URL validation using the same pattern as common validation
    if url.len() > 2048 {
        return Err(format!("URL length {} exceeds maximum {}", url.len(), 2048));
    }

    // Parse URL to validate format
    match url::Url::parse(url) {
        Ok(parsed_url) => {
            // Check scheme
            match parsed_url.scheme() {
                "http" | "https" => Ok(()),
                scheme => Err(format!("Unsupported URL scheme: {}", scheme)),
            }
        }
        Err(e) => Err(format!("Invalid URL format: {}", e)),
    }
}

/// Validate HTML structure using common patterns
pub fn validate_html_structure(html: &str) -> Result<(), ExtractionError> {
    if html.trim().is_empty() {
        return Err(ExtractionError::InvalidHtml(
            "Empty HTML content".to_string(),
        ));
    }

    // Basic HTML validation using consistent patterns
    let html_lower = html.to_lowercase();
    let has_html_tags = html_lower.contains("<html") || html_lower.contains("<!doctype");
    let has_body = html_lower.contains("<body");
    let has_content_tags = html_lower.contains("<p>")
        || html_lower.contains("<div")
        || html_lower.contains("<article")
        || html_lower.contains("<main");

    if has_html_tags && (has_body || has_content_tags) {
        Ok(())
    } else {
        Err(ExtractionError::InvalidHtml(
            "Invalid HTML structure - missing required HTML elements".to_string(),
        ))
    }
}

/// Validate content size against limits (follows same limits as riptide-core)
pub fn validate_content_size(size: usize) -> Result<(), ExtractionError> {
    const MAX_CONTENT_SIZE: usize = 20 * 1024 * 1024; // 20MB, same as common validation

    if size > MAX_CONTENT_SIZE {
        return Err(ExtractionError::InvalidHtml(format!(
            "Content size {} exceeds maximum {}",
            size, MAX_CONTENT_SIZE
        )));
    }

    Ok(())
}

/// Validate extraction mode parameters
pub fn validate_extraction_mode(mode: &crate::ExtractionMode) -> Result<(), ExtractionError> {
    match mode {
        crate::ExtractionMode::Article
        | crate::ExtractionMode::Full
        | crate::ExtractionMode::Metadata => {
            // Standard modes are always valid
            Ok(())
        }
        crate::ExtractionMode::Custom(selectors) => {
            if selectors.is_empty() {
                return Err(ExtractionError::InvalidHtml(
                    "Custom extraction mode requires at least one selector".to_string(),
                ));
            }

            // Validate CSS selectors format (basic validation)
            for selector in selectors {
                if selector.trim().is_empty() {
                    return Err(ExtractionError::InvalidHtml(
                        "CSS selector cannot be empty".to_string(),
                    ));
                }

                // Basic CSS selector validation - check for obvious invalid patterns
                if selector.contains("..") || selector.starts_with('>') {
                    return Err(ExtractionError::InvalidHtml(format!(
                        "Invalid CSS selector format: {}",
                        selector
                    )));
                }
            }

            Ok(())
        }
    }
}

/// Common patterns for parameter validation
pub mod parameter_validation {
    use super::*;

    /// Validate that a string parameter is not empty
    #[allow(dead_code)]
    pub fn validate_non_empty_string(value: &str, param_name: &str) -> Result<(), ExtractionError> {
        if value.trim().is_empty() {
            return Err(ExtractionError::InvalidHtml(format!(
                "{} cannot be empty",
                param_name
            )));
        }
        Ok(())
    }

    /// Validate that a number is within a valid range
    #[allow(dead_code)]
    pub fn validate_number_range<T>(
        value: T,
        min: T,
        max: T,
        param_name: &str,
    ) -> Result<(), ExtractionError>
    where
        T: PartialOrd + std::fmt::Display,
    {
        if value < min || value > max {
            return Err(ExtractionError::InvalidHtml(format!(
                "{} must be between {} and {}",
                param_name, min, max
            )));
        }
        Ok(())
    }

    /// Validate that a collection has elements within size limits
    #[allow(dead_code)]
    pub fn validate_collection_size<T>(
        collection: &[T],
        min_size: usize,
        max_size: usize,
        collection_name: &str,
    ) -> Result<(), ExtractionError> {
        if collection.len() < min_size {
            return Err(ExtractionError::InvalidHtml(format!(
                "{} must contain at least {} items",
                collection_name, min_size
            )));
        }

        if collection.len() > max_size {
            return Err(ExtractionError::InvalidHtml(format!(
                "{} cannot contain more than {} items",
                collection_name, max_size
            )));
        }

        Ok(())
    }
}

/// Error handling patterns that match common error conversion patterns
pub mod error_patterns {
    use super::*;

    /// Convert common validation errors to extraction errors
    #[allow(dead_code)]
    pub fn validation_error_to_extraction_error(field: &str, reason: &str) -> ExtractionError {
        ExtractionError::InvalidHtml(format!("Validation failed for {}: {}", field, reason))
    }

    /// Create a standard invalid input error
    #[allow(dead_code)]
    pub fn invalid_input_error(input_type: &str, details: &str) -> ExtractionError {
        ExtractionError::InvalidHtml(format!("Invalid {} input: {}", input_type, details))
    }

    /// Create a standard resource limit error
    #[allow(dead_code)]
    pub fn resource_limit_error(resource: &str, current: usize, limit: usize) -> ExtractionError {
        ExtractionError::InternalError(format!(
            "{} limit exceeded: {} > {}",
            resource, current, limit
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_extraction_input() {
        // Valid inputs
        let html = "<html><body><p>Test content</p></body></html>";
        let url = "https://example.com";
        assert!(validate_extraction_input(html, url).is_ok());

        // Empty HTML
        assert!(validate_extraction_input("", url).is_err());
        assert!(validate_extraction_input("   ", url).is_err());

        // Invalid URL
        assert!(validate_extraction_input(html, "not-a-url").is_err());
        assert!(validate_extraction_input(html, "ftp://example.com").is_err());
    }

    #[test]
    fn test_validate_url_format() {
        // Valid URLs
        assert!(validate_url_format("https://example.com").is_ok());
        assert!(validate_url_format("http://test.org/path").is_ok());

        // Invalid URLs
        assert!(validate_url_format("ftp://example.com").is_err());
        assert!(validate_url_format("not-a-url").is_err());
        assert!(validate_url_format(&"x".repeat(3000)).is_err()); // Too long
    }

    #[test]
    fn test_validate_html_structure() {
        // Valid HTML structures
        assert!(validate_html_structure("<html><body><p>content</p></body></html>").is_ok());
        assert!(validate_html_structure(
            "<!DOCTYPE html><html><body><div>content</div></body></html>"
        )
        .is_ok());

        // Invalid HTML structures
        assert!(validate_html_structure("").is_err());
        assert!(validate_html_structure("   ").is_err());
        assert!(validate_html_structure("plain text without html").is_err());
    }

    #[test]
    fn test_validate_content_size() {
        assert!(validate_content_size(1024).is_ok());
        assert!(validate_content_size(10 * 1024 * 1024).is_ok());
        assert!(validate_content_size(25 * 1024 * 1024).is_err()); // Over 20MB limit
    }

    #[test]
    fn test_parameter_validation() {
        use parameter_validation::*;

        // String validation
        assert!(validate_non_empty_string("test", "name").is_ok());
        assert!(validate_non_empty_string("", "name").is_err());
        assert!(validate_non_empty_string("   ", "name").is_err());

        // Number range validation
        assert!(validate_number_range(5, 0, 10, "count").is_ok());
        assert!(validate_number_range(-1, 0, 10, "count").is_err());
        assert!(validate_number_range(15, 0, 10, "count").is_err());

        // Collection size validation
        let items = vec![1, 2, 3];
        assert!(validate_collection_size(&items, 1, 5, "items").is_ok());
        assert!(validate_collection_size(&items, 5, 10, "items").is_err());
        assert!(validate_collection_size(&items, 1, 2, "items").is_err());
    }
}
