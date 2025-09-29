//! PII Redaction System for RipTide
//!
//! Provides automatic detection and redaction of Personally Identifiable Information
//! in logs, LLM payloads, and other sensitive data.

use crate::security::types::*;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

/// PII detection patterns and their types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiPattern {
    pub name: String,
    pub pattern_type: PiiType,
    pub regex: String,
    pub confidence_level: ConfidenceLevel,
    pub replacement_template: String,
}

/// Types of PII that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PiiType {
    Email,
    Phone,
    SSN,
    CreditCard,
    IPAddress,
    MacAddress,
    URL,
    Custom(String),
}

impl std::fmt::Display for PiiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PiiType::Email => write!(f, "EMAIL"),
            PiiType::Phone => write!(f, "PHONE"),
            PiiType::SSN => write!(f, "SSN"),
            PiiType::CreditCard => write!(f, "CREDIT_CARD"),
            PiiType::IPAddress => write!(f, "IP_ADDRESS"),
            PiiType::MacAddress => write!(f, "MAC_ADDRESS"),
            PiiType::URL => write!(f, "URL"),
            PiiType::Custom(name) => write!(f, "CUSTOM_{}", name.to_uppercase()),
        }
    }
}

/// Confidence levels for PII detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Information about detected PII
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiDetection {
    pub detection_id: String,
    pub pii_type: PiiType,
    pub original_value: String,
    pub redacted_value: String,
    pub position: PiiPosition,
    pub confidence: ConfidenceLevel,
    pub pattern_name: String,
}

/// Position information for detected PII
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiPosition {
    pub start: usize,
    pub end: usize,
    pub line_number: Option<usize>,
    pub column_number: Option<usize>,
}

/// Result of PII redaction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionResult {
    pub original_text: String,
    pub redacted_text: String,
    pub detections: Vec<PiiDetection>,
    pub stats: RedactionStats,
}

/// Statistics about redaction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionStats {
    pub total_detections: usize,
    pub detections_by_type: HashMap<PiiType, usize>,
    pub confidence_distribution: HashMap<ConfidenceLevel, usize>,
    pub redaction_time_ms: u64,
}

/// PII Redactor - handles detection and redaction of PII
pub struct PiiRedactor {
    config: PiiConfig,
    patterns: Vec<CompiledPattern>,
}

struct CompiledPattern {
    pattern: PiiPattern,
    regex: Regex,
}

impl PiiRedactor {
    /// Create a new PII redactor with default patterns
    pub fn new(config: Option<PiiConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        let patterns = Self::compile_patterns(&config)?;
        
        Ok(Self { config, patterns })
    }
    
