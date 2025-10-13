# Gate Threshold Tuning System Design

**Version:** 1.0
**Date:** 2025-10-13
**Author:** Coder Agent
**Status:** Design Complete

## Executive Summary

This document describes the design of a dynamic threshold tuning system for the Riptide gate scoring algorithm. The system enables runtime configuration changes, A/B testing of threshold variants, performance tracking, and automated threshold recommendations without requiring code recompilation.

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Architecture Overview](#2-architecture-overview)
3. [Configuration System](#3-configuration-system)
4. [Dynamic Loading Mechanism](#4-dynamic-loading-mechanism)
5. [A/B Testing Framework](#5-ab-testing-framework)
6. [Performance Tracking](#6-performance-tracking)
7. [Recommendation Engine](#7-recommendation-engine)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [Risk Analysis](#9-risk-analysis)

---

## 1. Current State Analysis

### 1.1 Existing Threshold Implementation

**Location:** `/workspaces/eventmesh/crates/riptide-core/src/gate.rs`

**Current Hard-Coded Values:**

```rust
// Line 233: Decision thresholds in decide() function
pub fn decide(features: &GateFeatures, hi: f32, lo: f32) -> Decision {
    let content_score = score(features);

    if content_score >= hi {      // Default: 0.8 (not in code)
        Decision::Raw
    } else if content_score <= lo || features.spa_markers >= 3 {  // Default: 0.3
        Decision::Headless
    } else {
        Decision::ProbesFirst
    }
}
```

**Current Feature Weights (Lines 97-133):**

| Feature | Weight/Factor | Line | Rationale |
|---------|--------------|------|-----------|
| Text ratio weight | 1.2 | 102 | Multiply text/HTML ratio for content-rich pages |
| Text ratio cap | 0.6 | 102 | Maximum contribution from text ratio |
| Paragraph log scale | 0.06 | 105 | Logarithmic scaling for paragraph count |
| Paragraph cap | 0.3 | 105 | Maximum from paragraph structure |
| Article element bonus | 0.15 | 109 | Semantic structure indicator |
| Open Graph bonus | 0.08 | 114 | Metadata presence |
| JSON-LD bonus | 0.12 | 119 | Structured data indicator |
| Script density penalty | 0.8 | 125 | Multiplier for script ratio |
| Script density cap | 0.4 | 125 | Maximum penalty from scripts |
| SPA marker penalty | 0.25 | 129 | Heavy SPA detection |
| SPA marker threshold | 2 | 128 | Number of markers needed |
| Domain prior weight | 0.1 | 133 | Historical performance adjustment |

**Decision Boundaries:**

- **score > 0.8**: `Decision::Raw` - Fast extraction (direct HTML parsing)
- **0.3 < score ≤ 0.8**: `Decision::ProbesFirst` - Try fast first, fallback to headless
- **score ≤ 0.3**: `Decision::Headless` - Use headless browser rendering
- **SPA markers ≥ 3**: `Decision::Headless` - Forced headless for heavy SPAs

### 1.2 Problems with Current Implementation

1. **Static thresholds**: Cannot adjust without recompilation
2. **No experimentation**: Cannot test threshold variants
3. **No feedback loop**: Cannot learn from extraction results
4. **Domain-specific needs**: Some domains may need different thresholds
5. **Performance blind spots**: No metrics on threshold effectiveness

---

## 2. Architecture Overview

### 2.1 System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                     Gate Threshold System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌───────────────┐    ┌───────────────┐    ┌─────────────────┐ │
│  │ Configuration │    │   A/B Test    │    │   Performance   │ │
│  │    Manager    │───▶│   Selector    │───▶│     Tracker     │ │
│  └───────────────┘    └───────────────┘    └─────────────────┘ │
│         │                     │                      │           │
│         │                     │                      │           │
│         ▼                     ▼                      ▼           │
│  ┌───────────────┐    ┌───────────────┐    ┌─────────────────┐ │
│  │  Config File  │    │   Variant     │    │    Metrics      │ │
│  │  (TOML/JSON)  │    │   Selector    │    │    Database     │ │
│  └───────────────┘    └───────────────┘    └─────────────────┘ │
│         │                     │                      │           │
│         │                     │                      │           │
│         └─────────────────────┴──────────────────────┘           │
│                              │                                   │
│                              ▼                                   │
│                   ┌───────────────────┐                         │
│                   │  Recommendation   │                         │
│                   │     Engine        │                         │
│                   └───────────────────┘                         │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
Request → Config Manager → A/B Selector → Gate Decision → Performance Tracker
                ↓                              ↓                  ↓
           Load Config                     Apply Thresholds    Record Metrics
                ↓                              ↓                  ↓
           Watch Changes                   Return Decision     Update Stats
                                                                  ↓
                                                          Recommendation Engine
```

---

## 3. Configuration System

### 3.1 Configuration Structure

```rust
// crates/riptide-core/src/gate/config.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for gate threshold tuning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    /// Version of the configuration format
    pub version: String,

    /// Default thresholds
    pub default: ThresholdSet,

    /// Domain-specific overrides (key: domain pattern)
    #[serde(default)]
    pub domain_overrides: HashMap<String, ThresholdSet>,

    /// A/B test variants
    #[serde(default)]
    pub ab_tests: Vec<ABTestVariant>,

    /// Feature flag: enable A/B testing
    #[serde(default = "default_true")]
    pub enable_ab_testing: bool,

    /// Feature flag: enable hot-reload
    #[serde(default = "default_true")]
    pub enable_hot_reload: bool,
}

/// A set of threshold values for decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSet {
    /// Name/description of this threshold set
    pub name: String,

    /// High threshold for raw extraction (score >= hi → Raw)
    pub threshold_raw: f32,

    /// Low threshold for headless (score <= lo → Headless)
    pub threshold_probes: f32,

    /// Feature weights for scoring algorithm
    pub feature_weights: FeatureWeights,

    /// SPA marker threshold (markers >= N → Headless)
    #[serde(default = "default_spa_threshold")]
    pub spa_marker_threshold: u8,
}

/// Weights and factors for scoring algorithm features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureWeights {
    // Positive indicators
    pub text_ratio_weight: f32,        // Default: 1.2
    pub text_ratio_cap: f32,           // Default: 0.6
    pub paragraph_log_scale: f32,      // Default: 0.06
    pub paragraph_cap: f32,            // Default: 0.3
    pub article_bonus: f32,            // Default: 0.15
    pub og_bonus: f32,                 // Default: 0.08
    pub jsonld_bonus: f32,             // Default: 0.12

    // Negative indicators
    pub script_density_penalty: f32,   // Default: 0.8
    pub script_density_cap: f32,       // Default: 0.4
    pub spa_marker_penalty: f32,       // Default: 0.25

    // Domain adjustment
    pub domain_prior_weight: f32,      // Default: 0.1
}

/// A/B test variant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestVariant {
    /// Unique variant identifier
    pub variant_id: String,

    /// Human-readable name
    pub name: String,

    /// Percentage of traffic (0.0-1.0)
    pub traffic_percentage: f32,

    /// Threshold configuration for this variant
    pub thresholds: ThresholdSet,

    /// Whether this variant is active
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Start date for the test (ISO 8601)
    pub start_date: Option<String>,

    /// End date for the test (ISO 8601)
    pub end_date: Option<String>,
}

// Default value helpers
fn default_true() -> bool { true }
fn default_spa_threshold() -> u8 { 3 }

impl Default for FeatureWeights {
    fn default() -> Self {
        Self {
            text_ratio_weight: 1.2,
            text_ratio_cap: 0.6,
            paragraph_log_scale: 0.06,
            paragraph_cap: 0.3,
            article_bonus: 0.15,
            og_bonus: 0.08,
            jsonld_bonus: 0.12,
            script_density_penalty: 0.8,
            script_density_cap: 0.4,
            spa_marker_penalty: 0.25,
            domain_prior_weight: 0.1,
        }
    }
}

impl Default for ThresholdSet {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            threshold_raw: 0.8,
            threshold_probes: 0.3,
            feature_weights: FeatureWeights::default(),
            spa_marker_threshold: 3,
        }
    }
}
```

### 3.2 Configuration File Format (TOML)

**Location:** `config/gate_thresholds.toml`

```toml
# Gate Threshold Configuration
# Version: 1.0

