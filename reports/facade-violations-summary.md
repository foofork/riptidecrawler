# Facade Layer Violations - Quick Reference

## 🔴 HIGH SEVERITY (Fix First)

### 1. HTTP Method Definition in Pipeline
**File:** `facades/pipeline.rs:441-445`
```rust
// ❌ HttpMethod enum (GET, POST) defined in facade
pub enum HttpMethod { Get, Post }
```
**Fix:** Move to `riptide-api/src/types.rs`, use domain operations

### 2. HTTP Headers in FetchOptions
**File:** `facades/pipeline.rs:427`
```rust
// ❌ Raw HTTP headers exposed
pub headers: Vec<(String, String)>
```
**Fix:** Replace with `metadata: HashMap<String, String>`

---

## 🟡 MEDIUM SEVERITY

### 3. JSON Serialization Throughout Pipeline
**Files:** `pipeline.rs`, `browser.rs`, `extractor.rs`
**Instances:** 37+
```rust
// ❌ serde_json::Value everywhere
async fn execute_fetch(...) -> RiptideResult<serde_json::Value>
pub struct PipelineResult { final_output: serde_json::Value }
```
**Fix:** Create domain types, move serialization to handlers

---

## Quick Check Commands

```bash
# Find HTTP types in facade
rg "HttpMethod|StatusCode|Request|Response" crates/riptide-facade/src/facades/

# Find JSON serialization
rg "serde_json::Value|::json\(" crates/riptide-facade/src/facades/

# Find transport headers
rg "headers.*Vec.*String.*String" crates/riptide-facade/src/
```

---

## Refactoring Priority

1. **Phase 1 (HIGH):** Remove HTTP types → 2-4 hours
2. **Phase 2 (HIGH):** Remove headers → 2-3 hours
3. **Phase 3 (MED):** Replace JSON → 8-12 hours
4. **Phase 4 (LOW):** Clean config → 1 hour

**Total Effort:** 13-20 hours

---

## Success Test

```bash
# These should return ZERO matches in facade/
rg "serde_json::Value" crates/riptide-facade/src/facades/
rg "HttpMethod" crates/riptide-facade/src/facades/
rg "headers.*Vec.*String" crates/riptide-facade/src/facades/
```

---

See `facade-layer-violations-analysis.md` for full details.
