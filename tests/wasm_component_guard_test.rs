//! Guard test to prevent WASM component regression
//!
//! This test ensures that the WASM component binary exists and has the expected structure.
//! It serves as a regression guard to prevent the issue where tests were being ignored
//! instead of properly using the Component Model approach.

use std::path::PathBuf;

#[test]
fn test_wasm_component_file_exists() {
    // Check that the componentized WASM file exists
    let component_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm");

    assert!(
        component_path.exists(),
        "WASM component file not found at {:?}. \
         Make sure to build the WASM component first with: \
         cargo build -p riptide-extractor-wasm --target wasm32-wasip2 --release && \
         wasm-tools component new target/wasm32-wasip2/release/riptide_extractor_wasm.wasm \
         --adapt wasi_snapshot_preview1=/tmp/wasi_snapshot_preview1.reactor.wasm \
         -o target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm",
        component_path
    );

    // Check that it's a reasonable size (should be > 100KB after componentization)
    let metadata = std::fs::metadata(&component_path).unwrap();
    assert!(
        metadata.len() > 100_000,
        "WASM component file is too small ({} bytes), may be corrupted",
        metadata.len()
    );

    println!("✅ WASM component file exists and is {} bytes", metadata.len());
}

#[test]
fn test_wasm_component_not_core_module() {
    // This test ensures we're using the componentized version, not the core module
    let component_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm");

    let core_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip2/release/riptide_extractor_wasm.wasm");

    if component_path.exists() && core_path.exists() {
        let component_size = std::fs::metadata(&component_path).unwrap().len();
        let core_size = std::fs::metadata(&core_path).unwrap().len();

        // Component should be larger than core module due to adapter inclusion
        assert!(
            component_size > core_size,
            "Component file ({} bytes) should be larger than core module ({} bytes)",
            component_size, core_size
        );

        println!("✅ Using componentized WASM ({} bytes) instead of core module ({} bytes)",
                 component_size, core_size);
    }
}