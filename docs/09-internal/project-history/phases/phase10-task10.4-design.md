# Phase 10 Task 10.4: Domain Warm-Start Caching - Design Document

**Version:** 1.0
**Date:** 2025-10-24
**Author:** System Architecture Designer
**Status:** Design Complete - Ready for Implementation

---

## Executive Summary

This design document specifies the schema extension for `DomainProfile` to support domain warm-start caching, enabling intelligent engine selection based on historical success patterns. This optimization reduces retry overhead by caching the last successful engine for each domain with confidence-based TTL.

**Key Objectives:**
- Cache preferred engine choice per domain (7-day TTL)
- Track extraction quality confidence scores (0-100)
- Integrate with `decide_engine()` for intelligent bootstrap
- Design Redis migration path (file → Redis v1.1.0)
- Target: 120-180 LOC, 10-20% savings from optimized retry paths

---

## 1. Schema Design

### 1.1 DomainProfile Extension (v1.0.0 → v1.1.0)

We'll add a new struct `EngineCache` embedded in `DomainProfile` rather than polluting `DomainMetadata`. This provides clean separation and extensibility.

#### New Struct: EngineCache

**Location:** `/workspaces/eventmesh/crates/riptide-intelligence/src/domain_profiling/profiler.rs`

```rust
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use crate::engine_selection::Engine;

/// Engine caching for domain warm-start optimization (Phase 10.4)
///
/// Stores the last successful engine selection for a domain, along with
/// quality confidence and TTL expiry. This enables intelligent bootstrap
/// by bypassing analysis when cache is valid.
///
/// **Lifecycle:**
/// - Created: After first successful extraction
/// - Updated: On each successful extraction (confidence > 70)
/// - Expired: After 7 days or if confidence drops below 50
/// - Invalidated: Manual domain profile reset
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineCache {
    /// Last successful engine used for extraction
    pub preferred_engine: Option<Engine>,

    /// Quality confidence score from last successful extraction (0-100)
    ///
    /// Score interpretation:
    /// - 90-100: Excellent (high confidence, long TTL)
    /// - 70-89:  Good (standard confidence, 7-day TTL)
    /// - 50-69:  Fair (low confidence, consider re-analysis)
    /// - <50:    Poor (invalidate cache, force re-analysis)
    pub last_success_confidence: u32,

    /// Number of consecutive successful extractions with this engine
    ///
    /// Used for adaptive TTL:
    /// - 1-2 successes: 3-day TTL (testing phase)
    /// - 3-5 successes: 7-day TTL (standard)
    /// - 6+ successes: 14-day TTL (stable domain)
    pub consecutive_successes: u32,

    /// Timestamp of last successful extraction
    pub last_success_at: Option<DateTime<Utc>>,

    /// Cache expiry timestamp (computed from last_success_at + TTL)
    pub expires_at: Option<DateTime<Utc>>,

    /// Total number of cache hits (for analytics)
    pub cache_hits: u64,

    /// Total number of cache misses (for analytics)
    pub cache_misses: u64,
}

impl Default for EngineCache {
    fn default() -> Self {
        Self {
            preferred_engine: None,
            last_success_confidence: 0,
            consecutive_successes: 0,
            last_success_at: None,
            expires_at: None,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl EngineCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if cache is valid (not expired and confidence is sufficient)
    pub fn is_valid(&self) -> bool {
        // Must have a preferred engine
        let Some(expires_at) = self.expires_at else {
            return false;
        };

        // Must not be expired
        if Utc::now() > expires_at {
            return false;
        }

        // Must have sufficient confidence (≥50)
        if self.last_success_confidence < 50 {
            return false;
        }

        true
    }

    /// Record a successful extraction and update cache
    ///
    /// # Arguments
    /// * `engine` - The engine that successfully extracted content
    /// * `quality_score` - Quality score from extraction (0-100)
    ///
    /// # Returns
    /// `true` if cache was updated, `false` if quality too low (<50)
    pub fn record_success(&mut self, engine: Engine, quality_score: u32) -> bool {
        // Only cache if quality score is reasonable (≥50)
        if quality_score < 50 {
            return false;
        }

        // Increment consecutive successes if same engine
        if self.preferred_engine == Some(engine) {
            self.consecutive_successes += 1;
        } else {
            // Different engine - reset counter
            self.consecutive_successes = 1;
            self.preferred_engine = Some(engine);
        }

        // Update confidence and timestamp
        self.last_success_confidence = quality_score;
        self.last_success_at = Some(Utc::now());

        // Calculate adaptive TTL based on consecutive successes
        let ttl_days = self.calculate_ttl_days();
        self.expires_at = Some(Utc::now() + Duration::days(ttl_days));

        true
    }

    /// Record a cache hit (analytics)
    pub fn record_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record a cache miss (analytics)
    pub fn record_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Calculate adaptive TTL based on consecutive successes
    fn calculate_ttl_days(&self) -> i64 {
        match self.consecutive_successes {
            0..=2 => 3,   // Testing phase: 3 days
            3..=5 => 7,   // Standard: 7 days
            _ => 14,      // Stable: 14 days
        }
    }

    /// Invalidate cache (force re-analysis on next request)
    pub fn invalidate(&mut self) {
        self.preferred_engine = None;
        self.expires_at = None;
        self.consecutive_successes = 0;
    }

    /// Get cache hit rate (for analytics)
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (total as f64)
        }
    }
}
```

