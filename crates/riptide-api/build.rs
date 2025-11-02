use std::env;

fn main() {
    // Generate build-time information using built crate
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = std::path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");

    built::write_built_file().expect("Failed to generate build-time information");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
}
