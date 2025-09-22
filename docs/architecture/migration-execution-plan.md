# WASM Component Model Migration Execution Plan

## Quick Start Implementation Guide

This document provides the step-by-step execution plan for migrating RipTide's WASM extractor from WASI command interface to Component Model.

## Pre-Migration Checklist

- [ ] Verify wasmtime 26.x is available (✅ Already configured)
- [ ] Confirm Component Model support in build environment
- [ ] Back up current WASM implementation
- [ ] Set up testing infrastructure

## Phase 1: Foundation Setup (Days 1-3)

### Day 1: Build Configuration

1. **Create Cargo configuration file**
   ```bash
   mkdir -p .cargo
   cp docs/architecture/cargo-config.toml .cargo/config.toml
   ```

2. **Update WASM component Cargo.toml**
   ```toml
   # In wasm/riptide-extractor-wasm/Cargo.toml
   [dependencies]
   wit-bindgen = "0.30"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   trek-rs = "0.2.1"  # Re-enable with correct version

   [profile.release]
   opt-level = "s"
   lto = true
   codegen-units = 1
   panic = "abort"
   ```

3. **Verify Component Model build**
   ```bash
   cd wasm/riptide-extractor-wasm
   cargo build --target wasm32-wasip2 --release
   ```

### Day 2: Enhanced WIT Interface

1. **Replace basic WIT interface**
   ```bash
   cp docs/architecture/enhanced-extractor.wit wasm/riptide-extractor-wasm/wit/extractor.wit
   cp docs/architecture/enhanced-extractor.wit crates/riptide-core/wit/extractor.wit
   ```

2. **Update package versions**
   - Ensure both WIT files use `riptide:extractor@0.2.0`
   - Verify world definitions match

3. **Test WIT compilation**
   ```bash
   wit-bindgen rust wit/extractor.wit --out-dir src/generated
   ```

### Day 3: Trek-rs Integration

1. **Update workspace dependencies**
   ```toml
   # In root Cargo.toml [workspace.dependencies]
   trek-rs = "0.2.1"
   ```

2. **Test trek-rs compatibility**
   ```bash
   cargo check --package riptide-extractor-wasm
   ```

## Phase 2: Core Implementation (Days 4-7)

### Day 4: Guest-Side Implementation

1. **Replace guest implementation**
   ```bash
   cp docs/architecture/enhanced-guest-implementation.rs wasm/riptide-extractor-wasm/src/lib.rs
   ```

2. **Add build script for metadata**
   ```rust
   // Create wasm/riptide-extractor-wasm/build.rs
   use std::process::Command;

   fn main() {
       println!("cargo:rustc-env=BUILD_TIMESTAMP={}",
           chrono::Utc::now().to_rfc3339());

       if let Ok(output) = Command::new("git")
           .args(&["rev-parse", "--short", "HEAD"])
           .output() {
           let git_hash = String::from_utf8(output.stdout).unwrap_or_default();
           println!("cargo:rustc-env=GIT_COMMIT={}", git_hash.trim());
       }
   }
   ```

3. **Add required dependencies**
   ```toml
   # In wasm/riptide-extractor-wasm/Cargo.toml
   [build-dependencies]
   chrono = { version = "0.4", features = ["serde"] }
   ```

### Day 5: Host-Side Integration

1. **Enhance component module**
   ```bash
   cp docs/architecture/enhanced-host-integration.rs crates/riptide-core/src/component.rs
   ```

2. **Update core dependencies**
   ```toml
   # In crates/riptide-core/Cargo.toml
   [dependencies]
   uuid = { workspace = true }
   tokio = { workspace = true, features = ["sync", "time"] }
   ```

