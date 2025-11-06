# Types Rule Violations Report - riptide-types Crate

**Report Date:** 2025-11-06
**Crate:** `crates/riptide-types`
**Total Lines:** 3,235

## Executive Summary

Analysis of `riptide-types` crate reveals **MAJOR VIOLATIONS** of the "Types Rule" principle. The crate contains significant business logic, calculations, and side effects that should be moved to domain or facade layers.

### Violation Summary

| Severity | Count | Category |
|----------|-------|----------|
| 🔴 **CRITICAL** | 8 | Business logic in types |
| 🟡 **MAJOR** | 6 | Complex calculations |
| 🟢 **MINOR** | 12 | Acceptable helpers |

---

## 🔴 CRITICAL VIOLATIONS

### 1. **Circuit Breaker State Management** (`reliability/circuit.rs`)

**Lines:** 75-201 (127 lines of implementation logic)

**Violation:** Complex state machine with business logic, atomics, and side effects.

```rust
// ❌ MAJOR VIOLATION: Business logic in types
impl CircuitBreaker {
    pub fn try_acquire(&self) -> Result<Option<OwnedSemaphorePermit>, &'static str> {
        match self.state() {
            State::Closed => Ok(None),
            State::Open => {
                let now = self.clock.now_ms();
                let open_until = self.open_until_ms.load(Relaxed);
                if now >= open_until {
                    // transition Open -> HalfOpen
                    self.state.store(State::HalfOpen as u8, Relaxed);
                } else {
                    return Err("circuit open");
                }
                // fallthrough to HalfOpen path
                self.try_acquire()
            }
            State::HalfOpen => match Arc::clone(&self.half_open_permits).try_acquire_owned() {
                Ok(permit) => Ok(Some(permit)),
                Err(_) => Err("half-open saturated"),
            },
        }
    }

    pub fn on_success(&self) { /* 19 lines of state management */ }
    pub fn on_failure(&self) { /* 14 lines of state management */ }
    fn trip_open(&self) { /* 13 lines of state management */ }
}

// Helper function with business logic
pub async fn guarded_call<T, E, F, Fut>(cb: &Arc<CircuitBreaker>, f: F)
    -> Result<T, anyhow::Error> { /* 18 lines */ }
```

**Rule Violated:**
- ❌ Business logic (state transitions, failure threshold checks)
- ❌ Side effects (atomic operations, state modifications)
- ❌ Complex coordination logic

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/reliability/circuit_breaker.rs
// Keep in types:
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub open_cooldown_ms: u64,
    pub half_open_max_in_flight: u32,
}

pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

**Impact:** HIGH - This is a complete reliability pattern implementation masquerading as a type definition.

---

### 2. **HTTP Conditional Request Logic** (`conditional.rs`)

**Lines:** 76-113 (38 lines of validation logic)

**Violation:** Complex validation and comparison logic for HTTP caching.

```rust
// ❌ VIOLATION: Business logic for cache validation
impl ConditionalResponse {
    pub fn check_conditions(&mut self, request: &ConditionalRequest) -> bool {
        let mut not_modified = false;

        // Check If-None-Match (ETag)
        if let (Some(client_etag), Some(response_etag)) = (&request.if_none_match, &self.etag) {
            if client_etag == "*" || client_etag == response_etag {
                not_modified = true;
            }
        }

        // Check If-Modified-Since
        if let (Some(if_modified), Some(last_modified)) =
            (&request.if_modified_since, &self.last_modified)
        {
            if *last_modified <= *if_modified {
                not_modified = true;
            }
        }

        // Check If-Match (for unsafe methods)
        if let (Some(client_etag), Some(response_etag)) = (&request.if_match, &self.etag) {
            if client_etag != "*" && client_etag != response_etag {
                not_modified = true;
            }
        }

        // Check If-Unmodified-Since
        if let (Some(if_unmodified), Some(last_modified)) =
            (&request.if_unmodified_since, &self.last_modified)
        {
            if *last_modified > *if_unmodified {
                not_modified = true;
            }
        }

        self.not_modified = not_modified;
        not_modified
    }
}
```

**Rule Violated:**
- ❌ Business rules (HTTP caching specification logic)
- ❌ Complex branching and validation
- ❌ Side effects (mutates self.not_modified)

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/http/conditional_validator.rs
// Keep in types only:
pub struct ConditionalRequest {
    pub if_none_match: Option<String>,
    pub if_modified_since: Option<DateTime<Utc>>,
    pub if_match: Option<String>,
    pub if_unmodified_since: Option<DateTime<Utc>>,
}