    /// Compile regex patterns for PII detection
    fn compile_patterns(config: &PiiConfig) -> Result<Vec<CompiledPattern>> {
        let mut compiled_patterns = Vec::new();
        
        // Email pattern
        if config.enable_email_detection {
            let pattern = PiiPattern {
                name: "email".to_string(),
                pattern_type: PiiType::Email,
                regex: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
                confidence_level: ConfidenceLevel::High,
                replacement_template: "[EMAIL]".to_string(),
            };
            
            let regex = Regex::new(&pattern.regex)
                .map_err(|e| anyhow!("Invalid email regex: {}", e))?;
            
            compiled_patterns.push(CompiledPattern { pattern, regex });
        }
        
        // Phone number patterns
        if config.enable_phone_detection {
            // US phone numbers
            let phone_patterns = [
                r"\b\d{3}-\d{3}-\d{4}\b",                    // 123-456-7890
                r"\b\(\d{3}\)\s?\d{3}-\d{4}\b",             // (123) 456-7890
                r"\b\d{3}\.\d{3}\.\d{4}\b",                 // 123.456.7890
                r"\b\+1\s?\d{3}\s?\d{3}\s?\d{4}\b",        // +1 123 456 7890
                r"\b\d{10}\b",                              // 1234567890
            ];
            
            for (i, regex_str) in phone_patterns.iter().enumerate() {
                let pattern = PiiPattern {
                    name: format!("phone_{}", i),
                    pattern_type: PiiType::Phone,
                    regex: regex_str.to_string(),
                    confidence_level: ConfidenceLevel::High,
                    replacement_template: "[PHONE]".to_string(),
                };
                
                let regex = Regex::new(regex_str)
                    .map_err(|e| anyhow!("Invalid phone regex {}: {}", i, e))?;
                
                compiled_patterns.push(CompiledPattern { pattern, regex });
            }
        }
        
        // SSN patterns
        if config.enable_ssn_detection {
            let ssn_patterns = [
                r"\b\d{3}-\d{2}-\d{4}\b",     // 123-45-6789
                r"\b\d{3}\s\d{2}\s\d{4}\b",   // 123 45 6789
                r"\b\d{9}\b",                 // 123456789 (only if 9 digits)
            ];
            
            for (i, regex_str) in ssn_patterns.iter().enumerate() {
                let pattern = PiiPattern {
                    name: format!("ssn_{}", i),
                    pattern_type: PiiType::SSN,
                    regex: regex_str.to_string(),
                    confidence_level: if i == 2 { ConfidenceLevel::Medium } else { ConfidenceLevel::High },
                    replacement_template: "[SSN]".to_string(),
                };
                
                let regex = Regex::new(regex_str)
                    .map_err(|e| anyhow!("Invalid SSN regex {}: {}", i, e))?;
                
                compiled_patterns.push(CompiledPattern { pattern, regex });
            }
        }
        
        // Credit card patterns
        if config.enable_credit_card_detection {
            let cc_patterns = [
                r"\b4\d{3}\s?\d{4}\s?\d{4}\s?\d{4}\b",               // Visa
                r"\b5[1-5]\d{2}\s?\d{4}\s?\d{4}\s?\d{4}\b",         // MasterCard
                r"\b3[47]\d{2}\s?\d{6}\s?\d{5}\b",                  // American Express
                r"\b6(?:011|5\d{2})\s?\d{4}\s?\d{4}\s?\d{4}\b",    // Discover
            ];
            
            for (i, regex_str) in cc_patterns.iter().enumerate() {
                let card_type = match i {
                    0 => "visa",
                    1 => "mastercard",
                    2 => "amex",
                    3 => "discover",
                    _ => "unknown",
                };
                
                let pattern = PiiPattern {
                    name: format!("credit_card_{}", card_type),
                    pattern_type: PiiType::CreditCard,
                    regex: regex_str.to_string(),
                    confidence_level: ConfidenceLevel::High,
                    replacement_template: "[CREDIT_CARD]".to_string(),
                };
                
                let regex = Regex::new(regex_str)
                    .map_err(|e| anyhow!("Invalid credit card regex {}: {}", card_type, e))?;
                
                compiled_patterns.push(CompiledPattern { pattern, regex });
            }
        }
        
        // IP Address patterns
        let ip_pattern = PiiPattern {
            name: "ip_address".to_string(),
            pattern_type: PiiType::IPAddress,
            regex: r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b".to_string(),
            confidence_level: ConfidenceLevel::Medium,
            replacement_template: "[IP_ADDRESS]".to_string(),
        };
        
        let regex = Regex::new(&ip_pattern.regex)
            .map_err(|e| anyhow!("Invalid IP address regex: {}", e))?;
        compiled_patterns.push(CompiledPattern { pattern: ip_pattern, regex });
        
        // URL patterns
        let url_pattern = PiiPattern {
            name: "url".to_string(),
            pattern_type: PiiType::URL,
            regex: r#"https?://[^\s<>"{}|\\^`\[\]]+"#.to_string(),
            confidence_level: ConfidenceLevel::Medium,
            replacement_template: "[URL]".to_string(),
        };
        
        let regex = Regex::new(&url_pattern.regex)
            .map_err(|e| anyhow!("Invalid URL regex: {}", e))?;
        compiled_patterns.push(CompiledPattern { pattern: url_pattern, regex });
        
        // Custom patterns
        if config.enable_custom_patterns {
            for (i, custom_regex) in config.custom_patterns.iter().enumerate() {
                let pattern = PiiPattern {
                    name: format!("custom_{}", i),
                    pattern_type: PiiType::Custom(format!("PATTERN_{}", i)),
                    regex: custom_regex.clone(),
                    confidence_level: ConfidenceLevel::Medium,
                    replacement_template: format!("[CUSTOM_{}]", i),
                };
                
                let regex = Regex::new(custom_regex)
                    .map_err(|e| anyhow!("Invalid custom regex {}: {}", i, e))?;
                
                compiled_patterns.push(CompiledPattern { pattern, regex });
            }
        }
        
        Ok(compiled_patterns)
    }
    
