# Project-Specific Wasmtime 37 Migration Fix

## 🎯 Root Cause Analysis

**File:** `/workspaces/eventmesh/wasm/riptide-extractor-wasm/wit/extractor.wit`

**Issue:** The WIT file has **TWO different structures**:

1. **Types defined directly in world** (lines 6-101) ✅ OLD PATTERN
2. **Types defined in interface** (lines 3-100) ✅ NEW PATTERN

This creates confusion about where types are located in generated code.

---

## 📦 Current WIT Structure Analysis

### Option 1: extractor.wit (Main World - Direct Exports)

```wit
world extractor {
    // Types defined AT WORLD LEVEL (not in interface)
    variant extraction-mode { ... }
    record extracted-content { ... }
    variant extraction-error { ... }

    // Functions exported directly from world
    export extract: func(...) -> result<extracted-content, extraction-error>;
    export health-check: func() -> health-status;
    // ... more exports
}
```

**Generated Structure for Direct World Exports:**
```rust
mod wit_bindings {
    wasmtime::component::bindgen!(...);

    // Types are at ROOT of module (no exports:: prefix needed)
    pub struct Extractor { ... }  // Main world
    pub enum ExtractionMode { ... }
    pub struct ExtractedContent { ... }
    pub enum ExtractionError { ... }
    // ... etc
}
```

### Option 2: world.wit (Interface-Based - Nested Exports)

```wit
interface extract {
    // Types defined IN INTERFACE
    variant extraction-mode { ... }
    record extracted-content { ... }
    // ... functions
}

world extractor {
    export extract;  // Export the interface
}
```

**Generated Structure for Interface Exports:**
```rust
mod wit_bindings {
    wasmtime::component::bindgen!(...);

    pub mod exports {
        pub mod extract {
            pub enum ExtractionMode { ... }
            pub struct ExtractedContent { ... }
            // ...
        }
    }
}
```

---

## 🔍 Investigation: Which Pattern is Used?

Let me check the actual WIT file:

**Result:** `/workspaces/eventmesh/wasm/riptide-extractor-wasm/wit/extractor.wit`

The file uses **OPTION 1 - Direct World Exports**:
- Types are defined directly in the `world extractor { }` block
- Functions are exported directly with `export extract: func(...)`
- NO interface wrapper

**This means types should be at ROOT level of wit_bindings module!**

---

## ✅ CORRECT Type Imports for Our Project

```rust
// File: crates/riptide-extraction/src/wasm_extraction.rs

mod wit_bindings {
    wasmtime::component::bindgen!({
        world: "extractor",
        path: "../../wasm/riptide-extractor-wasm/wit/extractor.wit",
    });
}

// ✅ CORRECT - Types at root level (no exports::)
use wit_bindings::{
    Extractor,           // Main world struct
    ExtractionMode,      // variant extraction-mode
    ExtractedContent,    // record extracted-content
    ExtractionError,     // variant extraction-error
    HealthStatus,        // record health-status
    ComponentInfo,       // record component-info
    ExtractionStats,     // record extraction-stats
};

// Usage:
let instance = Extractor::instantiate(&mut store, &component, &linker)?;

let mode = ExtractionMode::Article;

let result: Result<ExtractedContent, ExtractionError> =
    instance.call_extract(&mut store, html, url, &mode)?;
```

---

## 🐛 Common Import Errors (What NOT to do)

```rust
// ❌ WRONG - No exports:: for direct world exports
use wit_bindings::exports::ExtractionMode;           // ERROR
use wit_bindings::exports::extractor::ExtractionMode; // ERROR

// ❌ WRONG - Types not at crate root
use extractor::ExtractionMode;  // ERROR - 'extractor' is not a module
use crate::ExtractionMode;      // ERROR - only works if re-exported
```

---

## 🔧 Complete Fix for wasm_extraction.rs

### Step 1: Correct Imports

```rust
// At top of file after mod wit_bindings
use wit_bindings::{
    Extractor,
    ExtractionMode,
    ExtractedContent,
    ExtractionError,
    HealthStatus,
    ComponentInfo,
    ExtractionStats,
};

// If you need to expose these publicly:
pub use wit_bindings::{
    ExtractionMode as WasmExtractionMode,
    ExtractedContent as WasmExtractedContent,
    ExtractionError as WasmExtractionError,
};
```