3. **Add new types module entries**
   ```rust
   // In crates/riptide-core/src/types.rs
   #[derive(Debug, Clone)]
   pub enum ExtractionMode {
       Article,
       Full,
       Metadata,
       Custom(Vec<String>),
   }

   #[derive(Debug, Clone)]
   pub struct ExtractionStats {
       pub processing_time_ms: u64,
       pub memory_used: u64,
       pub nodes_processed: Option<u32>,
       pub links_found: u32,
       pub images_found: u32,
   }
   ```

### Day 6: Testing Infrastructure

1. **Create integration tests**
   ```rust
   // Create crates/riptide-core/tests/component_model_tests.rs
   use riptide_core::component::CmExtractor;
   use riptide_core::types::ExtractionMode;

   #[tokio::test]
   async fn test_component_model_extraction() {
       let config = Default::default();
       let wasm_path = "../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";

       let extractor = CmExtractor::new(wasm_path, config).await.unwrap();

       let html = r#"
           <html>
               <head><title>Test Article</title></head>
               <body>
                   <article>
                       <h1>Main Title</h1>
                       <p>Article content goes here.</p>
                   </article>
               </body>
           </html>
       "#;

       let result = extractor
           .extract(html, "https://example.com/test", ExtractionMode::Article)
           .await;

       assert!(result.is_ok());
       let doc = result.unwrap();
       assert_eq!(doc.title, Some("Test Article".to_string()));
       assert!(doc.text.contains("Article content"));
   }
   ```

2. **Create performance benchmarks**
   ```rust
   // Create crates/riptide-core/benches/component_model_bench.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   use riptide_core::component::CmExtractor;
   use riptide_core::types::ExtractionMode;

   async fn bench_extraction(extractor: &CmExtractor, html: &str) {
       let _ = extractor
           .extract(html, "https://bench.com", ExtractionMode::Article)
           .await;
   }

   fn component_model_benchmark(c: &mut Criterion) {
       let rt = tokio::runtime::Runtime::new().unwrap();
       let extractor = rt.block_on(async {
           CmExtractor::new("../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm", Default::default()).await.unwrap()
       });

       let html = include_str!("../test_data/sample_article.html");

       c.bench_function("component_model_extract", |b| {
           b.to_async(&rt).iter(|| bench_extraction(&extractor, black_box(html)))
       });
   }

   criterion_group!(benches, component_model_benchmark);
   criterion_main!(benches);
   ```

### Day 7: Error Handling & Validation

1. **Test error scenarios**
   ```rust
   #[tokio::test]
   async fn test_error_handling() {
       let extractor = create_test_extractor().await;

       // Test empty HTML
       let result = extractor.extract("", "https://example.com", ExtractionMode::Article).await;
       assert!(result.is_err());

       // Test invalid URL
       let result = extractor.extract("<html></html>", "", ExtractionMode::Article).await;
       assert!(result.is_err());

       // Test malformed HTML
       let result = extractor.extract("<html><body><unclosed tag", "https://example.com", ExtractionMode::Article).await;
       // Should handle gracefully
   }
   ```

2. **Validate resource limits**
   ```rust
   #[tokio::test]
   async fn test_resource_limits() {
       let config = ExtractorConfig {
           memory_limit: 1024 * 1024, // 1MB limit
           extraction_timeout: Duration::from_secs(5),
           ..Default::default()
       };

       let extractor = CmExtractor::new(wasm_path, config).await.unwrap();

       // Test with large document
       let large_html = "<html><body>".to_string() + &"<p>content</p>".repeat(100000) + "</body></html>";
       let result = extractor.extract(&large_html, "https://example.com", ExtractionMode::Article).await;

       // Should either succeed within limits or fail gracefully
       match result {
           Ok(_) => println!("Large document processed successfully"),
           Err(e) => println!("Large document rejected as expected: {}", e),
       }
   }
   ```

## Phase 3: Performance & Production (Days 8-10)

### Day 8: Performance Optimization

1. **Enable SIMD optimizations**
   ```bash
   export RUSTFLAGS="-C target-feature=+simd128"
   cargo build --target wasm32-wasip2 --release
   ```

