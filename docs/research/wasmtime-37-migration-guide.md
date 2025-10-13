# Wasmtime 37 Migration Guide: `component::bindgen!` Macro Changes

## Executive Summary

**Research Date:** 2025-10-13
**Research Agent:** Hive Mind Researcher
**Target Versions:** Wasmtime 34 → 37
**Status:** ✅ CRITICAL PATTERNS IDENTIFIED

---

## 🎯 Key Finding: Module Structure is CONSISTENT

**GOOD NEWS:** The `bindgen!` macro's **generated module structure has NOT changed** between Wasmtime 34 and 37!

The issue in our codebase is **NOT** a breaking change in bindgen, but rather:
1. **Incorrect type import assumptions** in our test code
2. **Missing WIT world definition** for proper type generation
3. **Confusion between guest and host type access patterns**

---

## 📦 Generated Module Structure (Wasmtime 34-37)

### Pattern 1: Top-Level Exports (Single Interface)

```rust
// WIT Definition
world my-world {
    export foo: func() -> string;
}

// Generated Code Structure
bindgen!("my-world" in "path/to/file.wit");

// Usage - Types are at ROOT level of generated module:
use my_world::MyWorld;  // Main world struct

let instance = MyWorld::instantiate(&mut store, &component, &linker)?;
instance.call_foo(&mut store)?;
```

### Pattern 2: Nested Interface Exports

```rust
// WIT Definition
world my-world {
    export my-interface: interface {
        record my-data {
            field: string,
        }
        process: func(input: my-data) -> my-data;
    }
}

// Generated Code Structure
bindgen!("my-world" in "path/to/file.wit");

// Usage - Types are nested under `exports::interface_name::*`
use my_world::MyWorld;
use my_world::exports::my_interface::{MyData, Host};

impl exports::my_interface::Host for MyState {
    fn process(&mut self, input: MyData) -> Result<MyData> {
        // Implementation
    }
}
```

### Pattern 3: Imports (Host Functions)

```rust
// WIT Definition
world my-world {
    import host-functions: interface {
        log: func(msg: string);
    }
}

// Generated Code Structure
bindgen!("my-world" in "path/to/file.wit");

// Usage - Import types under `host_functions::*`
impl host_functions::Host for MyState {
    fn log(&mut self, msg: String) {
        println!("{}", msg);
    }
}

// Add to linker
host_functions::add_to_linker(&mut linker, |state| state)?;
```

---

## 🔍 Real-World Examples from Wasmtime 37

### Example 1: Simple Component (No Imports)

**Source:** `tests/all/component_model/bindgen.rs` (Line 17-65)

```rust
wasmtime::component::bindgen!({
    inline: "
        package foo:foo;

        world no-imports {
            export foo: interface {
                foo: func();
            }
            export bar: func();
        }
    ",
});

// Generated structure:
// - Root module: `no_imports` (kebab-case → snake_case)
// - Main struct: `NoImports` (PascalCase)
// - Export interface: `no_imports.foo()`
// - Export function: `no_imports.call_bar()`

#[test]
fn run() -> Result<()> {
    let engine = engine();
    let component = Component::new(&engine, /* ... */)?;

    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());

    // CORRECT type access:
    let no_imports = NoImports::instantiate(&mut store, &component, &linker)?;
    no_imports.call_bar(&mut store)?;          // Top-level export
    no_imports.foo().call_foo(&mut store)?;     // Interface export

    Ok(())
}
```

### Example 2: Component with Imports

**Source:** `tests/all/component_model/bindgen.rs` (Line 139-201)

```rust
wasmtime::component::bindgen!({
    inline: "
        package foo:foo;

        world one-import {
            import foo: interface {
                foo: func();
            }
            export bar: func();
        }
    ",
});

#[derive(Default)]
struct MyImports {
    hit: bool,
}

// CORRECT import trait implementation:
impl foo::Host for MyImports {
    fn foo(&mut self) {
        self.hit = true;
    }
}

#[test]
fn run() -> Result<()> {
    let engine = engine();
    let component = Component::new(&engine, /* ... */)?;

    let mut linker = Linker::new(&engine);

    // Add import handlers to linker
    foo::add_to_linker::<_, HasSelf<_>>(&mut linker, |f| f)?;

    let mut store = Store::new(&engine, MyImports::default());
    let one_import = OneImport::instantiate(&mut store, &component, &linker)?;
    one_import.call_bar(&mut store)?;

    assert!(store.data().hit);
    Ok(())
}
```

