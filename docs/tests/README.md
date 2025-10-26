# Documentation Validation Scripts

Automated validation scripts to ensure documentation accuracy and quality.

## Overview

This directory contains five validation scripts:

1. **quick-validate.sh** - Fast validation for CI/CD (recommended)
2. **code-examples-validator.sh** - Comprehensive bash/curl code block validation
3. **link-checker.sh** - Comprehensive link and anchor checking
4. **api-endpoint-validator.sh** - API endpoint validation against OpenAPI spec
5. **run-all-validation.sh** - Master script that runs all comprehensive validators

## Quick Start

```bash
# Run quick validation (recommended for CI/CD - fast!)
bash docs/tests/quick-validate.sh

# Run all validators (comprehensive but slower)
bash docs/tests/run-all-validation.sh

# Run individual validators
bash docs/tests/code-examples-validator.sh
bash docs/tests/link-checker.sh
bash docs/tests/api-endpoint-validator.sh
```

### Recommended for CI/CD

Use **quick-validate.sh** for fast validation (< 5 seconds):
- Validates curl command syntax
- Counts markdown links and code blocks
- Checks for dangerous commands
- Validates OpenAPI YAML syntax
- No dependencies required

For comprehensive validation before releases, use **run-all-validation.sh**.

## Script Details

### 0. Quick Validator (Recommended for CI/CD)

**Purpose**: Fast validation for continuous integration pipelines

**Checks**:
- Unquoted JSON in curl commands
- Markdown link count
- Dangerous rm -rf commands (excluding safe paths)
- Code block statistics
- OpenAPI YAML syntax

**Usage**:
```bash
bash docs/tests/quick-validate.sh
```

**Performance**: < 5 seconds on large projects

**Output**:
- Pass/fail status for each check
- Summary with error/warning counts
- Exit code 0 (pass) or 1 (fail)

**When to use**:
- CI/CD pipelines
- Pre-commit hooks
- Quick local validation
- When you need results in seconds

---

### 1. Code Examples Validator

**Purpose**: Extract and validate bash/curl/json code blocks from markdown files

**Checks**:
- Missing quotes in curl commands with JSON
- Unquoted JSON keys
- Dangerous `rm -rf` commands
- Incomplete command chains (trailing `&&` or `|`)
- Balanced braces in JSON blocks

**Usage**:
```bash
bash docs/tests/code-examples-validator.sh
```

**Output**:
- Total code blocks found
- Valid/invalid blocks count
- List of errors with file:line references
- Warnings for potential issues

### 2. Link Checker

**Purpose**: Validate internal markdown links and anchors

**Checks**:
- File existence for internal links
- Anchor existence in target files
- Broken relative paths
- Missing anchor definitions

**Usage**:
```bash
bash docs/tests/link-checker.sh
```

**Output**:
- Total links checked
- Valid/broken links count
- Anchor statistics
- List of broken links with file:line references

### 3. API Endpoint Validator

**Purpose**: Compare documented API endpoints with OpenAPI specification

**Checks**:
- Endpoints mentioned in docs exist in spec
- Correct HTTP methods
- Parameterized paths (e.g., `/users/{id}`)
- Undocumented endpoints in spec

**Usage**:
```bash
bash docs/tests/api-endpoint-validator.sh
```

**Output**:
- Total endpoints in docs and spec
- Matching/invalid endpoints
- List of undocumented endpoints
- List of outdated endpoints with file:line references

### 4. Master Validation Script

**Purpose**: Run all validators and generate summary report

**Features**:
- Runs all three validators in sequence
- Generates comprehensive summary report
- Provides individual timing for each validator
- Returns overall pass/fail status
- Provides fix recommendations

**Usage**:
```bash
bash docs/tests/run-all-validation.sh
```

**Output**:
- Summary table with pass/fail status
- Individual validator results
- Total execution time
- Performance warnings if >60 seconds
- Detailed recommendations for failures

## CI/CD Integration

### Quick Validation (Recommended)

Fast validation for every commit:

```yaml
# GitHub Actions example
- name: Quick Documentation Validation
  run: bash docs/tests/quick-validate.sh
```

### Comprehensive Validation

For release branches or weekly schedules:

```yaml
# GitHub Actions example
- name: Comprehensive Documentation Validation
  run: bash docs/tests/run-all-validation.sh
  if: github.ref == 'refs/heads/main' || github.event_name == 'release'
```

```yaml
# GitLab CI example
validate-docs:
  script:
    - bash docs/tests/run-all-validation.sh
  only:
    changes:
      - docs/**/*
      - "*.md"
```

## Performance

- **Target**: < 60 seconds total execution time
- **Typical**: 5-15 seconds for small projects, 20-40 seconds for large projects
- **Optimization**: Scripts process only docs/, README.md, and sdk/ directories

