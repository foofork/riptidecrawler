use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use std::{fs, path::Path};
use walkdir::WalkDir;

static RE_LET_UNDERSCORE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?m)^\s*let\s+(_[A-Za-z0-9_]+)\s*=\s*(.+?);\s*$"#).unwrap());
static RE_TODO: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\b(TODO|FIXME|HACK|XXX)\b"#).unwrap());
static RE_GUARD: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(Mutex|RwLock|parking_lot|RwLockReadGuard|RwLockWriteGuard)"#).unwrap()
});
static RE_SPAWN: Lazy<Regex> = Lazy::new(|| Regex::new(r#"tokio::spawn\s*\("#).unwrap());
static RE_RESULT_HINT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(Result<|anyhow::Result|eyre::Result|std::io::Result|\?)"#).unwrap()
});
static RE_SPIDER_CONFIG: Lazy<Regex> = Lazy::new(|| Regex::new(r#"SpiderConfigBuilder"#).unwrap());
static RE_CONNECTION_POOL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"ConnectionPool|Pool<"#).unwrap());
static RE_AXUM_HANDLER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(Extension|State|Query|Json|Path)\s*\("#).unwrap());

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Code quality scanner for RipTide EventMesh", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan codebase for code quality issues
    Scan {
        /// Scan only a specific crate (e.g., riptide-api)
        #[arg(long)]
        crate_name: Option<String>,
    },
    /// Apply automated fixes
    Apply {
        /// Fix mode: simple (safe transformations only)
        #[arg(long, default_value = "simple")]
        mode: String,

        /// Apply fixes only to a specific crate
        #[arg(long)]
        crate_name: Option<String>,
    },
}

#[derive(Clone, Debug)]
struct Finding {
    category: &'static str,
    crate_name: String,
    file: String,
    line: usize,
    variable: String,
    snippet: String,
    initial_guess: &'static str,
}

fn get_crate_name(path: &Path) -> Option<String> {
    // Extract crate name from path like crates/riptide-api/src/...
    let components: Vec<_> = path.components().collect();
    for (i, comp) in components.iter().enumerate() {
        if comp.as_os_str() == "crates" && i + 1 < components.len() {
            return components[i + 1]
                .as_os_str()
                .to_str()
                .map(|s| s.to_string());
        }
        if comp.as_os_str() == "playground" {
            return Some("playground".to_string());
        }
    }
    None
}

fn scan_dir(root: &Path, crate_filter: Option<&str>) -> Result<Vec<Finding>> {
    let files: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            // Skip target and hidden directories
            !p.components().any(|c| {
                let os_str = c.as_os_str();
                os_str == "target" || os_str == "node_modules" || os_str == ".git"
            }) && !p
                .file_name()
                .map(|n| n.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            // Include both Rust (.rs) and JavaScript (.js, .jsx) files
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| matches!(ext, "rs" | "js" | "jsx"))
                .unwrap_or(false)
        })
        .filter(|e| {
            // Apply crate filter if specified
            if let Some(crate_name) = crate_filter {
                get_crate_name(e.path())
                    .map(|cn| cn == crate_name)
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .collect();

    println!("Scanning {} files...", files.len());

    let findings: Vec<Finding> = files
        .par_iter()
        .enumerate()
        .flat_map(|(idx, entry)| {
            if idx % 50 == 0 && idx > 0 {
                println!("  Progress: {}/{} files scanned", idx, files.len());
            }

            let path = entry.path();
            let Ok(text) = fs::read_to_string(path) else {
                eprintln!("Warning: Failed to read {}", path.display());
                return Vec::new();
            };
            let mut rows = Vec::new();
            let crate_name = get_crate_name(path).unwrap_or_else(|| "unknown".to_string());

            for (idx, line) in text.lines().enumerate() {
                let lineno = idx + 1;

                // TODO-like markers
                if RE_TODO.is_match(line) {
                    rows.push(Finding {
                        category: "TODOs",
                        crate_name: crate_name.clone(),
                        file: path.display().to_string(),
                        line: lineno,
                        variable: String::new(),
                        snippet: line.trim().to_string(),
                        initial_guess: "todo",
                    });
                }

                // let _var = ... (only for Rust files)
                if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Some(caps) = RE_LET_UNDERSCORE.captures(line) {
                        let var = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                        let rhs = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();

                        let guess = if RE_SPIDER_CONFIG.is_match(&rhs) {
                            "wire_config?"
                        } else if RE_CONNECTION_POOL.is_match(&rhs) {
                            "guard_lifetime?"
                        } else if RE_SPAWN.is_match(&rhs) {
                            "detached_task?"
                        } else if RE_GUARD.is_match(&rhs) {
                            "guard_lifetime?"
                        } else if RE_RESULT_HINT.is_match(&rhs) {
                            "handle_result?"
                        } else if RE_AXUM_HANDLER.is_match(&rhs) {
                            "axum_handler?"
                        } else {
                            "review"
                        };

                        rows.push(Finding {
                            category: "Underscore lets",
                            crate_name: crate_name.clone(),
                            file: path.display().to_string(),
                            line: lineno,
                            variable: var,
                            snippet: rhs.trim().to_string(),
                            initial_guess: guess,
                        });
                    }
                }
            }
            rows
        })
        .collect();

    println!("Scan complete. Found {} issues.", findings.len());
    Ok(findings)
}

fn write_markdown(findings: &[Finding]) -> Result<()> {
    fs::create_dir_all(".reports").ok();
    let mut s = String::new();

    s.push_str("# Triage: Underscore Bindings & TODOs\n\n");
    s.push_str("**Generated:** ");
    s.push_str(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    s.push_str("\n\n");
    s.push_str("Fill the **Decision** column using one of:\n\n");
    s.push_str("- `side_effect_only` · `promote_and_use` · `handle_result` · `guard_lifetime` · `detached_ok` · `wire_config` · `todo_tasked` · `axum_handler`\n\n");
    s.push_str("---\n\n");

    // Group by crate first
    let mut crates: Vec<String> = findings
        .iter()
        .map(|f| f.crate_name.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    crates.sort();

    for crate_name in &crates {
        let crate_findings: Vec<_> = findings
            .iter()
            .filter(|f| &f.crate_name == crate_name)
            .cloned()
            .collect();

        if crate_findings.is_empty() {
            continue;
        }

        s.push_str(&format!("## Crate: `{}`\n\n", crate_name));

        for category in ["Underscore lets", "TODOs"] {
            let mut group: Vec<_> = crate_findings
                .iter()
                .filter(|f| f.category == category)
                .cloned()
                .collect();

            if group.is_empty() {
                continue;
            }

            group.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

            s.push_str(&format!("### {}\n\n", category));
            s.push_str(
                "| ✔ | File | Line | Variable | Snippet / Context | Initial | Decision | Notes |\n",
            );
            s.push_str(
                "|:-:|:-----|-----:|:--------:|:------------------|:--------|:---------|:------|\n",
            );

            for f in group {
                let check = " "; // space placeholder for manual ticking
                let var = if f.variable.is_empty() {
                    "—"
                } else {
                    f.variable.as_str()
                };
                // Escape pipes inside snippet for GFM tables and truncate if too long
                let snippet = f.snippet.replace('|', r"\|");
                let snippet = if snippet.len() > 80 {
                    format!("{}...", &snippet[..77])
                } else {
                    snippet
                };

                // Make file path relative if possible
                let rel_file = f
                    .file
                    .strip_prefix("/workspaces/eventmesh/")
                    .unwrap_or(&f.file);

                s.push_str(&format!(
                    "| {} | `{}` | {} | `{}` | `{}` | `{}` |  |  |\n",
                    check, rel_file, f.line, var, snippet, f.initial_guess
                ));
            }
            s.push('\n');
        }
    }

    // Summary section
    s.push_str("---\n\n## Summary\n\n");
    s.push_str(&format!("- **Total findings:** {}\n", findings.len()));

    let underscore_count = findings
        .iter()
        .filter(|f| f.category == "Underscore lets")
        .count();
    let todo_count = findings.iter().filter(|f| f.category == "TODOs").count();

    s.push_str(&format!(
        "- **Underscore bindings:** {}\n",
        underscore_count
    ));
    s.push_str(&format!("- **TODOs/FIXMEs:** {}\n", todo_count));
    s.push_str(&format!("- **Crates analyzed:** {}\n", crates.len()));

    fs::write(".reports/triage.md", s)?;
    Ok(())
}

fn apply_simple_rewrites(root: &Path, crate_filter: Option<&str>) -> Result<usize> {
    // ONLY transform: single-line `let _name = expr;` → `let _ = expr;`
    let mut changed = 0usize;
    let mut files_changed = 0usize;

    for entry in WalkDir::new(root) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();

        // Skip non-Rust files
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        // Apply crate filter if specified
        if let Some(crate_name) = crate_filter {
            if get_crate_name(path)
                .map(|cn| cn != crate_name)
                .unwrap_or(true)
            {
                continue;
            }
        }

        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let mut line_changes = 0;

        let replaced = RE_LET_UNDERSCORE
            .replace_all(&text, |caps: &regex::Captures| {
                let rhs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                line_changes += 1;
                format!("let _ = {};", rhs.trim())
            })
            .to_string();

        if replaced != text {
            fs::write(path, replaced).with_context(|| format!("writing {}", path.display()))?;
            files_changed += 1;
            changed += line_changes;
            println!("  Fixed {} lines in {}", line_changes, path.display());
        }
    }

    println!("\nApplied simple rewrites:");
    println!("  {} lines changed", changed);
    println!("  {} files modified", files_changed);

    Ok(files_changed)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { crate_name } => {
            println!("🔍 Scanning RipTide EventMesh codebase...");
            if let Some(ref crate_filter) = crate_name {
                println!("   Filtering to crate: {}", crate_filter);
            }
            println!();

            let findings = scan_dir(Path::new("."), crate_name.as_deref())?;
            write_markdown(&findings)?;

            println!(
                "\n✅ Report written to .reports/triage.md ({} findings)",
                findings.len()
            );
            println!("\nNext steps:");
            println!("  1. Review .reports/triage.md");
            println!("  2. Fill in Decision column for each finding");
            println!("  3. Run: cargo run -p xtask -- apply --mode simple");
        }

        Commands::Apply { mode, crate_name } => {
            println!("🔧 Applying fixes...");
            if let Some(ref crate_filter) = crate_name {
                println!("   Filtering to crate: {}", crate_filter);
            }
            println!();

            match mode.as_str() {
                "simple" => {
                    let n = apply_simple_rewrites(Path::new("."), crate_name.as_deref())?;
                    if n > 0 {
                        println!("\n✅ Applied simple rewrites to {} files", n);
                        println!("\nNext steps:");
                        println!("  cargo clippy --all-targets --all-features");
                        println!("  cargo test");
                    } else {
                        println!("\n✅ No files needed changes");
                    }
                }
                _ => {
                    eprintln!("❌ Unknown mode: {}", mode);
                    eprintln!("   Available modes: simple");
                }
            }
        }
    }

    Ok(())
}
