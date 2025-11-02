//! CLI Specification Parser
//!
//! This library provides parsing and validation for YAML-based CLI specifications.
//! It reads cli.yaml files and generates structures that can be used to build clap
//! command definitions dynamically.

pub mod parser;

pub use parser::{
    ApiEndpoint, ApiMapping, Arg, CliSpec, Command, ErrorMapping, ExitCodes, Flag, HttpMethod,
    ParserError, SpecParser,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Ensure all main types are exported
        let _: Option<CliSpec> = None;
        let _: Option<Command> = None;
        let _: Option<Flag> = None;
        let _: Option<Arg> = None;
    }
}