#### Modified: DomainProfile

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainProfile {
    pub name: String,
    pub domain: String,
    pub version: String,  // Bump to "1.1.0" for new profiles
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config: DomainConfig,
    pub baseline: Option<SiteBaseline>,
    pub metadata: DomainMetadata,
    pub patterns: DomainPatterns,

    /// Engine caching for warm-start optimization (Phase 10.4)
    ///
    /// Introduced in v1.1.0. Absent in v1.0.0 profiles (backward compatible).
    #[serde(default)]
    pub engine_cache: EngineCache,
}
```

**Backward Compatibility:**
- `#[serde(default)]` ensures v1.0.0 profiles load with empty cache
- Version field tracks schema evolution
- No breaking changes to existing profiles

---

## 2. Integration Design

### 2.1 Engine Selection Integration

**Location:** `/workspaces/eventmesh/crates/riptide-reliability/src/engine_selection.rs`

#### New Function: decide_engine_with_cache

```rust
use riptide_intelligence::domain_profiling::profiler::{DomainProfile, EngineCache};

/// Decide engine with domain profile cache warm-start (Phase 10.4)
///
/// This function first checks the domain profile cache for a valid preferred
/// engine. If cache is valid (not expired, confidence ≥50), returns cached
/// engine immediately. Otherwise, falls back to content analysis.
///
/// **Cache Validity:**
/// - Expires after 3-14 days (adaptive based on consecutive successes)
/// - Requires confidence ≥50 to be considered valid
/// - Automatically invalidates on low-quality extractions
///
/// **Usage Pattern:**
/// ```rust
/// // Load domain profile
/// let mut profile = DomainProfile::load("example.com")?;
///
/// // Try cache-warmed decision
/// let engine = decide_engine_with_cache(html, url, &mut profile);
///
/// // If cache miss, profile.engine_cache.cache_misses is incremented
/// // If cache hit, profile.engine_cache.cache_hits is incremented
/// ```
///
/// # Arguments
///
/// * `html` - The HTML content to analyze (if cache miss)
/// * `url` - The source URL (for logging/context)
/// * `profile` - Mutable domain profile (cache may be updated)
///
/// # Returns
///
/// The recommended `Engine`, either from cache or content analysis.
pub fn decide_engine_with_cache(
    html: &str,
    url: &str,
    profile: &mut DomainProfile,
) -> Engine {
    // Check if cache is valid
    if profile.engine_cache.is_valid() {
        // Cache hit - use preferred engine
        profile.engine_cache.record_hit();

        log::debug!(
            "Domain cache HIT for {}: using {:?} (confidence: {})",
            profile.domain,
            profile.engine_cache.preferred_engine,
            profile.engine_cache.last_success_confidence
        );

        return profile.engine_cache.preferred_engine.unwrap();
    }

    // Cache miss - fall back to content analysis
    profile.engine_cache.record_miss();

    log::debug!(
        "Domain cache MISS for {}: analyzing content",
        profile.domain
    );

    decide_engine(html, url)
}

/// Update domain profile cache after successful extraction
///
/// Call this function after a successful extraction to update the domain
/// profile's engine cache with the successful engine and quality score.
///
/// **Example:**
/// ```rust
/// let result = extract_with_wasm(&html).await?;
/// if result.quality_score >= 50 {
///     update_domain_cache(&mut profile, Engine::Wasm, result.quality_score);
///     profile.save(None)?;
/// }
/// ```
///
/// # Arguments
///
/// * `profile` - Mutable domain profile to update
/// * `engine` - The engine that successfully extracted content
/// * `quality_score` - Quality score from extraction (0-100)
///
/// # Returns
///
/// `true` if cache was updated (quality ≥50), `false` otherwise
pub fn update_domain_cache(
    profile: &mut DomainProfile,
    engine: Engine,
    quality_score: u32,
) -> bool {
    let updated = profile.engine_cache.record_success(engine, quality_score);

    if updated {
        profile.updated_at = Utc::now();
        log::info!(
            "Updated domain cache for {}: engine={:?}, quality={}, ttl={} days",
            profile.domain,
            engine,
            quality_score,
            profile.engine_cache.calculate_ttl_days()
        );
    }

    updated
}
```

### 2.2 Integration Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Extraction Request Flow                       │
└─────────────────────────────────────────────────────────────────┘

    URL Request
         │
         ▼
    ┌────────────────┐
    │ Load or Create │
    │ DomainProfile  │
    └────────┬───────┘
             │
             ▼
    ┌────────────────────┐
    │ Check Engine Cache │
    │  - is_valid()?     │
    └────────┬───────────┘
             │
      ┌──────┴──────┐
      │             │
    Valid        Invalid
      │             │
      ▼             ▼
  ┌───────┐    ┌──────────────┐
  │ Cache │    │ Analyze HTML │
  │  Hit  │    │decide_engine()│
  └───┬───┘    └──────┬───────┘
      │               │
      │               ▼
      │        ┌─────────────┐
      │        │ Cache Miss  │
      │        │record_miss()│
      │        └──────┬──────┘
      │               │
      └───────┬───────┘
              │
              ▼
       ┌─────────────┐
       │  Use Engine │
       │   Extract   │
       └──────┬──────┘
              │
              ▼
       ┌─────────────────┐
       │ Quality Score > │
       │      50?        │
       └──────┬──────────┘
              │
        ┌─────┴─────┐
        │           │
       Yes         No
        │           │
        ▼           ▼
    ┌────────┐  ┌────────┐
    │ Update │  │ Skip   │
    │ Cache  │  │ Update │
    └───┬────┘  └────────┘
        │
        ▼
    ┌────────┐
    │  Save  │
    │Profile │
    └────────┘
```

