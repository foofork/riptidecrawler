# Output Directory Configuration Guide

## Overview

RipTide CLI provides flexible output directory configuration through environment variables and command-line flags. This allows you to organize extraction results, crawl data, and search outputs according to your project structure.

## Default Directory Structure

When no custom configuration is provided, RipTide uses the following structure:

```
./riptide-output/
├── extractions/          # Content extraction results
│   ├── <timestamp>/      # Organized by extraction time
│   └── <filename>.md     # Named output files
├── crawls/              # Web crawling results
│   ├── <domain>/        # Organized by crawled domain
│   └── <timestamp>/     # Time-based organization
├── searches/            # Search operation results
│   ├── <query-hash>/    # Organized by search query
│   └── results.json     # Search result data
├── cache/               # Local cache data
│   ├── http/            # HTTP response cache
│   └── wasm/            # WASM extraction cache
└── logs/                # Operation logs
    ├── cli.log          # CLI operation logs
    └── errors.log       # Error logs
```

## Environment Variables

### Base Configuration

#### `RIPTIDE_OUTPUT_DIR`
**Default**: `./riptide-output`

Base directory for all CLI output. All other directories are relative to this unless explicitly overridden.

```bash
export RIPTIDE_OUTPUT_DIR="/var/riptide/data"
```

### Operation-Specific Directories

#### `RIPTIDE_EXTRACT_DIR`
**Default**: `${RIPTIDE_OUTPUT_DIR}/extractions`

Directory for content extraction results.

```bash
export RIPTIDE_EXTRACT_DIR="/path/to/extractions"
```

#### `RIPTIDE_CRAWL_DIR`
**Default**: `${RIPTIDE_OUTPUT_DIR}/crawls`

Directory for crawling results.

```bash
export RIPTIDE_CRAWL_DIR="/path/to/crawl-data"
```

#### `RIPTIDE_SEARCH_DIR`
**Default**: `${RIPTIDE_OUTPUT_DIR}/searches`

Directory for search results.

```bash
export RIPTIDE_SEARCH_DIR="/path/to/search-results"
```

#### `RIPTIDE_CACHE_DIR`
**Default**: `${RIPTIDE_OUTPUT_DIR}/cache`

Directory for local cache data.

```bash
export RIPTIDE_CACHE_DIR="/tmp/riptide-cache"
```

#### `RIPTIDE_LOG_DIR`
**Default**: `${RIPTIDE_OUTPUT_DIR}/logs`

Directory for log files.

```bash
export RIPTIDE_LOG_DIR="/var/log/riptide"
```

### Cache Configuration

#### `RIPTIDE_CACHE_TTL`
**Default**: `3600` (1 hour)

Cache time-to-live in seconds.

```bash
export RIPTIDE_CACHE_TTL=7200  # 2 hours
```

#### `RIPTIDE_CACHE_MAX_SIZE`
**Default**: `1073741824` (1GB)

Maximum cache size in bytes.

```bash
export RIPTIDE_CACHE_MAX_SIZE=5368709120  # 5GB
```

## Command-Line Flags

Command-line flags override environment variables for the current operation.

### Global Flags

#### `--output-dir <path>`
Override base output directory for current command.

```bash
riptide extract --url "https://example.com" --output-dir ./custom-output
```

#### `--no-subdirs`
Save output directly to specified directory without subdirectories.

```bash
riptide extract --url "https://example.com" --output-dir ./ --no-subdirs
```

### Operation-Specific Flags

#### Extract Command

```bash
# Save to specific file (respects output dir)
riptide extract --url "https://example.com" -f article.md

# Save with custom directory
riptide extract --url "https://example.com" --output-dir ./articles -f article.md

# Save directly to current directory
riptide extract --url "https://example.com" --output-dir ./ --no-subdirs -f article.md
```

#### Crawl Command

```bash
# Save crawl results to directory
riptide crawl --url "https://example.com" -d ./crawl-results

# Organize by domain
riptide crawl --url "https://example.com" -d ./crawls --organize-by domain

# Organize by timestamp
riptide crawl --url "https://example.com" -d ./crawls --organize-by timestamp
```

#### Search Command

```bash
# Save search results
riptide search --query "rust web scraping" -o json --save-to ./search-results

# With custom directory
riptide search --query "content extraction" --output-dir ./searches
```

## Configuration Examples

### Example 1: Project-Specific Structure

```bash
# .env file
RIPTIDE_OUTPUT_DIR=./project-data
RIPTIDE_EXTRACT_DIR=./project-data/content
RIPTIDE_CRAWL_DIR=./project-data/raw-html
RIPTIDE_SEARCH_DIR=./project-data/queries
RIPTIDE_LOG_DIR=./project-data/logs
```

**Result Structure:**
```
./project-data/
├── content/          # Extractions
├── raw-html/         # Crawl results
├── queries/          # Search results
└── logs/            # Operation logs
```

