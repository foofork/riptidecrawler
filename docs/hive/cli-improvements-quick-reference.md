# CLI Improvements Quick Reference Guide

## 🚀 Quick Start

### Running the Comprehensive Test Suite

```bash
# Run all integration tests
cargo test --test cli_comprehensive_test

# Run specific test
cargo test --test cli_comprehensive_test test_cli_extract_basic

# Run full suite (all URLs, all engines, all methods)
cargo test --test cli_comprehensive_test test_comprehensive_suite -- --ignored

# Run crawl tests (long-running, ignored by default)
cargo test --test cli_comprehensive_test test_cli_crawl -- --ignored
```

### Using Progress Indicators

```rust
use crate::commands::progress::{ProgressIndicator, ProgressBar, MultiStepProgress};

// Simple spinner
let mut progress = ProgressIndicator::new("Processing");
progress.start();
// ... do work ...
progress.finish_success("Done!");

// Progress bar with known total
let mut bar = ProgressBar::new(100, "Downloading");
for i in 0..100 {
    bar.inc(1);
    // ... do work ...
}
bar.finish();

// Multi-step process
let mut steps = MultiStepProgress::new(vec![
    "Step 1".to_string(),
    "Step 2".to_string(),
    "Step 3".to_string(),
]);
while let Some(step) = steps.next_step() {
    // ... process step ...
}
steps.finish();
```

## 📁 File Locations

### Implementation Files
- **Progress System**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/progress.rs`
- **Enhanced Executor**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/optimized_executor.rs`
- **Command Definitions**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`

### Test Files
- **Test Framework**: `/workspaces/eventmesh/tests/integration/cli_comprehensive/real_world_tests.rs`
- **Test Module**: `/workspaces/eventmesh/tests/integration/cli_comprehensive/mod.rs`
- **Integration Tests**: `/workspaces/eventmesh/tests/integration/cli_comprehensive_test.rs`
- **Test Config**: `/workspaces/eventmesh/tests/integration/test_urls.json`

### Output & Documentation
- **Test Outputs**: `/workspaces/eventmesh/tests/integration/outputs/`
- **Full Report**: `/workspaces/eventmesh/docs/hive/cli-improvements-implemented.md`
- **This Guide**: `/workspaces/eventmesh/docs/hive/cli-improvements-quick-reference.md`

## 🎯 Key Features

### Progress Indicators
| Type | Use Case | Example |
|------|----------|---------|
| `ProgressIndicator` | Unknown duration tasks | Fetching, processing |
| `ProgressBar` | Known total items | Downloads, iterations |
| `MultiStepProgress` | Multi-phase operations | Build, test, deploy |

### Error Handling Improvements
- ✅ URL validation before processing
- ✅ Clear error messages with solutions
- ✅ Fallback recommendations
- ✅ Non-fatal cache errors

### Test Framework Features
- ✅ Real-world URL testing
- ✅ Automatic output storage
- ✅ Performance metrics
- ✅ Regression detection
- ✅ Session comparison

## 📊 Test Coverage

### Commands Tested
- `extract` - Content extraction with multiple methods
- `search` - Search functionality
- `crawl` - Website crawling

### Engines Tested
- `auto` - Automatic engine selection
- `raw` - Raw HTTP fetch
- `wasm` - WASM-based extraction

### Methods Tested
- `auto` - Automatic method selection
- `article` - Article extraction
- `full` - Full content extraction

### Total Test Variations
**45+ tests** = 3 methods × 3 engines × 5 URLs

## 🔧 Adding New Test URLs

Edit `/workspaces/eventmesh/tests/integration/test_urls.json`:

```json
{
  "id": "unique_test_id",
  "url": "https://example.com/page",
  "category": "news|blog|docs|api|custom",
  "expected": {
    "min_content_length": 500,
    "should_contain": ["keyword1", "keyword2"],
    "should_not_contain": ["error", "404"],
    "max_duration_ms": 10000,
    "expected_success": true
  },
  "notes": "Description of what this test validates"
}
```

## 🐛 Common Issues & Solutions

### Issue: Tests fail with "binary not found"
**Solution**: Build the binary first
```bash
cargo build --bin riptide
```

### Issue: Test outputs not stored
**Solution**: Ensure output directory exists
```bash
mkdir -p tests/integration/outputs
```

### Issue: Progress indicators not showing
**Solution**: Ensure terminal supports ANSI codes
```bash
export TERM=xterm-256color
```

## 📈 Performance Metrics

Test results include:
- **Duration** (ms)
- **Content Length** (bytes)
- **Exit Code**
- **Success/Failure Status**
- **Warnings**
- **Metadata** (JSON structure, line counts)

## 🔄 Session Comparison

Compare two test sessions to detect regressions:

```rust
use cli_comprehensive::CliTestHarness;

let harness = CliTestHarness::new(output_dir, "riptide".to_string());
let session1 = /* load from file */;
let session2 = /* load from file */;

harness.compare_sessions(&session1, &session2)?;
```

Output shows:
- Success/failure changes
- Performance regressions
- Content length differences

## 🎓 Best Practices

### When Writing Tests
1. Use descriptive test IDs
2. Set realistic timeouts
3. Include expected behaviors
4. Add meaningful notes

### When Using Progress Indicators
1. Start immediately before work
2. Update messages for clarity
3. Always finish (success/error/warning)
4. Keep messages concise

### When Handling Errors
1. Provide context
2. Suggest solutions
3. Log at appropriate level
4. Don't panic on recoverable errors

## 🚨 Important Notes

- Tests store ALL outputs for manual inspection
- Session files are JSON for easy parsing
- Progress indicators require ANSI terminal
- Some tests are `#[ignore]` due to duration
- Error messages designed for end users

## 📚 Related Documentation

- Full implementation report: `cli-improvements-implemented.md`
- Existing test harness: `tests/webpage-extraction/cli-test-harness.rs`
- CLI API integration tests: `tests/cli/cli_api_integration.rs`

## ✅ Checklist for New Contributors

- [ ] Read this quick reference
- [ ] Review `cli-improvements-implemented.md`
- [ ] Run basic tests to verify setup
- [ ] Understand progress indicator API
- [ ] Know where test outputs are stored
- [ ] Familiar with test URL configuration

## 🆘 Getting Help

1. Check the full implementation report
2. Review existing test cases
3. Check test output files
4. Review progress indicator tests
5. Coordinate via hive mind memory keys

---

**Quick Reference** | Coder Agent | Hive Mind Swarm
