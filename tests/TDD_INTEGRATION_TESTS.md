# TDD Integration Tests for Missing API Endpoints

This document describes the comprehensive Test-Driven Development (TDD) integration tests created for the missing API endpoints in RipTide API. These tests are designed to **FAIL FIRST** (RED phase of TDD) until the corresponding implementations are completed.

## 📍 Test File Location
`/workspaces/eventmesh/crates/riptide-api/tests/integration_tests.rs`

## 🔴 RED Phase - Tests That Will Fail

These tests document the expected behavior and API contracts for endpoints that need to be implemented:

### 1. Table Extraction API Tests (`table_extraction_tests`)

**Missing Endpoints:**
- `POST /api/v1/tables/extract` - Extract tables from HTML content or URLs
- `GET /api/v1/tables/{id}/export?format=csv` - Export tables as CSV
- `GET /api/v1/tables/{id}/export?format=markdown` - Export tables as Markdown

**Test Coverage:**
- ✅ `test_table_extraction_from_html()` - Extract tables from HTML content
- ✅ `test_table_extraction_from_url()` - Extract tables by fetching from URL
- ✅ `test_table_csv_export()` - Export extracted tables as CSV format
- ✅ `test_table_markdown_export()` - Export extracted tables as Markdown
- ✅ `test_complex_table_span_handling()` - Handle colspan/rowspan attributes
- ✅ `test_table_extraction_edge_cases()` - Error handling and edge cases

**Expected Features:**
- HTML table detection and parsing
- Metadata extraction (table ID, dimensions, headers)
- Complex table structure handling (spans)
- Multiple export formats with proper headers
- Performance optimization for large tables

### 2. LLM Provider Management Tests (`llm_provider_tests`)

**Missing Endpoints:**
- `GET /api/v1/llm/providers` - List available LLM providers
- `GET /api/v1/llm/providers/current` - Get current active provider
- `POST /api/v1/llm/providers/switch` - Switch to different provider
- `GET /api/v1/llm/config` - Get provider configuration
- `POST /api/v1/llm/config` - Update provider configuration

**Test Coverage:**
- ✅ `test_list_llm_providers()` - List all available providers with capabilities
- ✅ `test_get_current_llm_provider()` - Get current active provider info
- ✅ `test_switch_llm_provider()` - Switch between providers with validation
- ✅ `test_invalid_provider_switch()` - Error handling for invalid switches
- ✅ `test_llm_provider_configuration()` - Configuration management
- ✅ `test_llm_failover_chain()` - Automatic failover configuration
- ✅ `test_llm_provider_health_monitoring()` - Health and status monitoring

**Expected Features:**
- Support for multiple LLM providers (OpenAI, Anthropic, local models)
- Dynamic provider switching with validation
- Failover chain configuration
- Provider health monitoring and statistics
- Configuration validation and management

### 3. Advanced Chunking Configuration Tests (`advanced_chunking_tests`)

**Missing Features:**
- `chunking_mode` parameter in existing `/crawl` endpoint
- `POST /api/v1/content/chunk` - Standalone content chunking
- Topic-based chunking with TextTiling algorithm
- Sliding window chunking with overlap
- Performance requirements (<200ms for standard content)

**Test Coverage:**
- ✅ `test_crawl_with_chunking_strategy()` - Integration with crawl endpoint
- ✅ `test_topic_based_chunking()` - TextTiling algorithm implementation
- ✅ `test_sliding_window_chunking()` - Overlapping chunks with windows
- ✅ `test_chunking_performance()` - Performance under 200ms requirement
- ✅ `test_chunking_configuration_validation()` - Parameter validation
- ✅ `test_chunking_pipeline_integration()` - Integration with extraction
- ✅ `test_chunking_content_types()` - Support for HTML/Markdown/text

**Expected Features:**
- Multiple chunking algorithms (topic, sliding window, semantic)
- Configurable chunk size and overlap parameters
- Content type-aware chunking (HTML structure preservation)
- Performance optimization with metrics
- Integration with existing crawl pipeline

### 4. Integration Workflow Tests (`integration_workflow_tests`)

**End-to-End Scenarios:**
- ✅ `test_table_extraction_llm_analysis_workflow()` - Crawl → Extract tables → LLM analysis
- ✅ `test_llm_enhanced_chunking_workflow()` - LLM-powered topic detection
- ✅ `test_llm_failover_scenario()` - Automatic provider failover
- ✅ `test_concurrent_request_performance()` - Load testing and concurrency

## 🛠️ Implementation Requirements

### Core Components Needed:

1. **Table Extraction Engine**
   - HTML parsing with table detection
   - Span handling (colspan/rowspan)
   - Export formatters (CSV, Markdown)
   - Table metadata extraction

2. **LLM Provider Management System**
   - Provider registry and configuration
   - Dynamic switching mechanism
   - Health monitoring and failover
   - Usage statistics tracking

3. **Advanced Chunking System**
   - TextTiling algorithm implementation
   - Sliding window with overlap
   - Content type-aware processing
   - Performance optimization

4. **API Route Integration**
   - New endpoint handlers
   - Request/response models
   - Error handling and validation
   - Integration with existing middleware

## 🚀 Running the Tests (RED Phase)

These tests are designed to FAIL until implementations are complete:

```bash
# Run all integration tests (will fail)
cd /workspaces/eventmesh/crates/riptide-api
cargo test integration_tests

# Run specific test modules
cargo test table_extraction_tests
cargo test llm_provider_tests
cargo test advanced_chunking_tests
cargo test integration_workflow_tests
```

## 📋 TDD Workflow

1. **RED Phase** ✅ - Tests written and failing (current state)
2. **GREEN Phase** 🔄 - Implement minimal code to pass tests
3. **REFACTOR Phase** 🔄 - Optimize and clean up implementation

## 📊 Test Metrics and Performance Requirements

- **Chunking Performance**: < 200ms for standard documents (5KB)
- **Table Extraction**: Handle complex tables with spans
- **LLM Failover**: < 5 second failover time
- **Concurrent Requests**: Support 10+ simultaneous requests
- **Memory Usage**: Efficient processing without memory leaks

## 🔧 Test Utilities

The tests include comprehensive utilities in `test_utils` module:
- `create_test_app()` - Test app factory with all routes
- `make_json_request()` - HTTP request helper with JSON parsing
- `sample_html_with_tables()` - Test HTML content with various table structures
- `sample_long_text()` - Test content for chunking algorithms

## 📝 Next Steps

1. Implement the missing API endpoints one by one
2. Run tests to verify implementation correctness
3. Optimize performance to meet requirements
4. Add additional edge case handling as needed
5. Integration with existing codebase and middleware

---

**Note**: These tests represent a comprehensive specification of the missing functionality. They serve as both documentation and verification criteria for the implementation work ahead.