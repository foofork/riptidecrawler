# Test Scripts Directory

This directory contains executable test scripts for the webpage extraction comparison.

## Scripts

### run_comparison.rs
Main test harness that executes all extraction methods against all test URLs.

**Usage:**
```bash
cargo run --release --bin webpage_extraction_test
```

### parse_logs.py
Python script to parse JSON log files and extract statistics.

**Usage:**
```bash
python3 parse_logs.py logs/all_results.jsonl
```

### generate_report.py
Generates the markdown comparison report from parsed logs.

**Usage:**
```bash
python3 generate_report.py --input logs/all_results.jsonl --output results/report.md
```

### visualize_results.py
Creates charts and visualizations (optional).

**Usage:**
```bash
python3 visualize_results.py --data logs/all_results.jsonl --output results/charts/
```

## Development

To create the test harness, implement `run_comparison.rs` using the test plan specifications.
