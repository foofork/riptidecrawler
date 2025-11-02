fn main() {
    // Generate build-time information using built crate
    built::write_built_file().expect("Failed to generate build-time information");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
}