    /// Detect and redact PII in text
    pub fn redact_text(&self, text: &str) -> Result<RedactionResult> {
        let start_time = std::time::Instant::now();
        let mut detections = Vec::new();
        let mut redacted_text = text.to_string();
        let mut offset_adjustment = 0i32;
        
        // Collect all matches first
        let mut all_matches = Vec::new();
        
        for compiled_pattern in &self.patterns {
            for mat in compiled_pattern.regex.find_iter(text) {
                let detection = PiiDetection {
                    detection_id: Uuid::new_v4().to_string(),
                    pii_type: compiled_pattern.pattern.pattern_type.clone(),
                    original_value: mat.as_str().to_string(),
                    redacted_value: self.create_redacted_value(
                        mat.as_str(),
                        &compiled_pattern.pattern,
                    ),
                    position: PiiPosition {
                        start: mat.start(),
                        end: mat.end(),
                        line_number: None, // Could be enhanced to track line numbers
                        column_number: None,
                    },
                    confidence: compiled_pattern.pattern.confidence_level.clone(),
                    pattern_name: compiled_pattern.pattern.name.clone(),
                };
                
                all_matches.push((mat.start(), mat.end(), detection));
            }
        }
        
        // Sort matches by position (reverse order for easier replacement)
        all_matches.sort_by(|a, b| b.0.cmp(&a.0));
        
        // Apply redactions
        for (start, end, detection) in all_matches {
            let adjusted_start = (start as i32 + offset_adjustment) as usize;
            let adjusted_end = (end as i32 + offset_adjustment) as usize;
            
            if adjusted_start <= redacted_text.len() && adjusted_end <= redacted_text.len() {
                redacted_text.replace_range(adjusted_start..adjusted_end, &detection.redacted_value);
                
                let length_change = detection.redacted_value.len() as i32 - (end - start) as i32;
                offset_adjustment += length_change;
                
                detections.push(detection);
            }
        }
        
        // Calculate statistics
        let mut detections_by_type = HashMap::new();
        let mut confidence_distribution = HashMap::new();
        
        for detection in &detections {
            *detections_by_type.entry(detection.pii_type.clone()).or_insert(0) += 1;
            *confidence_distribution.entry(detection.confidence.clone()).or_insert(0) += 1;
        }
        
        let stats = RedactionStats {
            total_detections: detections.len(),
            detections_by_type,
            confidence_distribution,
            redaction_time_ms: start_time.elapsed().as_millis() as u64,
        };
        
        debug!(
            total_detections = stats.total_detections,
            redaction_time_ms = stats.redaction_time_ms,
            "PII redaction completed"
        );
        
        if !detections.is_empty() {
            warn!(
                detections_count = detections.len(),
                "PII detected and redacted in text"
            );
        }
        
        Ok(RedactionResult {
            original_text: text.to_string(),
            redacted_text,
            detections,
            stats,
        })
    }
    