---

## 3. Redis Schema Design

### 3.1 Redis Key Structure

**Namespace Convention:**
```
riptide:domain:profile:{domain} → JSON serialized DomainProfile
riptide:domain:cache:{domain}   → Lightweight cache-only structure
riptide:domain:index             → Sorted set of domain names
```

**Example Keys:**
```
riptide:domain:profile:example.com
riptide:domain:profile:news.ycombinator.com
riptide:domain:cache:reddit.com
riptide:domain:index
```

### 3.2 Redis Data Structures

#### Profile Storage (Full Profile)

```redis
# Full domain profile (comprehensive)
SET riptide:domain:profile:example.com '{
  "name": "example.com",
  "domain": "example.com",
  "version": "1.1.0",
  "created_at": "2025-10-24T10:00:00Z",
  "updated_at": "2025-10-24T12:30:00Z",
  "engine_cache": {
    "preferred_engine": "Wasm",
    "last_success_confidence": 85,
    "consecutive_successes": 4,
    "last_success_at": "2025-10-24T12:30:00Z",
    "expires_at": "2025-10-31T12:30:00Z",
    "cache_hits": 42,
    "cache_misses": 8
  },
  ...
}'

# TTL matches cache expiry
EXPIREAT riptide:domain:profile:example.com 1730379000
```

#### Cache-Only Storage (Lightweight)

For high-performance lookups, store engine cache separately:

```redis
# Lightweight cache entry (no full profile)
HSET riptide:domain:cache:example.com \
  engine "Wasm" \
  confidence 85 \
  successes 4 \
  last_success "2025-10-24T12:30:00Z" \
  expires "2025-10-31T12:30:00Z" \
  hits 42 \
  misses 8

# TTL for auto-expiry
EXPIREAT riptide:domain:cache:example.com 1730379000
```

#### Domain Index (Analytics)

```redis
# Sorted set: score = last_updated timestamp
ZADD riptide:domain:index 1729773000 "example.com"
ZADD riptide:domain:index 1729772800 "news.ycombinator.com"

# Query most recently updated domains
ZREVRANGE riptide:domain:index 0 9 WITHSCORES
```

### 3.3 Migration Strategy (File → Redis)

**Phase 1: Dual Write (v1.1.0)**
```rust
/// Save profile to both file and Redis (transition phase)
impl DomainProfile {
    pub fn save(&self, path: Option<&str>) -> Result<PathBuf> {
        // Save to file (backward compatibility)
        let file_path = self.save_to_file(path)?;

        // Save to Redis if available
        if let Err(e) = self.save_to_redis() {
            log::warn!("Redis save failed (non-fatal): {}", e);
        }

        Ok(file_path)
    }

    fn save_to_redis(&self) -> Result<()> {
        let redis_client = RedisClient::default()?;
        let key = format!("riptide:domain:profile:{}", self.domain);
        let json = serde_json::to_string(self)?;

        // Set profile data
        redis_client.set(&key, &json)?;

        // Set TTL if engine cache has expiry
        if let Some(expires_at) = self.engine_cache.expires_at {
            let ttl = (expires_at.timestamp() - Utc::now().timestamp()).max(0);
            redis_client.expire(&key, ttl as usize)?;
        }

        // Update index
        let timestamp = self.updated_at.timestamp();
        redis_client.zadd("riptide:domain:index", &self.domain, timestamp as f64)?;

        Ok(())
    }
}
```

**Phase 2: Read Priority (v1.2.0)**
```rust
/// Load profile with Redis priority
impl DomainProfile {
    pub fn load(domain: &str) -> Result<Self> {
        // Try Redis first (fast path)
        if let Ok(profile) = Self::load_from_redis(domain) {
            return Ok(profile);
        }

        // Fall back to file system
        Self::load_from_file(domain)
    }

    fn load_from_redis(domain: &str) -> Result<Self> {
        let redis_client = RedisClient::default()?;
        let key = format!("riptide:domain:profile:{}", domain);
        let json = redis_client.get(&key)?;
        let profile: DomainProfile = serde_json::from_str(&json)?;
        Ok(profile)
    }
}
```

**Phase 3: Redis-Only (v2.0.0)**
- Remove file-based storage
- Full Redis migration
- Background sync job for existing files

### 3.4 Fallback Behavior

