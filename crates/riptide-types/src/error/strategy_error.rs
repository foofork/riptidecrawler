//! Strategy-specific errors for extraction operations
//!
//! This module provides detailed, actionable error types for each extraction
//! strategy (CSS, LLM, Browser, Regex, WASM, JSON-LD, ICS, etc.)

use std::time::Duration;
use thiserror::Error;

/// Strategy-specific extraction errors with rich context
#[derive(Error, Debug, Clone)]
pub enum StrategyError {
    /// CSS selector failed to find elements
    #[error("CSS selector '{selector}' failed: {reason} (url: {url})")]
    CssSelectorFailed {
        selector: String,
        reason: String,
        url: String,
        html_snippet: String,
    },

    /// LLM provider timed out
    #[error("LLM provider {provider} timed out after {timeout_secs}s")]
    LlmTimeout {
        provider: String,
        timeout_secs: u64,
        request_id: String,
    },

    /// LLM circuit breaker is open
    #[error("LLM provider {provider} circuit breaker open, retry after {retry_after:?}")]
    LlmCircuitBreakerOpen {
        provider: String,
        retry_after: Duration,
    },

    /// Browser navigation failed
    #[error("Browser navigation to {url} failed: {reason}")]
    BrowserNavigationFailed {
        url: String,
        reason: String,
        status_code: Option<u16>,
    },

    /// Regex pattern is invalid
    #[error("Regex pattern '{pattern}' invalid: {reason}")]
    RegexPatternInvalid { pattern: String, reason: String },

    /// WASM module execution failed
    #[error("WASM module execution failed: {reason}")]
    WasmExecutionFailed {
        module_name: String,
        reason: String,
        stack_trace: Option<String>,
    },

    /// JSON-LD not found in HTML
    #[error("JSON-LD not found in HTML (url: {url})")]
    JsonLdNotFound { url: String, html_snippet: String },

    /// ICS calendar parsing failed
    #[error("ICS parsing failed: {reason}")]
    IcsParsingFailed {
        reason: String,
        content_snippet: String,
    },

    /// Generic strategy failure
    #[error("Strategy '{strategy}' failed: {reason}")]
    StrategyFailed { strategy: String, reason: String },
}

impl StrategyError {
    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::CssSelectorFailed { .. } => "CSS_001",
            Self::LlmTimeout { .. } => "LLM_001",
            Self::LlmCircuitBreakerOpen { .. } => "LLM_002",
            Self::BrowserNavigationFailed { .. } => "BROWSER_001",
            Self::RegexPatternInvalid { .. } => "REGEX_001",
            Self::WasmExecutionFailed { .. } => "WASM_001",
            Self::JsonLdNotFound { .. } => "JSONLD_001",
            Self::IcsParsingFailed { .. } => "ICS_001",
            Self::StrategyFailed { .. } => "STRATEGY_999",
        }
    }

    /// Get the strategy name
    pub fn strategy_name(&self) -> &str {
        match self {
            Self::CssSelectorFailed { .. } => "css",
            Self::LlmTimeout { .. } | Self::LlmCircuitBreakerOpen { .. } => "llm",
            Self::BrowserNavigationFailed { .. } => "browser",
            Self::RegexPatternInvalid { .. } => "regex",
            Self::WasmExecutionFailed { .. } => "wasm",
            Self::JsonLdNotFound { .. } => "json-ld",
            Self::IcsParsingFailed { .. } => "ics",
            Self::StrategyFailed { strategy, .. } => strategy,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::LlmTimeout { .. }
                | Self::LlmCircuitBreakerOpen { .. }
                | Self::BrowserNavigationFailed { .. }
                | Self::WasmExecutionFailed { .. }
        )
    }

    /// Get retry delay if retryable
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            Self::LlmCircuitBreakerOpen { retry_after, .. } => Some(*retry_after),
            Self::LlmTimeout { .. } => Some(Duration::from_secs(5)),
            Self::BrowserNavigationFailed { .. } => Some(Duration::from_secs(3)),
            Self::WasmExecutionFailed { .. } => Some(Duration::from_secs(2)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_selector_error_code() {
        let err = StrategyError::CssSelectorFailed {
            selector: "div.event".to_string(),
            reason: "Not found".to_string(),
            url: "https://example.com".to_string(),
            html_snippet: "<html>...".to_string(),
        };

        assert_eq!(err.error_code(), "CSS_001");
        assert_eq!(err.strategy_name(), "css");
        assert!(!err.is_retryable());
        assert!(err.retry_delay().is_none());
    }

    #[test]
    fn test_llm_timeout_error() {
        let err = StrategyError::LlmTimeout {
            provider: "openai".to_string(),
            timeout_secs: 30,
            request_id: "req_123".to_string(),
        };

        assert_eq!(err.error_code(), "LLM_001");
        assert_eq!(err.strategy_name(), "llm");
        assert!(err.is_retryable());
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_llm_circuit_breaker_error() {
        let err = StrategyError::LlmCircuitBreakerOpen {
            provider: "anthropic".to_string(),
            retry_after: Duration::from_secs(60),
        };

        assert_eq!(err.error_code(), "LLM_002");
        assert_eq!(err.strategy_name(), "llm");
        assert!(err.is_retryable());
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_browser_navigation_error() {
        let err = StrategyError::BrowserNavigationFailed {
            url: "https://example.com".to_string(),
            reason: "Timeout".to_string(),
            status_code: Some(504),
        };

        assert_eq!(err.error_code(), "BROWSER_001");
        assert_eq!(err.strategy_name(), "browser");
        assert!(err.is_retryable());
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(3)));
    }

    #[test]
    fn test_regex_pattern_error() {
        let err = StrategyError::RegexPatternInvalid {
            pattern: "[invalid".to_string(),
            reason: "Unclosed bracket".to_string(),
        };

        assert_eq!(err.error_code(), "REGEX_001");
        assert_eq!(err.strategy_name(), "regex");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_wasm_execution_error() {
        let err = StrategyError::WasmExecutionFailed {
            module_name: "extractor.wasm".to_string(),
            reason: "Memory limit exceeded".to_string(),
            stack_trace: Some("at function_name...".to_string()),
        };

        assert_eq!(err.error_code(), "WASM_001");
        assert_eq!(err.strategy_name(), "wasm");
        assert!(err.is_retryable());
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_json_ld_not_found_error() {
        let err = StrategyError::JsonLdNotFound {
            url: "https://example.com".to_string(),
            html_snippet: "<html><body>No JSON-LD</body></html>".to_string(),
        };

        assert_eq!(err.error_code(), "JSONLD_001");
        assert_eq!(err.strategy_name(), "json-ld");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_ics_parsing_error() {
        let err = StrategyError::IcsParsingFailed {
            reason: "Invalid format".to_string(),
            content_snippet: "BEGIN:VCALENDAR...".to_string(),
        };

        assert_eq!(err.error_code(), "ICS_001");
        assert_eq!(err.strategy_name(), "ics");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_generic_strategy_error() {
        let err = StrategyError::StrategyFailed {
            strategy: "custom".to_string(),
            reason: "Unknown failure".to_string(),
        };

        assert_eq!(err.error_code(), "STRATEGY_999");
        assert_eq!(err.strategy_name(), "custom");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_error_display() {
        let err = StrategyError::CssSelectorFailed {
            selector: "div.event".to_string(),
            reason: "Not found".to_string(),
            url: "https://example.com".to_string(),
            html_snippet: "<html>...".to_string(),
        };

        let display = format!("{}", err);
        assert!(display.contains("div.event"));
        assert!(display.contains("Not found"));
        assert!(display.contains("https://example.com"));
    }
}