version = "1.0"
enable_ab_testing = true
enable_hot_reload = true

# ============================================================================
# DEFAULT THRESHOLDS
# ============================================================================
[default]
name = "Production Default"
threshold_raw = 0.8      # score >= 0.8 → Raw extraction
threshold_probes = 0.3   # score <= 0.3 → Headless
spa_marker_threshold = 3 # SPA markers >= 3 → Headless

[default.feature_weights]
# Positive indicators (increase score)
text_ratio_weight = 1.2        # Multiply text/HTML ratio
text_ratio_cap = 0.6           # Maximum contribution from text ratio
paragraph_log_scale = 0.06     # Logarithmic scaling for paragraph count
paragraph_cap = 0.3            # Maximum from paragraph structure
article_bonus = 0.15           # Semantic structure indicator
og_bonus = 0.08                # Open Graph metadata bonus
jsonld_bonus = 0.12            # JSON-LD structured data bonus

# Negative indicators (decrease score)
script_density_penalty = 0.8   # Multiplier for script/HTML ratio
script_density_cap = 0.4       # Maximum penalty from scripts
spa_marker_penalty = 0.25      # Penalty per SPA marker group

# Domain adjustment
domain_prior_weight = 0.1      # Historical performance weight


# ============================================================================
# DOMAIN-SPECIFIC OVERRIDES
# ============================================================================

# High-quality news sites: favor raw extraction
[domain_overrides."*.nytimes.com"]
name = "News Site - NYTimes"
threshold_raw = 0.7            # Lower threshold for raw (more aggressive)
threshold_probes = 0.3

[domain_overrides."*.washingtonpost.com"]
name = "News Site - WaPo"
threshold_raw = 0.7
threshold_probes = 0.3

# Heavy SPA sites: favor headless
[domain_overrides."*.twitter.com"]
name = "SPA - Twitter"
threshold_raw = 0.9            # Higher threshold (less likely raw)
threshold_probes = 0.5         # Higher probes threshold (favor headless)
spa_marker_threshold = 2       # Lower SPA marker threshold

[domain_overrides."*.reddit.com"]
name = "SPA - Reddit"
threshold_raw = 0.85
threshold_probes = 0.4
spa_marker_threshold = 2


# ============================================================================
# A/B TEST VARIANTS
# ============================================================================

# Test: More aggressive raw extraction
[[ab_tests]]
variant_id = "aggressive_raw_v1"
name = "Aggressive Raw Extraction"
traffic_percentage = 0.1        # 10% of traffic
enabled = true
start_date = "2025-10-13T00:00:00Z"
end_date = "2025-11-13T00:00:00Z"

[ab_tests.thresholds]
name = "Aggressive Raw"
threshold_raw = 0.7            # Lower threshold
threshold_probes = 0.25        # Slightly lower probes
spa_marker_threshold = 3