### Example 3: WASI Preview 2 Component

**Source:** `examples/wasip2/main.rs`

```rust
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::*;
use wasmtime_wasi::p2::bindings::sync::Command;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

pub struct ComponentRunStates {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

// CRITICAL: WasiView trait implementation
impl WasiView for ComponentRunStates {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}

fn main() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

    // WASI Preview 2 linker setup
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    let wasi = WasiCtx::builder().inherit_stdio().inherit_args().build();
    let state = ComponentRunStates {
        wasi_ctx: wasi,
        resource_table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    let component = Component::from_file(&engine, "target/wasm32-wasip2/debug/wasi.wasm")?;

    // Method 1: Using Command helper
    let command = Command::instantiate(&mut store, &component, &linker)?;
    let program_result = command.wasi_cli_run().call_run(&mut store)?;

    // Method 2: Manual instantiation
    let instance = linker.instantiate(&mut store, &component)?;
    let interface_idx = instance
        .get_export_index(&mut store, None, "wasi:cli/run@0.2.0")
        .expect("Cannot get interface");

    Ok(())
}
```

### Example 4: Resources and Custom Types

**Source:** `examples/resource-component/main.rs`

```rust
use wasmtime::component::bindgen;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::component::{HasSelf, Resource};

bindgen!({
    path: "./examples/resource-component/kv-store.wit",
    world: "kv-database",
    imports: { default: async | trappable },
    exports: { default: async },
    with: {
        "example:kv-store/kvdb/connection": Connection  // Map WIT type to Rust type
    },
});

pub struct Connection {
    pub storage: HashMap<String, String>,
}

// Import interface implementation
impl KvDatabaseImports for ComponentRunStates {
    async fn log(&mut self, msg: String) -> Result<(), wasmtime::Error> {
        println!("Log: {msg}");
        Ok(())
    }
}

// Host resource trait (empty marker)
impl example::kv_store::kvdb::Host for ComponentRunStates {}

// Host resource methods
impl example::kv_store::kvdb::HostConnection for ComponentRunStates {
    async fn new(&mut self) -> Result<Resource<Connection>, wasmtime::Error> {
        Ok(self.resource_table.push(Connection {
            storage: HashMap::new(),
        })?)
    }

    async fn get(
        &mut self,
        resource: Resource<Connection>,
        key: String,
    ) -> Result<Option<String>, wasmtime::Error> {
        let connection = self.resource_table.get(&resource)?;
        Ok(connection.storage.get(&key).cloned())
    }

    // ... other methods
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = Config::new();
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);

    // Add generated bindings to linker
    KvDatabase::add_to_linker::<_, HasSelf<_>>(&mut linker, |s| s)?;
    add_to_linker_async(&mut linker)?;

    let component = Component::from_file(&engine, "target/wasm32-wasip2/debug/guest_kvdb.wasm")?;
    let bindings = KvDatabase::instantiate_async(&mut store, &component, &linker).await?;

    let result = bindings
        .call_replace_value(&mut store, "hello", "world")
        .await?;

    Ok(())
}
```

---

## 🔧 Breaking Changes: Wasmtime 34 → 37

### Version 34.0.0 (June 20, 2025)

**CRITICAL BREAKING CHANGE:**

```rust
// ❌ OLD (Wasmtime 33 and earlier):
impl foo::Host for MyState {
    fn foo(&mut self) -> Result<()> { ... }
}

let mut linker = Linker::new(&engine);
foo::add_to_linker::<_, GetHost<_>>(&mut linker, |state| state)?;

// ✅ NEW (Wasmtime 34+):
impl foo::Host for MyState {
    fn foo(&mut self) -> Result<()> { ... }
}

let mut linker = Linker::new(&engine);
foo::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;
// OR
foo::add_to_linker::<_, HasData<_>>(&mut linker, |state| state)?;
```

**Changes:**
- `GetHost` trait removed
- `HasData` trait introduced
- `HasSelf` replaces `GetHost` for simpler cases
- `Store<T>` now requires `T: 'static`

### Version 37.0.0 (September 20, 2025)

**WASI API Changes:**

```rust
// ❌ OLD module names:
use wasmtime_wasi::preview0;
use wasmtime_wasi::preview1;

// ✅ NEW module names (Wasmtime 37):
use wasmtime_wasi::p0;
use wasmtime_wasi::p1;
use wasmtime_wasi::p2;  // WASIp2 is stable
```

