use colored::*;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Table};
use serde::Serialize;

pub fn print_json<T: Serialize>(data: &T) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("{} Failed to serialize JSON: {}", "✗".red(), e),
    }
}

pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

pub fn create_table(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(
        headers
            .into_iter()
            .map(|h| Cell::new(h).set_alignment(CellAlignment::Center)),
    );
    table
}

pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

pub fn format_confidence(score: f64) -> String {
    let percentage = (score * 100.0) as u32;
    let color = if percentage >= 90 {
        "green"
    } else if percentage >= 70 {
        "yellow"
    } else {
        "red"
    };

    format!("{}%", percentage.to_string().color(color))
}

pub fn print_key_value(key: &str, value: &str) {
    println!("{}: {}", key.cyan().bold(), value);
}

pub fn print_section(title: &str) {
    println!("\n{}", title.bold().underline());
}
