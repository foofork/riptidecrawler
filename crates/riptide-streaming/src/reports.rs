//! HTML report generation with dynamic templates and visualizations
//!
//! This module provides comprehensive HTML report generation for extraction results,
//! including charts, tables, and interactive visualizations.

use crate::ExtractionResult;
use anyhow::Result;
use base64::Engine;
use chrono::Timelike;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Report format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Html,
    Json,
    Csv,
    Pdf,
}

/// Report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub title: String,
    pub include_charts: bool,
    pub include_raw_data: bool,
    pub include_metadata: bool,
    pub chart_width: u32,
    pub chart_height: u32,
    pub theme: ReportTheme,
}

/// Report theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportTheme {
    Light,
    Dark,
    Corporate,
    Modern,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            title: "RipTide Extraction Report".to_string(),
            include_charts: true,
            include_raw_data: false,
            include_metadata: true,
            chart_width: 800,
            chart_height: 400,
            theme: ReportTheme::Modern,
        }
    }
}

/// Report data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub extraction_id: String,
    pub title: String,
    pub generation_time: chrono::DateTime<chrono::Utc>,
    pub total_results: usize,
    pub total_pages: usize,
    pub total_words: usize,
    pub total_processing_time: u64,
    pub average_processing_time: f64,
    pub success_rate: f64,
    pub results: Vec<ExtractionResult>,
    pub domain_stats: HashMap<String, DomainStats>,
    pub timeline: Vec<TimelineEntry>,
    pub word_cloud_data: Vec<WordFrequency>,
    pub charts: HashMap<String, String>, // Base64 encoded chart images
}

/// Statistics for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    pub domain: String,
    pub count: usize,
    pub total_words: usize,
    pub average_words: f64,
    pub success_rate: f64,
    pub average_processing_time: f64,
}

/// Timeline entry for extraction progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event: String,
    pub count: usize,
    pub rate: f64,
}

/// Word frequency data for word clouds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordFrequency {
    pub word: String,
    pub frequency: usize,
    pub percentage: f64,
}

/// HTML report generator
#[derive(Debug, Clone)]
pub struct ReportGenerator {
    pub handlebars: Handlebars<'static>,
    pub config: ReportConfig,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new() -> Self {
        Self::with_config(ReportConfig::default())
    }

    /// Create a new report generator with custom configuration
    ///
    /// # Panics
    ///
    /// This function will panic if the built-in templates are malformed. This should never
    /// happen in practice as the templates are static constants that are part of the compiled binary.
    /// If a panic occurs, it indicates a critical bug in the template definitions that must be fixed.
    pub fn with_config(config: ReportConfig) -> Self {
        let mut handlebars = Handlebars::new();

        // Register built-in templates
        // These are compile-time constants, so errors indicate a critical programming bug
        // We use expect() here because:
        // 1. Templates are static and part of our codebase
        // 2. Failures indicate developer errors, not runtime conditions
        // 3. The application cannot function without valid templates
        handlebars
            .register_template_string("main", MAIN_TEMPLATE)
            .expect("BUG: Built-in main template is malformed - please report this issue");
        handlebars
            .register_template_string("summary", SUMMARY_TEMPLATE)
            .expect("BUG: Built-in summary template is malformed - please report this issue");
        handlebars
            .register_template_string("results", RESULTS_TEMPLATE)
            .expect("BUG: Built-in results template is malformed - please report this issue");
        handlebars
            .register_template_string("charts", CHARTS_TEMPLATE)
            .expect("BUG: Built-in charts template is malformed - please report this issue");

        // Register helpers
        handlebars.register_helper("format_duration", Box::new(format_duration));
        handlebars.register_helper("format_number", Box::new(format_number));
        handlebars.register_helper("format_percentage", Box::new(format_percentage));
        handlebars.register_helper("truncate", Box::new(truncate));
        handlebars.register_helper("highlight_keywords", Box::new(highlight_keywords));

        Self { handlebars, config }
    }

