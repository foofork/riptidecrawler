# RipTide Streaming - Report Generation

## Overview

The RipTide Streaming report generation system provides comprehensive HTML, JSON, and CSV reports for web extraction results, featuring:

- **Dynamic Templates**: Handlebars-based templating with 4 theme options
- **Interactive Charts**: Plotters-generated visualizations embedded as base64 PNG
- **Multi-Format Export**: HTML, JSON, CSV, and PDF (placeholder) formats
- **RESTful API**: Simple HTTP endpoints for report generation
- **Performance Metrics**: Detailed statistics and analytics

## Quick Start

### Generate a Report via API

```bash
curl -X POST http://localhost:3000/api/reports/generate \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "extraction-123",
    "format": "html",
    "include_charts": true,
    "theme": "modern"
  }' \
  > report.html
```

### Generate a Report Programmatically

```rust
use riptide_streaming::reports::{ReportConfig, ReportFormat, ReportGenerator, ReportTheme};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ReportConfig {
        title: "My Extraction Report".to_string(),
        include_charts: true,
        theme: ReportTheme::Modern,
        ..Default::default()
    };

    let generator = ReportGenerator::with_config(config);
    let report = generator
        .generate_report("extraction-id", ReportFormat::Html)
        .await?;

    std::fs::write("report.html", report)?;
    Ok(())
}
```

## API Endpoints

### POST /api/reports/generate

Generate a report for an extraction job.

**Request Body:**

```json
{
  "job_id": "extraction-uuid",
  "format": "html",
  "include_charts": true,
  "sections": ["summary", "details", "charts"],
  "title": "Custom Report Title",
  "theme": "modern",
  "include_raw_data": false,
  "include_metadata": true,
  "chart_width": 800,
  "chart_height": 400
}
```

**Response:**

- **200 OK**: Report content (HTML/JSON/CSV/PDF)
- **400 Bad Request**: Invalid request parameters
- **500 Internal Server Error**: Report generation failed

### GET /api/reports/formats

List available report formats and themes.

**Response:**

```json
{
  "formats": ["html", "json", "csv", "pdf"],
  "themes": ["light", "dark", "corporate", "modern"]
}
```

### GET /api/reports/config/defaults

Get default report configuration.

**Response:**

```json
{
  "title": "RipTide Extraction Report",
  "include_charts": true,
  "include_raw_data": false,
  "include_metadata": true,
  "chart_width": 800,
  "chart_height": 400,
  "theme": "Modern"
}
```

### GET /api/reports/health

Health check for reports service.

**Response:**

```json
{
  "status": "healthy",
  "service": "reports",
  "version": "0.1.0"
}
```

## Report Formats

### HTML Report

Full-featured HTML report with:

- **Responsive Design**: Mobile and desktop optimized
- **Interactive Charts**: Processing time, domain distribution, timeline, word count
- **Domain Statistics**: Per-domain metrics and analysis
- **Results Table**: Sortable extraction results
- **Timeline View**: Hourly extraction progress
- **Word Cloud**: Top keywords visualization
- **Print Support**: Optimized for PDF printing

### JSON Report

Structured JSON output with:

```json
{
  "extraction_id": "sample-123",
  "title": "Report Title",
  "generation_time": "2024-10-10T12:00:00Z",
  "total_results": 50,
  "total_words": 15000,
  "total_processing_time": 12500,
  "average_processing_time": 250.0,
  "success_rate": 98.5,
  "results": [...],
  "domain_stats": {...},
  "timeline": [...],
  "word_cloud_data": [...],
  "charts": {...}
}
```

### CSV Report

Simple CSV format for spreadsheet analysis:

```csv
ID,URL,Title,Word Count,Processing Time (ms),Timestamp
result-1,https://example.com,Page Title,150,250,2024-10-10 12:00:00
```

### PDF Report

PDF generation (currently returns HTML, can be converted with wkhtmltopdf or similar).

## Themes

### Modern Theme (Default)

- Gradient header (purple/blue)
- Clean card-based layout
- Rounded corners and shadows
- Optimized for presentations

### Light Theme

- Classic white background
- Blue accents
- High contrast for readability
- Professional appearance

### Dark Theme

- Dark gray/black background
- High contrast text
- Reduced eye strain
- Modern developer aesthetic

### Corporate Theme

- Traditional serif fonts
- Conservative color palette
- Formal business style
- Print-friendly

## Chart Types

### 1. Processing Time Distribution

Histogram showing distribution of extraction times across all results.

**Insights:**
- Identify slow extractions
- Performance consistency
- Outlier detection

### 2. Domain Distribution

Bar chart showing top domains by extraction count.

**Insights:**
- Most frequently scraped domains
- Source diversity
- Domain-specific patterns

### 3. Extraction Timeline

Line chart showing extraction rate over time.

**Insights:**
- Extraction velocity
- Peak activity periods
- Progress tracking

### 4. Word Count Distribution

Histogram showing content length distribution.

