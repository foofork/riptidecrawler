# Python Packaging Guide

This document describes how to build, test, and publish the `riptidecrawler` Python package.

## Prerequisites

```bash
# Install maturin
pip install maturin

# Install build tools
pip install build twine
```

## Local Development

### Development Build

For local development and testing:

```bash
cd crates/riptide-py

# Quick development build (installs in current Python env)
maturin develop

# Or use the build script
./build.sh dev
```

### Testing Local Build

```bash
# Run the test script
./test-wheel.sh

# Or manually:
python -c "import riptide; print(riptide.RipTide())"
pytest tests/ -v
```

## Building Wheels

### Release Build

```bash
cd crates/riptide-py

# Build release wheel
maturin build --release

# Or use the build script
./build.sh release

# Output: target/wheels/riptidecrawler-1.0.0-*.whl
```

### Multi-Platform Builds

The GitHub Actions workflow (`.github/workflows/python-wheels.yml`) automatically builds wheels for:

- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x64, x86

For each Python version: 3.8, 3.9, 3.10, 3.11, 3.12

### Manual Multi-Platform Build

Using Docker for Linux builds:

```bash
docker run --rm -v $(pwd):/io konstin2/maturin build --release
```

## Testing Wheels

### Install and Test Wheel

```bash
# Create clean virtual environment
python3 -m venv test-env
source test-env/bin/activate

# Install wheel
pip install target/wheels/riptidecrawler-1.0.0-*.whl

# Test import
python -c "import riptide; rt = riptide.RipTide(); print('Success!')"

# Run full test suite
pip install pytest pytest-asyncio
pytest tests/
```

### Test on Multiple Python Versions

Using `tox`:

```bash
# Install tox
pip install tox

# Run tests on all Python versions
tox
```

## Publishing to PyPI

### Test PyPI (Recommended First)

```bash
cd crates/riptide-py

# Build wheel
maturin build --release

# Publish to Test PyPI
maturin publish --repository testpypi

# Test installation from Test PyPI
pip install --index-url https://test.pypi.org/simple/ riptidecrawler
```

### Production PyPI

```bash
cd crates/riptide-py

# Build release wheel
maturin build --release

# Publish to PyPI (requires PYPI_API_TOKEN)
maturin publish

# Or use environment variable
MATURIN_PYPI_TOKEN=your-token-here maturin publish
```

### Automated Release

The GitHub Actions workflow automatically publishes to PyPI when you create a git tag:

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will:
# 1. Build wheels for all platforms
# 2. Run tests
# 3. Publish to PyPI
```

## Version Management

Update version in both files when releasing:

1. **Cargo.toml**:
   ```toml
   [package]
   version = "1.0.0"
   ```

2. **pyproject.toml**:
   ```toml
   [project]
   version = "1.0.0"
   ```

## Package Structure

```
crates/riptide-py/
├── Cargo.toml           # Rust package configuration
├── pyproject.toml       # Python package configuration
├── MANIFEST.in          # Package file inclusion rules
├── README.md            # Package documentation
├── riptide.pyi          # Type hints for Python
├── src/                 # Rust source code
│   ├── lib.rs          # PyO3 module definition
│   ├── riptide_class.rs # RipTide Python class
│   ├── document.rs      # Document Python class
│   └── errors.rs        # Error handling
├── tests/               # Python tests
│   ├── test_riptide.py  # Main test suite
│   └── test_performance.py # Performance benchmarks
├── examples/            # Python examples
│   ├── basic_usage.py   # Basic usage examples
│   └── spike_test.py    # Integration tests
├── build.sh            # Build script
└── test-wheel.sh       # Test script
```

## CI/CD Pipeline

The GitHub Actions workflow (`.github/workflows/python-wheels.yml`) provides:

1. **Automated Builds**: On every push and PR
2. **Multi-Platform Support**: Linux, macOS, Windows
3. **Multi-Python Support**: 3.8, 3.9, 3.10, 3.11, 3.12
4. **Automated Testing**: Run pytest on built wheels
5. **Automated Publishing**: Publish to PyPI on version tags

## Troubleshooting

### Build Fails

```bash
# Clean build artifacts
cargo clean

# Check Rust compilation
cargo check -p riptide-py

# Verify dependencies
cargo tree -p riptide-py
```

### Import Fails

```bash
# Check wheel contents
unzip -l target/wheels/riptidecrawler-1.0.0-*.whl

# Verify Python can find the module
python -c "import sys; print(sys.path)"
```

### Version Conflicts

```bash
# Ensure versions match
grep version crates/riptide-py/Cargo.toml
grep version crates/riptide-py/pyproject.toml
```

## Best Practices

1. **Always test locally** before publishing
2. **Use Test PyPI first** for new releases
3. **Update both version files** when releasing
4. **Run full test suite** before tagging
5. **Create git tags** for releases: `v1.0.0`, `v1.1.0`, etc.
6. **Follow semantic versioning**: MAJOR.MINOR.PATCH

## Resources

- [Maturin Documentation](https://www.maturin.rs/)
- [PyO3 Documentation](https://pyo3.rs/)
- [Python Packaging Guide](https://packaging.python.org/)
- [PyPI Publishing Guide](https://packaging.python.org/tutorials/packaging-projects/)
