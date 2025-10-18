//! CDP (Chrome DevTools Protocol) integration for stealth operations
//!
//! This module provides integration between stealth features and CDP
//! for batch operations, header injection, and fingerprint application.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::fingerprint_enhanced::EnhancedFingerprintGenerator;

/// CDP stealth integrator for batch operations
pub struct CdpStealthIntegrator {
    /// Enhanced fingerprint generator
    fingerprint_generator: EnhancedFingerprintGenerator,
}

impl CdpStealthIntegrator {
    /// Create a new CDP stealth integrator
    pub fn new() -> Self {
        Self {
            fingerprint_generator: EnhancedFingerprintGenerator::with_default_config(),
        }
    }

    /// Generate CDP commands for stealth setup
    pub fn generate_stealth_commands(
        &mut self,
        user_agent: &str,
        session_id: Option<&str>,
    ) -> Vec<CdpCommand> {
        let fingerprint = self
            .fingerprint_generator
            .generate_contextual(user_agent, session_id);
        let params = self.fingerprint_generator.generate_cdp_params(&fingerprint);

        vec![
            CdpCommand::new("Page.setUserAgentOverride", params.to_user_agent_override()),
            CdpCommand::new(
                "Emulation.setTimezoneOverride",
                params.to_timezone_override(),
            ),
            CdpCommand::new(
                "Emulation.setDeviceMetricsOverride",
                params.to_device_metrics_override(),
            ),
        ]
    }

    /// Generate batch headers for multiple requests with consistent fingerprint
    pub fn generate_batch_headers(
        &mut self,
        user_agent: &str,
        session_id: Option<&str>,
        count: usize,
    ) -> BatchHeadersResult {
        let fingerprint = self
            .fingerprint_generator
            .generate_contextual(user_agent, session_id);
        let headers_batch = self
            .fingerprint_generator
            .generate_batch_headers(&fingerprint, count);

        BatchHeadersResult {
            headers: headers_batch,
            fingerprint_id: session_id.map(|s| s.to_string()),
            consistent: true,
        }
    }

    /// Apply stealth headers to a single request
    pub fn apply_stealth_headers(
        &mut self,
        base_headers: HashMap<String, String>,
        user_agent: &str,
        session_id: Option<&str>,
    ) -> HashMap<String, String> {
        let fingerprint = self
            .fingerprint_generator
            .generate_contextual(user_agent, session_id);
        let stealth_headers =
            crate::fingerprint::FingerprintGenerator::generate_consistent_headers(&fingerprint);

        // Merge base headers with stealth headers (stealth takes precedence)
        let mut merged = base_headers;
        for (key, value) in stealth_headers {
            merged.insert(key, value);
        }
        merged
    }

    /// Create CDP command for JavaScript injection
    pub fn create_js_injection_command(&self, js_code: &str) -> CdpCommand {
        let mut params = HashMap::new();
        params.insert(
            "source".to_string(),
            serde_json::Value::String(js_code.to_string()),
        );

        CdpCommand::new("Page.addScriptToEvaluateOnNewDocument", params)
    }

    /// Generate complete stealth setup for a browser session
    pub fn generate_complete_setup(
        &mut self,
        user_agent: &str,
        session_id: Option<&str>,
        js_code: &str,
    ) -> Vec<CdpCommand> {
        let mut commands = self.generate_stealth_commands(user_agent, session_id);
        commands.push(self.create_js_injection_command(js_code));
        commands
    }

    /// Clear internal caches
    pub fn clear_cache(&mut self) {
        self.fingerprint_generator.clear_cache();
    }
}

impl Default for CdpStealthIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

/// CDP command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpCommand {
    pub method: String,
    pub params: HashMap<String, serde_json::Value>,
}

impl CdpCommand {
    pub fn new(method: &str, params: HashMap<String, serde_json::Value>) -> Self {
        Self {
            method: method.to_string(),
            params,
        }
    }

    /// Convert to JSON for sending via CDP
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "method": self.method,
            "params": self.params,
        })
    }
}

/// Result of batch header generation
#[derive(Debug, Clone)]
pub struct BatchHeadersResult {
    pub headers: Vec<HashMap<String, String>>,
    pub fingerprint_id: Option<String>,
    pub consistent: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdp_integrator_creation() {
        let integrator = CdpStealthIntegrator::new();
        assert!(integrator.fingerprint_generator.cache_size() == 0);
    }

    #[test]
    fn test_stealth_commands_generation() {
        let mut integrator = CdpStealthIntegrator::new();
        let commands = integrator.generate_stealth_commands(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            Some("session1"),
        );

        assert_eq!(commands.len(), 3);
        assert!(commands
            .iter()
            .any(|c| c.method == "Page.setUserAgentOverride"));
        assert!(commands
            .iter()
            .any(|c| c.method == "Emulation.setTimezoneOverride"));
        assert!(commands
            .iter()
            .any(|c| c.method == "Emulation.setDeviceMetricsOverride"));
    }

    #[test]
    fn test_batch_headers_generation() {
        let mut integrator = CdpStealthIntegrator::new();
        let result = integrator.generate_batch_headers(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            Some("session1"),
            5,
        );

        assert_eq!(result.headers.len(), 5);
        assert!(result.consistent);
        assert_eq!(result.fingerprint_id, Some("session1".to_string()));
    }

    #[test]
    fn test_header_merging() {
        let mut integrator = CdpStealthIntegrator::new();
        let mut base_headers = HashMap::new();
        base_headers.insert("Custom-Header".to_string(), "custom-value".to_string());

        let merged = integrator.apply_stealth_headers(
            base_headers,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            Some("session1"),
        );

        assert!(merged.contains_key("Custom-Header"));
        assert!(!merged.is_empty());
    }

    #[test]
    fn test_js_injection_command() {
        let integrator = CdpStealthIntegrator::new();
        let js_code = "console.log('stealth');";
        let command = integrator.create_js_injection_command(js_code);

        assert_eq!(command.method, "Page.addScriptToEvaluateOnNewDocument");
        assert!(command.params.contains_key("source"));
    }

    #[test]
    fn test_complete_setup() {
        let mut integrator = CdpStealthIntegrator::new();
        let commands = integrator.generate_complete_setup(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            Some("session1"),
            "console.log('test');",
        );

        assert_eq!(commands.len(), 4); // 3 setup commands + 1 JS injection
    }

    #[test]
    fn test_cdp_command_json() {
        let mut params = HashMap::new();
        params.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );
        let command = CdpCommand::new("Test.method", params);

        let json = command.to_json();
        assert_eq!(json["method"], "Test.method");
        assert!(json["params"].is_object());
    }
}