    /// Generate a report for extraction results
    pub async fn generate_report(
        &self,
        extraction_id: &str,
        format: ReportFormat,
    ) -> Result<Vec<u8>> {
        // This would normally fetch results from storage
        // For now, we'll generate sample data
        let results = self.fetch_extraction_results(extraction_id).await?;
        let report_data = self.prepare_report_data(extraction_id, results).await?;

        match format {
            ReportFormat::Html => self.generate_html_report(&report_data).await,
            ReportFormat::Json => self.generate_json_report(&report_data).await,
            ReportFormat::Csv => self.generate_csv_report(&report_data).await,
            ReportFormat::Pdf => self.generate_pdf_report(&report_data).await,
        }
    }

    /// Generate HTML report
    pub async fn generate_html_report(&self, data: &ReportData) -> Result<Vec<u8>> {
        let mut context = serde_json::to_value(data)?;

        // Add configuration to context
        if let Some(obj) = context.as_object_mut() {
            obj.insert("config".to_string(), serde_json::to_value(&self.config)?);
            obj.insert(
                "theme_css".to_string(),
                serde_json::Value::String(self.get_theme_css()),
            );
        }

        let html = self.handlebars.render("main", &context)?;
        Ok(html.into_bytes())
    }

    /// Generate JSON report
    async fn generate_json_report(&self, data: &ReportData) -> Result<Vec<u8>> {
        let json = serde_json::to_string_pretty(data)?;
        Ok(json.into_bytes())
    }

