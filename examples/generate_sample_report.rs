//! Example: Generate a sample HTML report with charts
//!
//! This example demonstrates how to use the ReportGenerator to create
//! comprehensive HTML reports with visualizations.
//!
//! Run with:
//! ```bash
//! cargo run --example generate_sample_report
//! ```

use riptide_streaming::reports::{
    ExtractionResult, ReportConfig, ReportFormat, ReportGenerator, ReportTheme,
};
use std::collections::HashMap;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎨 RipTide Report Generator Example\n");

    // Create sample extraction results
    let results = create_rich_sample_data();
    println!("✅ Created {} sample extraction results", results.len());

    // Configure report with modern theme
    let config = ReportConfig {
        title: "RipTide Sample Report - Technology News Analysis".to_string(),
        include_charts: true,
        include_raw_data: false,
        include_metadata: true,
        chart_width: 1000,
        chart_height: 500,
        theme: ReportTheme::Modern,
    };

    println!("📊 Generating HTML report with charts...");

    // Create generator
    let generator = ReportGenerator::with_config(config);

    // Prepare report data
    let report_data = generator
        .prepare_report_data("sample-extraction-2024", results)
        .await?;

    println!("   ├─ Total Results: {}", report_data.total_results);
    println!("   ├─ Total Words: {}", report_data.total_words);
    println!(
        "   ├─ Avg Processing Time: {:.2}ms",
        report_data.average_processing_time
    );
    println!("   ├─ Unique Domains: {}", report_data.domain_stats.len());
    println!("   └─ Charts Generated: {}", report_data.charts.len());

    // Generate HTML report
    let html_report = generator.generate_html_report(&report_data).await?;
    let output_path = "target/sample_report.html";
    fs::write(output_path, &html_report)?;
    println!("\n✅ HTML report saved to: {}", output_path);

    // Generate JSON report
    println!("\n📄 Generating JSON report...");
    let json_report = generator.generate_json_report(&report_data).await?;
    let json_path = "target/sample_report.json";
    fs::write(json_path, &json_report)?;
    println!("✅ JSON report saved to: {}", json_path);

    // Generate CSV report
    println!("\n📊 Generating CSV report...");
    let csv_report = generator.generate_csv_report(&report_data).await?;
    let csv_path = "target/sample_report.csv";
    fs::write(csv_path, &csv_report)?;
    println!("✅ CSV report saved to: {}", csv_path);

    // Generate reports with different themes
    println!("\n🎨 Generating themed reports...");
    for (theme_name, theme) in [
        ("light", ReportTheme::Light),
        ("dark", ReportTheme::Dark),
        ("corporate", ReportTheme::Corporate),
    ] {
        let themed_config = ReportConfig {
            theme: theme.clone(),
            include_charts: false, // Faster generation
            ..config.clone()
        };

        let themed_generator = ReportGenerator::with_config(themed_config);
        let themed_report = themed_generator
            .generate_report("themed-sample", ReportFormat::Html)
            .await?;

        let themed_path = format!("target/sample_report_{}.html", theme_name);
        fs::write(&themed_path, themed_report)?;
        println!("   ├─ {} theme: {}", theme_name, themed_path);
    }

    println!("\n🎉 Report generation complete!");
    println!("\n📖 Open the reports in your browser:");
    println!("   - Main report: file://{}/target/sample_report.html", env!("CARGO_MANIFEST_DIR"));

    Ok(())
}

/// Create rich sample data for demonstration
fn create_rich_sample_data() -> Vec<ExtractionResult> {
    let domains = vec![
        "techcrunch.com",
        "arstechnica.com",
        "theverge.com",
        "wired.com",
        "engadget.com",
    ];

    let titles = vec![
        "AI Breakthrough: New Language Model Achieves Human-Level Performance",
        "Quantum Computing Milestone: First Error-Corrected Quantum Processor",
        "Climate Tech: Revolutionary Carbon Capture Technology Deployed",
        "Space Exploration: Mars Mission Discovers Ancient Water Reserves",
        "Cybersecurity Alert: New Zero-Day Vulnerability Patched",
        "Renewable Energy: Solar Panel Efficiency Reaches 50%",
        "Biotechnology: CRISPR Gene Therapy Shows Promising Results",
        "5G Networks: Global Rollout Accelerates in Urban Centers",
        "Electric Vehicles: Battery Technology Breakthrough Announced",
        "Robotics: Humanoid Robots Enter Manufacturing Sector",
    ];

    let mut results = Vec::new();
    let base_time = chrono::Utc::now() - chrono::Duration::hours(24);

    for i in 0..50 {
        let domain = domains[i % domains.len()];
        let title_idx = i % titles.len();

        results.push(ExtractionResult {
            id: format!("result-{}", i + 1),
            url: format!("https://{}/article/{}", domain, i + 1),
            title: Some(titles[title_idx].to_string()),
            content: generate_sample_content(i),
            metadata: generate_metadata(i),
            timestamp: base_time + chrono::Duration::minutes((i * 30) as i64),
            extraction_time_ms: 150 + (i as u64 % 500),
            word_count: 300 + (i * 100),
            links: generate_links(domain, i),
            images: generate_images(domain, i),
        });
    }

    results
}

fn generate_sample_content(index: usize) -> String {
    let content_samples = vec![
        "Researchers have made significant progress in artificial intelligence, developing systems that can understand and generate human-like text with unprecedented accuracy.",
        "The latest advancements in quantum computing bring us closer to solving previously intractable computational problems in cryptography and optimization.",
        "Climate scientists are leveraging new technologies to combat global warming, with innovative carbon capture methods showing remarkable efficiency.",
        "Space agencies worldwide are collaborating on ambitious missions to explore our solar system and search for signs of extraterrestrial life.",
        "Cybersecurity experts warn of evolving threats as hackers develop more sophisticated techniques to exploit system vulnerabilities.",
    ];

    content_samples[index % content_samples.len()].to_string()
}

fn generate_metadata(index: usize) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    metadata.insert("author".to_string(), serde_json::json!(format!("Author {}", index % 10)));
    metadata.insert("category".to_string(), serde_json::json!("Technology"));
    metadata.insert("tags".to_string(), serde_json::json!(vec!["tech", "innovation", "news"]));
    metadata.insert("reading_time".to_string(), serde_json::json!(5 + (index % 10)));
    metadata
}

fn generate_links(domain: &str, index: usize) -> Vec<String> {
    (0..3)
        .map(|i| format!("https://{}/related/{}", domain, index + i))
        .collect()
}

fn generate_images(domain: &str, index: usize) -> Vec<String> {
    vec![
        format!("https://{}/images/header_{}.jpg", domain, index),
        format!("https://{}/images/content_{}.jpg", domain, index),
    ]
}