pub struct ConditionalResponse {
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub not_modified: bool,
    pub cache_control: Option<String>,
}
```

---

### 3. **ETag Generation** (`conditional.rs`)

**Lines:** 123-128

**Violation:** Cryptographic hashing logic.

```rust
// ❌ VIOLATION: Calculation logic with side effects
pub fn generate_etag(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hasher.finalize();
    format!("{:x}", hash)[..16].to_string() // Use first 16 chars for brevity
}

pub fn generate_weak_etag(content: &[u8]) -> String {
    format!("W/\"{}\"", generate_etag(content))
}
```

**Rule Violated:**
- ❌ Complex calculations (SHA-256 hashing)
- ❌ Business rules (weak vs strong ETag semantics)

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/http/etag_generator.rs
```

---

### 4. **HTTP Date Parsing** (`conditional.rs`)

**Lines:** 136-161 (26 lines)

**Violation:** Complex parsing logic with multiple format attempts.

```rust
// ❌ VIOLATION: Parsing business logic
pub fn parse_http_date(date_str: &str) -> Option<DateTime<Utc>> {
    // Try RFC 2822 format first
    if let Ok(date) = DateTime::parse_from_rfc2822(date_str) {
        return Some(date.with_timezone(&Utc));
    }

    // Try RFC 3339 format
    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        return Some(date.with_timezone(&Utc));
    }

    // Try common HTTP date formats
    let formats = [
        "%a, %d %b %Y %H:%M:%S GMT", // RFC 1123
        "%A, %d-%b-%y %H:%M:%S GMT", // RFC 1036
        "%a %b %d %H:%M:%S %Y",      // ANSI C asctime()
    ];

    for format in &formats {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
            return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
        }
    }

    None
}

pub fn format_http_date(date: DateTime<Utc>) -> String {
    date.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}
```

**Rule Violated:**
- ❌ Complex parsing logic
- ❌ Multiple fallback strategies
- ❌ HTTP specification knowledge

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/http/date_parser.rs
```

---

### 5. **Cache Validation Logic** (`conditional.rs`)

**Lines:** 180-205 (26 lines)

**Violation:** Multi-step validation algorithm.

```rust
// ❌ VIOLATION: Cache validation business logic
pub fn validate_cache(
    cached_etag: Option<&str>,
    cached_last_modified: Option<DateTime<Utc>>,
    server_etag: Option<&str>,
    server_last_modified: Option<DateTime<Utc>>,
) -> CacheValidation {
    // Check ETag first (more reliable)
    if let (Some(cached), Some(server)) = (cached_etag, server_etag) {
        if cached == server {
            return CacheValidation::Valid;
        } else {
            return CacheValidation::Stale;
        }
    }

    // Fall back to Last-Modified comparison
    if let (Some(cached), Some(server)) = (cached_last_modified, server_last_modified) {
        if cached >= server {
            return CacheValidation::Valid;
        } else {
            return CacheValidation::Stale;
        }
    }

    CacheValidation::Unknown
}
```

**Rule Violated:**
- ❌ Business rules (cache validation priority: ETag > Last-Modified)
- ❌ Domain logic (HTTP caching semantics)

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/cache/validator.rs
```

---

### 6. **Browser Timeout Helper** (`traits.rs`)

**Lines:** 140-148

**Violation:** Async orchestration logic.

```rust
// ❌ VIOLATION: Orchestration logic
pub async fn with_timeout<F, T>(timeout: Duration, future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(result) => result,
        Err(_) => Err(RiptideError::Timeout(timeout.as_millis() as u64)),
    }
}
```

**Rule Violated:**
- ❌ Orchestration/coordination logic
- ❌ Error handling logic (timeout wrapping)

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/orchestration/timeout_wrapper.rs
```

---

### 7. **Secret Redaction** (`secrets.rs`)

**Lines:** 85-111 (27 lines)

**Violation:** String manipulation and transformation logic.

```rust
// ❌ VIOLATION: Transformation logic
pub fn redact_secret(secret: &str) -> String {
    if secret.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = secret.chars().collect();
    if chars.len() <= 4 {
        format!("{}...", secret)
    } else {
        format!("{}...", chars[..4].iter().collect::<String>())
    }
}

pub fn redact_secrets(secrets: &[String]) -> Vec<String> {
    secrets.iter().map(|s| redact_secret(s)).collect()
}
```

**Rule Violated:**
- ❌ Transformation logic
- ❌ String manipulation algorithms
- ❌ Collection processing

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/security/redactor.rs
```

---

### 8. **RealClock Implementation** (`reliability/circuit.rs`)

**Lines:** 46-62

**Violation:** System time I/O and error handling.

```rust
// ❌ VIOLATION: I/O and side effects
impl Clock for RealClock {
    fn now_ms(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| {
                tracing::error!("System time is before Unix epoch: {}", e);
                // Fallback: return 0 for current time if system clock is broken
                // This allows the circuit breaker to continue functioning
                std::time::Duration::from_secs(0)
            });

        // Safe conversion: saturate to u64::MAX if duration exceeds u64 milliseconds
        // This handles the theoretical case where as_millis() returns u128 > u64::MAX
        u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
    }
}
```