2. **Profile memory usage**
   ```bash
   # Use wasmtime profiling
   wasmtime run --profile=jitdump target/wasm32-wasip2/release/riptide_extractor_wasm.wasm
   ```

3. **Benchmark against current implementation**
   ```bash
   cargo bench --package riptide-core
   ```

### Day 9: Production Configuration

1. **Configure instance pooling**
   ```rust
   let production_config = ExtractorConfig {
       max_instances: 32,
       extraction_timeout: Duration::from_secs(15),
       memory_limit: 128 * 1024 * 1024, // 128MB
       fuel_limit: 50_000_000,
       warmup_instances: 8,
       enable_simd: true,
       optimization_level: wasmtime::OptLevel::Speed,
       ..Default::default()
   };
   ```

2. **Set up monitoring**
   ```rust
   // Add metrics collection
   let metrics = extractor.get_metrics().await;
   tracing::info!("Extraction metrics: {:?}", metrics);
   ```

3. **Configure logging**
   ```toml
   # In Cargo.toml
   [dependencies]
   tracing = { workspace = true }
   tracing-subscriber = { workspace = true }
   ```

### Day 10: Integration & Testing

1. **Integration with main application**
   ```rust
   // In main application
   let extractor = Arc::new(
       CmExtractor::new(&wasm_path, production_config).await?
   );

   // Use in request handlers
   let doc = extractor.extract(&html, &url, ExtractionMode::Article).await?;
   ```

2. **Load testing**
   ```bash
   # Use your preferred load testing tool
   ab -n 1000 -c 10 http://localhost:8080/extract
   ```

3. **Regression testing**
   ```bash
   cargo test --package riptide-core --test component_model_tests
   ```

## Migration Commands Summary

```bash
# Phase 1: Setup
mkdir -p .cargo
cp docs/architecture/cargo-config.toml .cargo/config.toml
cp docs/architecture/enhanced-extractor.wit wasm/riptide-extractor-wasm/wit/extractor.wit

# Phase 2: Implementation
cp docs/architecture/enhanced-guest-implementation.rs wasm/riptide-extractor-wasm/src/lib.rs
cp docs/architecture/enhanced-host-integration.rs crates/riptide-core/src/component.rs

# Phase 3: Build & Test
cargo build --target wasm32-wasip2 --release
cargo test --package riptide-core
cargo bench --package riptide-core
```

## Rollback Plan

If issues occur, quickly rollback by:

1. **Revert git changes**
   ```bash
   git checkout HEAD~1 -- wasm/riptide-extractor-wasm/
   git checkout HEAD~1 -- crates/riptide-core/src/component.rs
   ```

2. **Rebuild old version**
   ```bash
   cargo build --target wasm32-wasi --release
   ```

3. **Update configuration**
   ```rust
   // Revert to WASI command interface
   use wasmtime_wasi::WasiCtxBuilder;
   ```

## Success Metrics

Track these metrics to validate migration success:

- **Performance**: Extraction speed improvement (target: 2x faster)
- **Memory**: Memory usage reduction (target: 50% less)
- **Reliability**: Error rate reduction (target: <1% for valid inputs)
- **Concurrency**: Concurrent extraction capacity (target: 100+ simultaneous)

## Monitoring Checklist

- [ ] Extraction latency metrics
- [ ] Memory usage tracking
- [ ] Error rate monitoring
- [ ] Pool utilization stats
- [ ] Instance lifecycle events
- [ ] Resource limit violations

## Post-Migration Tasks

1. **Documentation updates**
   - Update API documentation
   - Create usage examples
   - Update deployment guides

2. **Team training**
   - Component Model concepts
   - New error handling patterns
   - Performance optimization tips

3. **Monitoring setup**
   - Dashboards for new metrics
   - Alerts for performance degradation
   - Log aggregation for errors

This execution plan provides a systematic approach to migrating to WASM Component Model while minimizing risk and ensuring reliable operation.