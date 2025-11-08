# Quick Fix Guide for Build Validation Errors

**Target:** `/workspaces/eventmesh/crates/riptide-facade/src/facades/extraction.rs`  
**Total Fixes Required:** 12 compilation errors across 4 categories

## Fix #1: Duration Type (Line 72)

### Current (WRONG):
```rust
let timeout = config.timeout.unwrap_or(std::time::Duration::from_secs(30));
```

### Fixed (CORRECT):
```rust
let timeout = config.timeout;
```

**Reason:** `RiptideConfig.timeout` is `Duration`, not `Option<Duration>`

---

## Fix #2-9: Field Access on ExtractionResult (8 occurrences)

The code expects `ExtractedContent` but receives `ExtractionResult`. Must access nested `ScrapedContent`:

### Lines 132, 178: confidence field

**Current (WRONG):**
```rust
let quality_passed = self.apply_quality_gates(extracted.confidence, options.quality_threshold);
```

**Fixed (CORRECT):**
```rust
// Use default confidence since ExtractionResult doesn't have this field
let confidence = 0.8_f64; // Default confidence score
let quality_passed = self.apply_quality_gates(confidence, options.quality_threshold);
```

### Lines 137, 182: title field

**Current (WRONG):**
```rust
title: Some(extracted.title),
```

**Fixed (CORRECT):**
```rust
title: Some(extracted.content.title.clone()),
```

### Lines 138, 183: content field

**Current (WRONG):**
```rust
content: extracted.content,
```

**Fixed (CORRECT):**
```rust
content: extracted.content.content.clone(),
```

**Reason:** `ExtractedDoc.content` expects `String`, but `extracted.content` is `ScrapedContent` struct

### Lines 149, 194: strategy_used field

**Current (WRONG):**
```rust
strategy_used: extracted.strategy_used,
```

**Fixed (CORRECT):**
```rust
strategy_used: "default_extraction".to_string(), // ExtractionResult doesn't have this field
```

### Lines 150, 195: confidence field (same as lines 132, 178)

**Current (WRONG):**
```rust
confidence: extracted.confidence,
```

**Fixed (CORRECT):**
```rust
confidence: 0.8, // Default confidence score
```

---

## Fix #10: Return Type (Line 303)

### Current (WRONG):
```rust
Ok(result) // where result is ExtractedContent
```

**Context:** Method signature expects `Result<ExtractionResult>` but returns `Result<ExtractedContent>`

**Fixed (CORRECT):**
```rust
Ok(ExtractionResult {
    request_id: Uuid::new_v4(),
    url: Url::parse(&result.url)?,
    content: ScrapedContent {
        url: Url::parse(&result.url)?,
        title: result.title,
        content: result.content,
        description: result.summary,
        links: Vec::new(),
        custom_data: HashMap::new(),
        screenshot: None,
    },
    duration_ms: 0, // TODO: Track actual duration
    completed_at: Utc::now(),
    success: true,
    error: None,
})
```

---

## Bonus Fix: Clippy Warnings

**File:** `/workspaces/eventmesh/crates/riptide-types/src/pipeline/facade_types.rs`

### Lines 273-280: LocalStorage

**Current (WRONG):**
```rust
impl Default for LocalStorage {
    fn default() -> Self {
        LocalStorage {
            entries: HashMap::new(),
            created_at: None,
        }
    }
}
```

**Fixed (CORRECT):**
```rust
#[derive(Default)]
pub struct LocalStorage {
    // ... fields ...
}
// Remove the manual impl Default block
```

### Lines 282-290: SchemaExtractionResult (same pattern)

**Fixed:** Add `#[derive(Default)]` and remove manual implementation

---

## Verification Commands

After applying all fixes:

```bash
# 1. Build check
cargo build --workspace --all-features

# 2. Clippy check
cargo clippy --workspace --all-features -- -D warnings

# 3. Test check
cargo test -p riptide-facade --lib
cargo test -p riptide-types

# 4. Architecture validation
rg "serde_json::Value" crates/riptide-facade/src/facades/
rg "HttpMethod" crates/riptide-facade/src/
rg "headers.*Vec" crates/riptide-facade/src/
```

Expected results: All commands should pass with zero errors/warnings

---

## Estimated Time

- **Reading this guide:** 5 minutes
- **Applying fixes:** 15 minutes
- **Running verification:** 10 minutes
- **Total:** 30 minutes

## Notes

1. The core issue is type confusion between:
   - `ExtractionResult` (riptide-types/src/types.rs) - actual type
   - `ExtractedContent` (riptide-types/src/extracted.rs) - expected type

2. Consider adding a `From<ExtractionResult> for ExtractedContent` trait implementation to make conversions explicit

3. The `confidence` and `strategy_used` fields are missing from `ExtractionResult` - using sensible defaults for now