### Example 2: Centralized Data Directory

```bash
# .env file
RIPTIDE_OUTPUT_DIR=/var/riptide
RIPTIDE_CACHE_DIR=/tmp/riptide-cache
RIPTIDE_CACHE_TTL=7200
RIPTIDE_CACHE_MAX_SIZE=10737418240  # 10GB
```

**Result Structure:**
```
/var/riptide/
├── extractions/
├── crawls/
├── searches/
└── logs/

/tmp/riptide-cache/   # Separate cache location
├── http/
└── wasm/
```

### Example 3: Development vs Production

**Development (.env.development):**
```bash
RIPTIDE_OUTPUT_DIR=./dev-output
RIPTIDE_CACHE_DIR=./dev-cache
RIPTIDE_CACHE_TTL=300        # 5 minutes
RIPTIDE_LOG_DIR=./dev-logs
```

**Production (.env.production):**
```bash
RIPTIDE_OUTPUT_DIR=/var/riptide/production
RIPTIDE_CACHE_DIR=/var/cache/riptide
RIPTIDE_CACHE_TTL=3600       # 1 hour
RIPTIDE_LOG_DIR=/var/log/riptide
RIPTIDE_CACHE_MAX_SIZE=53687091200  # 50GB
```

### Example 4: User Home Directory

```bash
# .env file
RIPTIDE_OUTPUT_DIR=$HOME/.riptide/data
RIPTIDE_CACHE_DIR=$HOME/.riptide/cache
RIPTIDE_LOG_DIR=$HOME/.riptide/logs
```

**Result Structure:**
```
~/.riptide/
├── data/
│   ├── extractions/
│   ├── crawls/
│   └── searches/
├── cache/
│   ├── http/
│   └── wasm/
└── logs/
    ├── cli.log
    └── errors.log
```

## Organization Strategies

### By Domain (Crawling)

```bash
export RIPTIDE_CRAWL_ORGANIZE=domain

riptide crawl --url "https://example.com" -d ./crawls
riptide crawl --url "https://another.com" -d ./crawls
```

**Result:**
```
./crawls/
├── example.com/
│   ├── page1.md
│   ├── page2.md
│   └── index.json
└── another.com/
    ├── home.md
    └── index.json
```

### By Timestamp

```bash
export RIPTIDE_CRAWL_ORGANIZE=timestamp

riptide crawl --url "https://example.com" -d ./crawls
```

**Result:**
```
./crawls/
├── 2025-01-15T10-30-00/
│   ├── example.com/
│   └── metadata.json
└── 2025-01-15T14-45-00/
    ├── example.com/
    └── metadata.json
```

### By Query Hash (Search)

```bash
riptide search --query "rust web scraping" --save-to ./searches
```

**Result:**
```
./searches/
└── a3f5b9c2/              # Hash of "rust web scraping"
    ├── results.json
    ├── metadata.json
    └── timestamp.txt
```

## File Naming Conventions

### Extraction Files

**Default Format**: `<domain>-<timestamp>.md`

```bash
# Custom naming
riptide extract --url "https://example.com/article" -f custom-name.md

# Automatic naming
riptide extract --url "https://example.com/article"
# Creates: example.com-2025-01-15T10-30-00.md
```

### Crawl Files

**Default Format**: `<page-slug>.md`

```bash
# Custom prefix
riptide crawl --url "https://example.com" --prefix crawl-session-1

# Result files:
# - crawl-session-1-home.md
# - crawl-session-1-about.md
# - crawl-session-1-contact.md
```

### Search Files

**Default Format**: `results-<query-hash>.json`

```bash
# Custom filename
riptide search --query "content extraction" --save-as extraction-results.json
```

## Best Practices

### 1. Use Environment Variables for Consistency

Create a `.env` file in your project root:

```bash
# .env
RIPTIDE_OUTPUT_DIR=./data
RIPTIDE_CACHE_DIR=./cache
RIPTIDE_LOG_DIR=./logs
```

Load it in your shell:
```bash
source .env
riptide extract --url "https://example.com"
```

### 2. Organize by Use Case

**Content Research:**
```bash
RIPTIDE_OUTPUT_DIR=./research
RIPTIDE_EXTRACT_DIR=./research/articles
RIPTIDE_SEARCH_DIR=./research/queries
```

**Data Pipeline:**
```bash
RIPTIDE_OUTPUT_DIR=/data/pipeline
RIPTIDE_CRAWL_DIR=/data/pipeline/raw
RIPTIDE_EXTRACT_DIR=/data/pipeline/processed
```

### 3. Separate Cache from Data

```bash
RIPTIDE_OUTPUT_DIR=/var/riptide/data    # Persistent data
RIPTIDE_CACHE_DIR=/tmp/riptide-cache    # Temporary cache
```

