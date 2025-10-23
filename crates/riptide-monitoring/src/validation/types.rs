//! Validation result types and structures

use serde::{Deserialize, Serialize};

/// Status of a validation check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckStatus {
    /// Check passed successfully
    Pass,
    /// Check failed
    Fail,
    /// Check passed with warnings
    Warning,
    /// Check was skipped
    Skipped,
}

/// Result of a single validation check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the check
    pub name: String,
    /// Status of the check
    pub status: CheckStatus,
    /// Human-readable message
    pub message: String,
    /// Optional remediation steps
    pub remediation: Option<String>,
    /// Optional additional details
    pub details: Option<serde_json::Value>,
}

impl CheckResult {
    /// Create a passing check result
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Pass,
            message: message.into(),
            remediation: None,
            details: None,
        }
    }

    /// Create a failing check result with remediation steps
    pub fn fail(
        name: impl Into<String>,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Fail,
            message: message.into(),
            remediation: Some(remediation.into()),
            details: None,
        }
    }

    /// Create a warning check result
    pub fn warning(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warning,
            message: message.into(),
            remediation: None,
            details: None,
        }
    }

    /// Create a skipped check result
    pub fn skipped(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Skipped,
            message: message.into(),
            remediation: None,
            details: None,
        }
    }

    /// Add additional details to the check result
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Summary of validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of checks performed
    pub total_checks: usize,
    /// Number of checks that passed
    pub passed: usize,
    /// Number of checks that failed
    pub failed: usize,
    /// Number of checks with warnings
    pub warnings: usize,
    /// Number of checks that were skipped
    pub skipped: usize,
    /// Overall validation status
    pub overall_status: CheckStatus,
}

/// Complete validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Timestamp of the validation
    pub timestamp: String,
    /// Individual check results
    pub checks: Vec<CheckResult>,
    /// Summary of results
    pub summary: ValidationSummary,
}

impl ValidationReport {
    /// Create a new validation report from check results
    pub fn new(checks: Vec<CheckResult>) -> Self {
        let total_checks = checks.len();
        let passed = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Pass))
            .count();
        let failed = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Fail))
            .count();
        let warnings = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Warning))
            .count();
        let skipped = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Skipped))
            .count();

        let overall_status = if failed > 0 {
            CheckStatus::Fail
        } else if warnings > 0 {
            CheckStatus::Warning
        } else {
            CheckStatus::Pass
        };

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            checks,
            summary: ValidationSummary {
                total_checks,
                passed,
                failed,
                warnings,
                skipped,
                overall_status,
            },
        }
    }

    /// Get exit code based on validation status
    pub fn exit_code(&self) -> i32 {
        match self.summary.overall_status {
            CheckStatus::Pass => 0,
            CheckStatus::Warning => 0,
            CheckStatus::Fail => 1,
            CheckStatus::Skipped => 0,
        }
    }
}