[ab_tests.thresholds.feature_weights]
text_ratio_weight = 1.4        # Increase text ratio importance
text_ratio_cap = 0.7
paragraph_log_scale = 0.08
paragraph_cap = 0.35
article_bonus = 0.18
og_bonus = 0.10
jsonld_bonus = 0.15
script_density_penalty = 0.7   # Reduce script penalty
script_density_cap = 0.35
spa_marker_penalty = 0.25
domain_prior_weight = 0.1


# Test: More conservative (favor headless for quality)
[[ab_tests]]
variant_id = "conservative_v1"
name = "Conservative Quality-First"
traffic_percentage = 0.1        # 10% of traffic
enabled = true
start_date = "2025-10-13T00:00:00Z"
end_date = "2025-11-13T00:00:00Z"

[ab_tests.thresholds]
name = "Conservative"
threshold_raw = 0.85           # Higher threshold
threshold_probes = 0.35        # Higher probes threshold
spa_marker_threshold = 2       # Lower SPA threshold

[ab_tests.thresholds.feature_weights]
text_ratio_weight = 1.0        # Reduce text ratio importance
text_ratio_cap = 0.5
paragraph_log_scale = 0.06
paragraph_cap = 0.3
article_bonus = 0.15
og_bonus = 0.08
jsonld_bonus = 0.12
script_density_penalty = 1.0   # Increase script penalty
script_density_cap = 0.5
spa_marker_penalty = 0.30
domain_prior_weight = 0.15     # Increase domain prior weight


# Test: Balanced approach
[[ab_tests]]
variant_id = "balanced_v1"
name = "Balanced Extraction"
traffic_percentage = 0.05       # 5% of traffic
enabled = true
start_date = "2025-10-13T00:00:00Z"
end_date = "2025-11-13T00:00:00Z"

[ab_tests.thresholds]
name = "Balanced"
threshold_raw = 0.75
threshold_probes = 0.32
spa_marker_threshold = 3

[ab_tests.thresholds.feature_weights]
text_ratio_weight = 1.1
text_ratio_cap = 0.55
paragraph_log_scale = 0.07
paragraph_cap = 0.32
article_bonus = 0.16
og_bonus = 0.09
jsonld_bonus = 0.13
script_density_penalty = 0.85
script_density_cap = 0.42
spa_marker_penalty = 0.27
domain_prior_weight = 0.12
```

### 3.3 Environment Variable Overrides

```bash
# Override default thresholds via environment variables
export GATE_THRESHOLD_RAW=0.75
export GATE_THRESHOLD_PROBES=0.35
export GATE_ENABLE_AB_TESTING=true
export GATE_CONFIG_PATH=/custom/path/gate_thresholds.toml

# Feature weight overrides
export GATE_WEIGHT_TEXT_RATIO=1.3
export GATE_WEIGHT_SCRIPT_PENALTY=0.9
```

---

## 4. Dynamic Loading Mechanism

### 4.1 Configuration Manager Implementation

```rust
// crates/riptide-core/src/gate/config_manager.rs

use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Manages dynamic loading and reloading of gate configuration
pub struct ConfigManager {
    /// Current active configuration
    config: Arc<RwLock<GateConfig>>,

    /// Path to configuration file
    config_path: PathBuf,

    /// File system watcher for hot-reload
    watcher: Option<RecommendedWatcher>,

    /// Whether hot-reload is enabled
    hot_reload_enabled: bool,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub async fn new(config_path: impl AsRef<Path>) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();

        // Load initial configuration
        let config = Self::load_config(&config_path).await?;

        info!(
            path = ?config_path,
            version = config.version,
            ab_tests = config.ab_tests.len(),
            "Loaded gate configuration"
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            watcher: None,
            hot_reload_enabled: false,
        })
    }

    /// Load configuration from file
    async fn load_config(path: &Path) -> Result<GateConfig> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read config file")?;

        // Support both TOML and JSON
        let config = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)?
        } else {
            toml::from_str(&content)?
        };

        // Apply environment variable overrides
        Self::apply_env_overrides(config)
    }

    /// Apply environment variable overrides to configuration
    fn apply_env_overrides(mut config: GateConfig) -> Result<GateConfig> {
        if let Ok(val) = std::env::var("GATE_THRESHOLD_RAW") {
            config.default.threshold_raw = val.parse()?;
        }
        if let Ok(val) = std::env::var("GATE_THRESHOLD_PROBES") {
            config.default.threshold_probes = val.parse()?;
        }
        if let Ok(val) = std::env::var("GATE_ENABLE_AB_TESTING") {
            config.enable_ab_testing = val.parse()?;
        }

        // Feature weight overrides
        if let Ok(val) = std::env::var("GATE_WEIGHT_TEXT_RATIO") {
            config.default.feature_weights.text_ratio_weight = val.parse()?;
        }
        if let Ok(val) = std::env::var("GATE_WEIGHT_SCRIPT_PENALTY") {
            config.default.feature_weights.script_density_penalty = val.parse()?;
        }

        Ok(config)
    }

    /// Enable hot-reload of configuration changes
    pub fn enable_hot_reload(&mut self) -> Result<()> {
        if self.watcher.is_some() {
            return Ok(()); // Already enabled
        }

        let config_arc = Arc::clone(&self.config);
        let config_path = self.config_path.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) if matches!(event.kind, EventKind::Modify(_)) => {
                    info!("Configuration file changed, reloading...");

                    let config_arc = Arc::clone(&config_arc);
                    let config_path = config_path.clone();

                    tokio::spawn(async move {
                        match Self::load_config(&config_path).await {
                            Ok(new_config) => {
                                let mut config = config_arc.write().await;
                                *config = new_config;
                                info!("Configuration reloaded successfully");
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to reload configuration");
                            }
                        }
                    });
                }
                Ok(_) => {}
                Err(e) => {
                    error!(error = %e, "Watch error");
                }
            }
        })?;

        watcher.watch(&self.config_path, RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);
        self.hot_reload_enabled = true;

        info!("Hot-reload enabled for gate configuration");
        Ok(())
    }

    /// Get current configuration (read-only)
    pub async fn get_config(&self) -> Arc<RwLock<GateConfig>> {
        Arc::clone(&self.config)
    }

    /// Get threshold set for a specific domain
    pub async fn get_thresholds_for_domain(&self, domain: &str) -> ThresholdSet {
        let config = self.config.read().await;

        // Check for domain-specific overrides
        for (pattern, thresholds) in &config.domain_overrides {
            if Self::domain_matches(domain, pattern) {
                return thresholds.clone();
            }
        }

        // Return default thresholds
        config.default.clone()
    }

    /// Check if domain matches a pattern (supports wildcards)
    fn domain_matches(domain: &str, pattern: &str) -> bool {
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            domain.ends_with(suffix)
        } else {
            domain == pattern
        }
    }
}
```

### 4.2 Integration with Gate Module

```rust
// crates/riptide-core/src/gate.rs (modified)