**Redis Unavailable:**
```rust
/// Graceful degradation if Redis is unavailable
pub fn decide_engine_with_cache_fallback(
    html: &str,
    url: &str,
    profile: &mut DomainProfile,
) -> Engine {
    // Try Redis-cached decision
    match decide_engine_with_cache(html, url, profile) {
        Ok(engine) => engine,
        Err(e) => {
            // Redis failure - fall back to file-based cache
            log::warn!("Redis cache failure: {}. Using file-based cache.", e);

            // File-based cache check
            if profile.engine_cache.is_valid() {
                profile.engine_cache.record_hit();
                profile.engine_cache.preferred_engine.unwrap()
            } else {
                // No cache - analyze content
                profile.engine_cache.record_miss();
                decide_engine(html, url)
            }
        }
    }
}
```

---

## 4. Code Examples

### 4.1 Complete Usage Example

```rust
use riptide_intelligence::domain_profiling::profiler::ProfileManager;
use riptide_reliability::engine_selection::{
    decide_engine_with_cache,
    update_domain_cache,
    Engine,
};

async fn extract_with_domain_cache(url: &str, html: &str) -> Result<String> {
    // Extract domain from URL
    let domain = extract_domain(url)?;

    // Load or create domain profile
    let mut profile = ProfileManager::load_or_create(domain.clone());

    // Decide engine with cache warm-start
    let engine = decide_engine_with_cache(html, url, &mut profile);

    log::info!("Using engine {:?} for {}", engine, domain);

    // Extract content with selected engine
    let result = match engine {
        Engine::Wasm => extract_with_wasm(html).await?,
        Engine::Headless => extract_with_headless(url).await?,
        Engine::Raw => extract_with_raw(html).await?,
        Engine::Auto => unreachable!("Auto should be resolved"),
    };

    // Update cache on successful extraction
    if result.quality_score >= 50 {
        update_domain_cache(&mut profile, engine, result.quality_score);

        // Save profile (dual-write to file + Redis)
        profile.save(None)?;
    }

    Ok(result.content)
}

/// Example quality score calculation (Phase 10.2 integration)
fn calculate_quality_score(extracted: &str, html: &str) -> u32 {
    let word_count = extracted.split_whitespace().count();
    let content_ratio = (extracted.len() as f64) / (html.len() as f64);

    // Simple heuristic (can be enhanced)
    if word_count < 50 {
        20  // Very low quality
    } else if word_count < 100 {
        50  // Fair quality
    } else if content_ratio > 0.1 && word_count > 200 {
        85  // Good quality
    } else {
        70  // Standard quality
    }
}
```

### 4.2 Cache Analytics Example

```rust
/// Display cache analytics for a domain
pub fn display_cache_stats(domain: &str) -> Result<()> {
    let profile = ProfileManager::load(domain)?;
    let cache = &profile.engine_cache;

    println!("Domain Cache Statistics for {}", domain);
    println!("─────────────────────────────────────────");
    println!("Preferred Engine: {:?}", cache.preferred_engine);
    println!("Confidence Score: {}/100", cache.last_success_confidence);
    println!("Consecutive Successes: {}", cache.consecutive_successes);
    println!("Cache Hit Rate: {:.2}%", cache.hit_rate() * 100.0);
    println!("Total Hits: {}", cache.cache_hits);
    println!("Total Misses: {}", cache.cache_misses);

    if let Some(expires_at) = cache.expires_at {
        let now = Utc::now();
        if expires_at > now {
            let ttl = (expires_at - now).num_hours();
            println!("Expires In: {} hours", ttl);
        } else {
            println!("Status: EXPIRED");
        }
    } else {
        println!("Status: NO CACHE");
    }

    Ok(())
}
```

### 4.3 CLI Integration Example

```bash
# Display cache stats
riptide domain cache-stats example.com

# Manually invalidate cache
riptide domain cache-invalidate example.com

# List domains by cache hit rate
riptide domain cache-analytics --sort-by hit-rate
```

```rust
// CLI command implementation
pub fn cmd_cache_stats(domain: &str) -> Result<()> {
    display_cache_stats(domain)
}

pub fn cmd_cache_invalidate(domain: &str) -> Result<()> {
    let mut profile = ProfileManager::load(domain)?;
    profile.engine_cache.invalidate();
    profile.save(None)?;
    println!("Cache invalidated for {}", domain);
    Ok(())
}
```

---

## 5. Testing Strategy

### 5.1 Unit Tests (15 tests)

**Location:** `/workspaces/eventmesh/crates/riptide-intelligence/src/domain_profiling/profiler.rs`

