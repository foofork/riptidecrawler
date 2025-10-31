# Python SDK Consolidation Plan

**Goal:** Deprecate `/python-sdk` (riptide-client) and consolidate on `/sdk/python` (riptide-sdk)

**Status:** Planning Phase
**Priority:** High
**Target Timeline:** 90 days for full deprecation

---

## Executive Summary

Two Python SDKs currently exist in the codebase:

1. **Legacy SDK**: `/python-sdk` (riptide-client v1.0.0)
   - Synchronous, requests-based
   - 4 Python files (~24k lines with README)
   - Simple architecture focused on ease of use
   - Published to PyPI as `riptide-client`

2. **Modern SDK**: `/sdk/python` (riptide-sdk v0.2.0)
   - Asynchronous, httpx-based
   - 18+ Python files in modular architecture
   - 84% API coverage with advanced features
   - Published to PyPI as `riptide-sdk`

**Recommendation:** Deprecate riptide-client and migrate all users to riptide-sdk.

---

## 1. Deprecation Strategy

### 1.1 Communication Timeline

#### Phase 1: Initial Announcement (Week 1-2)
- **Action:** Add prominent deprecation notice to riptide-client README
- **Message Template:**
  ```markdown
  # ⚠️ DEPRECATION NOTICE

  **This package (`riptide-client`) is deprecated and will not receive further updates.**

  Please migrate to the new **`riptide-sdk`** package:

  ```bash
  pip install riptide-sdk
  ```

  ## Why Migrate?

  - ✅ **Async/await support** - Built on httpx for high-performance operations
  - ✅ **84% API coverage** - vs 34% in riptide-client
  - ✅ **Modern features** - Browser automation, WebSocket streaming, PDF processing
  - ✅ **Active development** - New features and bug fixes
  - ✅ **Type safety** - Full type hints and autocompletion

  ## Migration Timeline

  - **Jan 1, 2026**: No new features added to riptide-client
  - **Apr 1, 2026**: Security fixes only
  - **Jul 1, 2026**: Package archived, no further updates

  See [Migration Guide](#migration-guide) below for code examples.
  ```

#### Phase 2: PyPI Package Strategy (Week 3-4)
- **Upload final version** (v1.0.1) with deprecation warnings:
  ```python
  # riptide_client/__init__.py
  import warnings

  warnings.warn(
      "riptide-client is deprecated. Please use riptide-sdk instead: "
      "pip install riptide-sdk",
      DeprecationWarning,
      stacklevel=2
  )
  ```
- **Update PyPI metadata**:
  - Add "Development Status :: 7 - Inactive" classifier
  - Update description to indicate deprecation
  - Add link to riptide-sdk in project URLs
  - Consider yanking older versions (optional)

#### Phase 3: Documentation Updates (Week 3-4)
- Update main README to only reference riptide-sdk
- Add migration guide to documentation
- Update all code examples
- Create redirect notices in deprecated docs

#### Phase 4: Final Archive (Day 90)
- Mark PyPI package as "archived"
- Update GitHub repository settings
- Final announcement in changelog

### 1.2 PyPI Package Strategy

#### Immediate Actions:
1. **Publish riptide-client v1.0.1** with deprecation warnings
2. **Update package metadata**:
   ```toml
   [project]
   classifiers = [
       "Development Status :: 7 - Inactive",
       "Intended Audience :: Developers",
       # ... other classifiers
   ]
   description = "DEPRECATED: Use riptide-sdk instead - Official Python SDK for RipTide API"

   [project.urls]
   "Replacement Package" = "https://pypi.org/project/riptide-sdk/"
   Migration = "https://github.com/your-org/eventmesh/blob/main/docs/python-sdk-migration.md"
   ```

3. **Add installation warning**:
   ```python
   # setup.py or pyproject.toml post-install hook
   print("\n" + "="*60)
   print("⚠️  WARNING: riptide-client is DEPRECATED")
   print("Please install riptide-sdk instead:")
   print("    pip install riptide-sdk")
   print("="*60 + "\n")
   ```

#### Long-term Strategy:
- **Months 1-3**: Security fixes only
- **Month 4+**: Archive package, no updates
- **Do NOT yank packages** - Leave existing versions for backward compatibility
- **Consider**: Transfer PyPI package ownership to archive bot

---

