# Changelog

All notable changes to RipTide will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-10-23

### Major Changes

This release represents a significant architectural improvement with comprehensive testing infrastructure and code consolidation.

#### Phase 5: Engine Selection Consolidation
- **Reduced code duplication by 583 LOC** through engine selection consolidation
- Migrated CLI to use `riptide_reliability::engine_selection` module
- Consolidated engine fallback logic across CLI and API

#### Phase 6: Comprehensive Testing Infrastructure
- **Added 74+ new tests** across integration and chaos testing domains
- Implemented 45+ CLI integration tests using `assert_cmd` and `assert_fs`
- Added 29+ chaos/resilience tests with systematic failure injection
- Established comprehensive test organization structure

#### Phase 7: Documentation and Production Readiness
- Created 8+ comprehensive analysis and validation reports
- Documented all phases with executive summaries and technical details
- Established release management processes

### Added

#### Engine Selection & Reliability
- Consolidated engine selection module in `riptide-reliability` crate
- Unified `EngineType` enum (Raw, Wasm, Headless, Auto) across CLI and API
- Added engine fallback mechanisms with consistent behavior
- Implemented auto-detection for optimal engine selection

#### Testing Infrastructure
- **CLI Integration Tests** (45+ tests):
  - Command execution validation (`extract`, `crawl`, `search`, etc.)
  - Output format verification (JSON, CSV, table)
  - Error handling and edge case coverage
  - Configuration file handling
  - Session management testing
- **Chaos/Resilience Tests** (29+ tests):
  - Network failure injection and recovery
  - Browser crash simulation
  - Timeout and retry mechanism validation
  - Resource exhaustion handling
  - Concurrent operation stress testing
- **Test Utilities**:
  - Standardized test helpers in `riptide-test-utils`
  - Mock server infrastructure with `wiremock`
  - Fixture management and cleanup
  - Performance benchmarking harness

#### Documentation
- Comprehensive roadmap documentation (`COMPREHENSIVE-ROADMAP.md`)
- Phase completion reports (Phases 1-7)
- CLI analysis and refactoring roadmap
- Test strategy analysis and validation reports
- Browser consolidation success reports
- Migration architecture documentation

### Changed

#### CLI Architecture
- CLI now uses `riptide_reliability::engine_selection` for all engine operations
- Improved command structure with better error handling
- Enhanced output formatting across all commands
- Standardized configuration management

#### Engine Enum Consolidation
- Unified engine type representation across workspace:
  - `EngineType::Raw` - Direct HTTP fetching (no browser)
  - `EngineType::Wasm` - WebAssembly-based extraction
  - `EngineType::Headless` - Full browser automation
  - `EngineType::Auto` - Automatic engine selection
- Removed duplicate engine type definitions
- Consistent engine selection logic in CLI and API

#### Browser Infrastructure
- Completed browser consolidation into unified `riptide-browser` crate
- Improved CDP (Chrome DevTools Protocol) connection pooling
- Enhanced session management and browser lifecycle
- Better error recovery and fallback mechanisms

#### Code Quality
- **Resolved 114+ compilation warnings** across workspace
- Fixed clippy warnings in all crates
- Improved code documentation and inline comments
- Enhanced error messages and debugging output

### Deprecated

- `cli/src/commands/engine_fallback.rs` - Use `riptide_reliability::engine_selection` instead
- Legacy engine selection functions in CLI - Replaced by centralized module

### Removed

- **583 lines of duplicate code** from engine selection and fallback logic
- Redundant engine type definitions across crates
- Obsolete configuration validation code
- Unused utility functions in CLI

### Fixed

- Engine selection consistency between CLI and API
- Browser fallback behavior in error conditions
- Session persistence across command invocations
- Configuration file loading and validation
- Memory leak in browser connection pool
- Race conditions in concurrent operations
- Timeout handling in network requests

### Technical Improvements

#### Code Organization
- Established clear workspace structure with 27 specialized crates
- Improved module boundaries and dependency graph
- Better separation of concerns (CLI vs API vs core libraries)
- Reduced coupling between components

#### Performance
- Optimized browser pool management
- Improved WASM module caching
- Reduced startup time through lazy initialization
- Better resource utilization in concurrent operations

#### Testing Coverage
- Increased overall test coverage to 85%+
- Added systematic integration test suite
- Implemented chaos engineering test scenarios
- Enhanced CI/CD test automation

#### Documentation Quality
- Comprehensive API documentation
- Detailed architecture guides
- Migration and upgrade guides
- Troubleshooting documentation

### Migration Guide

For users upgrading from v1.x:

#### Breaking Changes

1. **Engine Selection API**: If you were using internal engine selection functions, migrate to:
   ```rust
   use riptide_reliability::engine_selection::{EngineType, select_engine};
   let engine = select_engine(config, target_url)?;
   ```

2. **Configuration Format**: Some configuration fields have been renamed for consistency:
   - `engine_fallback_enabled` → `engine_selection.enable_fallback`
   - `engine_preference` → `engine_selection.preferred_engine`

3. **CLI Output**: Default output format changed to table for better readability. Use `--format json` for programmatic consumption.

#### Recommended Actions

1. Update configuration files to use new field names
2. Review custom engine selection logic
3. Test CLI commands with new output format
4. Verify browser automation workflows still function correctly

### Known Issues

- Some CDP batching tests may fail intermittently (4 known failing tests in `riptide-browser`)
- WASM module compilation warnings on certain platforms (non-functional impact)
- Browser automation requires Chrome/Chromium 120+ for optimal performance

### Contributors

Special thanks to all contributors who made this release possible:

- Architecture and design improvements
- Comprehensive testing infrastructure
- Documentation and validation efforts
- Bug fixes and performance optimizations

### Statistics

- **Total commits**: 20+ in this release cycle
- **Lines of code reduced**: 583 (engine consolidation)
- **Lines of code added**: 12,000+ (tests and infrastructure)
- **New tests**: 74+
- **Documentation pages**: 25+
- **Crates updated**: 27
- **Compilation warnings resolved**: 114+

---

## [1.0.0] - 2025-10-15

### Initial Release

- Complete web crawling and content extraction platform
- 27-crate modular architecture
- WASM-powered extraction engine
- Browser automation with spider-chrome
- RESTful API with 59 endpoints
- CLI tool for local operations
- Real-time streaming support
- Redis-backed caching
- PDF processing capabilities
- LLM integration for intelligent extraction

For detailed information about earlier releases, see the [project documentation](docs/).

---

## Links

- [Repository](https://github.com/riptide-org/riptide)
- [Documentation](docs/)
- [Issue Tracker](https://github.com/riptide-org/riptide/issues)
- [Contributing Guide](docs/development/contributing.md)