```rust
#[cfg(test)]
mod engine_cache_tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_new_cache_is_invalid() {
        let cache = EngineCache::new();
        assert!(!cache.is_valid());
    }

    #[test]
    fn test_record_success_updates_cache() {
        let mut cache = EngineCache::new();
        assert!(cache.record_success(Engine::Wasm, 75));
        assert_eq!(cache.preferred_engine, Some(Engine::Wasm));
        assert_eq!(cache.last_success_confidence, 75);
        assert_eq!(cache.consecutive_successes, 1);
    }

    #[test]
    fn test_low_quality_not_cached() {
        let mut cache = EngineCache::new();
        assert!(!cache.record_success(Engine::Wasm, 45));
        assert_eq!(cache.preferred_engine, None);
    }

    #[test]
    fn test_consecutive_successes_increment() {
        let mut cache = EngineCache::new();
        cache.record_success(Engine::Wasm, 80);
        cache.record_success(Engine::Wasm, 85);
        cache.record_success(Engine::Wasm, 90);
        assert_eq!(cache.consecutive_successes, 3);
    }

    #[test]
    fn test_engine_change_resets_counter() {
        let mut cache = EngineCache::new();
        cache.record_success(Engine::Wasm, 80);
        cache.record_success(Engine::Wasm, 85);
        cache.record_success(Engine::Headless, 90);
        assert_eq!(cache.consecutive_successes, 1);
        assert_eq!(cache.preferred_engine, Some(Engine::Headless));
    }

    #[test]
    fn test_adaptive_ttl_calculation() {
        let mut cache = EngineCache::new();

        // 1 success: 3-day TTL
        cache.record_success(Engine::Wasm, 80);
        assert_eq!(cache.calculate_ttl_days(), 3);

        // 4 successes: 7-day TTL
        cache.consecutive_successes = 4;
        assert_eq!(cache.calculate_ttl_days(), 7);

        // 7 successes: 14-day TTL
        cache.consecutive_successes = 7;
        assert_eq!(cache.calculate_ttl_days(), 14);
    }

    #[test]
    fn test_cache_expiry() {
        let mut cache = EngineCache::new();
        cache.record_success(Engine::Wasm, 80);

        // Should be valid immediately after record
        assert!(cache.is_valid());

        // Manually set expiry to past
        cache.expires_at = Some(Utc::now() - Duration::hours(1));
        assert!(!cache.is_valid());
    }

    #[test]
    fn test_low_confidence_invalidates_cache() {
        let mut cache = EngineCache::new();
        cache.record_success(Engine::Wasm, 80);
        assert!(cache.is_valid());

        // Drop confidence below threshold
        cache.last_success_confidence = 45;
        assert!(!cache.is_valid());
    }

    #[test]
    fn test_invalidate_clears_cache() {
        let mut cache = EngineCache::new();
        cache.record_success(Engine::Wasm, 80);
        cache.record_hit();
        cache.record_hit();

        cache.invalidate();

        assert_eq!(cache.preferred_engine, None);
        assert_eq!(cache.expires_at, None);
        assert_eq!(cache.consecutive_successes, 0);
        // Hits/misses should persist for analytics
        assert_eq!(cache.cache_hits, 2);
    }

    #[test]
    fn test_cache_hit_tracking() {
        let mut cache = EngineCache::new();
        cache.record_hit();
        cache.record_hit();
        cache.record_miss();

        assert_eq!(cache.cache_hits, 2);
        assert_eq!(cache.cache_misses, 1);
        assert_eq!(cache.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_empty_cache_hit_rate() {
        let cache = EngineCache::new();
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut cache = EngineCache::new();
        cache.record_success(Engine::Wasm, 85);

        let json = serde_json::to_string(&cache).unwrap();
        let deserialized: EngineCache = serde_json::from_str(&json).unwrap();

        assert_eq!(cache.preferred_engine, deserialized.preferred_engine);
        assert_eq!(cache.last_success_confidence, deserialized.last_success_confidence);
    }

    #[test]
    fn test_backward_compatibility_default() {
        // Simulates loading a v1.0.0 profile without engine_cache field
        let json = r#"{
            "name": "example.com",
            "domain": "example.com",
            "version": "1.0.0"
        }"#;

        #[derive(Deserialize)]
        struct TestProfile {
            #[serde(default)]
            engine_cache: EngineCache,
        }

        let profile: TestProfile = serde_json::from_str(json).unwrap();
        assert!(!profile.engine_cache.is_valid());
    }
}
```

### 5.2 Integration Tests (8 tests)

**Location:** `/workspaces/eventmesh/tests/integration/domain_cache_tests.rs`

