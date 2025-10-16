// Temporary fix script to reorganize TopicChunker methods
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "/workspaces/eventmesh/crates/riptide-extraction/src/chunking/topic.rs";
    let content = fs::read_to_string(file_path)?;

    // Find the trait impl block and move the helper methods out
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut in_trait_impl = false;
    let mut found_fallback = false;
    let mut helper_methods = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.contains("impl ChunkingStrategy for TopicChunker") {
            in_trait_impl = true;
        }

        if in_trait_impl && line.trim() == "}" && i > 740 && i < 750 {
            // End of trait impl, add the closing brace and then add helper methods
            new_lines.push(line.to_string());
            new_lines.push("".to_string());
            new_lines.push("impl TopicChunker {".to_string());
            in_trait_impl = false;
            found_fallback = true;
            continue;
        }

        if found_fallback {
            // Collect the helper methods
            if line.contains("async fn fallback_chunk") ||
               line.contains("fn create_single_chunk") ||
               !helper_methods.is_empty() {
                helper_methods.push(line.to_string());
                if line.trim() == "}" && i > 745 {
                    // End of helper methods
                    new_lines.extend(helper_methods);
                    helper_methods.clear();
                    found_fallback = false;
                }
                continue;
            }
        }

        new_lines.push(line.to_string());
    }

    let new_content = new_lines.join("\n");
    fs::write(file_path, new_content)?;
    println!("Fixed TopicChunker trait implementation");

    Ok(())
}