# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Standardized on `wasm32-wasip2` target exclusively (removed all `wasip1` support)
- Consolidated WASM validation logic to eliminate code duplication
- Refactored validation functions into reusable `validate_input()` and `validate_html_structure()`
- Updated build scripts to use only `wasip2` target
- Fixed LTO configuration conflict by using `lto=thin` instead of `lto=fat`

### Removed
- Removed `wasm32-wasip1` target configuration from `.cargo/config.toml`
- Removed `wasip1` target from all build scripts
- Removed duplicate HTML validation logic in WASM extractor

## [0.1.0] - 2025-09-22

### Added
- Initial Component Model WASM extractor with typed interface
- Host integration with CmExtractor for wasmtime Component Model
- Headless browser rendering service using chromiumoxide
- REST API with crawling and deep search endpoints
- Redis caching layer for extracted content
- Docker-based infrastructure with compose setup
- CI/CD pipeline with automated builds and tests
- Component Model migration from WASI-stdin to typed exports

### Changed
- Standardized on wasm32-wasip2 target (removed wasip1 support)
- Replaced WASI I/O plumbing with typed function exports
- Updated build scripts and CI for Component Model support

### Technical Details
- WASM Component: `extract(html: string, url: string, mode: string) -> string`
- Host Integration: wasmtime Component Model with bindgen
- Architecture: Microservices with API, headless, workers, and core
- Testing: Unit tests with ignored integration tests for WASM-backed features