fn main() {
    println!("cargo:rerun-if-changed=../../wasm/riptide-extractor-wasm/wit/extractor.wit");

    // No build-time generation needed - we use runtime bindgen! macro instead
}
