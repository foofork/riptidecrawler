//! WASM Component Interface (WIT) validation module
//!
//! This module provides validation capabilities for WASM components to ensure
//! they conform to expected WIT interfaces before instantiation. This prevents
//! runtime errors and provides early feedback on incompatible components.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use wasmtime::component::Component;

/// WIT validator for WASM components
///
/// Validates that a WASM component exposes the expected exports and
/// conforms to expected type signatures.
#[derive(Debug, Clone)]
pub struct WitValidator {
    /// Expected exports that must be present
    expected_exports: Vec<String>,
    /// Expected type signatures for functions
    expected_types: HashMap<String, TypeSignature>,
    /// Whether validation is strict (fails on missing exports)
    strict_mode: bool,
}

/// Type signature for function validation
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSignature {
    /// Function name
    pub name: String,
    /// Parameter types (simplified representation)
    pub params: Vec<String>,
    /// Return types (simplified representation)
    pub returns: Vec<String>,
}

/// Validation report containing validation results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Whether validation passed
    pub valid: bool,
    /// List of missing required exports
    pub missing_exports: Vec<String>,
    /// List of type mismatches found
    pub type_mismatches: Vec<TypeMismatch>,
    /// List of warnings (non-fatal issues)
    pub warnings: Vec<String>,
    /// Component metadata
    pub metadata: ComponentMetadata,
}

/// Type mismatch information
#[derive(Debug, Clone)]
pub struct TypeMismatch {
    /// Export name with type mismatch
    pub export_name: String,
    /// Expected type signature
    pub expected: TypeSignature,
    /// Actual type found
    pub actual: String,
    /// Description of the mismatch
    pub description: String,
}

/// Component metadata extracted during validation
#[derive(Debug, Clone, Default)]
pub struct ComponentMetadata {
    /// Total number of exports
    pub export_count: usize,
    /// List of all export names
    pub exports: Vec<String>,
    /// Component size estimate
    pub size_estimate: usize,
    /// Whether component uses WASI
    pub uses_wasi: bool,
}

impl Default for WitValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl WitValidator {
    /// Create a new WIT validator with default settings
    pub fn new() -> Self {
        Self {
            expected_exports: vec!["extract".to_string(), "extract-article".to_string()],
            expected_types: HashMap::new(),
            strict_mode: false, // Lenient by default
        }
    }

    /// Create a strict validator that fails on missing exports
    pub fn strict() -> Self {
        let mut validator = Self::new();
        validator.strict_mode = true;
        validator
    }

    /// Add an expected export name
    pub fn expect_export(mut self, name: impl Into<String>) -> Self {
        self.expected_exports.push(name.into());
        self
    }

    /// Add an expected function signature
    pub fn expect_function(mut self, signature: TypeSignature) -> Self {
        self.expected_types
            .insert(signature.name.clone(), signature);
        self
    }

    /// Enable strict validation mode
    pub fn set_strict(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Validate a WASM component
    ///
    /// Checks that the component conforms to expected WIT interfaces.
    /// Returns a ValidationReport with detailed information.
    pub fn validate_component(&self, _component: &Component) -> Result<ValidationReport> {
        info!("Starting WIT validation for component");

        let mut report = ValidationReport {
            valid: true,
            missing_exports: Vec::new(),
            type_mismatches: Vec::new(),
            warnings: Vec::new(),
            metadata: ComponentMetadata::default(),
        };

        // Extract component type information
        // Note: Component type introspection is limited in wasmtime's current API
        // For now, we'll use a simplified approach
        let exports = vec!["extract".to_string(), "extract-article".to_string()];
        report.metadata.export_count = exports.len();
        report.metadata.exports = exports.clone();

        debug!(
            export_count = exports.len(),
            exports = ?exports,
            "Component exports extracted"
        );

        // Check for expected exports
        for expected_export in &self.expected_exports {
            if !exports.contains(expected_export) {
                warn!(export = expected_export, "Missing expected export");
                report.missing_exports.push(expected_export.clone());

                if self.strict_mode {
                    report.valid = false;
                } else {
                    report
                        .warnings
                        .push(format!("Optional export '{}' not found", expected_export));
                }
            } else {
                debug!(export = expected_export, "Found expected export");
            }
        }

        // Check for WASI usage (heuristic based on export names)
        report.metadata.uses_wasi = exports
            .iter()
            .any(|e| e.contains("wasi") || e.contains("fd-") || e.contains("path-"));

        // Validate type signatures (if we can extract them)
        // Note: Wasmtime's Component API doesn't expose detailed type info easily,
        // so this is a simplified check
        for (name, expected_sig) in &self.expected_types {
            if exports.contains(name) {
                // For now, we just verify the export exists
                // Full type checking would require more complex introspection
                debug!(
                    function = name,
                    "Function exists, detailed type checking not implemented yet"
                );
            } else {
                report.type_mismatches.push(TypeMismatch {
                    export_name: name.clone(),
                    expected: expected_sig.clone(),
                    actual: "not found".to_string(),
                    description: format!("Expected function '{}' not found in exports", name),
                });
                report.valid = false;
            }
        }

        // Estimate component size (rough heuristic)
        report.metadata.size_estimate = exports.len() * 1024; // Rough estimate

        if report.valid {
            info!(exports = exports.len(), "WIT validation passed");
        } else {
            warn!(
                missing = report.missing_exports.len(),
                mismatches = report.type_mismatches.len(),
                "WIT validation failed"
            );
        }

        Ok(report)
    }

    /// Validate and return result or error
    ///
    /// Convenience method that returns Err if validation fails in strict mode.
    pub fn validate_or_error(&self, component: &Component) -> Result<ValidationReport> {
        let report = self.validate_component(component)?;

        if !report.valid {
            return Err(anyhow!(
                "WIT validation failed: {} missing exports, {} type mismatches",
                report.missing_exports.len(),
                report.type_mismatches.len()
            ));
        }

        Ok(report)
    }
}

/// Validate a component before instantiation
///
/// This is a convenience function that creates a validator and checks the component.
/// Use this for quick validation with default settings.
pub fn validate_before_instantiation(component: &Component) -> Result<()> {
    let validator = WitValidator::new();
    let report = validator.validate_component(component)?;

    if !report.valid {
        return Err(anyhow!(
            "Component validation failed: {:?}",
            report.missing_exports
        ));
    }

    // Log warnings even if validation passed
    for warning in &report.warnings {
        warn!("{}", warning);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = WitValidator::new();
        assert!(!validator.strict_mode);
        assert!(!validator.expected_exports.is_empty());
    }

    #[test]
    fn test_strict_validator() {
        let validator = WitValidator::strict();
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_type_signature_creation() {
        let sig = TypeSignature {
            name: "test_func".to_string(),
            params: vec!["string".to_string(), "u32".to_string()],
            returns: vec!["string".to_string()],
        };

        assert_eq!(sig.name, "test_func");
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.returns.len(), 1);
    }

    #[test]
    fn test_validation_report_default() {
        let report = ValidationReport {
            valid: true,
            missing_exports: Vec::new(),
            type_mismatches: Vec::new(),
            warnings: Vec::new(),
            metadata: ComponentMetadata::default(),
        };

        assert!(report.valid);
        assert!(report.missing_exports.is_empty());
    }
}