## 2. Migration Path

### 2.1 Breaking Changes Summary

| Feature | riptide-client | riptide-sdk | Breaking? |
|---------|---------------|-------------|-----------|
| **Async/Sync** | Synchronous | Async/await | ✅ **YES** |
| **Client Import** | `from riptide_client import RipTide` | `from riptide_sdk import RipTideClient` | ✅ **YES** |
| **HTTP Library** | `requests` | `httpx` | Internal only |
| **Context Manager** | Optional | Required for async | ⚠️ Pattern change |
| **Return Types** | Dicts | Typed models | ⚠️ Access pattern |
| **API Coverage** | 34% (21/62) | 84% (52/62) | ➕ Additive |

### 2.2 Code Migration Examples

#### Example 1: Basic Crawling

**Before (riptide-client):**
```python
from riptide_client import RipTide

# Synchronous
client = RipTide('http://localhost:8080')
result = client.crawl(['https://example.com'])
print(result['results'][0]['document']['title'])
```

**After (riptide-sdk):**
```python
from riptide_sdk import RipTideClient
import asyncio

async def main():
    # Asynchronous with context manager
    async with RipTideClient(base_url='http://localhost:8080') as client:
        result = await client.crawl.batch(['https://example.com'])
        print(result.results[0].document.title)  # Typed attributes

asyncio.run(main())
```

#### Example 2: Context Manager Usage

**Before (riptide-client):**
```python
# Optional context manager
with RipTide('http://localhost:8080') as client:
    result = client.crawl(['https://example.com'])
    print(result)
```

**After (riptide-sdk):**
```python
# Required async context manager
async with RipTideClient(base_url='http://localhost:8080') as client:
    result = await client.crawl.batch(['https://example.com'])
    print(result)
```

#### Example 3: Streaming

**Before (riptide-client):**
```python
# NDJSON streaming
for result in client.stream_crawl(['https://example.com']):
    print(f"Got: {result['url']}")
```

**After (riptide-sdk):**
```python
# Async NDJSON streaming
async for result in client.streaming.crawl_ndjson(['https://example.com']):
    print(f"Got: {result.data['url']}")
```

#### Example 4: Error Handling

**Before (riptide-client):**
```python
from riptide_client import RipTide, APIError, RateLimitError

try:
    result = client.crawl(['https://example.com'])
except RateLimitError:
    print("Rate limited")
except APIError as e:
    print(f"Error: {e}")
```

**After (riptide-sdk):**
```python
from riptide_sdk import RipTideClient, APIError, ValidationError

try:
    result = await client.crawl.batch(['https://example.com'])
except ValidationError as e:
    print(f"Validation: {e}")
except APIError as e:
    print(f"API error [{e.status_code}]: {e.message}")
```

#### Example 5: New Features (SDK-only)

**Browser Automation (Not available in riptide-client):**
```python
from riptide_sdk import RipTideClient
from riptide_sdk.models import BrowserSessionConfig

async with RipTideClient() as client:
    # Create browser session
    config = BrowserSessionConfig(stealth_preset="medium")
    session = await client.browser.create_session(config)

    # Navigate and interact
    await client.browser.navigate(session.session_id, "https://example.com")
    await client.browser.click(session.session_id, "button.submit")

    # Take screenshot
    screenshot = await client.browser.screenshot(session.session_id)
```

### 2.3 Compatibility Layer (Optional)

For gradual migration, create a sync wrapper:

```python
# riptide_sdk/sync.py (optional helper)
import asyncio
from functools import wraps
from .client import RipTideClient

class SyncRipTideClient:
    """Synchronous wrapper for gradual migration"""

    def __init__(self, base_url: str, **kwargs):
        self.base_url = base_url
        self.kwargs = kwargs
        self._client = None

    def crawl(self, urls, options=None):
        """Sync wrapper for crawl.batch"""
        async def _crawl():
            async with RipTideClient(self.base_url, **self.kwargs) as client:
                return await client.crawl.batch(urls, options)
        return asyncio.run(_crawl())

    # Add other methods as needed...
```

Usage:
```python
from riptide_sdk.sync import SyncRipTideClient

# Drop-in replacement with minimal changes
client = SyncRipTideClient('http://localhost:8080')
result = client.crawl(['https://example.com'])
```

---

## 3. Documentation Updates