    /// Generate CSV report
    async fn generate_csv_report(&self, data: &ReportData) -> Result<Vec<u8>> {
        let mut csv = String::new();

        // Header
        csv.push_str(
            "ID,URL,Title,Word Count,Processing Time (ms),Timestamp\
",
        );

        // Data rows
        for result in &data.results {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\
",
                result.id,
                result.url,
                result.title.as_deref().unwrap_or("N/A"),
                result.word_count,
                result.extraction_time_ms,
                result.timestamp.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        Ok(csv.into_bytes())
    }

    /// Generate PDF report (placeholder)
    async fn generate_pdf_report(&self, _data: &ReportData) -> Result<Vec<u8>> {
        // This would require a PDF generation library like wkhtmltopdf or similar
        // For now, return HTML as bytes
        self.generate_html_report(_data).await
    }

    /// Prepare report data from extraction results
    pub async fn prepare_report_data(
        &self,
        extraction_id: &str,
        results: Vec<ExtractionResult>,
    ) -> Result<ReportData> {
        let total_results = results.len();
        let total_words: usize = results.iter().map(|r| r.word_count).sum();
        let total_processing_time: u64 = results.iter().map(|r| r.extraction_time_ms).sum();
        let average_processing_time = if total_results > 0 {
            total_processing_time as f64 / total_results as f64
        } else {
            0.0
        };

        // Calculate domain statistics
        let domain_stats = self.calculate_domain_stats(&results);

        // Generate timeline
        let timeline = self.generate_timeline(&results);

        // Generate word cloud data
        let word_cloud_data = self.generate_word_cloud_data(&results);

        // Generate charts if enabled
        let charts = if self.config.include_charts {
            self.generate_charts(&results, &domain_stats, &timeline)
                .await?
        } else {
            HashMap::new()
        };

        Ok(ReportData {
            extraction_id: extraction_id.to_string(),
            title: self.config.title.clone(),
            generation_time: chrono::Utc::now(),
            total_results,
            total_pages: results.len(), // Assuming one page per result
            total_words,
            total_processing_time,
            average_processing_time,
            success_rate: 100.0, // Would be calculated based on actual success/failure data
            results,
            domain_stats,
            timeline,
            word_cloud_data,
            charts,
        })
    }

    /// Calculate domain statistics
    pub fn calculate_domain_stats(
        &self,
        results: &[ExtractionResult],
    ) -> HashMap<String, DomainStats> {
        let mut stats: HashMap<String, Vec<&ExtractionResult>> = HashMap::new();

        // Group results by domain
        for result in results {
            if let Ok(url) = url::Url::parse(&result.url) {
                if let Some(domain) = url.domain() {
                    stats.entry(domain.to_string()).or_default().push(result);
                }
            }
        }

        // Calculate statistics for each domain
        stats
            .into_iter()
            .map(|(domain, domain_results)| {
                let count = domain_results.len();
                let total_words: usize = domain_results.iter().map(|r| r.word_count).sum();
                let average_words = total_words as f64 / count as f64;
                let total_time: u64 = domain_results.iter().map(|r| r.extraction_time_ms).sum();
                let average_processing_time = total_time as f64 / count as f64;

                (
                    domain.clone(),
                    DomainStats {
                        domain,
                        count,
                        total_words,
                        average_words,
                        success_rate: 100.0, // Would be calculated from actual data
                        average_processing_time,
                    },
                )
            })
            .collect()
    }

    /// Generate timeline entries
    ///
    /// Groups extraction results by hour to create a timeline of activity.
    /// Timestamps are normalized to the beginning of each hour for grouping.
    pub fn generate_timeline(&self, results: &[ExtractionResult]) -> Vec<TimelineEntry> {
        let mut timeline = Vec::new();
        let mut sorted_results: Vec<_> = results.iter().collect();
        sorted_results.sort_by_key(|r| r.timestamp);

        // Group by hour for timeline
        let mut hourly_counts: HashMap<chrono::DateTime<chrono::Utc>, usize> = HashMap::new();

        for result in sorted_results {
            // Normalize timestamp to the beginning of the hour
            // with_minute/with_second/with_nanosecond return None only for invalid values
            // Since we're setting to 0, this should always succeed for valid DateTime objects
            let hour = result
                .timestamp
                .with_minute(0)
                .and_then(|dt| dt.with_second(0))
                .and_then(|dt| dt.with_nanosecond(0))
                .unwrap_or_else(|| {
                    // Fallback: If normalization somehow fails (should be impossible for valid DateTime),
                    // use the original timestamp and log the issue for investigation
                    tracing::warn!(
                        result_id = %result.id,
                        timestamp = %result.timestamp,
                        "Failed to normalize timestamp to hour - using original timestamp. \
                         This indicates a DateTime object in an unexpected state."
                    );
                    result.timestamp
                });

            *hourly_counts.entry(hour).or_insert(0) += 1;
        }

        for (timestamp, count) in hourly_counts {
            timeline.push(TimelineEntry {
                timestamp,
                event: "Extractions Completed".to_string(),
                count,
                rate: count as f64, // Could calculate rate per hour
            });
        }

        timeline.sort_by_key(|t| t.timestamp);
        timeline
    }

    /// Generate word cloud data
    fn generate_word_cloud_data(&self, results: &[ExtractionResult]) -> Vec<WordFrequency> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut total_words = 0;

        for result in results {
            if let Some(title) = &result.title {
                let words: Vec<&str> = title
                    .split_whitespace()
                    .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
                    .filter(|w| w.len() > 3) // Filter out short words
                    .collect();

                for word in words {
                    let word = word.to_lowercase();
                    *word_counts.entry(word).or_insert(0) += 1;
                    total_words += 1;
                }
            }
        }

        let mut word_frequencies: Vec<_> = word_counts
            .into_iter()
            .map(|(word, frequency)| WordFrequency {
                word,
                frequency,
                percentage: frequency as f64 / total_words as f64 * 100.0,
            })
            .collect();

        word_frequencies.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        word_frequencies.truncate(50); // Top 50 words

        word_frequencies
    }

    /// Generate charts
    pub async fn generate_charts(
        &self,
        results: &[ExtractionResult],
        domain_stats: &HashMap<String, DomainStats>,
        timeline: &[TimelineEntry],
    ) -> Result<HashMap<String, String>> {
        let mut charts = HashMap::new();

        // Processing time distribution chart
        if let Ok(chart_data) = self.create_processing_time_chart(results) {
            charts.insert("processing_time".to_string(), chart_data);
        }

        // Domain distribution pie chart
        if let Ok(chart_data) = self.create_domain_pie_chart(domain_stats) {
            charts.insert("domain_distribution".to_string(), chart_data);
        }

        // Timeline chart
        if let Ok(chart_data) = self.create_timeline_chart(timeline) {
            charts.insert("timeline".to_string(), chart_data);
        }

        // Word count distribution
        if let Ok(chart_data) = self.create_word_count_chart(results) {
            charts.insert("word_count".to_string(), chart_data);
        }

        Ok(charts)
    }

