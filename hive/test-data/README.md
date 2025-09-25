# Test Data Coordination for WASM Extractor

This directory contains test data and coordination files used by the WASM extractor test suite.

## Directory Structure

```
test-data/
├── manifest.json          # Test data manifest and expectations
├── fixtures/              # Test HTML fixtures (symlinked from WASM tests)
├── expectations/           # Expected extraction results
└── README.md              # This file
```

## Test Data Manifest

The `manifest.json` file contains:

- **Test Data Version**: Version tracking for test data compatibility
- **Fixtures Metadata**: Information about each HTML fixture including:
  - Content type (news, blog, gallery, etc.)
  - Language and word count
  - Expected features (author, published date, categories)
- **Performance Expectations**: Baseline performance metrics
- **Quality Thresholds**: Minimum quality scores for each fixture type

## Usage

This test data is used by:

1. **Golden Tests**: Validate extraction accuracy against known-good results
2. **Performance Benchmarks**: Measure extraction speed and efficiency
3. **Integration Tests**: End-to-end validation with realistic content
4. **Memory Tests**: Validate memory usage patterns
5. **Cache Tests**: Measure AOT compilation performance improvements

## Coordination Protocol

The test suite coordinates results through:

- **Input**: Test fixtures and expectations from this directory
- **Output**: Test results written to `/hive/test-results/`
- **Reports**: Comprehensive reports generated at `/reports/last-run/wasm/`

## Updating Test Data

When updating test fixtures:

1. Update the fixture files in `/wasm/riptide-extractor-wasm/tests/fixtures/`
2. Update `manifest.json` with new expectations
3. Regenerate golden snapshots if needed
4. Update performance baselines if improvements are made

## Test Data Types

### News Site (`news_site.html`)
- **Type**: News article
- **Features**: Author, published date, categories
- **Expected Quality**: ≥85%
- **Performance Target**: <50ms extraction

### Blog Post (`blog_post.html`)
- **Type**: Technical blog post
- **Features**: Code blocks, table of contents, author bio
- **Expected Quality**: ≥80%
- **Performance Target**: <100ms extraction

### Gallery Site (`gallery_site.html`)
- **Type**: Photo gallery
- **Features**: Image metadata, photographer info, collection structure
- **Expected Quality**: ≥75%
- **Performance Target**: <80ms extraction

### Navigation Heavy (`nav_heavy_site.html`)
- **Type**: Application dashboard
- **Features**: Complex navigation, breadcrumbs, UI elements
- **Expected Quality**: ≥70%
- **Performance Target**: <60ms extraction

## Quality Metrics

Quality scores are calculated based on:

- **Content Extraction**: Accuracy of main content identification
- **Metadata Extraction**: Title, author, date, description accuracy
- **Link Extraction**: Relevant links identified and extracted
- **Media Extraction**: Images and media properly catalogued
- **Language Detection**: Correct language identification
- **Category Classification**: Appropriate content categorization

## Performance Baselines

Current performance baselines (updated with each release):

- **Average Extraction Time**: 15-25ms
- **Peak Memory Usage**: <128MB
- **Throughput**: 100+ ops/sec
- **Cache Hit Rate**: >80%
- **Memory Growth**: <0.01MB per operation

## Compatibility

Test data is compatible with:

- **WASM Extractor**: v0.1.0+
- **Test Suite**: v1.0.0+
- **Component Model**: v0.2.0+

Test data versioning ensures compatibility across different versions of the extractor.