**Insights:**
- Content depth analysis
- Quality indicators
- Content type patterns

## Report Sections

### Summary Statistics

- Total results processed
- Total words extracted
- Total processing time
- Success rate percentage
- Average processing time
- Pages processed
- Average words per page

### Domain Statistics Table

Per-domain metrics including:

- Extraction count
- Total words
- Average words per extraction
- Average processing time
- Success rate

### Results Table

Detailed extraction results with:

- URL (clickable link)
- Page title
- Word count
- Processing time
- Timestamp

### Timeline

Hourly breakdown of extraction activity with:

- Timestamp
- Event description
- Extraction count
- Rate (extractions/hour)

### Word Cloud

Top 50 keywords from extracted content with:

- Word frequency
- Percentage of total
- Visual size representation

## Configuration Options

### ReportConfig

```rust
pub struct ReportConfig {
    /// Report title
    pub title: String,

    /// Include chart visualizations
    pub include_charts: bool,

    /// Include raw extraction data
    pub include_raw_data: bool,

    /// Include metadata fields
    pub include_metadata: bool,

    /// Chart width in pixels
    pub chart_width: u32,

    /// Chart height in pixels
    pub chart_height: u32,

    /// Visual theme
    pub theme: ReportTheme,
}
```

### Default Configuration

```rust
ReportConfig {
    title: "RipTide Extraction Report".to_string(),
    include_charts: true,
    include_raw_data: false,
    include_metadata: true,
    chart_width: 800,
    chart_height: 400,
    theme: ReportTheme::Modern,
}
```

## Advanced Usage

### Custom Chart Dimensions

```rust
let config = ReportConfig {
    chart_width: 1200,
    chart_height: 600,
    ..Default::default()
};
```

### Disable Charts for Faster Generation

```rust
let config = ReportConfig {
    include_charts: false,
    ..Default::default()
};
```

### Include Raw Data for Analysis

```rust
let config = ReportConfig {
    include_raw_data: true,
    ..Default::default()
};
```

### Concurrent Report Generation

```rust
let generator = ReportGenerator::new();

let reports: Vec<_> = (0..10)
    .map(|i| {
        let gen = generator.clone();
        tokio::spawn(async move {
            gen.generate_report(
                &format!("extraction-{}", i),
                ReportFormat::Html
            ).await
        })
    })
    .collect();

for report in reports {
    let result = report.await??;
    // Process result
}
```

## Performance Considerations

### Chart Generation

Chart generation adds ~200-500ms per report depending on dataset size. Disable charts for faster generation:

```rust
config.include_charts = false;
```

### Large Datasets

For datasets >1000 results:

1. Generate JSON/CSV instead of HTML
2. Disable charts
3. Use pagination in custom reports
4. Consider streaming reports

### Memory Usage

Reports are generated in-memory. For very large datasets:

- Use streaming NDJSON format
- Generate reports in batches
- Store intermediate results

## Customization

### Custom Templates

Templates are embedded in the binary but can be customized by forking and modifying:

- `crates/riptide-streaming/src/reports.rs` - Template constants
- Handlebars syntax for dynamic content
- CSS themes for styling

### Custom Helpers

Add custom Handlebars helpers:

```rust
handlebars.register_helper("my_helper", Box::new(my_helper_fn));
```

### Custom Charts

Extend chart generation in `reports.rs`:

```rust
fn create_custom_chart(&self, data: &[Data]) -> Result<String> {
    // Use Plotters to create custom visualization
    // Return base64 encoded PNG
}
```

## Testing

### Unit Tests

```bash
cargo test --package riptide-streaming --lib reports
```

### Integration Tests

```bash
cargo test --package riptide-streaming report_generation_tests
```

### Generate Sample Report

```bash
cargo run --example generate_sample_report
open target/sample_report.html
```

## Troubleshooting

### Report Generation Fails

**Error:** "Failed to render template"

**Solution:** Check Handlebars template syntax and ensure all required fields are present in data.

### Charts Not Appearing

**Error:** No chart images in HTML

**Solution:** Ensure `include_charts: true` in config and Plotters dependencies are installed.

### Large Report Timeout

**Error:** Report generation times out

**Solution:** Reduce dataset size or disable charts. Consider generating JSON instead.

### Theme Not Applied

**Error:** Report uses default theme

**Solution:** Verify theme is passed correctly in ReportConfig and CSS is embedded.

## Future Enhancements

- [ ] PDF generation with wkhtmltopdf integration
- [ ] Interactive JavaScript charts (Chart.js, D3.js)
- [ ] Report templates library
- [ ] Report scheduling and automation
- [ ] Email delivery integration
- [ ] Report versioning and history
- [ ] Custom branding and logos
- [ ] Multi-language support
- [ ] Real-time report updates via WebSocket

## See Also

- [NDJSON Streaming](./NDJSON_STREAMING.md)
- [Progress Tracking](./PROGRESS_TRACKING.md)
- [API Documentation](./API.md)
- [Examples](../../examples/)