    /// Create processing time distribution chart
    fn create_processing_time_chart(&self, results: &[ExtractionResult]) -> Result<String> {
        let mut buffer = Vec::new();
        {
            let root = BitMapBackend::with_buffer(
                &mut buffer,
                (self.config.chart_width, self.config.chart_height),
            )
            .into_drawing_area();
            root.fill(&WHITE)?;

            let times: Vec<u64> = results.iter().map(|r| r.extraction_time_ms).collect();

            // Create histogram buckets
            let max_time = times.iter().max().cloned().unwrap_or(0);
            let bucket_size = (max_time / 10).max(1);
            let mut buckets = [0; 10];

            for time in times {
                let bucket = ((time / bucket_size) as usize).min(9);
                buckets[bucket] += 1;
            }

            let mut chart = ChartBuilder::on(&root)
                .caption("Processing Time Distribution", ("sans-serif", 30))
                .margin(20)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .build_cartesian_2d(0..10, 0..*buckets.iter().max().unwrap_or(&1))?;

            chart.configure_mesh().draw()?;

            chart.draw_series(buckets.iter().enumerate().map(|(i, &count)| {
                Rectangle::new([(i as i32, 0), (i as i32, count)], BLUE.filled())
            }))?;

            root.present()?;
        }

        let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
        Ok(format!("data:image/png;base64,{}", base64))
    }

    /// Create domain distribution pie chart (simplified)
    fn create_domain_pie_chart(
        &self,
        domain_stats: &HashMap<String, DomainStats>,
    ) -> Result<String> {
        let mut buffer = Vec::new();
        {
            let root = BitMapBackend::with_buffer(
                &mut buffer,
                (self.config.chart_width, self.config.chart_height),
            )
            .into_drawing_area();
            root.fill(&WHITE)?;

            // For simplicity, create a bar chart instead of pie chart
            let mut domains: Vec<_> = domain_stats.values().collect();
            domains.sort_by(|a, b| b.count.cmp(&a.count));
            domains.truncate(10); // Top 10 domains

            let max_count = domains.iter().map(|d| d.count).max().unwrap_or(1);

            let mut chart = ChartBuilder::on(&root)
                .caption("Top Domains by Count", ("sans-serif", 30))
                .margin(20)
                .x_label_area_size(80)
                .y_label_area_size(40)
                .build_cartesian_2d(0..domains.len(), 0..max_count)?;

            chart
                .configure_mesh()
                .x_desc("Domain")
                .y_desc("Count")
                .draw()?;

            chart.draw_series(
                domains
                    .iter()
                    .enumerate()
                    .map(|(i, domain)| Rectangle::new([(i, 0), (i, domain.count)], BLUE.filled())),
            )?;

            root.present()?;
        }

        let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
        Ok(format!("data:image/png;base64,{}", base64))
    }

    /// Create timeline chart
    fn create_timeline_chart(&self, timeline: &[TimelineEntry]) -> Result<String> {
        let mut buffer = Vec::new();
        {
            let root = BitMapBackend::with_buffer(
                &mut buffer,
                (self.config.chart_width, self.config.chart_height),
            )
            .into_drawing_area();
            root.fill(&WHITE)?;

            if timeline.is_empty() {
                return Ok(String::new());
            }

            let start_time = timeline.first().unwrap().timestamp;
            let end_time = timeline.last().unwrap().timestamp;
            let max_count = timeline.iter().map(|t| t.count).max().unwrap_or(1);

            let mut chart = ChartBuilder::on(&root)
                .caption("Extraction Timeline", ("sans-serif", 30))
                .margin(20)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .build_cartesian_2d(start_time..end_time, 0..max_count)?;

            chart.configure_mesh().draw()?;

            chart.draw_series(LineSeries::new(
                timeline.iter().map(|t| (t.timestamp, t.count)),
                &BLUE,
            ))?;

            root.present()?;
        }

        let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
        Ok(format!("data:image/png;base64,{}", base64))
    }