## Dependencies

**Required**:
- bash 4.0+
- Standard Unix tools (grep, awk, sed, find)

**Optional**:
- `python3` - For JSON validation (falls back to simple brace counting)
- `yq` - For OpenAPI parsing (falls back to Python)

## File Scope

Scripts validate markdown files in:
- `docs/` directory (all .md files)
- `README.md` (project root)
- `sdk/` directory (excluding node_modules)

## Error Codes

- `0` - All validations passed
- `1` - One or more validations failed

## Common Issues and Fixes

### Code Examples Validator

**Error**: `curl -d flag with unquoted JSON`
```bash
# Wrong
curl -X POST http://localhost:8080/api -d {"key": "value"}

# Correct
curl -X POST http://localhost:8080/api -d '{"key": "value"}'
```

**Error**: `JSON with unquoted keys`
```json
# Wrong
{key: "value"}

# Correct
{"key": "value"}
```

### Link Checker

**Error**: `Broken link: ./path/to/file.md`
- Check file exists at relative path
- Verify path casing (case-sensitive on Linux)
- Use relative paths from current file location

**Error**: `Broken anchor: #section-name`
- Anchors are auto-generated from headers
- Use lowercase, replace spaces with dashes
- Remove special characters

### API Endpoint Validator

**Error**: `Endpoint not in spec`
- Update OpenAPI spec to include endpoint
- Or update documentation to match spec
- Check HTTP method matches (GET vs POST)

**Warning**: `Endpoint in spec but not documented`
- Add endpoint to API documentation
- Include examples and parameters
- Document request/response format

## Extending the Validators

To add new validation rules:

1. **Code Examples**: Add checks in `validate_bash()` function
2. **Link Checker**: Add patterns in `validate_markdown_links()` function
3. **API Endpoints**: Add patterns in `extract_doc_endpoints()` function

## Examples

### Successful Run

```
╔════════════════════════════════════════════════╗
║  RipTide Documentation Validation Suite      ║
╚════════════════════════════════════════════════╝

Running comprehensive documentation validation...

[Output from individual validators...]

╔════════════════════════════════════════════════╗
║  Validation Summary Report                    ║
╚════════════════════════════════════════════════╝

Total Validators: 3
Passed: 3
Failed: 0
Total Time: 12s

┌─────────────────────────┬────────┬──────────┐
│ Validator               │ Status │ Duration │
├─────────────────────────┼────────┼──────────┤
│ Code Examples           │ ✓ PASS │      4s │
│ Link Checker            │ ✓ PASS │      5s │
│ API Endpoints           │ ✓ PASS │      3s │
└─────────────────────────┴────────┴──────────┘

✓ Performance: Under 60 second target

╔════════════════════════════════════════════════╗
║  ✓ ALL VALIDATIONS PASSED                     ║
╚════════════════════════════════════════════════╝
```

### Failed Run with Recommendations

```
[Individual validator outputs showing errors...]

╔════════════════════════════════════════════════╗
║  Validation Summary Report                    ║
╚════════════════════════════════════════════════╝

Total Validators: 3
Passed: 1
Failed: 2
Total Time: 15s

Recommendations:
  ✗ Fix issues in: Code Examples
    - Review code blocks in markdown files
    - Ensure proper syntax and quoting
    - Run: bash docs/tests/code-examples-validator.sh

  ✗ Fix issues in: Link Checker
    - Fix broken internal links
    - Update file paths and anchors
    - Run: bash docs/tests/link-checker.sh

╔════════════════════════════════════════════════╗
║  ✗ VALIDATION FAILED                           ║
╚════════════════════════════════════════════════╝
```

## Troubleshooting

**Scripts hang or run slow:**
- Reduce scope by modifying `find` commands
- Check for large binary files inadvertently matching `*.md`
- Exclude additional directories (add to grep filter)

**False positives:**
- Code examples validator: Adjust regex patterns for your use case
- Link checker: Add exceptions for generated anchors
- API validator: Handle custom path parameters

**Dependencies missing:**
- Scripts degrade gracefully without optional dependencies
- Install python3 for better JSON validation
- Install yq for faster OpenAPI parsing

## Contributing

When adding new validation rules:
1. Keep execution time minimal
2. Provide clear error messages with file:line numbers
3. Add warnings for potential issues (not just errors)
4. Test on various markdown formats
5. Document the new rule in this README

## Support

For issues or feature requests:
- Check existing documentation
- Review error messages and recommendations
- Open an issue with validator output and sample markdown

---

**Version**: 1.0.0
**Last Updated**: 2025-10-26
**Maintainer**: RipTide Hive Mind - Validation Specialist
