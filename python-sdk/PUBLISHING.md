# Publishing to PyPI

## Prerequisites

1. Create accounts on:
   - [PyPI](https://pypi.org/account/register/)
   - [Test PyPI](https://test.pypi.org/account/register/)

2. Generate API tokens:
   - PyPI: https://pypi.org/manage/account/token/
   - Test PyPI: https://test.pypi.org/manage/account/token/

3. Add tokens to GitHub secrets:
   - `PYPI_API_TOKEN`
   - `TEST_PYPI_API_TOKEN`

## Manual Publishing

### 1. Install Build Tools

```bash
pip install build twine
```

### 2. Build Package

```bash
cd python-sdk
python -m build
```

This creates:
- `dist/riptide_client-1.0.0-py3-none-any.whl`
- `dist/riptide-client-1.0.0.tar.gz`

### 3. Test on Test PyPI

```bash
# Upload to Test PyPI
python -m twine upload --repository testpypi dist/*

# Install from Test PyPI
pip install --index-url https://test.pypi.org/simple/ riptide-client

# Test it works
python -c "from riptide_client import RipTide; print('Success!')"
```

### 4. Publish to PyPI

```bash
# Upload to PyPI
python -m twine upload dist/*

# Install from PyPI
pip install riptide-client
```

## Automated Publishing with GitHub Actions

### Via Git Tag

```bash
# Tag release
git tag python-v1.0.0
git push origin python-v1.0.0

# GitHub Actions will automatically:
# 1. Run tests
# 2. Build package
# 3. Publish to PyPI
```

### Manual Trigger

1. Go to GitHub Actions
2. Select "Publish Python SDK" workflow
3. Click "Run workflow"
4. Package will be published to Test PyPI

## Version Bumping

Edit `pyproject.toml`:

```toml
[project]
version = "1.0.1"  # Increment version
```

Commit and tag:

```bash
git commit -am "Bump version to 1.0.1"
git tag python-v1.0.1
git push origin python-v1.0.1
```

## Verification

After publishing:

```bash
# Install from PyPI
pip install riptide-client

# Verify version
python -c "import riptide_client; print(riptide_client.__version__)"

# Test basic functionality
python -c "from riptide_client import RipTide; client = RipTide(); print(client)"
```

## Checklist Before Publishing

- [ ] All tests pass (`pytest`)
- [ ] Version number updated in `pyproject.toml`
- [ ] CHANGELOG.md updated
- [ ] README.md up to date
- [ ] Documentation reviewed
- [ ] Built locally and tested (`python -m build`)
- [ ] Tested on Test PyPI
- [ ] Tagged in git

## Troubleshooting

### Upload Fails

```bash
# Check credentials
python -m twine upload --repository testpypi dist/* --verbose

# Clear old builds
rm -rf dist/ build/ *.egg-info
python -m build
```

### Wrong Version on PyPI

You cannot re-upload the same version. Increment version and republish:

```toml
version = "1.0.1"  # or "1.0.0.post1" for hotfix
```

### Import Errors After Install

Check package structure:

```bash
# List package contents
tar -tzf dist/riptide-client-1.0.0.tar.gz
```

Ensure `__init__.py` exists and imports are correct.

## Links

- PyPI Package: https://pypi.org/project/riptide-client/
- Test PyPI: https://test.pypi.org/project/riptide-client/
- PyPI Publishing Guide: https://packaging.python.org/en/latest/tutorials/packaging-projects/