    /// Create word count distribution chart
    fn create_word_count_chart(&self, results: &[ExtractionResult]) -> Result<String> {
        let mut buffer = Vec::new();
        {
            let root = BitMapBackend::with_buffer(
                &mut buffer,
                (self.config.chart_width, self.config.chart_height),
            )
            .into_drawing_area();
            root.fill(&WHITE)?;

            let word_counts: Vec<usize> = results.iter().map(|r| r.word_count).collect();

            // Create histogram buckets
            let max_words = word_counts.iter().max().cloned().unwrap_or(0);
            let bucket_size = (max_words / 10).max(1);
            let mut buckets = [0; 10];

            for count in word_counts {
                let bucket = (count / bucket_size).min(9);
                buckets[bucket] += 1;
            }

            let mut chart = ChartBuilder::on(&root)
                .caption("Word Count Distribution", ("sans-serif", 30))
                .margin(20)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .build_cartesian_2d(0..10, 0..*buckets.iter().max().unwrap_or(&1))?;

            chart.configure_mesh().draw()?;

            chart.draw_series(buckets.iter().enumerate().map(|(i, &count)| {
                Rectangle::new([(i as i32, 0), (i as i32, count)], GREEN.filled())
            }))?;

            root.present()?;
        }

        let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
        Ok(format!("data:image/png;base64,{}", base64))
    }

    /// Get theme CSS
    fn get_theme_css(&self) -> String {
        match self.config.theme {
            ReportTheme::Light => LIGHT_THEME_CSS.to_string(),
            ReportTheme::Dark => DARK_THEME_CSS.to_string(),
            ReportTheme::Corporate => CORPORATE_THEME_CSS.to_string(),
            ReportTheme::Modern => MODERN_THEME_CSS.to_string(),
        }
    }

    /// Fetch extraction results (placeholder)
    async fn fetch_extraction_results(
        &self,
        _extraction_id: &str,
    ) -> Result<Vec<ExtractionResult>> {
        // This would normally fetch from storage
        // For now, return sample data
        Ok(vec![
            ExtractionResult {
                id: "result-1".to_string(),
                url: "https://example.com/page1".to_string(),
                title: Some("Example Page 1".to_string()),
                content: "This is sample content from page 1".to_string(),
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
                extraction_time_ms: 250,
                word_count: 8,
                links: vec![],
                images: vec![],
            },
            ExtractionResult {
                id: "result-2".to_string(),
                url: "https://example.com/page2".to_string(),
                title: Some("Example Page 2".to_string()),
                content: "This is sample content from page 2 with more words".to_string(),
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
                extraction_time_ms: 180,
                word_count: 11,
                links: vec![],
                images: vec![],
            },
        ])
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Handlebars helpers
fn format_duration(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let duration_ms = h.param(0).and_then(|v| v.value().as_u64()).unwrap_or(0);

    let formatted = if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else if duration_ms < 60000 {
        format!("{:.1}s", duration_ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", duration_ms as f64 / 60000.0)
    };

    out.write(&formatted)?;
    Ok(())
}

fn format_number(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let number = h.param(0).and_then(|v| v.value().as_u64()).unwrap_or(0);

    let formatted = if number < 1000 {
        number.to_string()
    } else if number < 1000000 {
        format!("{:.1}K", number as f64 / 1000.0)
    } else {
        format!("{:.1}M", number as f64 / 1000000.0)
    };

    out.write(&formatted)?;
    Ok(())
}

fn format_percentage(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let percentage = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);

    let formatted = format!("{:.1}%", percentage);
    out.write(&formatted)?;
    Ok(())
}

fn truncate(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    let length = h.param(1).and_then(|v| v.value().as_u64()).unwrap_or(100) as usize;

    let truncated = if text.len() > length {
        format!("{}...", &text[..length])
    } else {
        text.to_string()
    };

    out.write(&truncated)?;
    Ok(())
}

fn highlight_keywords(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");

    // Simple keyword highlighting (would be more sophisticated in real implementation)
    let highlighted = text
        .replace("extraction", "<mark>extraction</mark>")
        .replace("data", "<mark>data</mark>")
        .replace("content", "<mark>content</mark>");

    out.write(&highlighted)?;
    Ok(())
}

// Template constants
const MAIN_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{title}}</title>
    <style>{{{theme_css}}}</style>
</head>
<body>
    <div class="container">
        <header>
            <h1>{{title}}</h1>
            <div class="generation-info">
                Generated on {{generation_time}} for extraction: <code>{{extraction_id}}</code>
            </div>
        </header>
        
