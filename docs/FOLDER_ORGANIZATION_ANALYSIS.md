# Folder Organization Analysis

## Summary of Findings

This document explains the apparent "duplicate" folders in the project root and provides recommendations.

---

## 1. CLI Directories: `/cli` vs `/crates/riptide-cli`

### Status: ✅ **CORRECT - Keep Both**

**Purpose:** Two separate CLIs for different ecosystems

#### `/cli` - Node.js/JavaScript CLI
- **Language**: JavaScript/Node.js
- **Package**: `@riptide/cli` (npm)
- **Function**: Lightweight HTTP client wrapper for RipTide API
- **Dependencies**: axios, commander, chalk, inquirer
- **Target**: npm ecosystem, JavaScript developers
- **Requires**: Running API server
- **Use case**: Easy `npx` execution, scripting, CI/CD integration

#### `/crates/riptide-cli` - Native Rust CLI
- **Language**: Rust
- **Package**: `riptide-cli` (cargo)
- **Function**: Full-featured native CLI with local processing
- **Dependencies**: All Rust crates (extraction, browser, PDF, workers)
- **Target**: Rust ecosystem, performance-critical use cases
- **Requires**: No API server needed
- **Use case**: Local extraction, browser automation, PDF processing

**Recommendation**: ✅ **Keep both** - This is a smart multi-ecosystem strategy. The CLI README has been updated to clarify the distinction.

---

## 2. Benchmark Directories: `/benches` vs `/benchmarks`

### Status: ✅ **CORRECT - Keep Both**

**Purpose:** Standard Rust convention - code vs configuration

#### `/benches` - Benchmark Code (Criterion.rs Convention)
- **Contents**: Actual Rust benchmark files (`.rs`)
  - `facade_benchmark.rs` (548 lines)
  - `hybrid_launcher_benchmark.rs` (374 lines)
  - `performance_benchmarks.rs` (370 lines)
  - `wasm_performance.rs` (392 lines)
- **Purpose**: Executable benchmark suites using Criterion.rs
- **Standard**: This is the standard Cargo convention (`/benches/` directory)

#### `/benchmarks` - Benchmark Configuration & Documentation
- **Contents**:
  - `baseline-config.toml` - Performance baselines and thresholds
  - `README.md` - Documentation for benchmark suite
  - `*.log` - Benchmark results and history
- **Purpose**: Configuration, documentation, and results storage
- **Not code**: No executable benchmarks here

**Recommendation**: ✅ **Keep both** - This follows standard Rust project conventions. The separation is intentional and correct.

**Analogy**: It's like having `/src` (code) and `/docs` (documentation) - different purposes.

---

## 3. Configuration Directories: `/config` vs `/configs`

### Status: ⚠️ **POTENTIALLY REDUNDANT - Consider Consolidation**

**Purpose:** Both contain configuration files, but with different scopes

#### `/config` - Operational Configuration
- **Contents**:
  - `feature-flags/` - Compile-time and runtime feature toggles
  - `monitoring/` - Grafana dashboards, streaming alerts
  - `gate_thresholds.toml.example` - Decision threshold templates
- **Focus**: Build-time and operational configs
- **Subdirectories**: Organized by function

#### `/configs` - Runtime Application Configuration
- **Contents**:
  - `riptide.yml` - Main application configuration
  - `features.yml` - Feature configurations
  - `policies.yml` - Policy definitions
  - `fingerprints.yml` - Fingerprint configurations
  - `resource_management.toml` - Resource limits
  - `ua_list.txt` - User agent strings
  - `grafana/dashboards/` - Parser dashboard
- **Focus**: Runtime application configs
- **Structure**: Flat with one grafana subdirectory

### Analysis

**Overlap Detected:**
- Both have Grafana dashboard files
- Both contain feature-related configs (`config/feature-flags` vs `configs/features.yml`)
- Purpose distinction is unclear

**Potential Issues:**
1. Developers may not know where to add new configs
2. Related configs are separated
3. Grafana configs are duplicated across both

### Recommendations

#### Option 1: Consolidate into `/config` (Recommended)
```
/config/
├── application/          # Runtime app configs
│   ├── riptide.yml
│   ├── features.yml
│   ├── policies.yml
│   ├── fingerprints.yml
│   └── resource_management.toml
├── feature-flags/        # Feature toggles
│   ├── compile-time.toml
│   └── runtime.json
├── monitoring/           # All monitoring configs
│   ├── dashboards/
│   │   ├── grafana-streaming-dashboard.json
│   │   └── grafana-parser-dashboard.json
│   └── alerts/
│       └── streaming-alerts.yaml
├── gate_thresholds.toml.example
└── ua_list.txt
```

#### Option 2: Keep Separate with Clear Purpose
```
/config/                  # BUILD-TIME & OPERATIONAL
├── feature-flags/
├── gate_thresholds.toml.example
└── monitoring/

/configs/                 # RUNTIME APPLICATION
├── application.yml       # Renamed from riptide.yml for clarity
├── features.yml
├── policies.yml
└── ...
```

**Preferred**: **Option 1** - Single `/config` directory with logical subdirectories is clearer and more maintainable.

---

## Summary Table

| Directory Pair | Status | Recommendation |
|----------------|--------|----------------|
| `/cli` vs `/crates/riptide-cli` | ✅ Correct | Keep both - different ecosystems |
| `/benches` vs `/benchmarks` | ✅ Correct | Keep both - code vs config |
| `/config` vs `/configs` | ⚠️ Redundant | Consolidate into `/config` |

---

## Action Items

- [x] **CLI clarification** - Updated `/cli/README.md` with ecosystem distinction
- [ ] **Configuration consolidation** - Decide on Option 1 or Option 2 for config folders
- [ ] **Documentation update** - Update CONTRIBUTING.md with folder structure guidelines
- [ ] **Migration script** - If consolidating, create script to move files safely

---

## Notes

The `/benches` vs `/benchmarks` pattern is actually a best practice in the Rust ecosystem:
- Similar to having `/src` (code) and `/docs` (documentation)
- Cargo automatically looks for benchmarks in `/benches`
- Configuration and results naturally go in `/benchmarks`

Many mature Rust projects follow this pattern (tokio, serde, etc.).