### 3.1 Files Requiring Deprecation Notices

#### High Priority (User-facing):
1. **`/python-sdk/README.md`** ✅ CRITICAL
   - Add prominent deprecation banner at top
   - Link to migration guide
   - Show side-by-side comparison

2. **`/python-sdk/PUBLISHING.md`**
   - Mark as obsolete
   - Reference riptide-sdk publishing process

3. **`/python-sdk/examples/*`**
   - Add deprecation notice to all example files
   - Link to new examples in `/sdk/python/examples/`

#### Medium Priority (Developer docs):
4. **`/docs/README.md`**
   - Update SDK references
   - Point to `/sdk/python` as canonical

5. **`/docs/01-guides/usage/api-usage.md`**
   - Update Python examples to use riptide-sdk
   - Remove riptide-client references

6. **`/docs/PYTHON_SDK_DUPLICATION_ANALYSIS.md`**
   - Update with final decision
   - Mark as resolved

7. **`/cli/README.md`**
   - Update PyPI link from riptide-client to riptide-sdk

### 3.2 Files Requiring SDK Reference Updates

#### Root Level:
1. **`/README.md`** ✅ CRITICAL
   - Line 82: Change `pip install riptide-sdk` (already correct!)
   - Verify all Python examples use riptide-sdk

#### Documentation:
2. **`/docs/04-architecture/components/system-diagram.md`**
   - Line mentioning `riptide-client` → change to `riptide-sdk`

3. **`/docs/02-api-reference/security.md`**
   - Update client_id example if relevant

4. **All tutorial/guide files** (`/docs/01-guides/**/*.md`)
   - Search and replace Python SDK references
   - Update installation instructions
   - Verify code examples

### 3.3 New Documentation Needed

#### Create Migration Guide:
**`/docs/python-sdk-migration.md`**
```markdown
# Migrating from riptide-client to riptide-sdk

## Quick Migration Checklist

- [ ] Install riptide-sdk: `pip install riptide-sdk`
- [ ] Update imports: `riptide_client` → `riptide_sdk`
- [ ] Convert sync code to async/await
- [ ] Update result access: dict keys → typed attributes
- [ ] Test with new SDK
- [ ] Uninstall old package: `pip uninstall riptide-client`

## Detailed Migration Steps

### 1. Install New SDK
[...]

### 2. Update Code
[... include examples from section 2.2 ...]

### 3. Verify Functionality
[... testing guidance ...]
```

#### Update SDK Documentation:
**`/sdk/python/README.md`** - Add migration section:
```markdown
## Migrating from riptide-client

If you were using the older `riptide-client` package (located in `/python-sdk`),
please see our [Migration Guide](../../docs/python-sdk-migration.md).

**Quick comparison:**

| Feature | riptide-client | riptide-sdk |
|---------|---------------|-------------|
| Async Support | ❌ No | ✅ Yes |
| API Coverage | 34% | 84% |
| Browser Automation | ❌ No | ✅ Yes |
| WebSocket Streaming | ❌ No | ✅ Yes |
| Type Hints | Partial | Full |
```

### 3.4 README Updates Summary

**Files to update:**
- `/python-sdk/README.md` - Add deprecation banner
- `/sdk/python/README.md` - Add migration section (partially done)
- `/README.md` - Already references riptide-sdk ✅
- `/docs/README.md` - Update Python SDK links
- All example READMEs in both SDK directories

---

## 4. Archive Strategy

### 4.1 Should We Delete `/python-sdk`?

**Recommendation: Keep for Reference (Archive State)**

**Rationale:**
- Preserves git history without requiring `git log --follow`
- Helps users understand migration path
- Useful for debugging legacy issues
- Minimal storage cost (~24KB of Python code)

**Alternative: Delete After Grace Period**
- Wait 6-12 months post-deprecation
- Ensure all users migrated
- Delete directory but preserve in git history

### 4.2 Archive Implementation