**Rule Violated:**
- ❌ I/O operations (SystemTime::now())
- ❌ Side effects (logging via tracing)
- ❌ Error recovery logic

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/time/clock_impl.rs
// Keep only the trait in types:
pub trait Clock: Send + Sync + std::fmt::Debug {
    fn now_ms(&self) -> u64;
}
```

---

## 🟡 MAJOR VIOLATIONS

### 9. **Quality Score Calculation** (`extracted.rs`)

**Lines:** 61-68

```rust
// ❌ VIOLATION: Calculation logic
impl ExtractionQuality {
    pub fn overall_score(&self) -> f64 {
        (self.title_quality
            + self.content_quality
            + self.structure_score
            + self.metadata_completeness)
            / 4.0
    }
}
```

**Rule Violated:** Business calculation (weighted average)

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/extraction/quality_scorer.rs
```

---

### 10. **Content Truncation** (`http_types.rs`)

**Lines:** 252-267

```rust
// ❌ VIOLATION: Content manipulation logic
impl CrawledPage {
    pub fn truncate_content(&mut self, max_content_bytes: usize) {
        if let Some(content) = &mut self.content {
            if content.len() > max_content_bytes {
                content.truncate(max_content_bytes);
                self.truncated = Some(true);
            }
        }

        if let Some(markdown) = &mut self.markdown {
            if markdown.len() > max_content_bytes {
                markdown.truncate(max_content_bytes);
                self.truncated = Some(true);
            }
        }
    }
}
```

**Rule Violated:**
- ❌ Content manipulation
- ❌ Side effects (mutates multiple fields)

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/content/truncator.rs
```

---

### 11. **Error Classification Methods** (`error/riptide_error.rs`)

**Lines:** 94-124

```rust
// ❌ VIOLATION: Business logic for error classification
impl RiptideError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            RiptideError::Network(_) | RiptideError::Timeout(_) | RiptideError::BrowserOperation(_)
        )
    }

    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            RiptideError::InvalidUrl(_)
                | RiptideError::Configuration(_)
                | RiptideError::NotFound(_)
                | RiptideError::AlreadyExists(_)
                | RiptideError::PermissionDenied(_)
        )
    }

    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            RiptideError::BrowserInitialization(_)
                | RiptideError::BrowserOperation(_)
                | RiptideError::Extraction(_)
                | RiptideError::Cache(_)
                | RiptideError::Storage(_)
        )
    }
}
```

**Rule Violated:**
- ❌ Business rules (retry policies)
- ❌ HTTP semantics (4xx vs 5xx mapping)

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/error/classifier.rs
pub trait ErrorClassifier {
    fn is_retryable(&self) -> bool;
    fn is_client_error(&self) -> bool;
    fn is_server_error(&self) -> bool;
}
```

---

### 12. **ExtractedDoc Conversion** (`extracted.rs`)

**Lines:** 71-85

```rust
// ❌ VIOLATION: Transformation logic
impl From<BasicExtractedDoc> for ExtractedContent {
    fn from(doc: BasicExtractedDoc) -> Self {
        Self {
            title: doc.title.unwrap_or_else(|| "Untitled".to_string()),
            content: doc.text,
            summary: doc.description,
            url: doc.url,
            strategy_used: "wasm_extraction".to_string(),
            extraction_confidence: doc
                .quality_score
                .map(|score| score as f64 / 100.0_f64)
                .unwrap_or(0.8_f64),
        }
    }
}
```

**Rule Violated:**
- ❌ Transformation logic
- ❌ Default values and business rules

**Suggested Fix:**
```rust
// ✅ Move to: crates/riptide-domain/src/extraction/converter.rs
```

---

### 13-14. **Builder Pattern Methods**

Multiple builder-style methods that do more than simple field assignment:

- `ConditionalResponse::with_etag_from_content()` - Line 52-55 (calls generate_etag)
- `ComponentMeta::with_description()` - Line 41-44 (simple builder, acceptable)

**Rule Violated:** Builder methods that invoke complex logic

---

## 🟢 ACCEPTABLE (Not Violations)

The following are **acceptable** in a types crate:

1. **Simple constructors**: `ComponentId::new()`, `ExtractionRequest::new()`
2. **Getters**: `ComponentId::as_str()`, `SecretString::expose_secret()`
3. **Simple checks**: `ConditionalRequest::has_conditions()`
4. **Default implementations**: All `impl Default` blocks
5. **From/Into traits**: Simple field mapping conversions
6. **Derive macros**: `#[derive(Debug, Clone, Serialize, Deserialize)]`
7. **Type aliases**: `pub type Result<T> = ...`
8. **Simple property methods**: `SecretString::len()`, `SecretString::is_empty()`
9. **Marker trait implementations**: `CombinedPipelineExecutor`
10. **Enum `from` conversions**: `State::from(u8)`
11. **Simple formatters**: `impl fmt::Debug for SecretString` (delegating to helper)
12. **New() constructors**: `CrawledPage::new()`