### 4. Use Descriptive Directory Names

```bash
# Good
export RIPTIDE_EXTRACT_DIR=./marketing-content
export RIPTIDE_CRAWL_DIR=./competitor-analysis

# Less descriptive
export RIPTIDE_EXTRACT_DIR=./ext
export RIPTIDE_CRAWL_DIR=./c
```

### 5. Set Appropriate Cache Limits

```bash
# Development (smaller cache)
export RIPTIDE_CACHE_MAX_SIZE=1073741824    # 1GB
export RIPTIDE_CACHE_TTL=300                # 5 minutes

# Production (larger cache)
export RIPTIDE_CACHE_MAX_SIZE=53687091200   # 50GB
export RIPTIDE_CACHE_TTL=3600               # 1 hour
```

## Troubleshooting

### Issue: Permission Denied

**Problem**: Cannot write to output directory

**Solution**:
```bash
# Check permissions
ls -ld $RIPTIDE_OUTPUT_DIR

# Fix permissions
sudo chmod 755 $RIPTIDE_OUTPUT_DIR
sudo chown $USER:$USER $RIPTIDE_OUTPUT_DIR
```

### Issue: Disk Space Full

**Problem**: Cache or output directory fills disk

**Solution**:
```bash
# Check disk usage
du -sh $RIPTIDE_OUTPUT_DIR
du -sh $RIPTIDE_CACHE_DIR

# Clear cache
riptide cache clear

# Reduce cache size
export RIPTIDE_CACHE_MAX_SIZE=1073741824  # 1GB
```

### Issue: Files Not Found

**Problem**: Output files not in expected location

**Solution**:
```bash
# Check current configuration
riptide config show

# Verify environment variables
echo $RIPTIDE_OUTPUT_DIR
echo $RIPTIDE_EXTRACT_DIR

# Use absolute path
riptide extract --url "..." --output-dir /absolute/path
```

### Issue: Subdirectory Creation Fails

**Problem**: Cannot create subdirectories

**Solution**:
```bash
# Disable subdirectories
riptide extract --url "..." --no-subdirs

# Or create manually
mkdir -p $RIPTIDE_EXTRACT_DIR
```

## Configuration Management

### View Current Configuration

```bash
riptide config show
```

**Output:**
```
RipTide Configuration:
  Output Directory: /var/riptide/data
  Extract Directory: /var/riptide/data/extractions
  Crawl Directory: /var/riptide/data/crawls
  Search Directory: /var/riptide/data/searches
  Cache Directory: /tmp/riptide-cache
  Log Directory: /var/log/riptide
  Cache TTL: 3600 seconds
  Cache Max Size: 1073741824 bytes (1.0 GB)
```

### Reset to Defaults

```bash
riptide config reset
```

### Export Configuration

```bash
riptide config export > riptide-config.env
```

### Import Configuration

```bash
source riptide-config.env
# or
riptide config import riptide-config.env
```

## Integration Examples

### Shell Script

```bash
#!/bin/bash
# extraction-pipeline.sh

# Configuration
export RIPTIDE_OUTPUT_DIR="./pipeline-data"
export RIPTIDE_EXTRACT_DIR="./pipeline-data/raw"
export RIPTIDE_CACHE_TTL=7200

# Extract content
riptide extract --url "$1" -f raw-content.md

# Process results
python process_content.py ./pipeline-data/raw/raw-content.md
```

### Python Integration

```python
import os
import subprocess

# Configure
os.environ['RIPTIDE_OUTPUT_DIR'] = './python-pipeline'
os.environ['RIPTIDE_EXTRACT_DIR'] = './python-pipeline/extractions'

# Run extraction
subprocess.run([
    'riptide', 'extract',
    '--url', 'https://example.com',
    '-f', 'article.md'
])

# Read result
output_path = os.path.join(
    os.environ['RIPTIDE_EXTRACT_DIR'],
    'article.md'
)
with open(output_path) as f:
    content = f.read()
```

### GitHub Actions

```yaml
- name: Configure RipTide
  run: |
    echo "RIPTIDE_OUTPUT_DIR=${{ github.workspace }}/output" >> $GITHUB_ENV
    echo "RIPTIDE_EXTRACT_DIR=${{ github.workspace }}/output/extractions" >> $GITHUB_ENV

- name: Extract Content
  run: |
    riptide extract --url "${{ inputs.url }}" -f result.md

- name: Upload Results
  uses: actions/upload-artifact@v3
  with:
    name: extraction-results
    path: ${{ github.workspace }}/output/extractions/
```

## Next Steps

- Review [System Design](../architecture/SYSTEM_DESIGN.md) for architecture details
- See [Migration Guide](../guides/MIGRATION_GUIDE.md) for upgrade instructions
- Check [Rollout Plan](../ROLLOUT_PLAN.md) for implementation timeline