        {{> summary}}
        
        {{#if config.include_charts}}
        {{> charts}}
        {{/if}}
        
        {{> results}}
    </div>
</body>
</html>
"#;

const SUMMARY_TEMPLATE: &str = r#"
<section class="summary">
    <h2>Summary</h2>
    <div class="stats-grid">
        <div class="stat-card">
            <h3>{{format_number total_results}}</h3>
            <p>Total Results</p>
        </div>
        <div class="stat-card">
            <h3>{{format_number total_words}}</h3>
            <p>Total Words</p>
        </div>
        <div class="stat-card">
            <h3>{{format_duration total_processing_time}}</h3>
            <p>Total Processing Time</p>
        </div>
        <div class="stat-card">
            <h3>{{format_percentage success_rate}}</h3>
            <p>Success Rate</p>
        </div>
    </div>
</section>
"#;

const RESULTS_TEMPLATE: &str = r#"
<section class="results">
    <h2>Extraction Results</h2>
    <div class="results-table">
        <table>
            <thead>
                <tr>
                    <th>URL</th>
                    <th>Title</th>
                    <th>Word Count</th>
                    <th>Processing Time</th>
                    <th>Timestamp</th>
                </tr>
            </thead>
            <tbody>
                {{#each results}}
                <tr>
                    <td><a href="{{url}}" target="_blank">{{truncate url 50}}</a></td>
                    <td>{{title}}</td>
                    <td>{{word_count}}</td>
                    <td>{{format_duration extraction_time_ms}}</td>
                    <td>{{timestamp}}</td>
                </tr>
                {{/each}}
            </tbody>
        </table>
    </div>
</section>
"#;

const CHARTS_TEMPLATE: &str = r#"
<section class="charts">
    <h2>Analytics</h2>
    <div class="charts-grid">
        {{#if charts.processing_time}}
        <div class="chart">
            <h3>Processing Time Distribution</h3>
            <img src="{{charts.processing_time}}" alt="Processing Time Chart" />
        </div>
        {{/if}}
        
        {{#if charts.domain_distribution}}
        <div class="chart">
            <h3>Domain Distribution</h3>
            <img src="{{charts.domain_distribution}}" alt="Domain Distribution Chart" />
        </div>
        {{/if}}
        
        {{#if charts.timeline}}
        <div class="chart">
            <h3>Extraction Timeline</h3>
            <img src="{{charts.timeline}}" alt="Timeline Chart" />
        </div>
        {{/if}}
        
        {{#if charts.word_count}}
        <div class="chart">
            <h3>Word Count Distribution</h3>
            <img src="{{charts.word_count}}" alt="Word Count Chart" />
        </div>
        {{/if}}
    </div>
</section>
"#;

const MODERN_THEME_CSS: &str = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    margin: 0;
    padding: 20px;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    background: white;
    border-radius: 12px;
    box-shadow: 0 20px 40px rgba(0,0,0,0.1);
    overflow: hidden;
}

header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 40px;
    text-align: center;
}

header h1 {
    margin: 0;
    font-size: 2.5em;
    font-weight: 300;
}

.generation-info {
    margin-top: 10px;
    opacity: 0.9;
}

section {
    padding: 40px;
}

.stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin-top: 20px;
}

.stat-card {
    background: #f8f9fa;
    padding: 30px;
    border-radius: 8px;
    text-align: center;
    border-left: 4px solid #667eea;
}

.stat-card h3 {
    font-size: 2em;
    margin: 0;
    color: #667eea;
}

.stat-card p {
    margin: 10px 0 0 0;
    color: #666;
}

.charts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 30px;
    margin-top: 20px;
}

.chart {
    text-align: center;
}

.chart img {
    max-width: 100%;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}

table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 20px;
}

th, td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid #ddd;
}

th {
    background: #f8f9fa;
    font-weight: 600;
    color: #333;
}

tr:hover {
    background: #f8f9fa;
}

a {
    color: #667eea;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

code {
    background: rgba(102, 126, 234, 0.1);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: 'Monaco', 'Courier New', monospace;
}

mark {
    background: #fff3cd;
    padding: 1px 3px;
    border-radius: 2px;
}
"#;

const LIGHT_THEME_CSS: &str = r#"
body { font-family: Arial, sans-serif; background: #f5f5f5; }
.container { max-width: 1200px; margin: 0 auto; background: white; padding: 20px; }
header { background: #007bff; color: white; padding: 20px; }
.stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; }
.stat-card { background: #f8f9fa; padding: 20px; border-radius: 5px; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 10px; border: 1px solid #ddd; }
"#;

const DARK_THEME_CSS: &str = r#"
body { font-family: Arial, sans-serif; background: #1a1a1a; color: #ffffff; }
.container { max-width: 1200px; margin: 0 auto; background: #2d2d2d; padding: 20px; }
header { background: #0d6efd; color: white; padding: 20px; }
.stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; }
.stat-card { background: #3d3d3d; padding: 20px; border-radius: 5px; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 10px; border: 1px solid #555; }
th { background: #3d3d3d; }
"#;

const CORPORATE_THEME_CSS: &str = r#"
body { font-family: 'Times New Roman', serif; background: #ffffff; color: #333; }
.container { max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border: 1px solid #ccc; }
header { background: #2c3e50; color: white; padding: 30px; text-align: center; }
.stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; }
.stat-card { background: #ecf0f1; padding: 20px; border: 1px solid #bdc3c7; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 12px; border: 1px solid #bdc3c7; }
th { background: #ecf0f1; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_generator_creation() {
        let generator = ReportGenerator::new();
        assert_eq!(generator.config.title, "RipTide Extraction Report");
    }

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert!(config.include_charts);
        assert!(config.include_metadata);
        assert!(!config.include_raw_data);
    }

    #[tokio::test]
    async fn test_generate_json_report() {
        let generator = ReportGenerator::new();
        let results = vec![ExtractionResult {
            id: "test-1".to_string(),
            url: "https://example.com".to_string(),
            title: Some("Test".to_string()),
            content: "Test content".to_string(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            extraction_time_ms: 100,
            word_count: 2,
            links: vec![],
            images: vec![],
        }];

        let report_data = generator
            .prepare_report_data("test-extraction", results)
            .await
            .unwrap();
        let json_report = generator.generate_json_report(&report_data).await.unwrap();

        let json_str = String::from_utf8(json_report).unwrap();
        assert!(json_str.contains("test-extraction"));
        assert!(json_str.contains("Test"));
    }

    #[tokio::test]
    async fn test_generate_csv_report() {
        let generator = ReportGenerator::new();
        let results = vec![ExtractionResult {
            id: "test-1".to_string(),
            url: "https://example.com".to_string(),
            title: Some("Test".to_string()),
            content: "Test content".to_string(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            extraction_time_ms: 100,
            word_count: 2,
            links: vec![],
            images: vec![],
        }];

        let report_data = generator
            .prepare_report_data("test-extraction", results)
            .await
            .unwrap();
        let csv_report = generator.generate_csv_report(&report_data).await.unwrap();

        let csv_str = String::from_utf8(csv_report).unwrap();
        assert!(csv_str.contains("ID,URL,Title"));
        assert!(csv_str.contains("test-1"));
        assert!(csv_str.contains("https://example.com"));
    }

    #[test]
    fn test_domain_stats_calculation() {
        let generator = ReportGenerator::new();
        let results = vec![
            ExtractionResult {
                id: "1".to_string(),
                url: "https://example.com/page1".to_string(),
                title: None,
                content: "".to_string(),
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
                extraction_time_ms: 100,
                word_count: 10,
                links: vec![],
                images: vec![],
            },
            ExtractionResult {
                id: "2".to_string(),
                url: "https://example.com/page2".to_string(),
                title: None,
                content: "".to_string(),
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
                extraction_time_ms: 200,
                word_count: 20,
                links: vec![],
                images: vec![],
            },
        ];

        let domain_stats = generator.calculate_domain_stats(&results);
        let example_stats = domain_stats.get("example.com").unwrap();

        assert_eq!(example_stats.count, 2);
        assert_eq!(example_stats.total_words, 30);
        assert_eq!(example_stats.average_words, 15.0);
        assert_eq!(example_stats.average_processing_time, 150.0);
    }
}