#### Option A: Keep with Deprecation Markers (Recommended)
```bash
# Add DEPRECATED file
echo "This SDK is deprecated. Use /sdk/python instead." > /python-sdk/DEPRECATED

# Update .gitignore
echo "# Legacy SDK - deprecated" >> .gitignore
echo "python-sdk/**/*.pyc" >> .gitignore

# Add archive notice to directory
cat > /python-sdk/ARCHIVE_NOTICE.md << 'EOF'
# ⚠️ ARCHIVED SDK

This directory contains the legacy `riptide-client` SDK which is **no longer maintained**.

**Archived:** March 2026
**Replacement:** `/sdk/python` (riptide-sdk)

## For Historical Reference Only

Do not use this code for new projects. See:
- Migration Guide: `/docs/python-sdk-migration.md`
- Modern SDK: `/sdk/python`
EOF
```

#### Option B: Move to Archive Directory
```bash
# Create archive directory
mkdir -p /archive

# Move legacy SDK
git mv /python-sdk /archive/python-sdk-legacy

# Update README
echo "# Archived Legacy SDK - DO NOT USE" > /archive/python-sdk-legacy/README.md
```

### 4.3 Git History Preservation

**Best Practice: Keep in Repository**

- Git history is already preserved
- No special action needed
- Use git tags for milestones:

```bash
# Tag final version
git tag -a riptide-client-v1.0.1-final -m "Final release of riptide-client before deprecation"

# Tag deprecation point
git tag -a riptide-client-deprecated -m "riptide-client officially deprecated, use riptide-sdk"
```

### 4.4 Backup Strategy

**GitHub as Source of Truth:**
- Main repository contains full history
- No external backup needed for code
- Consider archiving PyPI package separately

**Documentation Backup:**
```bash
# Create archive snapshot
tar -czf python-sdk-archive-$(date +%Y%m%d).tar.gz /python-sdk

# Store in releases or separate archive repository
gh release create python-sdk-archive-final \
  --title "Legacy Python SDK Archive" \
  --notes "Final archive of riptide-client before deprecation"
```

### 4.5 Recommended Approach

**Timeline:**
1. **Day 0-90**: Keep `/python-sdk` with deprecation notices
2. **Day 90-180**: Add `DEPRECATED` marker, keep code
3. **Day 180-365**: Move to `/archive/python-sdk-legacy`
4. **After 1 year**: Consider deletion (git history preserved)

**Implementation:**
```bash
# Phase 1 (Today - Day 90): Add deprecation
echo "⚠️ DEPRECATED - Use /sdk/python instead" > /python-sdk/DEPRECATED

# Phase 2 (Day 90): Mark as archived
git mv /python-sdk/README.md /python-sdk/README.DEPRECATED.md
cp /templates/ARCHIVE_NOTICE.md /python-sdk/README.md

# Phase 3 (Day 180): Move to archive
mkdir -p /archive
git mv /python-sdk /archive/python-sdk-legacy

# Phase 4 (Day 365+): Optional deletion
# git rm -r /archive/python-sdk-legacy  # Only if absolutely necessary
```

---

## 5. Testing Requirements

### 5.1 Pre-Deprecation Testing

#### Test Coverage Verification:
```bash
cd /sdk/python
pytest --cov=riptide_sdk --cov-report=html
# Target: >80% coverage (currently achieved)
```

#### API Compatibility Matrix:

| Endpoint Category | riptide-client | riptide-sdk | Notes |
|-------------------|---------------|-------------|-------|
| Core Crawling | ✅ | ✅ | Fully compatible |
| Streaming | ✅ (NDJSON/SSE) | ✅ (NDJSON/SSE/WS) | WS is new |
| Search | ✅ | ✅ | Compatible |
| Spider | ✅ | ✅ | Enhanced in SDK |
| Sessions | ✅ | ✅ | Compatible |
| PDF | ✅ | ✅ | Compatible |
| Workers | ✅ | ✅ | Compatible |
| **Browser** | ❌ | ✅ | SDK-only |
| **Profiles** | ❌ | ✅ | SDK-only |
| **Engine Selection** | ❌ | ✅ | SDK-only |

#### Feature Parity Tests:
```python
# tests/test_migration_parity.py
import pytest
from riptide_sdk import RipTideClient

@pytest.mark.asyncio
async def test_crawl_returns_compatible_structure():
    """Verify riptide-sdk returns expected data structure"""
    async with RipTideClient() as client:
        result = await client.crawl.batch(["https://example.com"])

        # Verify structure matches expected format
        assert hasattr(result, 'results')
        assert hasattr(result, 'successful')
        assert hasattr(result, 'total_urls')

        # Verify first result has document
        if result.results:
            assert hasattr(result.results[0], 'document')
            assert hasattr(result.results[0].document, 'title')

@pytest.mark.asyncio
async def test_error_compatibility():
    """Verify error handling matches expectations"""
    from riptide_sdk import ValidationError, APIError

    async with RipTideClient() as client:
        with pytest.raises((ValidationError, APIError)):
            await client.crawl.batch([])  # Empty list should error
```

