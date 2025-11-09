# Quick Fix Guide - Remaining 7 Errors

## Error Summary
**Total:** 7 errors in 2 files (memory.rs, monitoring.rs)
**Time to fix:** 5-10 minutes
**Complexity:** LOW - simple field name corrections

---

## Fix 1: memory.rs (Lines 20-26)

### Current Code (WRONG):
```rust
pub async fn memory_profile_handler(State(_state): State<AppState>) -> Result<Json<MemoryUsageResponse>, ApiError> {
    Ok(Json(MemoryUsageResponse {
        allocated_mb: 0,     // ❌ Error
        resident_mb: 0,      // ❌ Error
        metadata_mb: 0,      // ❌ Error
    }))
}
```

### Fixed Code:
```rust
pub async fn memory_profile_handler(State(_state): State<AppState>) -> Result<Json<MemoryUsageResponse>, ApiError> {
    Ok(Json(MemoryUsageResponse {
        total_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        usage_percentage: 0.0,
        pressure_level: "normal".to_string(),
        recommendations: vec![],
    }))
}
```

**Edit command:**
```rust
old_string:
    Ok(Json(MemoryUsageResponse {
        allocated_mb: 0,
        resident_mb: 0,
        metadata_mb: 0,
    }))

new_string:
    Ok(Json(MemoryUsageResponse {
        total_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        usage_percentage: 0.0,
        pressure_level: "normal".to_string(),
        recommendations: vec![],
    }))
```

---

## Fix 2: monitoring.rs - HealthScoreResponse (Lines 33-37)

### Current Code (WRONG):
```rust
pub async fn get_health_score(State(_state): State<AppState>) -> Result<Json<HealthScoreResponse>, ApiError> {
    Ok(Json(HealthScoreResponse {
        score: 100,  // ❌ Error
        status: "healthy".to_string(),
    }))
}
```

### Fixed Code:
```rust
pub async fn get_health_score(State(_state): State<AppState>) -> Result<Json<HealthScoreResponse>, ApiError> {
    Ok(Json(HealthScoreResponse {
        health_score: 100.0,
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}
```

**Edit command:**
```rust
old_string:
    Ok(Json(HealthScoreResponse {
        score: 100,
        status: "healthy".to_string(),
    }))

new_string:
    Ok(Json(HealthScoreResponse {
        health_score: 100.0,
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
```

---

## Fix 3: monitoring.rs - PerformanceReportResponse (Lines 41-46)

### Current Code (WRONG):
```rust
pub async fn get_performance_report(State(_state): State<AppState>) -> Result<Json<PerformanceReportResponse>, ApiError> {
    Ok(Json(PerformanceReportResponse {
        uptime_seconds: 0,    // ❌ Error
        total_requests: 0,    // ❌ Error
        avg_latency_ms: 0.0,  // ❌ Error
    }))
}
```

### Fixed Code:
```rust
pub async fn get_performance_report(State(_state): State<AppState>) -> Result<Json<PerformanceReportResponse>, ApiError> {
    Ok(Json(PerformanceReportResponse {
        metrics: std::collections::HashMap::new(),
        summary: "No performance data available".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}
```

**Edit command:**
```rust
old_string:
    Ok(Json(PerformanceReportResponse {
        uptime_seconds: 0,
        total_requests: 0,
        avg_latency_ms: 0.0,
    }))

new_string:
    Ok(Json(PerformanceReportResponse {
        metrics: std::collections::HashMap::new(),
        summary: "No performance data available".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
```

---

## Validation Commands

After making all 3 fixes:

```bash
# Check compilation
cargo check -p riptide-api --lib

# Expected result:
# ✅ Compiling riptide-api v0.9.0
# ✅ Finished (warning: 256 warnings)
# ❌ 0 errors

# Count errors (should be 0)
cargo check -p riptide-api --lib 2>&1 | grep "^error" | wc -l

# Run tests
cargo test -p riptide-api --lib
```

---

## Agent 7 Instructions

**Mission:** Fix the 7 remaining type definition errors

**Tasks:**
1. Apply Fix 1 to `/workspaces/eventmesh/crates/riptide-api/src/handlers/memory.rs`
2. Apply Fix 2 to `/workspaces/eventmesh/crates/riptide-api/src/handlers/monitoring.rs`
3. Apply Fix 3 to `/workspaces/eventmesh/crates/riptide-api/src/handlers/monitoring.rs`
4. Run validation: `cargo check -p riptide-api --lib`
5. Verify 0 errors
6. Report completion

**Coordination:**
```bash
npx claude-flow@alpha hooks pre-task --description "Fix remaining 7 stub handler errors"
# ... make fixes ...
npx claude-flow@alpha hooks post-task --task-id "stub-handler-fixes"
npx claude-flow@alpha hooks notify --message "Agent 7: All 7 errors fixed - riptide-api compiles with 0 errors"
```

---

## Type Reference

### MemoryUsageResponse (from facade):
```rust
pub struct MemoryUsageResponse {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percentage: f32,
    pub pressure_level: String,
    pub recommendations: Vec<String>,
}
```

### HealthScoreResponse (from facade):
```rust
pub struct HealthScoreResponse {
    pub health_score: f32,
    pub status: String,
    pub timestamp: String,
}
```

### PerformanceReportResponse (from facade):
```rust
pub struct PerformanceReportResponse {
    pub metrics: std::collections::HashMap<String, f64>,
    pub summary: String,
    pub timestamp: String,
}
```

---

**Created by:** Agent 6 - Validation Coordinator
**Date:** 2025-11-09
**Status:** Ready for Agent 7 execution
