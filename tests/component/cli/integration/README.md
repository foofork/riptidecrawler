# CLI Integration Tests

**Phase 6 Task 6.1: Comprehensive CLI Testing with assert_cmd and assert_fs**

## Quick Stats
- **Total Tests**: 71
- **Test Categories**: 7
- **Lines of Code**: 1,113
- **Target Execution Time**: <30 seconds

## Test Modules

### 1. `basic_commands.rs` (20 tests)
Core CLI functionality testing:
- Help and version information
- Extract command (file, stdin, URL)
- Output formats (text, JSON, table)
- Metadata and confidence scoring
- Health and validation commands

### 2. `cache_commands.rs` (9 tests)
Cache management operations:
- Status and statistics
- Clear and validate
- Cache warming from URL files
- Domain-specific operations

### 3. `wasm_commands.rs` (8 tests)
WASM runtime testing:
- Runtime information and health
- Performance benchmarking
- Path configuration
- Timeout handling

### 4. `error_handling.rs` (15 tests)
Error scenarios and validation:
- Invalid commands and arguments
- Missing files and URLs
- Malformed HTML handling
- API mode errors
- Input validation

### 5. `filesystem_scenarios.rs` (9 tests)
File operations with assert_fs:
- Output file creation
- Directory handling
- CSV and JSON exports
- File overwriting
- Temporary file management

### 6. `session_commands.rs` (4 tests)
Session management:
- Session creation and listing
- Session-based extraction

### 7. `job_commands.rs` (6 tests)
Job operations:
- Local and API-based jobs
- Job listing and status
- JSON output formats

## Running Tests

### All CLI Integration Tests
```bash
cargo test --test '*' component::cli::integration
```

### Specific Categories
```bash
cargo test --test '*' component::cli::integration::basic_commands
cargo test --test '*' component::cli::integration::cache_commands
cargo test --test '*' component::cli::integration::error_handling
```

### Quick Validation (3 tests in ~5s)
```bash
cargo test --test '*' component::cli::integration::basic_commands::test_cli_help
cargo test --test '*' component::cli::integration::basic_commands::test_extract_from_file
cargo test --test '*' component::cli::integration::error_handling::test_extract_missing_file
```

## Test Design

### Key Features
1. **Fast Execution**: 2-10 second timeouts per test
2. **Isolation**: Each test uses temporary directories
3. **Realistic Data**: Well-formed HTML and real scenarios
4. **CI/CD Ready**: No external dependencies required
5. **Comprehensive Coverage**: Success and error paths

### Dependencies
- `assert_cmd` - CLI command testing framework
- `assert_fs` - Filesystem assertions and temp directories
- `predicates` - Flexible assertion predicates

## Example Tests

### Basic Command Test
```rust
#[test]
fn test_extract_from_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("test.html");
    input_file.write_str("<html><body><h1>Test</h1></body></html>").unwrap();

    Command::cargo_bin("riptide").unwrap()
        .arg("extract")
        .arg("--input-file").arg(input_file.path())
        .arg("--local")
        .timeout(Duration::from_secs(5))
        .assert()
        .success();
}
```

### Filesystem Test
```rust
#[test]
fn test_extract_output_to_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let input = temp.child("input.html");
    let output = temp.child("output.txt");

    input.write_str("<html><body><p>Content</p></body></html>").unwrap();

    Command::cargo_bin("riptide").unwrap()
        .arg("extract")
        .arg("--input-file").arg(input.path())
        .arg("--file").arg(output.path())
        .arg("--local")
        .assert()
        .success();

    output.assert(predicate::path::exists());
}
```

### Error Handling Test
```rust
#[test]
fn test_extract_missing_file() {
    Command::cargo_bin("riptide").unwrap()
        .arg("extract")
        .arg("--input-file").arg("/nonexistent/file.html")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
```

## Coverage Map

| CLI Command | Tested | Error Handling | Filesystem |
|------------|--------|----------------|------------|
| extract | ✅ | ✅ | ✅ |
| render | ✅ | - | - |
| tables | ✅ | ✅ | ✅ |
| cache | ✅ | ✅ | ✅ |
| wasm | ✅ | ✅ | - |
| health | ✅ | ✅ | - |
| validate | ✅ | - | - |
| session | ✅ | - | - |
| job-local | ✅ | - | ✅ |

## Success Criteria ✅

- [x] 10+ tests implemented (71 total)
- [x] Multiple CLI commands covered (9 commands)
- [x] Error handling scenarios (15 tests)
- [x] Filesystem operations (9 tests)
- [x] Fast execution (<30s target)
- [x] CI/CD ready
- [x] Real-world scenarios
- [x] Well-documented

## See Also

- [Full Phase 6 Documentation](../../docs/PHASE6_CLI_TESTING.md)
- [Testing Guide](../../docs/TESTING_GUIDE.md)
- [CLI Source Code](../../../crates/riptide-cli/)