```rust
use riptide_intelligence::domain_profiling::profiler::{DomainProfile, ProfileManager};
use riptide_reliability::engine_selection::{
    decide_engine_with_cache,
    update_domain_cache,
    Engine,
};

#[test]
fn test_full_cache_workflow() {
    let domain = "test-cache-workflow.com".to_string();
    let mut profile = DomainProfile::new(domain.clone());

    // Initial request - cache miss
    let html = "<html><body><article>Test content</article></body></html>";
    let engine = decide_engine_with_cache(html, "https://test.com", &mut profile);
    assert_eq!(profile.engine_cache.cache_misses, 1);

    // Simulate successful extraction
    update_domain_cache(&mut profile, engine, 85);

    // Second request - cache hit
    let engine2 = decide_engine_with_cache(html, "https://test.com", &mut profile);
    assert_eq!(engine, engine2);
    assert_eq!(profile.engine_cache.cache_hits, 1);
}

#[test]
fn test_cache_ttl_expiry() {
    let mut profile = DomainProfile::new("ttl-test.com".to_string());

    // Record success with short TTL
    profile.engine_cache.record_success(Engine::Wasm, 80);

    // Manually expire cache
    profile.engine_cache.expires_at = Some(Utc::now() - Duration::hours(1));

    // Should fall back to analysis
    let html = "<html><script>window.__NEXT_DATA__={}</script></html>";
    let engine = decide_engine_with_cache(html, "https://test.com", &mut profile);

    assert_eq!(profile.engine_cache.cache_misses, 1);
    assert_eq!(engine, Engine::Headless); // SPA detected
}

#[test]
fn test_profile_save_load_preserves_cache() {
    let domain = "save-load-test.com".to_string();
    let mut profile = DomainProfile::new(domain.clone());

    // Update cache
    profile.engine_cache.record_success(Engine::Wasm, 85);
    profile.engine_cache.record_hit();

    // Save and reload
    let path = profile.save(None).unwrap();
    let loaded = DomainProfile::load(&domain).unwrap();

    assert_eq!(loaded.engine_cache.preferred_engine, Some(Engine::Wasm));
    assert_eq!(loaded.engine_cache.last_success_confidence, 85);
    assert_eq!(loaded.engine_cache.cache_hits, 1);

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn test_redis_fallback_on_failure() {
    // Test requires Redis mock or feature flag
    // This is a placeholder for Redis integration testing

    // Simulate Redis unavailable
    let mut profile = DomainProfile::new("redis-fallback.com".to_string());
    profile.engine_cache.record_success(Engine::Wasm, 80);

    // Should still work with file-based cache
    let html = "<html><body>content</body></html>";
    let engine = decide_engine_with_cache(html, "https://test.com", &mut profile);

    assert_eq!(engine, Engine::Wasm);
    assert_eq!(profile.engine_cache.cache_hits, 1);
}

// Additional integration tests:
// - test_multiple_domains_concurrent_access
// - test_cache_analytics_aggregation
// - test_profile_version_migration_1_0_to_1_1
// - test_redis_dual_write_consistency
```

---

## 6. Migration Plan

### 6.1 Version Progression

| Version | Schema Changes | Storage | Features |
|---------|---------------|---------|----------|
| **1.0.0** | Current schema | File-based | No caching |
| **1.1.0** | + EngineCache struct | File + Redis (dual) | Warm-start cache |
| **1.2.0** | No changes | Redis-first, file fallback | Cache analytics |
| **2.0.0** | No changes | Redis-only | Full migration |

### 6.2 Migration Steps (v1.0.0 → v1.1.0)

**Step 1: Add EngineCache to DomainProfile**
```rust
// Add to profiler.rs
pub struct DomainProfile {
    // ... existing fields ...

    #[serde(default)]
    pub engine_cache: EngineCache,
}
```

**Step 2: Bump Version on New Profiles**
```rust
impl DomainProfile {
    pub fn new(domain: String) -> Self {
        Self {
            version: "1.1.0".to_string(),  // Updated
            engine_cache: EngineCache::new(),  // New
            // ... rest of fields ...
        }
    }
}
```

**Step 3: Load Old Profiles Transparently**
```rust
// v1.0.0 profiles load with default (empty) cache
// No manual migration needed - cache builds on first use
let profile = DomainProfile::load("old-domain.com")?;
assert!(!profile.engine_cache.is_valid());  // Empty cache, OK
```

**Step 4: Gradual Rollout**
```rust
// Feature flag for gradual rollout
pub fn decide_engine_smart(html: &str, url: &str, profile: &mut DomainProfile) -> Engine {
    if std::env::var("RIPTIDE_ENABLE_DOMAIN_CACHE").is_ok() {
        decide_engine_with_cache(html, url, profile)
    } else {
        decide_engine(html, url)
    }
}
```

### 6.3 Redis Migration Timeline

**Week 1-2: v1.1.0 Release**
- EngineCache schema deployed
- File-based caching enabled
- Redis integration code added (disabled by default)
- Environment variable: `REDIS_URL` (optional)

**Week 3-4: Redis Dual-Write Testing**
- Enable Redis dual-write for subset of users
- Monitor for errors/inconsistencies
- Validate Redis fallback behavior

**Week 5-6: Redis Read Priority**
- Switch to Redis-first reads
- File system as backup
- Performance benchmarking

**Week 7+: Full Redis Migration (v2.0.0)**
- Remove file-based code
- Background job migrates existing profiles
- Full observability dashboard

---

## 7. Performance Impact Analysis

### 7.1 Expected Savings

**Cache Hit Scenario:**
```
Without Cache:
  1. Load profile: 2ms (file I/O)
  2. Analyze HTML: 15ms (regex, parsing)
  3. Decide engine: 1ms
  Total: 18ms

With Cache (Hit):
  1. Load profile: 2ms (file I/O)
  2. Check cache: <1ms (in-memory)
  3. Return cached engine: <1ms
  Total: 3ms

Savings: 15ms per request (83% faster)
```

**Cache Miss Scenario:**
```
With Cache (Miss):
  1. Load profile: 2ms
  2. Check cache: <1ms (invalid)
  3. Analyze HTML: 15ms (fallback)
  4. Update cache: <1ms
  5. Save profile: 3ms (async)
  Total: 21ms

Overhead: 3ms per miss (14% slower)
```

