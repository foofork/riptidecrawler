# Development Guide

Resources for developers contributing to RipTide.

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+ with `cargo`
- Docker and Docker Compose
- Redis 7.0+
- Node.js 18+ (for tooling)

### Quick Setup

```bash
# Clone repository
git clone https://github.com/your-org/eventmesh.git
cd eventmesh

# Install dependencies
cargo build

# Run tests
cargo test

# Start development server
cargo run --bin riptide-api
```

## 📚 Core Documentation

### Essential Reading
- **[Getting Started](./getting-started.md)** - Developer setup guide (⏱️ 15 min)
- **[Contributing Guide](./contributing.md)** - Contribution guidelines (⏱️ 20 min)
- **[Coding Standards](./coding-standards.md)** - Code style and best practices (⏱️ 25 min)

### Build & CI/CD
- **[Build Infrastructure](./BUILD-INFRASTRUCTURE.md)** - Build system overview (⏱️ 20 min)
- **[Local CI Guide](./LOCAL_CI_GUIDE.md)** - Running CI locally (⏱️ 15 min)
- **[Workflow Guide](./WORKFLOW_GUIDE.md)** - GitHub Actions workflows (⏱️ 15 min)

### Testing
- **[Testing Guide](./testing.md)** - Comprehensive testing documentation (⏱️ 35 min)
- **[Integration Testing Guide](./integration-testing.md)** - Comprehensive integration testing setup and best practices
- **[Golden Test Infrastructure](./testing/golden-test-infrastructure-analysis.md)** - Snapshot testing
- **[CI Timeout Configuration](./deployment/ci-timeout-configuration.md)** - Test timeout settings

### Performance & Security
- **[Memory Profiling Guide](./performance/memory-profiling-activation-guide.md)** - Performance analysis
- **[Memory Profiling Examples](./performance/memory-profiling-examples.md)** - Profiling examples
- **[Safety Audit](./safety-audit.md)** - Security and safety guidelines

## 🏗️ Development Workflow

### 1. Local Development

```bash
# Create feature branch
git checkout -b feature/your-feature

# Make changes and test
cargo fmt
cargo clippy
cargo test

# Build in debug mode
cargo build

# Run integration tests
./scripts/test-integration.sh
```

### 2. Testing Strategy

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Specific test suite
cargo test -p riptide-api --test api_tests

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

### 3. Code Quality

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all-targets --all-features

# Check for common mistakes
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### 4. Running Locally

```bash
# Development mode with hot reload
cargo watch -x 'run --bin riptide-api'

# With custom config
RUST_LOG=debug cargo run --bin riptide-api

# Full stack with Docker
docker-compose up -d
```

## 🧪 Testing Guide

### Test Organization

```
tests/
├── unit/           # Unit tests
├── integration/    # Integration tests
├── e2e/           # End-to-end tests
└── fixtures/      # Test data
```

### Running Tests

```bash
# All tests
cargo test

# Specific package
cargo test -p riptide-extractor

# Integration tests only
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html
```

### Golden Tests

```bash
# Run golden tests
cargo test --test golden_tests

# Update golden files
UPDATE_GOLDEN=1 cargo test --test golden_tests
```

## 📦 Project Structure

```
riptide/
├── crates/
│   ├── riptide-api/        # HTTP API server
│   ├── riptide-extractor/  # WASM content extraction
│   ├── riptide-spider/     # Web crawling engine
│   └── riptide-shared/     # Shared utilities
├── docs/                   # Documentation
├── tests/                  # Integration tests
├── scripts/               # Build and utility scripts
└── docker/                # Docker configurations
```

## 🔧 Build System

### Build Targets

```bash
# Debug build (fast, unoptimized)
cargo build

# Release build (optimized)
cargo build --release

# Specific package
cargo build -p riptide-api

# All features
cargo build --all-features
```

### CI/CD Pipeline

See [Workflow Guide](./WORKFLOW_GUIDE.md) for complete CI/CD documentation.

**Pipeline stages**:
1. **Lint** - Format and clippy checks
2. **Test** - Unit and integration tests
3. **Build** - Release builds
4. **Docker** - Container image creation
5. **Deploy** - Production deployment (main branch)

## 🎯 Common Tasks

### Adding a New Feature

1. **Plan** - Document in architecture docs
2. **Design** - Create ADR if needed
3. **Implement** - Write code following standards
4. **Test** - Add comprehensive tests
5. **Document** - Update relevant docs
6. **Review** - Submit PR for review

### Adding a New Endpoint

1. Update `riptide-api/src/routes.rs`
2. Add handler in appropriate module
3. Add OpenAPI documentation
4. Write integration tests
5. Update [API Reference](../02-api-reference/README.md)

### Debugging Tips

```bash
# Verbose logging
RUST_LOG=trace cargo run

# Specific module logging
RUST_LOG=riptide_api=debug cargo run

# Backtrace on panic
RUST_BACKTRACE=1 cargo test

# Memory debugging
valgrind target/debug/riptide-api
```

## 📊 Performance Profiling

### Memory Profiling

```bash
# Install profiling tools
cargo install heaptrack

# Profile application
heaptrack target/release/riptide-api

# Analyze results
heaptrack_gui heaptrack.riptide-api.*.gz
```

See [Memory Profiling Guide](./performance/memory-profiling-activation-guide.md) for details.

### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin riptide-api

# Open flamegraph.svg in browser
```

## 🔒 Security Guidelines

- Never commit secrets or credentials
- Use environment variables for configuration
- Follow [Safety Audit](./safety-audit.md) guidelines
- Run `cargo audit` regularly
- Review dependencies for vulnerabilities

## 📝 Documentation Standards

- Update docs with code changes
- Follow markdown lint rules
- Include code examples
- Add time estimates for reading
- Cross-reference related docs

## 🤝 Contributing

### Pull Request Process

1. **Fork & Clone** - Fork repo and create feature branch
2. **Develop** - Make changes following coding standards
3. **Test** - Ensure all tests pass
4. **Document** - Update relevant documentation
5. **Commit** - Use conventional commit messages
6. **PR** - Submit with clear description

### Commit Message Format

```
type(scope): brief description

Longer description if needed

Closes #issue-number
```

**Types**: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

## 🎓 Learning Resources

**Beginner** (2 hours):
1. [Getting Started](./getting-started.md)
2. [Contributing Guide](./contributing.md)
3. Build and run locally

**Intermediate** (4 hours):
1. [Coding Standards](./coding-standards.md)
2. [Testing Guide](./testing.md)
3. [Build Infrastructure](./BUILD-INFRASTRUCTURE.md)
4. Submit first PR

**Advanced** (Full day):
1. Architecture documents
2. Performance profiling
3. Complex feature implementation
4. CI/CD customization

## 🔗 Related Documentation

- **[Architecture](../04-architecture/README.md)** - System design
- **[API Reference](../02-api-reference/README.md)** - API documentation
- **[Deployment](../06-deployment/README.md)** - Production deployment
- **[Advanced Topics](../07-advanced/README.md)** - Performance optimization

## 🆘 Need Help?

- **[GitHub Issues](https://github.com/your-org/eventmesh/issues)** - Bug reports
- **[Discussions](https://github.com/your-org/eventmesh/discussions)** - Questions
- **[Contributing Guide](./contributing.md)** - Contribution help

---

**Ready to contribute?** → [Contributing Guide](./contributing.md)