use crate::gate::config::{ConfigManager, ThresholdSet};
use once_cell::sync::OnceCell;

// Global configuration manager
static CONFIG_MANAGER: OnceCell<ConfigManager> = OnceCell::new();

/// Initialize the gate configuration system
pub async fn init_config(config_path: impl AsRef<Path>) -> Result<()> {
    let mut manager = ConfigManager::new(config_path).await?;

    // Enable hot-reload if configured
    if manager.get_config().await.read().await.enable_hot_reload {
        manager.enable_hot_reload()?;
    }

    CONFIG_MANAGER.set(manager)
        .map_err(|_| anyhow::anyhow!("Config already initialized"))?;

    Ok(())
}

/// Get configuration manager instance
pub fn get_config_manager() -> Option<&'static ConfigManager> {
    CONFIG_MANAGER.get()
}

/// Score with dynamic feature weights
pub async fn score_dynamic(features: &GateFeatures, weights: &FeatureWeights) -> f32 {
    let text_ratio = if features.html_bytes == 0 {
        0.0
    } else {
        features.visible_text_chars as f32 / features.html_bytes as f32
    };

    let script_density = if features.html_bytes == 0 {
        0.0
    } else {
        features.script_bytes as f32 / features.html_bytes as f32
    };

    let mut score = 0.0;

    // Apply dynamic weights
    score += (text_ratio * weights.text_ratio_weight).clamp(0.0, weights.text_ratio_cap);
    score += ((features.p_count as f32 + 1.0).ln() * weights.paragraph_log_scale)
        .clamp(0.0, weights.paragraph_cap);

    if features.article_count > 0 {
        score += weights.article_bonus;
    }
    if features.has_og {
        score += weights.og_bonus;
    }
    if features.has_jsonld_article {
        score += weights.jsonld_bonus;
    }

    score -= (script_density * weights.script_density_penalty)
        .clamp(0.0, weights.script_density_cap);

    if features.spa_markers >= 2 {
        score -= weights.spa_marker_penalty;
    }

    let domain_adjustment = (features.domain_prior - 0.5) * weights.domain_prior_weight;

    (score + domain_adjustment).clamp(0.0, 1.0)
}

/// Decide with dynamic thresholds
pub async fn decide_dynamic(
    features: &GateFeatures,
    thresholds: &ThresholdSet,
) -> Decision {
    let content_score = score_dynamic(features, &thresholds.feature_weights).await;

    if content_score >= thresholds.threshold_raw {
        Decision::Raw
    } else if content_score <= thresholds.threshold_probes
        || features.spa_markers >= thresholds.spa_marker_threshold {
        Decision::Headless
    } else {
        Decision::ProbesFirst
    }
}
```

---

## 5. A/B Testing Framework

### 5.1 Variant Selection Strategy

```rust
// crates/riptide-core/src/gate/ab_testing.rs

use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A/B test selector for threshold variants
pub struct ABTestSelector {
    /// Active A/B test variants
    variants: Vec<ABTestVariant>,

    /// Cache of URL → variant assignments
    assignment_cache: HashMap<String, String>,
}

