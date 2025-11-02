// Spec Validator
//
// Validates CLI specifications against requirements

use crate::{CliSpec, Result, SpecError};

pub struct SpecValidator;

impl SpecValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate a CLI spec
    pub fn validate(&self, spec: &CliSpec) -> Result<()> {
        self.validate_metadata(spec)?;
        self.validate_commands(spec)?;
        self.validate_exit_codes(spec)?;
        self.validate_error_mapping(spec)?;
        Ok(())
    }

    fn validate_metadata(&self, spec: &CliSpec) -> Result<()> {
        if spec.version.is_empty() {
            return Err(SpecError::MissingField("version".to_string()));
        }
        if spec.name.is_empty() {
            return Err(SpecError::MissingField("name".to_string()));
        }
        if spec.about.is_empty() {
            return Err(SpecError::MissingField("about".to_string()));
        }
        Ok(())
    }

    fn validate_commands(&self, spec: &CliSpec) -> Result<()> {
        if spec.commands.is_empty() {
            return Err(SpecError::ValidationError(
                "Spec must define at least one command".to_string(),
            ));
        }

        for cmd in &spec.commands {
            if cmd.name.is_empty() {
                return Err(SpecError::ValidationError(
                    "Command name cannot be empty".to_string(),
                ));
            }
            if cmd.about.is_empty() {
                return Err(SpecError::ValidationError(format!(
                    "Command '{}' must have description",
                    cmd.name
                )));
            }
            if cmd.examples.is_empty() {
                return Err(SpecError::ValidationError(format!(
                    "Command '{}' must have at least one example",
                    cmd.name
                )));
            }
        }

        Ok(())
    }

    fn validate_exit_codes(&self, spec: &CliSpec) -> Result<()> {
        if spec.exit_codes.success != 0 {
            return Err(SpecError::ValidationError(
                "Success exit code must be 0".to_string(),
            ));
        }
        if spec.exit_codes.user_error <= 0 {
            return Err(SpecError::ValidationError(
                "User error exit code must be positive".to_string(),
            ));
        }
        if spec.exit_codes.server_error <= 0 {
            return Err(SpecError::ValidationError(
                "Server error exit code must be positive".to_string(),
            ));
        }
        if spec.exit_codes.invalid_args <= 0 {
            return Err(SpecError::ValidationError(
                "Invalid args exit code must be positive".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_error_mapping(&self, spec: &CliSpec) -> Result<()> {
        // Validate 4xx errors map to user_error
        let client_errors = ["400", "401", "403", "404", "429"];
        for status in &client_errors {
            if let Some(&code) = spec.error_mapping.get(*status) {
                if code != spec.exit_codes.user_error {
                    return Err(SpecError::ValidationError(format!(
                        "HTTP {} should map to user error ({}), got {}",
                        status, spec.exit_codes.user_error, code
                    )));
                }
            }
        }

        // Validate 5xx errors map to server_error
        let server_errors = ["500", "502", "503", "504"];
        for status in &server_errors {
            if let Some(&code) = spec.error_mapping.get(*status) {
                if code != spec.exit_codes.server_error {
                    return Err(SpecError::ValidationError(format!(
                        "HTTP {} should map to server error ({}), got {}",
                        status, spec.exit_codes.server_error, code
                    )));
                }
            }
        }

        Ok(())
    }
}

impl Default for SpecValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Example, ExitCodes};

    fn minimal_valid_spec() -> CliSpec {
        CliSpec {
            version: "1.0.0".to_string(),
            name: "test".to_string(),
            about: "Test CLI".to_string(),
            config: Default::default(),
            global_flags: vec![],
            exit_codes: ExitCodes {
                success: 0,
                user_error: 1,
                server_error: 2,
                invalid_args: 3,
            },
            commands: vec![Command {
                name: "test".to_string(),
                about: "Test command".to_string(),
                api: Default::default(),
                args: vec![],
                flags: vec![],
                examples: vec![Example {
                    command: "test".to_string(),
                    description: "Test example".to_string(),
                }],
                logic: None,
            }],
            error_mapping: Default::default(),
            output_formats: None,
        }
    }

    #[test]
    fn test_valid_spec() {
        let validator = SpecValidator::new();
        let spec = minimal_valid_spec();
        assert!(validator.validate(&spec).is_ok());
    }

    #[test]
    fn test_missing_version() {
        let validator = SpecValidator::new();
        let mut spec = minimal_valid_spec();
        spec.version = String::new();
        assert!(validator.validate(&spec).is_err());
    }

    #[test]
    fn test_invalid_exit_codes() {
        let validator = SpecValidator::new();
        let mut spec = minimal_valid_spec();
        spec.exit_codes.success = 1;
        assert!(validator.validate(&spec).is_err());
    }

    #[test]
    fn test_missing_command_example() {
        let validator = SpecValidator::new();
        let mut spec = minimal_valid_spec();
        spec.commands[0].examples.clear();
        assert!(validator.validate(&spec).is_err());
    }
}
