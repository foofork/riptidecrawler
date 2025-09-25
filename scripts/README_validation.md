# PDF Pipeline Validation Scripts

This directory contains comprehensive validation scripts to verify the PDF processing pipeline integration is complete and working correctly.

## Available Scripts

### 1. `validate_pdf_simple.sh` ⚡ (Quick Check)

**Purpose**: Basic structural validation of PDF pipeline components.

**Usage**:
```bash
./scripts/validate_pdf_simple.sh
```

**Checks**:
- ✅ PDF module files exist
- ✅ API integration (routes, handlers)
- ✅ Worker integration
- ✅ Test files exist
- ❌ Code quality (unwrap calls)

**Runtime**: ~5 seconds

### 2. `validate_pdf_pipeline.sh` 🔍 (Comprehensive)

**Purpose**: Complete validation of PDF pipeline integration with compilation, testing, and performance checks.

**Usage**:
```bash
./scripts/validate_pdf_pipeline.sh
```

**Comprehensive Checks**:

#### 1. Compilation Verification
- riptide-core builds without errors
- riptide-api builds without errors
- riptide-workers builds without errors

#### 2. PDF Module Structure
- All required PDF module files exist
- Module hierarchy is correct

#### 3. API Endpoint Registration
- PDF routes are registered in main.rs
- PDF handlers are implemented
- Route modules are properly declared

#### 4. Worker Service Integration
- PDF processor is registered in workers
- Job types include PDF processing
- Worker service can handle PDF jobs

#### 5. Memory Management Validation
- Memory configuration is present
- Memory hooks are implemented
- Memory benchmarks are available

#### 6. Progress Tracking Integration
- Progress types are defined
- Progress tracking is implemented in processor
- Progress callbacks are functional

#### 7. Test Suite Validation
- Unit tests exist and pass
- Integration tests exist and pass
- Test coverage is adequate (5+ tests)

#### 8. Code Quality Checks
- No unwrap() calls in production code
- No panic! calls in PDF code
- Proper error handling patterns used

#### 9. Metrics Collection
- Metrics module exists
- Metrics are integrated in processor
- Telemetry is configured

#### 10. Performance Guardrails
- Memory limits are configured
- Timeout configurations exist
- Concurrency limits are set

#### 11. Integration Smoke Test
- Full workspace builds successfully
- No clippy warnings

#### 12. Configuration Validation
- All required config files exist
- PDF feature flags are configured

**Runtime**: ~3-5 minutes

### 3. `validate_pdf_pipeline_quick.sh` 🚀 (Balanced)

**Purpose**: Mid-level validation focusing on core integration points.

**Usage**:
```bash
./scripts/validate_pdf_pipeline_quick.sh
```

**Checks**:
- File structure validation
- Basic compilation check
- API and worker integration
- Error handling patterns
- Test existence

**Runtime**: ~30 seconds

## Validation Results

### Current Status (as of latest run):

```
==============================================
           VALIDATION SUMMARY
==============================================

Total Checks: 13
Passed:       12
Failed:       1

❌ 1 CHECK(S) FAILED!
```

### Known Issues:

1. **unwrap() calls found in 4 PDF files**:
   - `/pdf/benchmarks.rs` - Benchmark code (acceptable)
   - `/pdf/utils_corrupted.rs` - Test utility (acceptable)
   - `/pdf/utils.rs` - Utility functions (needs review)
   - `/pdf/tests.rs` - Test code (acceptable)

   **Action Required**: Review unwrap() calls in `utils.rs` and replace with proper error handling.

## Output Format

All scripts provide color-coded output:
- 🔵 **Blue**: Information messages
- ✅ **Green**: Successful checks
- ❌ **Red**: Failed checks
- ⚠️ **Yellow**: Warnings

## Integration with CI/CD

### GitHub Actions Integration

```yaml
- name: Validate PDF Pipeline
  run: |
    chmod +x ./scripts/validate_pdf_pipeline.sh
    ./scripts/validate_pdf_pipeline.sh
```

### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
./scripts/validate_pdf_simple.sh
```

## Troubleshooting

### Common Issues:

1. **Compilation Failures**:
   ```bash
   # Check specific crate
   cargo check -p riptide-core
   cargo check -p riptide-api
   cargo check -p riptide-workers
   ```

2. **Missing Files**:
   ```bash
   # Verify file structure
   find crates/riptide-core/src/pdf -name "*.rs" | sort
   ```

3. **Test Failures**:
   ```bash
   # Run specific tests
   cargo test -p riptide-core pdf --lib
   ```

### Performance Issues:

If validation scripts take too long:
1. Use `validate_pdf_simple.sh` for quick checks
2. Check for hanging compilation processes
3. Verify disk space and memory availability

## Advanced Usage

### Custom Validation Patterns

```bash
# Check specific component
grep -r "PdfProcessor" crates/riptide-workers/

# Verify API endpoints
grep -r "pdf_routes" crates/riptide-api/

# Check error handling
find crates/riptide-core/src/pdf -name "*.rs" -exec grep -l "Result\|Error" {} +
```

### Integration with Development Workflow

1. **Before Committing**: Run `validate_pdf_simple.sh`
2. **Before PR**: Run `validate_pdf_pipeline_quick.sh`
3. **Before Release**: Run `validate_pdf_pipeline.sh`

## Metrics and Reporting

The comprehensive validation script tracks:
- Success rate percentage
- Execution time
- Memory usage during validation
- Detailed failure locations

## Contributing

When adding new PDF components:
1. Update the validation scripts to include new checks
2. Add file existence checks for new modules
3. Include integration verification steps
4. Update this README with new validation points

## Future Enhancements

- [ ] Performance benchmarking integration
- [ ] Memory leak detection
- [ ] Load testing validation
- [ ] Security vulnerability scanning
- [ ] Automated fix suggestions
- [ ] Integration with monitoring systems

## Support

For issues with validation scripts:
1. Check the script output for specific error messages
2. Verify file paths and permissions
3. Ensure all dependencies are installed
4. Review the troubleshooting section above

---

**Last Updated**: September 25, 2025
**Version**: 1.0.0
**Maintainer**: PDF Pipeline Team