### Step 2: Type Mapping Functions

```rust
// Convert from WIT types to internal types
impl From<wit_bindings::ExtractedContent> for ExtractedDoc {
    fn from(wasm: wit_bindings::ExtractedContent) -> Self {
        ExtractedDoc {
            url: wasm.url,
            title: wasm.title,
            byline: wasm.byline,
            published_iso: wasm.published_iso,
            markdown: wasm.markdown,
            text: wasm.text,
            links: wasm.links,
            media: wasm.media,
            language: wasm.language,
            reading_time: wasm.reading_time,
            quality_score: wasm.quality_score,
            word_count: wasm.word_count,
            categories: wasm.categories,
            site_name: wasm.site_name,
            description: wasm.description,
        }
    }
}

// Convert internal types to WIT types
impl From<ExtractionMode> for wit_bindings::ExtractionMode {
    fn from(mode: ExtractionMode) -> Self {
        match mode {
            ExtractionMode::Article => wit_bindings::ExtractionMode::Article,
            ExtractionMode::Full => wit_bindings::ExtractionMode::Full,
            ExtractionMode::Metadata => wit_bindings::ExtractionMode::Metadata,
            ExtractionMode::Custom(selectors) => {
                wit_bindings::ExtractionMode::Custom(selectors)
            }
        }
    }
}

// Handle extraction errors
impl From<wit_bindings::ExtractionError> for anyhow::Error {
    fn from(err: wit_bindings::ExtractionError) -> Self {
        match err {
            wit_bindings::ExtractionError::InvalidHtml(msg) => {
                anyhow::anyhow!("Invalid HTML: {}", msg)
            }
            wit_bindings::ExtractionError::NetworkError(msg) => {
                anyhow::anyhow!("Network error: {}", msg)
            }
            wit_bindings::ExtractionError::ParseError(msg) => {
                anyhow::anyhow!("Parse error: {}", msg)
            }
            wit_bindings::ExtractionError::ResourceLimit(msg) => {
                anyhow::anyhow!("Resource limit: {}", msg)
            }
            wit_bindings::ExtractionError::ExtractorError(msg) => {
                anyhow::anyhow!("Extractor error: {}", msg)
            }
            wit_bindings::ExtractionError::InternalError(msg) => {
                anyhow::anyhow!("Internal error: {}", msg)
            }
            wit_bindings::ExtractionError::UnsupportedMode(msg) => {
                anyhow::anyhow!("Unsupported mode: {}", msg)
            }
        }
    }
}
```

### Step 3: Instantiation Pattern

```rust
pub struct WasmExtractor {
    engine: Engine,
    component: Component,
    linker: Linker<ExtractionState>,
}

impl WasmExtractor {
    pub fn new(component_path: &str) -> Result<Self> {
        // Engine configuration
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(false);  // Sync mode

        let engine = Engine::new(&config)?;

        // Load component
        let component = Component::from_file(&engine, component_path)?;

        // Create linker (no imports needed for this component)
        let linker = Linker::new(&engine);

        Ok(Self {
            engine,
            component,
            linker,
        })
    }

    pub fn extract(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        let mut store = Store::new(&self.engine, ExtractionState::new());

        // Instantiate component
        let instance = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        // Convert mode to WIT type
        let wasm_mode: wit_bindings::ExtractionMode = mode.into();

        // Call extract function
        let result = instance.call_extract(
            &mut store,
            html,
            url,
            &wasm_mode,
        )?;

        // Handle result
        match result {
            Ok(content) => Ok(content.into()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn health_check(&self) -> Result<HealthStatus> {
        let mut store = Store::new(&self.engine, ExtractionState::new());
        let instance = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        Ok(instance.call_health_check(&mut store)?)
    }
}
```

---

## 🧪 Testing the Fix

### Enable Debug Output