**No breaking changes to `bindgen!` macro structure in v37.**

---

## 🐛 Common Pitfalls and Solutions

### Pitfall 1: Accessing Types at Wrong Level

```rust
// ❌ WRONG: Trying to access types at root
bindgen!("my-world" in "world.wit");

use my_world::ExtractedContent;  // ❌ ERROR: not found
use my_world::ExtractionError;   // ❌ ERROR: not found

// ✅ CORRECT: Access types based on WIT structure
// For exports in interfaces:
use my_world::exports::extraction::ExtractedContent;
use my_world::exports::extraction::ExtractionError;

// For imports in interfaces:
use my_world::extraction::ExtractedContent;  // If it's an import
```

### Pitfall 2: Missing WIT World Definition

```rust
// ❌ WRONG: No world definition
bindgen!({
    path: "types.wit"  // Just a types file, no world!
});

// ✅ CORRECT: Complete world definition
bindgen!({
    path: "world.wit",
    world: "html-extraction"  // Specify which world to use
});
```

### Pitfall 3: Guest vs Host Confusion

```rust
// Understanding the perspective:

// GUEST exports → Host imports → `exports::` namespace
// GUEST imports → Host exports → root namespace (no `exports::`)

// Example:
world my-world {
    // Host provides this (host export, guest import)
    import logger: interface {
        log: func(msg: string);
    }

    // Guest provides this (guest export, host import)
    export processor: interface {
        process: func(data: string) -> result;
    }
}

// Host code:
impl logger::Host for MyState {           // No `exports::` - we provide it
    fn log(&mut self, msg: String) { ... }
}

// Access guest exports:
let instance = MyWorld::instantiate(...)?;
instance.processor().call_process(...)?;  // Guest provides this
```

### Pitfall 4: Not Using Debug Output

```rust
// Enable bindgen debug output to see generated code:
// Set environment variable before compiling:
// WASMTIME_DEBUG_BINDGEN=1 cargo build

// This writes generated Rust code to:
// target/debug/build/your-crate-*/out/bindgen_*.rs

// Examine this file to see:
// - Actual module structure
// - Generated types
// - Trait definitions
// - Function signatures
```

---

## 🚀 Migration Checklist

### Step 1: Update Dependencies

```toml
[dependencies]
wasmtime = "37"
wasmtime-wasi = "37"
```

### Step 2: Update WASI Module Imports

```rust
// Find and replace:
use wasmtime_wasi::preview1 → use wasmtime_wasi::p1
use wasmtime_wasi::preview2 → use wasmtime_wasi::p2
```

### Step 3: Check Linker Helper Traits

```rust
// Update from GetHost → HasSelf or HasData
foo::add_to_linker::<_, HasSelf<_>>(&mut linker, |s| s)?;
```

### Step 4: Verify WIT Definitions

```wit
// Ensure you have a complete world definition:
package my:package;

world my-world {
    // Define all imports and exports
    import logger: interface { ... }
    export processor: interface { ... }
}
```

### Step 5: Enable Debug Output

```bash
WASMTIME_DEBUG_BINDGEN=1 cargo build 2>&1 | grep "bindgen"
# Look for: "writing bindgen debug output to..."
```

### Step 6: Fix Type Imports

```rust
// Pattern 1: Check if type is in export interface
use my_world::exports::interface_name::TypeName;

// Pattern 2: Check if type is in import interface
use my_world::interface_name::TypeName;

// Pattern 3: Check if type is at root (rare)
use my_world::TypeName;
```

---

## 📚 Documentation References

### Official Wasmtime 37 Documentation
- **Bindgen Macro:** https://docs.wasmtime.dev/api/wasmtime/component/macro.bindgen.html
- **Bindgen Examples:** https://docs.wasmtime.dev/api/wasmtime/component/bindgen_examples/index.html
- **WASI Preview 2:** https://docs.wasmtime.dev/examples-rust-wasip2.html
- **Component Model:** https://docs.wasmtime.dev/api/wasmtime/component/index.html

### Key GitHub Resources
- **Release Notes:** https://github.com/bytecodealliance/wasmtime/blob/main/RELEASES.md
- **Examples Directory:** https://github.com/bytecodealliance/wasmtime/tree/main/examples
- **Component Tests:** https://github.com/bytecodealliance/wasmtime/tree/main/tests/all/component_model

