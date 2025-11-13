use std::env;

/// Execution mode for CLI operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Try API first, fallback to direct execution if unavailable
    ApiFirst,
    /// API only - error if API is unavailable (no fallback)
    ApiOnly,
    /// Direct execution only (offline/development mode)
    DirectOnly,
}

#[allow(dead_code)]
impl ExecutionMode {
    /// Get execution mode from CLI flags and environment
    pub fn from_flags(direct: bool, api_only: bool) -> Self {
        // CLI flags take precedence
        if direct {
            return ExecutionMode::DirectOnly;
        }

        if api_only {
            return ExecutionMode::ApiOnly;
        }

        // Check environment variable
        if let Ok(mode) = env::var("RIPTIDE_EXECUTION_MODE") {
            match mode.to_lowercase().as_str() {
                "direct" | "offline" => return ExecutionMode::DirectOnly,
                "api-only" | "api_only" => return ExecutionMode::ApiOnly,
                "api-first" | "api_first" => return ExecutionMode::ApiFirst,
                _ => {}
            }
        }

        // Default to API-first with fallback
        ExecutionMode::ApiFirst
    }

    /// Check if mode allows API execution
    pub fn allows_api(&self) -> bool {
        matches!(self, ExecutionMode::ApiFirst | ExecutionMode::ApiOnly)
    }

    /// Check if mode allows direct execution
    pub fn allows_direct(&self) -> bool {
        matches!(self, ExecutionMode::ApiFirst | ExecutionMode::DirectOnly)
    }

    /// Check if fallback is allowed
    pub fn allows_fallback(&self) -> bool {
        matches!(self, ExecutionMode::ApiFirst)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionMode::ApiFirst => "API-first with fallback",
            ExecutionMode::ApiOnly => "API-only (no fallback)",
            ExecutionMode::DirectOnly => "Direct execution (offline)",
        }
    }
}

/// Get execution mode from environment and CLI flags
#[allow(dead_code)] // TODO: Remove once wired into main CLI
pub fn get_execution_mode(direct: bool, api_only: bool) -> ExecutionMode {
    ExecutionMode::from_flags(direct, api_only)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_flag_takes_precedence() {
        let mode = ExecutionMode::from_flags(true, true);
        assert_eq!(mode, ExecutionMode::DirectOnly);
    }

    #[test]
    fn test_api_only_flag() {
        let mode = ExecutionMode::from_flags(false, true);
        assert_eq!(mode, ExecutionMode::ApiOnly);
    }

    #[test]
    fn test_default_mode() {
        let mode = ExecutionMode::from_flags(false, false);
        assert_eq!(mode, ExecutionMode::ApiFirst);
    }

    #[test]
    fn test_mode_permissions() {
        let api_first = ExecutionMode::ApiFirst;
        assert!(api_first.allows_api());
        assert!(api_first.allows_direct());
        assert!(api_first.allows_fallback());

        let api_only = ExecutionMode::ApiOnly;
        assert!(api_only.allows_api());
        assert!(!api_only.allows_direct());
        assert!(!api_only.allows_fallback());

        let direct_only = ExecutionMode::DirectOnly;
        assert!(!direct_only.allows_api());
        assert!(direct_only.allows_direct());
        assert!(!direct_only.allows_fallback());
    }
}