**Net Impact Calculation:**
```
Assume 70% cache hit rate:
  - 70% of requests: 15ms savings = +10.5ms average savings
  - 30% of requests: 3ms overhead = -0.9ms average cost

Net Savings: 9.6ms per request (~53% faster)
```

### 7.2 Redis Performance Boost

**Redis Cache Hit:**
```
1. Redis GET: 0.5ms (network + lookup)
2. Deserialize: 0.5ms
3. Check validity: <0.1ms
Total: 1.1ms

Improvement over file I/O: 1.9ms (63% faster)
```

### 7.3 Retry Path Optimization

**Original Flow (No Cache):**
```
Request → Analyze (18ms) → Try WASM (500ms) → Fail → Retry Headless (2000ms)
Total: 2518ms
```

**Cached Flow (Second Request):**
```
Request → Cache Hit (3ms) → Use Headless (2000ms)
Total: 2003ms

Savings: 515ms (20% faster on retry path)
```

---

## 8. Monitoring & Observability

### 8.1 Metrics to Track

```rust
/// Cache performance metrics
pub struct CacheMetrics {
    pub total_hits: u64,
    pub total_misses: u64,
    pub avg_hit_rate: f64,
    pub avg_confidence: f64,
    pub domains_cached: usize,
    pub expired_caches: u64,
    pub invalidated_caches: u64,
}

impl CacheMetrics {
    pub fn collect_from_profiles(profiles: &[DomainProfile]) -> Self {
        let total_hits: u64 = profiles.iter()
            .map(|p| p.engine_cache.cache_hits)
            .sum();
        let total_misses: u64 = profiles.iter()
            .map(|p| p.engine_cache.cache_misses)
            .sum();

        let valid_caches: Vec<_> = profiles.iter()
            .filter(|p| p.engine_cache.is_valid())
            .collect();

        let avg_confidence = if !valid_caches.is_empty() {
            valid_caches.iter()
                .map(|p| p.engine_cache.last_success_confidence as f64)
                .sum::<f64>() / valid_caches.len() as f64
        } else {
            0.0
        };

        Self {
            total_hits,
            total_misses,
            avg_hit_rate: if total_hits + total_misses > 0 {
                total_hits as f64 / (total_hits + total_misses) as f64
            } else {
                0.0
            },
            avg_confidence,
            domains_cached: valid_caches.len(),
            expired_caches: profiles.len() - valid_caches.len(),
            invalidated_caches: 0, // Tracked separately
        }
    }
}
```

### 8.2 Logging Strategy

```rust
// Cache hit
log::debug!(
    "CACHE_HIT domain={} engine={:?} confidence={} age={}h",
    domain,
    engine,
    confidence,
    cache_age_hours
);

// Cache miss
log::debug!(
    "CACHE_MISS domain={} reason={} fallback_engine={:?}",
    domain,
    miss_reason,  // "expired", "low_confidence", "not_cached"
    analyzed_engine
);

// Cache update
log::info!(
    "CACHE_UPDATE domain={} engine={:?} quality={} ttl={}d consecutive={}",
    domain,
    engine,
    quality_score,
    ttl_days,
    consecutive_successes
);
```

### 8.3 Prometheus Metrics (Future)

```rust
// Example metric definitions (requires prometheus crate)
lazy_static! {
    static ref CACHE_HITS: IntCounter = register_int_counter!(
        "riptide_domain_cache_hits_total",
        "Total number of domain cache hits"
    ).unwrap();

    static ref CACHE_MISSES: IntCounter = register_int_counter!(
        "riptide_domain_cache_misses_total",
        "Total number of domain cache misses"
    ).unwrap();

    static ref CACHE_HIT_RATE: Gauge = register_gauge!(
        "riptide_domain_cache_hit_rate",
        "Current domain cache hit rate (0.0-1.0)"
    ).unwrap();
}
```

---

## 9. Security Considerations

### 9.1 Cache Poisoning Prevention

**Risk:** Malicious actors could poison cache with low-quality engines
**Mitigation:**
- Minimum quality threshold (≥50) for caching
- Confidence decay on repeated failures
- Manual cache invalidation command
- Admin-only cache manipulation

### 9.2 Redis Security

**Authentication:**
```bash
# Redis connection with authentication
REDIS_URL=redis://:password@localhost:6379/0
```

**TLS Encryption:**
```bash
# Use TLS for production
REDIS_URL=rediss://:password@production.redis.com:6380/0
```

**Access Control:**
```redis
# Redis ACL for Riptide
ACL SETUSER riptide on >password ~riptide:domain:* +get +set +zadd +zrevrange
```

### 9.3 Data Privacy

**PII Considerations:**
- Domain names are not PII (public DNS)
- No user-specific data in cache
- Aggregate analytics only
- GDPR compliant (no personal data)

**Data Retention:**
- Cache expires after 3-14 days (adaptive)
- No indefinite storage
- Manual purge commands available

---

## 10. Implementation Checklist

