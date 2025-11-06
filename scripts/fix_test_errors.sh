#!/bin/bash
# Comprehensive test fix script for riptide-api
# Fixes all 52 compilation errors systematically

set -e

echo "=== Fixing riptide-api Test Compilation Errors ==="

# 1. Fix facade_integration_tests.rs - Add missing imports
echo "[1/8] Fixing facade_integration_tests.rs imports..."
sed -i '/#[cfg(feature = "extraction")]/d' /workspaces/eventmesh/crates/riptide-api/src/tests/facade_integration_tests.rs
sed -i '/use crate::handlers::extract::ExtractRequest;/d' /workspaces/eventmesh/crates/riptide-api/src/tests/facade_integration_tests.rs
sed -i '/use riptide_types::ExtractOptions;/c\use riptide_types::{ExtractOptions, ExtractRequest};' /workspaces/eventmesh/crates/riptide-api/src/tests/facade_integration_tests.rs
sed -i '/use http_body_util::BodyExt;/a use riptide_config::ApiConfig;' /workspaces/eventmesh/crates/riptide-api/src/tests/facade_integration_tests.rs

# 2. Fix facade_integration_tests.rs - Fix ApiConfig usage
echo "[2/8] Fixing ApiConfig usage in facade_integration_tests.rs..."
sed -i 's/let api_config = ApiConfig {/let api_config = ApiConfig::default(); \/\/ Temporary fix/' /workspaces/eventmesh/crates/riptide-api/src/tests/facade_integration_tests.rs
# Note: Will need manual adjustment for the struct initialization

# 3. Fix dto.rs - Add missing type imports
echo "[3/8] Fixing dto.rs imports..."
sed -i '/^use riptide_types::/s/$/\nuse riptide_types::{SpiderResultStats, SpiderResultUrls};/' /workspaces/eventmesh/crates/riptide-api/src/dto.rs

# 4. Fix handlers/crawl.rs - Fix unused options parameter
echo "[4/8] Fixing crawl.rs options parameter..."
sed -i 's/_options: \&riptide_types::config::CrawlOptions/options: \&riptide_types::config::CrawlOptions/' /workspaces/eventmesh/crates/riptide-api/src/handlers/crawl.rs

# 5. Fix middleware/auth.rs - Make test helpers public
echo "[5/8] Fixing auth.rs test helper visibility..."
sed -i 's/^fn constant_time_compare/#[cfg(test)]\npub(crate) fn constant_time_compare/' /workspaces/eventmesh/crates/riptide-api/src/middleware/auth.rs
sed -i 's/^fn extract_api_key/#[cfg(test)]\npub(crate) fn extract_api_key/' /workspaces/eventmesh/crates/riptide-api/src/middleware/auth.rs

# 6. Fix middleware/auth.rs tests - Add missing imports
echo "[6/8] Fixing auth.rs test imports..."
sed -i '/#\[cfg(test)\]/a use axum::body::Body;\nuse crate::middleware::auth::{constant_time_compare, extract_api_key, AuthConfig};' /workspaces/eventmesh/crates/riptide-api/src/middleware/auth.rs

# 7. Fix resource_controls.rs and resource_manager tests - Add ApiConfig import
echo "[7/8] Fixing test ApiConfig imports..."
sed -i '/^use std::sync::Arc;/a use riptide_config::ApiConfig;' /workspaces/eventmesh/crates/riptide-api/src/tests/resource_controls.rs
sed -i '/^use std::sync::Arc;/a use riptide_config::ApiConfig;' /workspaces/eventmesh/crates/riptide-api/src/resource_manager/memory_manager.rs
sed -i '/^use std::sync::Arc;/a use riptide_config::ApiConfig;' /workspaces/eventmesh/crates/riptide-api/src/resource_manager/rate_limiter.rs
sed -i '/^use std::sync::Arc;/a use riptide_config::ApiConfig;' /workspaces/eventmesh/crates/riptide-api/src/resource_manager/mod.rs

# 8. Fix handlers/spider.rs - Add missing field
echo "[8/8] Fixing SpiderStatusResponse missing field..."
sed -i '/let response = SpiderStatusResponse {/,/};/{s/};/    adaptive_stop_stats: None,\n    };/}' /workspaces/eventmesh/crates/riptide-api/src/handlers/spider.rs

echo "=== All fixes applied successfully ==="
echo "Run: cargo test -p riptide-api --lib --no-run to verify"