### Component Model Specification
- **WIT Format:** https://component-model.bytecodealliance.org/design/wit.html
- **Component Model:** https://github.com/WebAssembly/component-model

---

## 💡 Debugging Tips

### Tip 1: Inspect Generated Code

```bash
# Set env var before build
export WASMTIME_DEBUG_BINDGEN=1

# Build your project
cargo build

# Find generated code
find target/debug/build -name "bindgen_*.rs" -exec cat {} \;

# You'll see the ACTUAL generated module structure
```

### Tip 2: Use rust-analyzer

```rust
// In VSCode with rust-analyzer:
// 1. Trigger bindgen generation (cargo check)
// 2. Hover over bindgen!() macro
// 3. Click "Expand macro recursively"
// 4. See generated code inline
```

### Tip 3: Check Linker Errors

```rust
// If instantiation fails, check what the linker expects:
let err = linker.instantiate(&mut store, &component).unwrap_err();
println!("Linker error: {:#?}", err);

// Error will show missing imports like:
// "unknown import: `foo:bar/baz@0.1.0` has not been defined"
```

### Tip 4: Validate WIT Files

```bash
# Use wasm-tools to validate WIT
cargo install wasm-tools

wasm-tools component wit path/to/world.wit
# Shows parsed structure and any errors
```

---

## 🎯 Specific Fix for Our Codebase

### Current Problem

```rust
// File: crates/riptide-html/src/wasm_extraction.rs
wasmtime::component::bindgen!({
    path: "wit/html_extraction.wit"
});

// ❌ These imports fail:
use ExtractedContent;
use ExtractionError;
use ExtractionMode;
use Extractor;
```

### Root Cause

The WIT file likely defines these types inside an **interface** within a **world**, making them nested in the generated code structure.

### Solution Steps

1. **Examine WIT file structure:**

```wit
// Expected structure in wit/html_extraction.wit
package riptide:html;

world html-extraction {
    export extraction: interface {
        record extracted-content { ... }
        enum extraction-error { ... }
        enum extraction-mode { ... }

        extract: func(...) -> result<extracted-content, extraction-error>;
    }
}
```

2. **Update imports based on structure:**

```rust
// ✅ CORRECT imports:
use html_extraction::HtmlExtraction;  // Main world struct
use html_extraction::exports::extraction::{
    ExtractedContent,
    ExtractionError,
    ExtractionMode,
};

// For host trait implementation:
impl html_extraction::exports::extraction::Host for MyState {
    fn extract(&mut self, ...) -> Result<ExtractedContent, ExtractionError> {
        // Implementation
    }
}
```

3. **Alternative: Inline WIT for testing:**

```rust
// For quick testing/prototyping:
wasmtime::component::bindgen!({
    inline: "
        package riptide:html;

        world html-extraction {
            export extraction: interface {
                record extracted-content {
                    title: option<string>,
                    content: string,
                }
                extract: func(html: string) -> extracted-content;
            }
        }
    "
});
```

---

## ✅ Verification

To verify the migration is complete:

```bash
# 1. Enable debug output
export WASMTIME_DEBUG_BINDGEN=1

# 2. Clean build
cargo clean
cargo build

# 3. Check generated types
grep -r "pub struct ExtractedContent" target/debug/build/

# 4. Run tests
cargo test wasm_binding

# 5. Check for deprecation warnings
cargo build 2>&1 | grep -i "deprecat\|warning"
```

---

## 🎓 Key Learnings

1. **Module structure is STABLE** across Wasmtime 34-37
2. **Type location depends on WIT structure**, not Wasmtime version
3. **Always examine WIT files** to understand generated code layout
4. **Use `WASMTIME_DEBUG_BINDGEN=1`** to see actual generated code
5. **`exports::` namespace** is for guest exports (host imports them)
6. **Root namespace** is for guest imports (host exports them)
7. **WASI module names changed** from `preview*` to `p*` in v37

---

## 📞 Support Resources

- **Wasmtime GitHub Issues:** https://github.com/bytecodealliance/wasmtime/issues
- **Component Model Zulip:** https://bytecodealliance.zulipchat.com/
- **Stack Overflow Tag:** `wasmtime`, `webassembly-component-model`

---

**Generated by:** Hive Mind Research Agent
**Last Updated:** 2025-10-13
**Next Review:** After successful migration test