### 5.2 Migration Testing Strategy

#### Phase 1: Side-by-Side Comparison Tests
Create test suite that verifies both SDKs produce equivalent results:

```python
# tests/test_sdk_comparison.py
import pytest
import asyncio
from riptide_client import RipTide as LegacyClient
from riptide_sdk import RipTideClient as ModernClient

@pytest.fixture
def test_urls():
    return ["https://example.com", "https://httpbin.org/html"]

def test_crawl_results_equivalent(test_urls):
    """Verify both SDKs produce compatible results"""
    # Legacy SDK (sync)
    legacy_client = LegacyClient('http://localhost:8080')
    legacy_result = legacy_client.crawl(test_urls)

    # Modern SDK (async)
    async def get_modern_result():
        async with ModernClient(base_url='http://localhost:8080') as client:
            return await client.crawl.batch(test_urls)

    modern_result = asyncio.run(get_modern_result())

    # Compare key fields
    assert len(legacy_result['results']) == len(modern_result.results)
    assert legacy_result['successful'] == modern_result.successful

    # Compare first document
    if legacy_result['results']:
        legacy_doc = legacy_result['results'][0]['document']
        modern_doc = modern_result.results[0].document

        assert legacy_doc['title'] == modern_doc.title
        assert legacy_doc['url'] == modern_doc.url
```

#### Phase 2: Integration Tests
```python
# tests/integration/test_end_to_end.py
@pytest.mark.integration
@pytest.mark.asyncio
async def test_full_workflow_riptide_sdk():
    """End-to-end test of common workflow with new SDK"""
    async with RipTideClient() as client:
        # 1. Basic crawl
        crawl_result = await client.crawl.batch(["https://example.com"])
        assert crawl_result.successful > 0

        # 2. Extract content
        extract_result = await client.extract.extract("https://example.com")
        assert extract_result.content

        # 3. Create session
        session = await client.sessions.create(SessionConfig(ttl_seconds=3600))
        assert session.id

        # 4. Cleanup
        await client.sessions.delete(session.id)
```

#### Phase 3: Performance Benchmarks
```python
# tests/benchmarks/test_performance.py
import time
import asyncio

def test_sync_client_performance():
    """Benchmark legacy sync client"""
    from riptide_client import RipTide

    client = RipTide('http://localhost:8080')
    urls = ["https://example.com"] * 10

    start = time.time()
    result = client.crawl(urls)
    duration = time.time() - start

    print(f"Legacy SDK: {duration:.2f}s for {len(urls)} URLs")
    return duration

@pytest.mark.asyncio
async def test_async_client_performance():
    """Benchmark modern async client"""
    from riptide_sdk import RipTideClient

    async with RipTideClient() as client:
        urls = ["https://example.com"] * 10

        start = time.time()
        result = await client.crawl.batch(urls)
        duration = time.time() - start

        print(f"Modern SDK: {duration:.2f}s for {len(urls)} URLs")
        return duration
```

### 5.3 Breaking Changes Validation

**Critical Test Cases:**

1. **Import Changes:**
   ```python
   # Verify old import fails with helpful message
   def test_legacy_import_warning():
       with pytest.warns(DeprecationWarning, match="riptide-sdk"):
           from riptide_client import RipTide
   ```

2. **Async Requirement:**
   ```python
   # Verify sync usage raises clear error
   def test_sync_usage_error():
       from riptide_sdk import RipTideClient

       client = RipTideClient()
       with pytest.raises(RuntimeError, match="async"):
           # Should fail - not awaited
           client.crawl.batch(["https://example.com"])
   ```

3. **Type Safety:**
   ```python
   # Verify typed responses work correctly
   @pytest.mark.asyncio
   async def test_typed_responses():
       async with RipTideClient() as client:
           result = await client.crawl.batch(["https://example.com"])

           # Should have typed attributes
           assert isinstance(result.successful, int)
           assert isinstance(result.results, list)

           # Should support IDE autocomplete
           first_result = result.results[0]
           assert hasattr(first_result, 'document')
           assert hasattr(first_result.document, 'title')
   ```