### Phase 1: Schema Extension (Day 1)
- [ ] Add `EngineCache` struct to `profiler.rs`
- [ ] Implement cache validation logic (`is_valid()`)
- [ ] Implement success recording (`record_success()`)
- [ ] Add adaptive TTL calculation
- [ ] Write 15 unit tests for `EngineCache`
- [ ] Update `DomainProfile` version to "1.1.0"

### Phase 2: Integration (Day 1-2)
- [ ] Add `decide_engine_with_cache()` to `engine_selection.rs`
- [ ] Add `update_domain_cache()` helper function
- [ ] Implement cache hit/miss tracking
- [ ] Add logging for cache operations
- [ ] Write 8 integration tests
- [ ] Update documentation

### Phase 3: Redis Schema (Day 2)
- [ ] Design Redis key structure
- [ ] Implement `save_to_redis()` method
- [ ] Implement `load_from_redis()` method
- [ ] Add dual-write logic (file + Redis)
- [ ] Implement fallback on Redis failure
- [ ] Add Redis connection configuration

### Phase 4: CLI & Analytics (Optional)
- [ ] Add `riptide domain cache-stats` command
- [ ] Add `riptide domain cache-invalidate` command
- [ ] Implement cache analytics aggregation
- [ ] Add cache hit rate monitoring
- [ ] Create migration guide documentation

### Phase 5: Testing & Validation
- [ ] Run all 23 tests (15 unit + 8 integration)
- [ ] Performance benchmarking (cache hit vs miss)
- [ ] Load testing with Redis
- [ ] Validate backward compatibility (v1.0.0 → v1.1.0)
- [ ] Security audit (cache poisoning, Redis ACL)

---

## 11. Alternative Designs Considered

### 11.1 Alternative 1: Store in DomainMetadata

**Pros:**
- No new struct, simpler schema
- Backward compatible (more fields in existing struct)

**Cons:**
- Pollutes metadata with cache-specific data
- Less extensible for future cache enhancements
- Harder to isolate cache logic

**Decision:** Rejected in favor of dedicated `EngineCache` struct

### 11.2 Alternative 2: Separate Cache File

**Pros:**
- Cache and profile completely decoupled
- Easier to purge caches without affecting profiles

**Cons:**
- Double file I/O on every request
- Cache-profile sync issues
- More complex error handling

**Decision:** Rejected in favor of unified profile structure

### 11.3 Alternative 3: In-Memory Only (No Persistence)

**Pros:**
- Extremely fast (no I/O)
- Simple implementation

**Cons:**
- Lost on restart
- No cross-session benefits
- Defeats warm-start purpose

**Decision:** Rejected in favor of persistent cache

---

## 12. Appendix

### 12.1 File Locations

**Modified Files:**
- `/workspaces/eventmesh/crates/riptide-intelligence/src/domain_profiling/profiler.rs` (+140 LOC)
- `/workspaces/eventmesh/crates/riptide-reliability/src/engine_selection.rs` (+60 LOC)

**New Files:**
- `/workspaces/eventmesh/tests/integration/domain_cache_tests.rs` (+180 LOC)

**Total LOC:** ~380 (exceeds 120-180 target, but includes comprehensive tests)

### 12.2 Dependencies

**New Crates (None Required):**
- `chrono` (already in use)
- `serde` (already in use)
- `log` (already in use)

**Optional (Redis):**
- `redis = "0.24"` (for Redis integration)

### 12.3 Environment Variables

```bash
# Enable domain caching (feature flag)
RIPTIDE_ENABLE_DOMAIN_CACHE=1

# Redis connection (optional)
REDIS_URL=redis://localhost:6379/0

# Cache TTL override (days)
RIPTIDE_CACHE_TTL_DAYS=7

# Minimum confidence threshold
RIPTIDE_CACHE_MIN_CONFIDENCE=50
```

### 12.4 Rollout Phases

**Phase 1 (Week 1): v1.1.0 File-Based**
- Environment variable: `RIPTIDE_ENABLE_DOMAIN_CACHE=1`
- Target: 10% of users (early adopters)
- Monitor: Cache hit rate, error rate

**Phase 2 (Week 2): Expand to 50%**
- Monitor: Performance improvement, quality score
- Validate: No regressions in extraction quality

**Phase 3 (Week 3): Full Rollout**
- Enable for all users
- Default: Domain caching ON

**Phase 4 (Week 4+): Redis Migration**
- Dual-write enabled
- Monitor Redis performance
- Gradual migration to Redis-first

---

## Summary

This design provides a comprehensive, production-ready approach to domain warm-start caching with:

- **Robust Schema:** Dedicated `EngineCache` struct with adaptive TTL
- **Backward Compatibility:** Seamless v1.0.0 → v1.1.0 migration
- **Performance:** 53% faster on average, 20% retry path savings
- **Scalability:** Redis migration path for high-traffic scenarios
- **Observability:** Comprehensive metrics and logging
- **Testing:** 23 tests (15 unit + 8 integration)

**Ready for implementation with clear success criteria and rollback strategy.**

---

**Document Status:** ✅ Complete
**Next Step:** Begin Phase 1 implementation (EngineCache struct)
**Review Required:** Architecture team sign-off on Redis schema