impl ABTestSelector {
    /// Create new A/B test selector
    pub fn new(config: &GateConfig) -> Self {
        let variants = if config.enable_ab_testing {
            config.ab_tests.iter()
                .filter(|v| v.enabled && Self::is_active(v))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        Self {
            variants,
            assignment_cache: HashMap::new(),
        }
    }

    /// Check if variant is currently active based on dates
    fn is_active(variant: &ABTestVariant) -> bool {
        let now = chrono::Utc::now();

        if let Some(start) = &variant.start_date {
            if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(start) {
                if now < start_time {
                    return false;
                }
            }
        }

        if let Some(end) = &variant.end_date {
            if let Ok(end_time) = chrono::DateTime::parse_from_rfc3339(end) {
                if now > end_time {
                    return false;
                }
            }
        }

        true
    }

    /// Select threshold set for a given URL (deterministic)
    pub fn select_variant(&mut self, url: &str, default: &ThresholdSet) -> (ThresholdSet, Option<String>) {
        // Check cache first
        if let Some(variant_id) = self.assignment_cache.get(url) {
            if let Some(variant) = self.variants.iter().find(|v| &v.variant_id == variant_id) {
                return (variant.thresholds.clone(), Some(variant_id.clone()));
            }
        }

        // No active variants
        if self.variants.is_empty() {
            return (default.clone(), None);
        }

        // Deterministic assignment based on URL hash
        let hash = Self::hash_url(url);
        let roll = (hash % 100) as f32 / 100.0; // 0.0 - 0.99

        let mut cumulative = 0.0;
        for variant in &self.variants {
            cumulative += variant.traffic_percentage;
            if roll < cumulative {
                self.assignment_cache.insert(url.to_string(), variant.variant_id.clone());
                return (variant.thresholds.clone(), Some(variant.variant_id.clone()));
            }
        }

        // Default (control group)
        (default.clone(), None)
    }

    /// Hash URL to 0-99 range for consistent assignment
    fn hash_url(url: &str) -> u32 {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let result = hasher.finalize();
        u32::from_be_bytes([result[0], result[1], result[2], result[3]]) % 100
    }

    /// Get variant by ID
    pub fn get_variant(&self, variant_id: &str) -> Option<&ABTestVariant> {
        self.variants.iter().find(|v| v.variant_id == variant_id)
    }
}
```

### 5.2 Integration with Decision Flow

```rust
// Modified decide function with A/B testing

pub async fn decide_with_ab_testing(
    features: &GateFeatures,
    url: &str,
) -> (Decision, Option<String>) {
    let manager = get_config_manager()
        .expect("Config manager not initialized");

    let config = manager.get_config().await;
    let config_guard = config.read().await;

    // Get domain-specific thresholds
    let domain = extract_domain(url);
    let default_thresholds = manager.get_thresholds_for_domain(&domain).await;

    // A/B test variant selection
    let mut selector = ABTestSelector::new(&config_guard);
    let (thresholds, variant_id) = selector.select_variant(url, &default_thresholds);

    // Make decision with selected thresholds
    let decision = decide_dynamic(features, &thresholds).await;

    (decision, variant_id)
}
```

---

## 6. Performance Tracking

### 6.1 Metrics Schema

```rust
// crates/riptide-core/src/gate/metrics.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metrics for gate decision performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateMetrics {
    /// Request ID
    pub request_id: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// URL being processed
    pub url: String,

    /// Domain
    pub domain: String,

    /// A/B test variant (if any)
    pub variant_id: Option<String>,

    /// Threshold set used
    pub threshold_set_name: String,

    /// Calculated score
    pub score: f32,

    /// Decision made
    pub decision: String, // "Raw", "ProbesFirst", "Headless"

    /// Feature values
    pub features: GateFeatures,

    /// Extraction outcome
    pub outcome: ExtractionOutcome,

    /// Performance measurements
    pub performance: PerformanceMetrics,
}

/// Outcome of the extraction attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionOutcome {
    /// Whether extraction succeeded
    pub success: bool,

    /// Quality score (0.0-1.0)
    pub quality_score: Option<f32>,

    /// Extracted text length
    pub extracted_text_length: Option<usize>,

    /// Number of fallback attempts
    pub fallback_attempts: u8,

    /// Final extraction method used
    pub final_method: String, // "raw", "headless"

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Performance measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total time (milliseconds)
    pub total_time_ms: u64,

    /// Gate decision time (milliseconds)
    pub gate_time_ms: u64,

    /// Extraction time (milliseconds)
    pub extraction_time_ms: u64,

    /// Memory usage (bytes)
    pub memory_bytes: Option<usize>,
}

/// Aggregated statistics for a variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantStats {
    /// Variant ID
    pub variant_id: String,

    /// Variant name
    pub variant_name: String,

    /// Time period
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    /// Sample size
    pub total_requests: u64,

    /// Success rate
    pub success_rate: f64,

    /// Average quality score
    pub avg_quality_score: f64,

    /// Decision breakdown
    pub decision_raw_count: u64,
    pub decision_probes_count: u64,
    pub decision_headless_count: u64,

    /// Performance
    pub avg_total_time_ms: f64,
    pub avg_extraction_time_ms: f64,
    pub p95_total_time_ms: u64,
    pub p99_total_time_ms: u64,

    /// Fallback rate
    pub fallback_rate: f64,
}
```

### 6.2 Metrics Storage

```rust
// Metrics tracker with storage backend

use tokio::sync::mpsc;

pub struct MetricsTracker {
    /// Channel for async metrics submission
    sender: mpsc::Sender<GateMetrics>,

    /// Storage backend
    storage: Box<dyn MetricsStorage + Send + Sync>,
}

#[async_trait::async_trait]
pub trait MetricsStorage: Send + Sync {
    /// Store a single metric
    async fn store(&self, metric: GateMetrics) -> Result<()>;

