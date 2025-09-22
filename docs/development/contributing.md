# Contributing to RipTide Crawler

Thank you for your interest in contributing to RipTide! This guide will help you get started with contributing code, documentation, and bug reports.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please read and follow our Code of Conduct.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Set up the development environment** following the [Getting Started Guide](getting-started.md)
4. **Create a feature branch** from `main`
5. **Make your changes**
6. **Submit a pull request**

## Development Process

### Branch Naming

Use descriptive branch names with prefixes:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test improvements

Examples:
```
feature/add-pdf-support
fix/memory-leak-in-extractor
docs/update-api-guide
refactor/simplify-gate-logic
```

### Commit Messages

Follow the conventional commit format:

```
type(scope): short description

Longer description if needed.

Fixes #123
```

Types:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `style` - Formatting changes
- `refactor` - Code refactoring
- `test` - Adding tests
- `chore` - Maintenance tasks

Examples:
```
feat(core): add PDF content extraction support
fix(api): resolve memory leak in concurrent requests
docs(readme): update installation instructions
```

## Code Standards

### Rust Guidelines

Follow the [Coding Standards](coding-standards.md) document and these principles:

#### Code Style
- Use `cargo fmt` for consistent formatting
- Follow Rust naming conventions (snake_case, PascalCase)
- Prefer explicit types when unclear
- Use meaningful variable and function names

#### Error Handling
```rust
// ✅ Good: Use anyhow for application errors
use anyhow::{Context, Result};

pub fn fetch_url(url: &str) -> Result<String> {
    reqwest::get(url)
        .context("Failed to send request")?
        .text()
        .context("Failed to read response body")
}

// ❌ Avoid: Unwrapping in library code
let response = reqwest::get(url).unwrap();
```

#### Documentation
```rust
/// Extracts content from HTML using WebAssembly.
///
/// # Arguments
///
/// * `html` - Raw HTML content as bytes
/// * `url` - Source URL for context
/// * `mode` - Extraction mode ("article", "full", etc.)
///
/// # Returns
///
/// Extracted document with title, content, and metadata
///
/// # Errors
///
/// Returns error if WASM execution fails or JSON parsing fails
pub fn extract_content(html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
    // Implementation
}
```

### Testing Requirements

#### Unit Tests
- Test all public functions
- Cover error cases
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_decision_with_high_quality_content() {
        let features = GateFeatures {
            html_bytes: 5000,
            visible_text_chars: 2500,
            p_count: 10,
            article_count: 1,
            has_og: true,
            // ... other fields
        };

        assert_eq!(decide(&features, 0.7, 0.3), Decision::Raw);
    }

    #[test]
    fn test_gate_decision_with_spa_content() {
        let features = GateFeatures {
            spa_markers: 3,
            script_bytes: 8000,
            html_bytes: 10000,
            // ... other fields
        };

        assert_eq!(decide(&features, 0.7, 0.3), Decision::Headless);
    }
}
```

#### Integration Tests
- Test API endpoints end-to-end
- Test Docker containers
- Test configuration loading

### Performance Guidelines

#### Memory Usage
- Use streaming where possible
- Avoid cloning large data structures
- Pool expensive resources (HTTP clients, WASM instances)

```rust
// ✅ Good: Stream processing
use tokio_stream::StreamExt;

pub async fn process_urls(urls: impl Stream<Item = String>) -> Result<Vec<ExtractedDoc>> {
    let mut results = Vec::new();

    tokio::pin!(urls);
    while let Some(url) = urls.next().await {
        results.push(process_single_url(&url).await?);
    }

    Ok(results)
}

// ❌ Avoid: Loading everything into memory
pub async fn process_urls(urls: Vec<String>) -> Result<Vec<ExtractedDoc>> {
    let mut results = Vec::with_capacity(urls.len());
    // ... process all at once
}
```

#### Async Best Practices
- Use `tokio::spawn` for CPU-bound tasks
- Limit concurrency with semaphores
- Use timeouts for external requests

## Pull Request Process

### Before Submitting

1. **Run all checks locally**:
   ```bash
   just fmt
   just lint
   just test
   cargo deny check
   ```

2. **Update documentation** if needed
3. **Add tests** for new functionality
4. **Update CHANGELOG.md** if applicable

### PR Description Template

```markdown
## Summary
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)

## Related Issues
Fixes #123
Related to #456
```

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Testing** on multiple environments
4. **Approval** and merge

## Types of Contributions

### Bug Reports

Use the GitHub issue template and include:
- **Environment**: OS, Rust version, Docker version
- **Steps to reproduce**
- **Expected vs actual behavior**
- **Error messages** and logs
- **Minimal reproduction case**

### Feature Requests

- **Describe the problem** you're solving
- **Explain the proposed solution**
- **Consider alternatives**
- **Show examples** of usage
- **Discuss breaking changes**

### Documentation

- **Fix typos** and improve clarity
- **Add examples** and use cases
- **Update for new features**
- **Translate** documentation
- **Improve code comments**

### Performance Improvements

- **Benchmark before and after**
- **Profile memory usage**
- **Test under load**
- **Document trade-offs**

## Coding Areas

### High Priority
- **WASM optimization**: Improve extraction speed
- **Cache efficiency**: Better Redis utilization
- **Error handling**: More descriptive errors
- **Configuration**: Dynamic reloading

### Medium Priority
- **New extractors**: Support more content types
- **Monitoring**: Better observability
- **Testing**: Increase coverage
- **Documentation**: More examples

### Future
- **Distributed crawling**: Multi-node support
- **ML integration**: Smart content classification
- **Plugin system**: Custom extractors
- **Web UI**: Management interface

## Development Environment

### Required Tools

```bash
# Core tools
rustup toolchain install stable
rustup target add wasm32-wasi wasm32-wasip2
cargo install just cargo-deny cargo-audit

# Optional but recommended
cargo install cargo-watch    # Auto-rebuild on changes
cargo install cargo-expand   # Macro expansion
cargo install tokio-console  # Async debugging
```

### Editor Configuration

#### VS Code Settings
```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "files.watcherExclude": {
        "**/target/**": true
    }
}
```

### Git Hooks

Install pre-commit hooks:
```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

## Release Process

### Version Bumping
1. Update version in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.2.0`
4. Push: `git push origin v0.2.0`

### Release Notes
- Summarize new features
- List breaking changes
- Document migration steps
- Include performance improvements

## Community

### Communication Channels
- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and ideas
- **Pull Requests**: Code review and collaboration

### Recognition
Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- GitHub contributors page

## Security

### Reporting Vulnerabilities
- **Do not** open public issues for security bugs
- **Email** maintainers privately
- **Include** reproduction steps
- **Allow** 90 days for fixing before disclosure

### Security Guidelines
- Validate all inputs
- Use secure defaults
- Audit dependencies regularly
- Follow OWASP guidelines

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

## Questions?

- **Getting Started**: See [Getting Started Guide](getting-started.md)
- **Code Style**: See [Coding Standards](coding-standards.md)
- **Testing**: See [Testing Guide](testing.md)
- **Issues**: Open a GitHub issue
- **Discussions**: Start a GitHub discussion