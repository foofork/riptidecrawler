use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckStatus {
    Pass,
    Fail,
    Warning,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub remediation: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl CheckResult {
    pub fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Pass,
            message: message.into(),
            remediation: None,
            details: None,
        }
    }

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

    pub fn warning(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warning,
            message: message.into(),
            remediation: None,
            details: None,
        }
    }

    pub fn skipped(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Skipped,
            message: message.into(),
            remediation: None,
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub timestamp: String,
    pub checks: Vec<CheckResult>,
    pub summary: ValidationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub overall_status: CheckStatus,
}

impl ValidationReport {
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

    pub fn exit_code(&self) -> i32 {
        match self.summary.overall_status {
            CheckStatus::Pass => 0,
            CheckStatus::Warning => 0,
            CheckStatus::Fail => 1,
            CheckStatus::Skipped => 0,
        }
    }
}