    /// Query metrics by variant
    async fn query_by_variant(
        &self,
        variant_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<GateMetrics>>;

    /// Get aggregated stats
    async fn get_variant_stats(
        &self,
        variant_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<VariantStats>;
}

// Implementation options:
// 1. PostgreSQL/TimescaleDB for production
// 2. SQLite for local development
// 3. Redis for fast access
// 4. File-based JSON lines for simplicity
```

### 6.3 Storage Implementation Options

#### Option 1: PostgreSQL/TimescaleDB (Production)

```sql
-- Schema for gate metrics

CREATE TABLE gate_metrics (
    id BIGSERIAL PRIMARY KEY,
    request_id VARCHAR(64) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    url TEXT NOT NULL,
    domain VARCHAR(255) NOT NULL,
    variant_id VARCHAR(64),
    threshold_set_name VARCHAR(64) NOT NULL,
    score REAL NOT NULL,
    decision VARCHAR(32) NOT NULL,

    -- Feature values (JSONB for flexibility)
    features JSONB NOT NULL,

    -- Outcome
    success BOOLEAN NOT NULL,
    quality_score REAL,
    extracted_text_length INTEGER,
    fallback_attempts SMALLINT NOT NULL,
    final_method VARCHAR(32) NOT NULL,
    error TEXT,

    -- Performance
    total_time_ms BIGINT NOT NULL,
    gate_time_ms BIGINT NOT NULL,
    extraction_time_ms BIGINT NOT NULL,
    memory_bytes BIGINT
);

-- Indexes for efficient queries
CREATE INDEX idx_gate_metrics_timestamp ON gate_metrics (timestamp DESC);
CREATE INDEX idx_gate_metrics_variant ON gate_metrics (variant_id, timestamp DESC);
CREATE INDEX idx_gate_metrics_domain ON gate_metrics (domain, timestamp DESC);
CREATE INDEX idx_gate_metrics_decision ON gate_metrics (decision, timestamp DESC);

-- Hypertable for TimescaleDB (time-series optimization)
SELECT create_hypertable('gate_metrics', 'timestamp');

-- Aggregate view for variant stats
CREATE MATERIALIZED VIEW variant_stats_hourly AS
SELECT
    variant_id,
    time_bucket('1 hour', timestamp) AS bucket,
    COUNT(*) AS total_requests,
    AVG(CASE WHEN success THEN 1 ELSE 0 END) AS success_rate,
    AVG(quality_score) AS avg_quality_score,
    SUM(CASE WHEN decision = 'Raw' THEN 1 ELSE 0 END) AS decision_raw_count,
    SUM(CASE WHEN decision = 'ProbesFirst' THEN 1 ELSE 0 END) AS decision_probes_count,
    SUM(CASE WHEN decision = 'Headless' THEN 1 ELSE 0 END) AS decision_headless_count,
    AVG(total_time_ms) AS avg_total_time_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY total_time_ms) AS p95_total_time_ms,
    AVG(CASE WHEN fallback_attempts > 0 THEN 1 ELSE 0 END) AS fallback_rate
FROM gate_metrics
GROUP BY variant_id, bucket;
```

#### Option 2: File-Based JSON Lines (Development/Simple)

```rust
pub struct JsonLinesStorage {
    file_path: PathBuf,
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl JsonLinesStorage {
    pub async fn new(file_path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;

        Ok(Self {
            file_path: file_path.as_ref().to_path_buf(),
            writer: Arc::new(Mutex::new(BufWriter::new(file))),
        })
    }
}

#[async_trait::async_trait]
impl MetricsStorage for JsonLinesStorage {
    async fn store(&self, metric: GateMetrics) -> Result<()> {
        let mut writer = self.writer.lock().await;
        let line = serde_json::to_string(&metric)?;
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    // ... other methods
}
```

---

## 7. Recommendation Engine

### 7.1 Analysis Algorithm

```rust
// crates/riptide-core/src/gate/recommendations.rs

use std::collections::HashMap;

/// Recommendation engine for optimal threshold selection
pub struct RecommendationEngine {
    /// Metrics storage backend
    storage: Box<dyn MetricsStorage + Send + Sync>,
}

impl RecommendationEngine {
    /// Analyze variant performance and recommend best thresholds
    pub async fn analyze_variants(
        &self,
        period_days: u32,
    ) -> Result<VariantRecommendation> {
        let end = Utc::now();
        let start = end - chrono::Duration::days(period_days as i64);

        // Get stats for all variants
        let variants = self.get_all_variant_ids().await?;
        let mut stats_map = HashMap::new();

        for variant_id in variants {
            let stats = self.storage.get_variant_stats(
                &variant_id,
                start,
                end,
            ).await?;
            stats_map.insert(variant_id, stats);
        }

        // Analyze performance
        let recommendation = self.compute_recommendation(stats_map)?;

        Ok(recommendation)
    }

    /// Compute recommendation based on variant stats
    fn compute_recommendation(
        &self,
        stats: HashMap<String, VariantStats>,
    ) -> Result<VariantRecommendation> {
        // Scoring criteria:
        // 1. Success rate (40%)
        // 2. Quality score (30%)
        // 3. Performance/speed (20%)
        // 4. Fallback rate (10% - lower is better)

        let mut scored_variants = Vec::new();

        for (variant_id, stats) in stats {
            let success_score = stats.success_rate * 0.4;
            let quality_score = stats.avg_quality_score * 0.3;
            let speed_score = self.normalize_speed(stats.avg_total_time_ms) * 0.2;
            let fallback_score = (1.0 - stats.fallback_rate) * 0.1;

            let total_score = success_score + quality_score + speed_score + fallback_score;

            scored_variants.push((variant_id, stats, total_score));
        }

        // Sort by total score (descending)
        scored_variants.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Generate recommendation
        let best_variant = scored_variants.first()
            .ok_or_else(|| anyhow::anyhow!("No variants to analyze"))?;

        Ok(VariantRecommendation {
            recommended_variant_id: best_variant.0.clone(),
            recommended_variant_name: best_variant.1.variant_name.clone(),
            overall_score: best_variant.2,
            analysis_period_days: period_days,
            comparison: scored_variants.into_iter()
                .map(|(id, stats, score)| VariantComparison {
                    variant_id: id,
                    variant_name: stats.variant_name,
                    overall_score: score,
                    success_rate: stats.success_rate,
                    avg_quality_score: stats.avg_quality_score,
                    avg_time_ms: stats.avg_total_time_ms,
                    fallback_rate: stats.fallback_rate,
                })
                .collect(),
            recommendation_text: self.generate_recommendation_text(&scored_variants),
        })
    }

