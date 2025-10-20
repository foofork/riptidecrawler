# RipTide CLI Test Script

## Overview

The `test-cli-commands.sh` script provides comprehensive testing for all RipTide CLI commands with spider-chrome integration.

## Location

```bash
/workspaces/eventmesh/scripts/test-cli-commands.sh
```

## Usage

### Basic Usage

```bash
# Run all tests
./scripts/test-cli-commands.sh

# Or with bash explicitly
bash scripts/test-cli-commands.sh
```

### Test Output

Test results and logs are stored in:
```
/tmp/riptide-cli-tests/
```

Each test creates its own log file with detailed output.

## Test Categories

### 1. **Version & Help Tests**
- Version check
- Main help
- Command-specific help

### 2. **Extract Command Tests**
- Extract from file
- Extract with CSS selector
- Extract with metadata
- Invalid file handling

### 3. **WASM Tests**
- WASM info
- WASM health check
- WASM benchmarks (optional)

### 4. **Cache Tests**
- Cache status
- Cache statistics
- Cache validation

### 5. **Stealth Tests**
- Stealth configuration
- Stealth info
- JavaScript injection generation

### 6. **System Tests**
- System check
- Configuration validation
- Health checks

### 7. **Tables Extraction Tests**
- JSON format extraction
- CSV format extraction
- Markdown format extraction

### 8. **Session & Job Tests**
- Session management
- Local job management

### 9. **Error Handling Tests**
- Invalid commands
- Missing arguments
- Invalid URLs

### 10. **Engine & Method Tests**
- Auto, Raw, WASM engines
- Auto, CSS, WASM methods
- Different extraction strategies

### 11. **Output Format Tests**
- JSON output
- Text output
- Table output

## Exit Codes

- `0` - All tests passed
- `1` - One or more tests failed

## Test Results

The script provides:
- Color-coded output (PASS/FAIL/WARN)
- Individual test logs
- Summary report with counts
- Detailed failure information

## Example Output

```
================================
  RipTide CLI Test Results
================================
Total Tests Run:    50
Tests Passed:       48
Tests Failed:       2
================================
✓ Most tests passed!

Check logs in: /tmp/riptide-cli-tests
```

## Requirements

- Rust toolchain (cargo)
- RipTide project built
- curl (optional, for some tests)

## Features

1. **Comprehensive Coverage**: Tests all major CLI commands
2. **Error Handling**: Tests both success and failure scenarios
3. **Multiple Formats**: Tests JSON, text, and table outputs
4. **Engine Testing**: Tests all extraction engines (auto, raw, wasm, headless)
5. **Detailed Logging**: Individual log files for each test
6. **Color Output**: Easy-to-read test results
7. **Exit Codes**: Proper exit codes for CI/CD integration

## CI/CD Integration

The script can be integrated into CI/CD pipelines:

```bash
# In your CI pipeline
./scripts/test-cli-commands.sh
if [ $? -eq 0 ]; then
    echo "CLI tests passed"
else
    echo "CLI tests failed"
    exit 1
fi
```

## Customization

You can modify the script to:
- Add more test cases
- Change test timeout values
- Modify output directory
- Add custom validation logic

## Troubleshooting

### Build Fails
```bash
# Ensure dependencies are up to date
cargo update
cargo check
```

### Tests Hang
- Check for infinite loops in commands
- Verify timeout values are appropriate
- Check for deadlocks in concurrent operations

### Permission Errors
```bash
# Ensure script is executable
chmod +x scripts/test-cli-commands.sh

# Ensure output directory is writable
mkdir -p /tmp/riptide-cli-tests
chmod 755 /tmp/riptide-cli-tests
```

## Test Development

To add new tests:

1. Create a new test function:
```bash
test_new_feature() {
    log "=== Testing New Feature ==="
    run_test "Feature: Description" \
        "cargo run --bin riptide -- command --args"
}
```

2. Add to main execution:
```bash
main() {
    # ... existing tests
    test_new_feature
    # ...
}
```

## Spider-Chrome Integration

The tests specifically validate spider-chrome integration:
- Browser abstraction layer
- CDP communication
- Headless rendering
- Engine fallback mechanisms

## Success Criteria

A test passes when:
- Exit code matches expected (0 for success, non-zero for errors)
- Output is generated (where applicable)
- No crashes or panics occur
- Resources are cleaned up properly

## Related Files

- `/workspaces/eventmesh/crates/riptide-cli/src/main.rs` - CLI entry point
- `/workspaces/eventmesh/crates/riptide-cli/src/commands/` - Command implementations
- `/workspaces/eventmesh/crates/riptide-browser-abstraction/` - Browser abstraction layer
- `/workspaces/eventmesh/crates/riptide-headless-hybrid/` - Spider-chrome integration

## Support

For issues or questions:
1. Check test logs in `/tmp/riptide-cli-tests/`
2. Review individual command help: `cargo run --bin riptide -- <command> --help`
3. Check project documentation
4. Review recent commits and changes