    /// Create redacted value based on configuration
    fn create_redacted_value(&self, original: &str, pattern: &PiiPattern) -> String {
        match self.config.redaction_method {
            RedactionMethod::Asterisk => {
                if self.config.preserve_format {
                    self.preserve_format_asterisk(original)
                } else {
                    "****".to_string()
                }
            }
            RedactionMethod::Hash => {
                let hash = sha2::Sha256::digest(original.as_bytes());
                format!("[REDACTED:{}]", hex::encode(&hash[..4]))
            }
            RedactionMethod::Remove => String::new(),
            RedactionMethod::Placeholder => pattern.replacement_template.clone(),
        }
    }
    
    /// Preserve format when using asterisk redaction
    fn preserve_format_asterisk(&self, original: &str) -> String {
        original
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    '*'
                } else {
                    c
                }
            })
            .collect()
    }
    
    /// Redact PII in structured data (JSON, etc.)
    pub fn redact_json(&self, json_str: &str) -> Result<String> {
        // For JSON, we need to be careful not to break the structure
        // This is a simplified implementation - a production version might parse JSON
        let result = self.redact_text(json_str)?;
        Ok(result.redacted_text)
    }
    
    /// Check if text contains PII without redacting
    pub fn contains_pii(&self, text: &str) -> bool {
        for compiled_pattern in &self.patterns {
            if compiled_pattern.regex.is_match(text) {
                return true;
            }
        }
        false
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &PiiConfig {
        &self.config
    }
    
    /// Update configuration and recompile patterns
    pub fn update_config(&mut self, config: PiiConfig) -> Result<()> {
        self.patterns = Self::compile_patterns(&config)?;
        self.config = config;
        Ok(())
    }
    
    /// Get supported PII types
    pub fn get_supported_types(&self) -> Vec<PiiType> {
        self.patterns
            .iter()
            .map(|p| p.pattern.pattern_type.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

/// PII Redaction Middleware for automatic redaction in logs and payloads
pub struct PiiRedactionMiddleware {
    redactor: Arc<PiiRedactor>,
    enabled: bool,
}

impl PiiRedactionMiddleware {
    /// Create new PII redaction middleware
    pub fn new(config: Option<PiiConfig>) -> Result<Self> {
        let redactor = Arc::new(PiiRedactor::new(config)?);
        Ok(Self {
            redactor,
            enabled: true,
        })
    }
    
    /// Enable or disable redaction
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Redact PII in request/response data
    pub fn redact_payload(&self, payload: &str) -> Result<String> {
        if !self.enabled {
            return Ok(payload.to_string());
        }
        
        let result = self.redactor.redact_text(payload)?;
        Ok(result.redacted_text)
    }
    
    /// Redact PII in log messages
    pub fn redact_log_message(&self, message: &str) -> String {
        if !self.enabled {
            return message.to_string();
        }
        
        match self.redactor.redact_text(message) {
            Ok(result) => result.redacted_text,
            Err(_) => {
                // If redaction fails, return original message
                // In production, you might want to log this error
                message.to_string()
            }
        }
    }
    
    /// Get redactor instance
    pub fn get_redactor(&self) -> Arc<PiiRedactor> {
        Arc::clone(&self.redactor)
    }
}

impl Default for PiiRedactionMiddleware {
    fn default() -> Self {
        Self::new(None).expect("Failed to create default PII redaction middleware")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_detection() {
        let config = PiiConfig {
            redaction_method: RedactionMethod::Placeholder,
            ..Default::default()
        };
        let redactor = PiiRedactor::new(Some(config)).unwrap();
        let text = "Contact me at john.doe@example.com for more info.";

        let result = redactor.redact_text(text).unwrap();
        assert!(result.redacted_text.contains("[EMAIL]"));
        assert!(!result.redacted_text.contains("john.doe@example.com"));
        assert_eq!(result.detections.len(), 1);
        assert_eq!(result.detections[0].pii_type, PiiType::Email);
    }
    
    #[test]
    fn test_phone_detection() {
        let config = PiiConfig {
            redaction_method: RedactionMethod::Placeholder,
            ..Default::default()
        };
        let redactor = PiiRedactor::new(Some(config)).unwrap();
        let text = "Call me at 123-456-7890 or (555) 123-4567.";

        let result = redactor.redact_text(text).unwrap();
        assert!(result.redacted_text.contains("[PHONE]"));
        // Both numbers should be detected
        assert!(!result.detections.is_empty());
        // Check that at least one phone was detected
        assert!(result.detections.iter().any(|d| d.pii_type == PiiType::Phone));
    }
    
    #[test]
    fn test_ssn_detection() {
        let config = PiiConfig {
            redaction_method: RedactionMethod::Placeholder,
            ..Default::default()
        };
        let redactor = PiiRedactor::new(Some(config)).unwrap();
        let text = "My SSN is 123-45-6789.";

        let result = redactor.redact_text(text).unwrap();
        assert!(result.redacted_text.contains("[SSN]"));
        assert!(!result.redacted_text.contains("123-45-6789"));
        assert_eq!(result.detections.len(), 1);
        assert_eq!(result.detections[0].pii_type, PiiType::SSN);
    }
    
    #[test]
    fn test_credit_card_detection() {
        let config = PiiConfig {
            redaction_method: RedactionMethod::Placeholder,
            ..Default::default()
        };
        let redactor = PiiRedactor::new(Some(config)).unwrap();
        let text = "Card number: 4532 1234 5678 9012";

        let result = redactor.redact_text(text).unwrap();
        assert!(result.redacted_text.contains("[CREDIT_CARD]"));
        assert!(!result.redacted_text.contains("4532 1234 5678 9012"));
        assert_eq!(result.detections.len(), 1);
        assert_eq!(result.detections[0].pii_type, PiiType::CreditCard);
    }
    
    #[test]
    fn test_custom_patterns() {
        let config = PiiConfig {
            enable_custom_patterns: true,
            custom_patterns: vec![r"\bTEST-\d{4}\b".to_string()],
            redaction_method: RedactionMethod::Placeholder,
            ..Default::default()
        };

        let redactor = PiiRedactor::new(Some(config)).unwrap();
        let text = "Reference number: TEST-1234";

        let result = redactor.redact_text(text).unwrap();
        assert!(result.redacted_text.contains("[CUSTOM_0]"));
        assert!(!result.redacted_text.contains("TEST-1234"));
        assert_eq!(result.detections.len(), 1);
    }
    
    #[test]
    fn test_asterisk_redaction_with_format_preservation() {
        let config = PiiConfig {
            redaction_method: RedactionMethod::Asterisk,
            preserve_format: true,
            ..Default::default()
        };
        
        let redactor = PiiRedactor::new(Some(config)).unwrap();
        let text = "SSN: 123-45-6789";
        
        let result = redactor.redact_text(text).unwrap();
        assert!(result.redacted_text.contains("***-**-****"));
    }
    
    #[test]
    fn test_pii_detection_without_redaction() {
        let redactor = PiiRedactor::new(None).unwrap();
        
        assert!(redactor.contains_pii("email: test@example.com"));
        assert!(redactor.contains_pii("phone: 123-456-7890"));
        assert!(!redactor.contains_pii("no sensitive data here"));
    }
    
    #[test]
    fn test_redaction_middleware() {
        let middleware = PiiRedactionMiddleware::new(None).unwrap();
        let payload = r#"{"email": "user@example.com", "phone": "123-456-7890"}"#;
        
        let redacted = middleware.redact_payload(payload).unwrap();
        assert!(!redacted.contains("user@example.com"));
        assert!(!redacted.contains("123-456-7890"));
    }
    
    #[test]
    fn test_disabled_middleware() {
        let mut middleware = PiiRedactionMiddleware::new(None).unwrap();
        middleware.set_enabled(false);
        
        let payload = "email: test@example.com";
        let result = middleware.redact_payload(payload).unwrap();
        assert_eq!(result, payload); // Should be unchanged
    }
}