---

## 📊 Detailed Statistics

### File-by-File Breakdown

| File | Total Lines | Logic Lines | Violation % |
|------|-------------|-------------|-------------|
| `reliability/circuit.rs` | 373 | ~150 | 🔴 40% |
| `conditional.rs` | 300 | ~120 | 🔴 40% |
| `secrets.rs` | 189 | ~30 | 🟡 16% |
| `traits.rs` | 455 | ~10 | 🟢 2% |
| `extracted.rs` | 184 | ~20 | 🟡 11% |
| `http_types.rs` | 300 | ~20 | 🟡 7% |
| `error/riptide_error.rs` | 164 | ~35 | 🟡 21% |
| `types.rs` | 198 | 0 | ✅ 0% |
| `component.rs` | 68 | 0 | ✅ 0% |
| `config.rs` | 149 | 0 | ✅ 0% |
| `pipeline/traits.rs` | 180 | 0 | ✅ 0% |
| `pipeline/results.rs` | 274 | 0 | ✅ 0% |

### Violation Density

```
Total Lines:           3,235
Logic Lines:           ~385
Violation Percentage:  11.9%
```

---

## 🎯 Recommendations

### Priority 1: IMMEDIATE (Critical)

1. **Extract Circuit Breaker** → `riptide-domain/src/reliability/`
   - Impact: HIGH - 150 lines of complex state machine logic
   - Blocks: Proper domain separation

2. **Extract HTTP Conditional Logic** → `riptide-domain/src/http/`
   - Impact: HIGH - 120 lines of HTTP caching specification
   - Blocks: Clean HTTP domain layer

3. **Extract Clock Implementation** → `riptide-domain/src/time/`
   - Impact: MEDIUM - I/O operations in types layer
   - Blocks: Proper I/O abstraction

### Priority 2: SHORT-TERM (Major)

4. **Extract Secret Redaction** → `riptide-domain/src/security/`
5. **Extract Quality Scoring** → `riptide-domain/src/extraction/`
6. **Extract Error Classification** → `riptide-domain/src/error/`

### Priority 3: MEDIUM-TERM (Minor)

7. Consolidate all transformation logic into domain converters
8. Review all `impl` blocks for hidden business logic
9. Add linting rules to prevent future violations

---

## 📝 Migration Strategy

### Step 1: Create Domain Modules
```bash
mkdir -p crates/riptide-domain/src/{reliability,http,time,security,extraction,error}
```

### Step 2: Move Implementations (Keep Contracts)
```rust
// BEFORE (in riptide-types):
pub struct CircuitBreaker { /* fields */ }
impl CircuitBreaker { /* 150 lines of logic */ }

// AFTER:
// In riptide-types (contract only):
pub struct CircuitBreakerConfig { /* config fields */ }
pub enum CircuitState { Closed, Open, HalfOpen }

// In riptide-domain (implementation):
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    // ... internal fields
}
impl CircuitBreaker { /* all the logic */ }
```

### Step 3: Update Dependencies
```toml
# riptide-facade/Cargo.toml
[dependencies]
riptide-types = { path = "../riptide-types" }      # Contracts
riptide-domain = { path = "../riptide-domain" }    # Implementations
```

---

## 🚨 Critical Quote from Codebase

From `conditional.rs` line 6:
> "HTTP conditional request support for ETag and Last-Modified headers"

**This entire module (300 lines) implements HTTP caching specification logic and should be in a domain layer, not types.**

---

## ✅ Success Criteria

A properly refactored `riptide-types` should:

1. ✅ Contain ONLY struct definitions
2. ✅ Have ONLY simple getters/setters
3. ✅ Include ONLY constructors (`new()`)
4. ✅ Feature ONLY derive macros
5. ✅ Provide ONLY trait definitions (no implementations with logic)
6. ❌ **NO** business logic
7. ❌ **NO** calculations
8. ❌ **NO** validations
9. ❌ **NO** side effects
10. ❌ **NO** I/O operations

**Current Status:** 🔴 FAILING (8 critical violations, 6 major violations)

---

## 📚 References

- **Types Rule**: Define contracts only — not behavior
- **Architectural Goal**: `riptide-facade → riptide-types` (no circular deps)
- **Related Docs**: P2-F1 Phase 2 architectural refactoring

---

**Report Generated By:** Code Review Agent (Reviewer)
**Next Action:** Schedule P3-F1 refactoring sprint to extract domain logic