    /// Normalize speed metric (faster = higher score)
    fn normalize_speed(&self, avg_time_ms: f64) -> f64 {
        // Normalize to 0-1 range
        // Assuming 100ms is ideal, 2000ms is poor
        let ideal_ms = 100.0;
        let poor_ms = 2000.0;

        ((poor_ms - avg_time_ms) / (poor_ms - ideal_ms)).clamp(0.0, 1.0)
    }

    /// Generate human-readable recommendation text
    fn generate_recommendation_text(&self, variants: &[(String, VariantStats, f64)]) -> String {
        if variants.is_empty() {
            return "Insufficient data for recommendation.".to_string();
        }

        let best = &variants[0];
        let control = variants.iter().find(|v| v.0 == "default");

        if let Some(control) = control {
            let improvement = ((best.2 - control.2) / control.2) * 100.0;

            format!(
                "Variant '{}' shows {:.1}% improvement over control.\n\
                 - Success rate: {:.1}% → {:.1}%\n\
                 - Quality score: {:.2} → {:.2}\n\
                 - Avg time: {:.0}ms → {:.0}ms\n\
                 Recommended to promote this variant to production.",
                best.1.variant_name,
                improvement,
                control.1.success_rate * 100.0,
                best.1.success_rate * 100.0,
                control.1.avg_quality_score,
                best.1.avg_quality_score,
                control.1.avg_total_time_ms,
                best.1.avg_total_time_ms
            )
        } else {
            format!(
                "Variant '{}' has the best overall performance:\n\
                 - Success rate: {:.1}%\n\
                 - Quality score: {:.2}\n\
                 - Avg time: {:.0}ms\n\
                 - Fallback rate: {:.1}%",
                best.1.variant_name,
                best.1.success_rate * 100.0,
                best.1.avg_quality_score,
                best.1.avg_total_time_ms,
                best.1.fallback_rate * 100.0
            )
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariantRecommendation {
    pub recommended_variant_id: String,
    pub recommended_variant_name: String,
    pub overall_score: f64,
    pub analysis_period_days: u32,
    pub comparison: Vec<VariantComparison>,
    pub recommendation_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariantComparison {
    pub variant_id: String,
    pub variant_name: String,
    pub overall_score: f64,
    pub success_rate: f64,
    pub avg_quality_score: f64,
    pub avg_time_ms: f64,
    pub fallback_rate: f64,
}
```

### 7.2 CLI for Recommendations

```bash
# Generate recommendation report
riptide-cli gate recommend --period-days 14

# Output:
# ┌─────────────────────────────────────────────────────────────┐
# │  Gate Threshold Recommendation Report                       │
# │  Analysis Period: 14 days (2025-09-29 to 2025-10-13)       │
# └─────────────────────────────────────────────────────────────┘
#
# RECOMMENDED VARIANT: Aggressive Raw Extraction
# Overall Score: 0.87
#
# Variant 'Aggressive Raw Extraction' shows 12.5% improvement over control.
# - Success rate: 92.3% → 94.1%
# - Quality score: 0.78 → 0.82
# - Avg time: 850ms → 720ms
# Recommended to promote this variant to production.
#
# ┌───────────────────────────────┬──────┬─────────┬─────────┬──────────┬──────────┐
# │ Variant                       │ Score│ Success │ Quality │ Avg Time │ Fallback │
# ├───────────────────────────────┼──────┼─────────┼─────────┼──────────┼──────────┤
# │ Aggressive Raw Extraction     │ 0.87 │  94.1%  │  0.82   │  720ms   │   8.2%   │
# │ Balanced Extraction           │ 0.85 │  93.5%  │  0.81   │  780ms   │   9.5%   │
# │ Production Default (control)  │ 0.77 │  92.3%  │  0.78   │  850ms   │  12.1%   │
# │ Conservative Quality-First    │ 0.74 │  95.2%  │  0.84   │ 1120ms   │   4.3%   │
# └───────────────────────────────┴──────┴─────────┴─────────┴──────────┴──────────┘
```

---

## 8. Implementation Roadmap

### Phase 1: Configuration System (Week 1-2)

**Tasks:**
1. ✅ Create configuration structs (`GateConfig`, `ThresholdSet`, `FeatureWeights`)
2. ✅ Implement TOML/JSON parsing with serde
3. ✅ Add environment variable override support
4. ✅ Write configuration validation
5. ✅ Create default configuration file template
6. ✅ Add configuration unit tests

**Deliverables:**
- `crates/riptide-core/src/gate/config.rs`
- `config/gate_thresholds.toml`
- Configuration tests

### Phase 2: Dynamic Loading (Week 2-3)

**Tasks:**
1. Implement `ConfigManager` with file watching
2. Add hot-reload functionality with `notify` crate
3. Integrate with existing gate module
4. Add graceful fallback for config errors
5. Implement domain-specific override logic
6. Add initialization to application startup

**Deliverables:**
- `crates/riptide-core/src/gate/config_manager.rs`
- Integration tests for hot-reload
- Documentation for configuration setup

### Phase 3: A/B Testing Framework (Week 3-4)

**Tasks:**
1. Implement `ABTestSelector` with deterministic assignment
2. Add variant activation date logic
3. Integrate with decision flow
4. Add variant assignment caching
5. Implement traffic percentage control
6. Add variant selection tests

**Deliverables:**
- `crates/riptide-core/src/gate/ab_testing.rs`
- A/B testing integration tests
- Example A/B test configurations

### Phase 4: Metrics & Performance Tracking (Week 4-6)

**Tasks:**
1. Design metrics schema
2. Implement `MetricsTracker` with async channel
3. Create storage backends:
   - JSON Lines (simple)
   - PostgreSQL (production)
4. Add metrics collection to extraction pipeline
5. Implement aggregated stats queries
6. Create metrics visualization dashboard

**Deliverables:**
- `crates/riptide-core/src/gate/metrics.rs`
- Database migration scripts
- Metrics dashboard (Grafana or custom)

### Phase 5: Recommendation Engine (Week 6-7)

**Tasks:**
1. Implement `RecommendationEngine`
2. Add multi-criteria scoring algorithm
3. Create variant comparison logic
4. Implement recommendation text generation
5. Add CLI command for recommendations
6. Create automated reports

**Deliverables:**
- `crates/riptide-core/src/gate/recommendations.rs`
- CLI command: `riptide-cli gate recommend`
- Recommendation report templates

### Phase 6: Testing & Documentation (Week 7-8)

**Tasks:**
1. Comprehensive integration tests
2. Performance benchmarking
3. Load testing with different configurations
4. User documentation
5. API documentation
6. Migration guide from static thresholds

**Deliverables:**
- Full test suite
- Performance benchmarks
- User guide documentation
- API reference

### Phase 7: Production Rollout (Week 8-10)

**Tasks:**
1. Deploy configuration management
2. Enable A/B testing with 10% traffic
3. Monitor metrics collection
4. Analyze initial results
5. Adjust variants based on data
6. Gradual rollout of winning variant

**Deliverables:**
- Production configuration
- Monitoring dashboards
- Initial performance report
- Rollout post-mortem

---

## 9. Risk Analysis

### 9.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Configuration parsing errors crash system | High | Low | Graceful fallback to default thresholds, comprehensive validation |
| Hot-reload causes race conditions | Medium | Medium | Use `RwLock` for config, atomic updates |
| A/B testing biases results | Medium | Medium | Deterministic URL-based assignment, stratified sampling |
| Metrics collection impacts performance | Medium | Low | Async metrics submission, batching, sampling |
| Database storage failures | High | Low | Fallback to file storage, circuit breaker pattern |

### 9.2 Operational Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Incorrect threshold tuning degrades quality | High | Medium | Gradual rollout, automated quality monitoring, rollback capability |
| A/B test misconfiguration | Medium | Medium | Configuration validation, dry-run mode |
| Metrics storage grows unbounded | Medium | High | Data retention policy, automated cleanup, aggregation |
| Dashboard overwhelms users | Low | Medium | Simplified views, automated recommendations |

### 9.3 Business Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Complexity delays development | Medium | Low | Phased rollout, MVP first |
| Feature adoption is low | Low | Medium | Clear documentation, compelling use cases |
| Overhead doesn't justify benefits | Medium | Low | Measure ROI, showcase improvements |

---

## 10. Success Metrics

### 10.1 System Performance

- **Configuration loading time**: < 50ms
- **Hot-reload latency**: < 100ms
- **A/B variant selection**: < 1ms
- **Metrics collection overhead**: < 5% of request time

### 10.2 Extraction Quality

- **Success rate improvement**: +5-10% over baseline
- **Quality score improvement**: +0.05-0.10 over baseline
- **Fallback rate reduction**: -20-30% over baseline

### 10.3 Operational Efficiency

- **Time to deploy new thresholds**: < 5 minutes (vs. hours for recompile)
- **A/B test setup time**: < 10 minutes
- **Time to recommendation**: < 1 hour for 7-day analysis

---

## 11. Example Usage

### 11.1 Initial Setup

```bash
# Copy default configuration
cp config/gate_thresholds.toml.example config/gate_thresholds.toml

# Edit configuration
vim config/gate_thresholds.toml

# Initialize application with config
GATE_CONFIG_PATH=config/gate_thresholds.toml riptide-server start
```

### 11.2 Running A/B Test

```bash
# Add variant to config
vim config/gate_thresholds.toml

# Example: Add new variant
[[ab_tests]]
variant_id = "test_v2"
name = "Experimental Thresholds"
traffic_percentage = 0.15
enabled = true
# ... threshold values

# Hot-reload (automatically detected)
# OR restart service
systemctl restart riptide

# Monitor metrics
riptide-cli gate metrics --variant test_v2 --period 24h
```

### 11.3 Analyzing Results

```bash
# Get recommendation after 14 days
riptide-cli gate recommend --period-days 14

# Compare variants
riptide-cli gate compare --variants test_v2,default --period 7d

# Export metrics for analysis
riptide-cli gate export --format csv --output gate_metrics.csv
```

### 11.4 Promoting Winning Variant

```bash
# Copy winning variant thresholds to default
vim config/gate_thresholds.toml

# Update [default] section with winning values
[default]
name = "Production Default (updated from test_v2)"
threshold_raw = 0.75  # From test_v2
threshold_probes = 0.32
# ...

# Disable A/B test
[[ab_tests]]
variant_id = "test_v2"
enabled = false  # Disable after promotion

# Hot-reload applies changes immediately
```

---

## 12. Appendix

### 12.1 Dependencies

```toml
# Add to Cargo.toml

[dependencies]
# Configuration
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Hot-reload
notify = "6.0"

# Async
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# Hashing for A/B testing
sha2 = "0.10"
rand = "0.8"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Storage
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"], optional = true }

# Metrics
prometheus = { version = "0.13", optional = true }
```

### 12.2 Configuration Schema

See `/workspaces/eventmesh/docs/design/gate_thresholds_schema.json` for JSON Schema validation.

### 12.3 References

- [Riptide Core Gate Module](/workspaces/eventmesh/crates/riptide-core/src/gate.rs)
- [SPARC Methodology](https://github.com/ruvnet/sparc)
- [Feature Flags Best Practices](https://martinfowler.com/articles/feature-toggles.html)
- [A/B Testing Guide](https://exp-platform.com/Documents/2013%20controlled%20experiments%20CACM.pdf)

---

**End of Design Document**
