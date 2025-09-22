use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate build and git information using the correct vergen v8 API
    EmitBuilder::builder()
        .build_timestamp()
        .git_sha(false)
        .git_commit_timestamp()
        .emit()?;

    Ok(())
}