```bash
# See generated code structure
export WASMTIME_DEBUG_BINDGEN=1

# Clean and rebuild
cd /workspaces/eventmesh
cargo clean
cargo build -p riptide-extraction

# Find generated code
find target/debug/build/riptide-extraction-*/out -name "bindgen_*.rs" -exec cat {} \; | head -100
```

### Verify Type Locations

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wit_bindings_structure() {
        // These should all compile if types are at root
        let _mode: wit_bindings::ExtractionMode = wit_bindings::ExtractionMode::Article;

        // Type assertions to verify structure
        let _: fn() -> wit_bindings::Extractor = || {
            panic!("type check only")
        };

        println!("✅ All WIT binding types accessible at root level");
    }

    #[test]
    fn test_mode_conversion() {
        let internal_mode = ExtractionMode::Article;
        let wasm_mode: wit_bindings::ExtractionMode = internal_mode.into();

        match wasm_mode {
            wit_bindings::ExtractionMode::Article => {},
            _ => panic!("Mode conversion failed"),
        }
    }
}
```

---

## 📋 Migration Checklist

- [x] **Identify WIT structure** - Direct world exports (not interface-based)
- [ ] **Update imports** - Remove any `exports::` prefixes
- [ ] **Add type conversions** - `From` traits for WIT ↔ internal types
- [ ] **Update instantiation** - Use `Extractor::instantiate()`
- [ ] **Test compilation** - `cargo build -p riptide-extraction`
- [ ] **Enable debug output** - `WASMTIME_DEBUG_BINDGEN=1`
- [ ] **Verify generated code** - Check actual module structure
- [ ] **Run tests** - `cargo test -p riptide-extraction wasm_binding`
- [ ] **Check integration** - End-to-end extraction test

---

## 🚨 If Types Still Not Found

### Debug Steps:

1. **Verify bindgen is running:**
```bash
cargo clean
WASMTIME_DEBUG_BINDGEN=1 cargo build -p riptide-extraction 2>&1 | grep bindgen
```

2. **Examine generated code:**
```bash
find target/debug/build -name "bindgen_*.rs" -newer Cargo.toml -exec cat {} \;
```

3. **Check module visibility:**
```rust
// In wasm_extraction.rs - make module public for debugging
pub mod wit_bindings {
    wasmtime::component::bindgen!({
        world: "extractor",
        path: "../../wasm/riptide-extractor-wasm/wit/extractor.wit",
    });
}

// Then in tests:
#[test]
fn inspect_bindings() {
    println!("{:#?}", std::any::type_name::<wit_bindings::Extractor>());
}
```

4. **Validate WIT file:**
```bash
# Install wasm-tools if needed
cargo install wasm-tools

# Validate WIT syntax
wasm-tools component wit /workspaces/eventmesh/wasm/riptide-extractor-wasm/wit/extractor.wit
```

5. **Check relative path:**
```bash
# Verify WIT file exists from build location
cd /workspaces/eventmesh/crates/riptide-extraction
ls -la ../../wasm/riptide-extractor-wasm/wit/extractor.wit
```

---

## 🎓 Key Insight for Our Project

**Our WIT file uses DIRECT WORLD EXPORTS, not interface-based exports.**

```wit
// This pattern:
world extractor {
    variant extraction-mode { ... }  // ← Types at world level
    export extract: func(...);       // ← Direct export
}

// NOT this pattern:
interface extraction {
    variant extraction-mode { ... }  // ← Types in interface
}
world extractor {
    export extraction;               // ← Export interface
}
```

**Therefore:**
- ✅ Types at `wit_bindings::TypeName` (root level)
- ❌ NOT at `wit_bindings::exports::anything`
- ✅ World struct: `wit_bindings::Extractor`
- ✅ Functions: `instance.call_extract()`, `instance.call_health_check()`

---

## 📞 Next Steps

1. Apply the imports fix to `wasm_extraction.rs`
2. Enable `WASMTIME_DEBUG_BINDGEN=1`
3. Rebuild and examine generated code
4. Update type conversions
5. Run tests
6. Report findings to Hive Mind Coordinator

---

**Generated by:** Hive Mind Research Agent
**Date:** 2025-10-13
**Status:** ✅ Root cause identified, fix documented