### 5.4 User Acceptance Testing

**Test Migration Guide Examples:**
1. Run all code examples from migration guide
2. Verify they work without modification
3. Check error messages are helpful
4. Validate documentation accuracy

**Checklist:**
- [ ] All migration examples execute successfully
- [ ] Error messages guide users to correct usage
- [ ] Documentation links work
- [ ] Installation process is smooth
- [ ] Import deprecation warnings show
- [ ] No silent failures or confusing errors

### 5.5 Continuous Testing

**GitHub Actions Workflow:**
```yaml
# .github/workflows/sdk-compatibility.yml
name: SDK Compatibility Testing

on: [push, pull_request]

jobs:
  test-both-sdks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Test Legacy SDK
        run: |
          cd python-sdk
          pip install -e ".[dev]"
          pytest tests/ --cov

      - name: Test Modern SDK
        run: |
          cd sdk/python
          pip install -e ".[dev]"
          pytest tests/ --cov

      - name: Compare Coverage
        run: |
          echo "Legacy SDK coverage: $(cat python-sdk/.coverage)"
          echo "Modern SDK coverage: $(cat sdk/python/.coverage)"
```

---

## 6. Implementation Checklist

### Week 1-2: Preparation
- [ ] Review and approve this consolidation plan
- [ ] Create migration guide document
- [ ] Set up deprecation branch
- [ ] Notify stakeholders of deprecation timeline

### Week 3-4: Deprecation Implementation
- [ ] Add deprecation warnings to riptide-client code
- [ ] Update riptide-client README with banner
- [ ] Publish riptide-client v1.0.1 to PyPI
- [ ] Update PyPI metadata and classifiers
- [ ] Add DEPRECATED marker to `/python-sdk`

### Week 5-6: Documentation Updates
- [ ] Update main README.md
- [ ] Update all docs references (see section 3.2)
- [ ] Create migration guide page
- [ ] Update SDK comparison documentation
- [ ] Add migration examples to riptide-sdk docs

### Week 7-8: Testing & Validation
- [ ] Run compatibility tests (section 5.1)
- [ ] Execute migration tests (section 5.2)
- [ ] Performance benchmarking
- [ ] User acceptance testing with sample projects

### Week 9-10: Communication
- [ ] Publish blog post announcing deprecation
- [ ] Email notification to known users (if list exists)
- [ ] Update GitHub repository description
- [ ] Post in relevant communities (Reddit, Discord, etc.)

### Week 11-12: Monitoring
- [ ] Monitor GitHub issues for migration problems
- [ ] Track PyPI download statistics
- [ ] Address user feedback and questions
- [ ] Update FAQ based on common questions

### Day 90+: Finalization
- [ ] Mark riptide-client as inactive on PyPI
- [ ] Move `/python-sdk` to `/archive/python-sdk-legacy`
- [ ] Create final archive snapshot
- [ ] Tag final versions in git
- [ ] Remove riptide-client from active documentation

---

## 7. Risk Assessment & Mitigation

### 7.1 Identified Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Users don't migrate | Medium | Medium | Clear timeline, helpful warnings, migration guide |
| Breaking changes cause issues | High | Medium | Comprehensive testing, compatibility layer option |
| PyPI package confusion | Low | Low | Clear naming, deprecation warnings |
| Lost features | Low | High | Feature parity verification (riptide-sdk has MORE features) |
| Documentation gaps | Medium | Medium | Thorough doc review, examples for all use cases |
| Support burden | Medium | Low | Self-service migration guide, automated warnings |

### 7.2 Rollback Plan

**If critical issues arise:**

1. **Immediate Actions:**
   - Restore riptide-client to active status
   - Publish bug fix release
   - Communicate issue and timeline

2. **Assessment:**
   - Identify root cause
   - Determine if riptide-sdk needs fixes
   - Evaluate if issue affects many users

3. **Decision Points:**
   - Can issue be fixed in riptide-sdk quickly? → Do it, resume deprecation
   - Is issue fundamental? → Delay deprecation, improve SDK
   - Only affects few users? → Provide workaround, continue deprecation

### 7.3 Success Metrics

