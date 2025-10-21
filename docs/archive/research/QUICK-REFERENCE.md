# Wasmtime 37 Migration - Quick Reference Card

## 🎯 TL;DR - The Fix

```rust
// ✅ CORRECT imports for our project:
mod wit_bindings {
    wasmtime::component::bindgen!({
        world: "extractor",
        path: "../../wasm/riptide-extractor-wasm/wit/extractor.wit",
    });
}

use wit_bindings::{
    Extractor,          // World struct
    ExtractionMode,     // Types at root level
    ExtractedContent,   // (no exports:: prefix)
    ExtractionError,
};

// Usage:
let instance = Extractor::instantiate(&mut store, &component, &linker)?;
let result = instance.call_extract(&mut store, html, url, &mode)?;
```

---

## 🔍 Pattern Recognition

### Is it `type_name` or `exports::type_name`?

**Rule:** Check your WIT file structure!

```wit
// Pattern A: Direct exports → ROOT level
world my-world {
    variant my-type { ... }  // ← Type here
    export func: ...;        // ← Export here
}
// Import as: use wit_bindings::MyType;

// Pattern B: Interface exports → NESTED level
interface my-interface {
    variant my-type { ... }  // ← Type in interface
}
world my-world {
    export my-interface;     // ← Export interface
}
// Import as: use wit_bindings::exports::my_interface::MyType;
```

**Our project uses Pattern A (direct exports).**

---

## 🚨 Breaking Changes by Version

### Wasmtime 34 (June 2025)
```rust
// Changed:
GetHost → HasSelf / HasData
Store<T> requires T: 'static
```

### Wasmtime 37 (September 2025)
```rust
// Changed:
wasmtime_wasi::preview1 → wasmtime_wasi::p1
wasmtime_wasi::preview2 → wasmtime_wasi::p2
```

---

## 🔧 Debug Commands

```bash
# See generated code
export WASMTIME_DEBUG_BINDGEN=1
cargo build -p riptide-extraction

# Find generated file
find target/debug/build/riptide-extraction-*/out -name "bindgen_*.rs" -exec cat {} \; | less

# Validate WIT
wasm-tools component wit path/to/file.wit

# Check relative paths
cd crates/riptide-extraction
ls -la ../../wasm/riptide-extractor-wasm/wit/extractor.wit
```

---

## ⚡ Common Errors → Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| `cannot find type ExtractedContent` | Wrong import path | Use `wit_bindings::ExtractedContent` |
| `no field exports in module wit_bindings` | Pattern A, not B | Remove `exports::` prefix |
| `failed to find world extractor` | Wrong path or typo | Check `path:` in bindgen! |
| `cannot find trait Host` | Missing trait impl | Check if interface-based or world-based |

---

## 📋 5-Minute Fix Checklist

- [ ] Open `crates/riptide-extraction/src/wasm_extraction.rs`
- [ ] Find `use wit_bindings::` or `use ExtractedContent`
- [ ] Replace with: `use wit_bindings::{Extractor, ExtractionMode, ExtractedContent, ExtractionError}`
- [ ] Remove any `exports::` prefixes
- [ ] Set `export WASMTIME_DEBUG_BINDGEN=1`
- [ ] Run `cargo build -p riptide-extraction`
- [ ] Check build output for generated file location
- [ ] Run `cargo test -p riptide-extraction`

---

## 📚 Full Documentation

- **Detailed Guide:** `wasmtime-37-migration-guide.md` (38KB)
- **Project Fix:** `PROJECT-SPECIFIC-FIX.md` (13KB)
- **Summary:** `RESEARCH-SUMMARY.md` (7KB)

---

## 🆘 If Still Broken

1. Check WIT file exists: `ls -la wasm/riptide-extractor-wasm/wit/extractor.wit`
2. Validate WIT: `wasm-tools component wit <file>`
3. Enable debug: `WASMTIME_DEBUG_BINDGEN=1`
4. Read generated code (location shown in build output)
5. Post in Zulip: https://bytecodealliance.zulipchat.com/

---

**Quick Reference Version 1.0**
**Research by:** Hive Mind Research Agent
**Date:** 2025-10-13
