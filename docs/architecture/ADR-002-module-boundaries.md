# ADR-002: Module Boundaries and Dependency Management

## Status
**Accepted** - Implementation in progress (Phase 1)

## Context
The EventMesh/RipTide codebase grew organically, resulting in circular dependencies and unclear module boundaries. This created several issues:

1. **Circular Dependencies**: `riptide-core` ↔ `riptide-cli` causing compilation issues
2. **Unclear Responsibilities**: Overlapping functionality between crates
3. **Testing Complexity**: Difficult to test modules in isolation
4. **Maintenance Burden**: Changes cascade across multiple crates

### Current Structure Issues
```
riptide-core (lib)
├── depends on: riptide-cli types (circular!)
├── contains: Browser automation, extraction
└── problems: Mixed responsibilities

riptide-cli (bin)
├── depends on: riptide-core
├── contains: CLI, API client, validation
└── problems: Too much business logic
```

## Decision
**Restructure crates with clear module boundaries and break circular dependencies.**

### New Architecture
```
riptide-types (shared types)
├── Core data structures
├── Error types
├── Configuration types
└── No dependencies on other riptide crates

riptide-browser (browser automation)
├── depends on: riptide-types
├── Engine facade (spider-chrome, chromiumoxide)
├── Browser management
└── Session handling

riptide-extraction (content extraction)
├── depends on: riptide-types
├── CSS selector extraction
├── Regex pattern extraction
├── DOM parsing
├── Spider integration
└── WASM module support

riptide-stealth (fingerprint protection)
├── depends on: riptide-types
├── 8-category fingerprinting system
├── Randomization strategies
└── Detection evasion

riptide-pdf (PDF handling)
├── depends on: riptide-types
├── PDF generation
├── Table extraction
└── Text extraction

riptide-core (orchestration)
├── depends on: riptide-{types,browser,extraction,stealth,pdf}
├── High-level workflows
├── Engine coordination
└── Resource management

riptide-cli (user interface)
├── depends on: riptide-core
├── CLI commands
├── API client
├── Output formatting
└── User interaction
```

## Implementation Details

### Module Responsibilities

#### riptide-types
- **Purpose**: Shared types and interfaces
- **Exports**: Structs, enums, traits, errors
- **Dependencies**: None (only std, serde)
- **Size**: Small, stable API

#### riptide-browser
- **Purpose**: Browser automation abstraction
- **Key Types**: `BrowserEngine`, `Session`, `Page`
- **Pattern**: Engine facade for multiple backends
- **Dependencies**: spider-chrome, chromiumoxide (optional)

#### riptide-extraction
- **Purpose**: Content extraction strategies
- **Key Types**: `Extractor`, `ExtractionMode`, `ExtractionResult`
- **Strategies**: CSS, Regex, DOM, Spider, WASM
- **Dependencies**: scraper, regex, spider

#### riptide-stealth
- **Purpose**: Anti-detection and fingerprinting
- **Key Types**: `FingerprintProfile`, `StealthLevel`
- **Categories**: 8 fingerprinting categories
- **Dependencies**: Browser crates for CDP integration

#### riptide-pdf
- **Purpose**: PDF operations
- **Key Types**: `PdfGenerator`, `TableExtractor`
- **Features**: Generation, text extraction, table extraction
- **Dependencies**: headless-chrome, pdf libraries

#### riptide-core
- **Purpose**: High-level orchestration
- **Key Types**: `RenderConfig`, `ExtractConfig`, `Engine`
- **Role**: Coordinate between specialized crates
- **Dependencies**: All other riptide crates

#### riptide-cli
- **Purpose**: User interface
- **Key Types**: `CliArgs`, `Commands`, `OutputFormat`
- **Role**: Parse input, call core, format output
- **Dependencies**: riptide-core, clap, serde

### Dependency Rules
1. **No circular dependencies** - Enforced by cargo
2. **Types flow downward** - Shared types in riptide-types
3. **CLI depends on Core** - Never the reverse
4. **Core orchestrates** - Doesn't implement domain logic
5. **Specialized crates are independent** - Can be used separately

## Migration Strategy

### Phase 1: Create Shared Types (Week 1)
```bash
cargo new --lib crates/riptide-types
# Move common types
# Update all crates to use riptide-types
```

### Phase 2: Extract Browser Logic (Week 2)
```bash
cargo new --lib crates/riptide-browser
# Move browser code from riptide-core
# Implement engine facade
```

### Phase 3: Extract Specialized Crates (Week 3)
```bash
cargo new --lib crates/riptide-extraction
cargo new --lib crates/riptide-stealth
cargo new --lib crates/riptide-pdf
# Move specialized logic
```

### Phase 4: Refactor Core (Week 4)
```rust
// riptide-core becomes orchestration layer
use riptide_browser::BrowserEngine;
use riptide_extraction::Extractor;
use riptide_stealth::StealthProfile;

pub struct Engine {
    browser: Box<dyn BrowserEngine>,
    extractor: Extractor,
    stealth: StealthProfile,
}
```

## Consequences

### Positive
- **Clear Responsibilities**: Each crate has a single, well-defined purpose
- **No Circular Dependencies**: Clean dependency graph
- **Better Testing**: Test crates in isolation
- **Reusability**: Use riptide-extraction without full CLI
- **Parallel Development**: Teams can work on different crates
- **Easier Onboarding**: Clear module boundaries

### Negative
- **More Crates**: 7 crates vs 3 current
- **Migration Effort**: 3-4 weeks to complete
- **Import Complexity**: More explicit imports needed
- **Build Time**: Potentially longer (but can cache better)

### Mitigation
1. **Gradual Migration**: Phase-based approach
2. **Workspace Features**: Use Cargo workspace for unified builds
3. **Re-exports**: Core can re-export common types
4. **Documentation**: Clear module guide
5. **Examples**: Show how to use individual crates

## Testing Strategy

### Per-Crate Testing
```rust
// riptide-extraction tests (no browser needed)
#[test]
fn test_css_extraction() {
    let html = "<div class='title'>Test</div>";
    let result = extract_css(html, ".title");
    assert_eq!(result, "Test");
}

// riptide-browser tests (mock CDP)
#[test]
async fn test_navigation() {
    let browser = MockBrowser::new();
    browser.navigate("https://example.com").await?;
    assert_eq!(browser.current_url(), "https://example.com");
}
```

### Integration Testing
```rust
// riptide-core integration tests
#[tokio::test]
async fn test_full_workflow() {
    let engine = Engine::builder()
        .browser(BrowserType::Spider)
        .stealth(StealthLevel::Maximum)
        .build()?;

    let result = engine
        .render("https://example.com")
        .await?;

    assert!(result.html.contains("<html"));
}
```

## Performance Impact

### Build Performance
- **Before**: Single large crate, incremental builds difficult
- **After**: Parallel crate builds, better caching
- **Expected**: 20-30% faster incremental builds

### Runtime Performance
- **No impact**: Same compiled code
- **Binary size**: Slightly smaller (dead code elimination per crate)

## Related ADRs
- ADR-001: Browser Automation Strategy
- ADR-003: Stealth Architecture
- ADR-004: Extraction Strategies

## Success Metrics
- ✅ Zero circular dependencies
- ✅ Each crate <5000 lines of code
- ✅ 100% test pass rate during migration
- ✅ All crates compile independently
- ✅ <500ms build time for individual crate changes

## References
- [Rust API Guidelines - Module Organization](https://rust-lang.github.io/api-guidelines/)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)

---
**Last Updated**: 2025-10-17
**Approved By**: Architecture Team
**Review Date**: 2025-11-17