**Track these metrics:**
- PyPI download statistics (riptide-client declining, riptide-sdk increasing)
- GitHub issue mentions of migration
- Documentation page views (migration guide)
- User feedback sentiment
- Test coverage maintenance (>80% for riptide-sdk)

**Success Criteria:**
- 80% of active users migrated within 90 days
- Zero critical bugs in riptide-sdk blocking migration
- Migration guide receives positive feedback
- Support requests decline after month 2

---

## 8. Communication Templates

### 8.1 PyPI Description Update

```markdown
# ⚠️ DEPRECATED - Use riptide-sdk instead

This package is no longer maintained. Please migrate to **riptide-sdk** for continued support and new features.

## Quick Migration

```bash
pip uninstall riptide-client
pip install riptide-sdk
```

See [Migration Guide](https://github.com/your-org/eventmesh/blob/main/docs/python-sdk-migration.md) for code examples.

## Why riptide-sdk?

- ✅ Async/await support
- ✅ 84% API coverage (vs 34%)
- ✅ Active development
- ✅ Modern features (browser automation, WebSocket streaming)

## Deprecation Timeline

- **Jan 2026**: No new features
- **Apr 2026**: Security fixes only
- **Jul 2026**: Package archived

---

[Continue to package details...]
```

### 8.2 GitHub Issue Template

```markdown
---
name: Migration Support
about: Help migrating from riptide-client to riptide-sdk
title: '[Migration] '
labels: migration, help-wanted
assignees: ''
---

## Migration Question

**Current code (riptide-client):**
```python
# Paste your current code here
```

**What I tried (riptide-sdk):**
```python
# Paste your migration attempt here
```

**Error or issue:**
```
Paste error message or describe issue
```

**Resources checked:**
- [ ] Read [Migration Guide](link)
- [ ] Reviewed [API Documentation](link)
- [ ] Checked [Examples](link)

**Additional context:**
Add any other context about the migration issue here.
```

### 8.3 Email Announcement (If Applicable)

```markdown
Subject: Action Required: Migrate from riptide-client to riptide-sdk

Hi RipTide Users,

We're consolidating our Python SDKs to provide you with better features and support.

**What's changing:**
- riptide-client is being deprecated
- riptide-sdk is now the official Python SDK
- You have 90 days to migrate (security fixes continue for 180 days)

**Why migrate:**
✅ Async/await for better performance
✅ 2.5x more API coverage (84% vs 34%)
✅ New features: Browser automation, WebSocket streaming, PDF processing
✅ Active development and support

**Migration is easy:**
Most code changes are straightforward async/await updates.
See our migration guide: [link]

**Timeline:**
- Today: Deprecation announcement
- Mar 2026: No new features for riptide-client
- Jun 2026: Security fixes only
- Sep 2026: Package archived

**Need help?**
- Migration Guide: [link]
- GitHub Issues: [link]
- Examples: [link]

Thank you for using RipTide!

The RipTide Team
```

---

## 9. Post-Deprecation Maintenance

### 9.1 Security Patches (Months 1-6)

**Policy:**
- Critical security vulnerabilities: Patch immediately
- High severity: Patch within 7 days
- Medium/Low: No patches after month 3

**Process:**
1. Receive security report
2. Assess severity
3. If critical/high: Create patch, publish v1.0.x
4. Notify users via GitHub Security Advisory
5. Encourage migration to riptide-sdk

### 9.2 Issue Management

**Response Templates:**

For new feature requests:
```markdown
Thank you for the suggestion! However, riptide-client is deprecated.

Please check if riptide-sdk supports this feature: [link to docs]

If not, please open an issue in the riptide-sdk tracker: [link]

Migration guide: [link]
```

For bug reports:
```markdown
Thank you for reporting this. riptide-client is in maintenance mode.

If this is a security issue, we'll patch it. Otherwise, please migrate to riptide-sdk which has this issue fixed.

Migration guide: [link]
```

### 9.3 Final Shutdown Checklist (Day 180)

- [ ] Archive GitHub repository section for riptide-client
- [ ] Update PyPI to "Development Status :: 7 - Inactive"
- [ ] Final announcement in README and docs
- [ ] Move code to `/archive/` directory
- [ ] Update all documentation to remove references
- [ ] Close remaining non-security issues
- [ ] Create final git tags
- [ ] Remove from CI/CD pipelines

---

## 10. Lessons Learned & Future Prevention

### 10.1 What Led to Duplication?

**Root Causes:**
1. Separate development efforts without coordination
2. Different design philosophies (sync vs async)
3. Lack of clear SDK ownership/roadmap
4. Both SDKs published to PyPI independently

### 10.2 Prevention Strategies

**Going Forward:**
1. **Single source of truth**: Only `/sdk/python` for Python SDK
2. **Clear ownership**: Designate SDK maintainer
3. **RFC process**: Major changes require design doc
4. **Version planning**: Roadmap prevents parallel efforts
5. **Deprecation policy**: Document when/how to deprecate features

**Documentation:**
- Maintain `ARCHITECTURE.md` showing package relationships
- SDK development guidelines in `CONTRIBUTING.md`
- Regular architecture review meetings

### 10.3 Recommended Policies

**SDK Governance:**
```markdown
# Python SDK Governance Policy

1. **Single Official SDK**: /sdk/python (riptide-sdk)
2. **No parallel implementations** without RFC
3. **Deprecation requires**:
   - 90-day notice minimum
   - Migration guide
   - Compatibility testing
4. **Breaking changes require**:
   - Major version bump
   - Changelog entry
   - Migration examples
5. **PyPI publishing**:
   - Only official SDK on PyPI
   - No experimental packages without -beta suffix
```

---

## Appendix A: File Reference

### Files to Modify

**Python SDK Code:**
- `/python-sdk/README.md` - Add deprecation notice
- `/python-sdk/__init__.py` - Add warning
- `/python-sdk/pyproject.toml` - Update classifiers
- `/sdk/python/README.md` - Add migration section

**Documentation:**
- `/README.md` - Verify references
- `/docs/python-sdk-migration.md` - CREATE NEW
- `/docs/README.md` - Update links
- `/docs/01-guides/usage/api-usage.md` - Update examples
- `/docs/04-architecture/components/system-diagram.md` - Update references
- `/docs/PYTHON_SDK_DUPLICATION_ANALYSIS.md` - Mark resolved

**CI/CD:**
- `.github/workflows/` - Remove riptide-client tests (optional)
- PyPI publishing workflows

### New Files to Create

- `/docs/python-sdk-migration.md` - Migration guide
- `/python-sdk/DEPRECATED` - Marker file
- `/python-sdk/ARCHIVE_NOTICE.md` - Archive instructions

---

## Appendix B: Timeline Gantt Chart

```
Week 1-2   [Prep & Planning] ████████
Week 3-4   [Deprecation]     ░░░░████████
Week 5-6   [Documentation]   ░░░░░░░░████████
Week 7-8   [Testing]         ░░░░░░░░░░░░████████
Week 9-10  [Communication]   ░░░░░░░░░░░░░░░░████████
Week 11-12 [Monitoring]      ░░░░░░░░░░░░░░░░░░░░████████
Day 90     [Finalization]    ░░░░░░░░░░░░░░░░░░░░░░░░████
```

---

## Appendix C: Contact & Escalation

### Project Contacts

- **SDK Owner**: [Name/Email]
- **DevOps**: [Name/Email]
- **Documentation**: [Name/Email]
- **Support**: [support@riptide.dev]

### Escalation Path

1. **User Question** → GitHub Issues / Migration Guide
2. **Bug Report** → GitHub Issues → SDK Owner
3. **Security Issue** → security@riptide.dev → Immediate patch
4. **Breaking Issue** → SDK Owner → Rollback decision within 24h

---

## Conclusion

This consolidation plan provides a clear path to deprecate the legacy `riptide-client` SDK while ensuring a smooth migration for users to the modern `riptide-sdk`. By following this timeline and checklist, we can eliminate technical debt while minimizing user disruption.

**Key Success Factors:**
- Clear, proactive communication
- Comprehensive migration guide with examples
- Adequate timeline (90 days + security patches)
- Strong testing to prevent issues
- Helpful error messages and warnings

**Next Steps:**
1. Review and approve this plan
2. Begin Week 1-2 preparation tasks
3. Execute deprecation in phases
4. Monitor and adjust based on user feedback

---

**Document Version:** 1.0
**Last Updated:** 2025-10-31
**Status:** Planning Phase
**Approvers:** [TBD]
