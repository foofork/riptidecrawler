# Schema Command Implementation

## Overview
Comprehensive schema management suite for RipTide CLI, enabling schema learning, testing, comparison, and registry management.

## Location
- **File**: `/workspaces/eventmesh/crates/riptide-cli/src/commands/schema.rs`
- **Module**: Added to `mod.rs` and wired to `main.rs`

## Subcommands

### 1. `schema learn`
Learn extraction schema from a URL by analyzing content patterns.

**Usage:**
```bash
riptide schema learn --url <URL> --goal article --output schema.json --confidence 0.7
```

**Features:**
- Analyzes URL to detect extraction patterns
- Supports goal-based learning (article, product, listing, form, etc.)
- Generates JSON schema file with selectors and field definitions
- Configurable confidence thresholds
- Verbose mode for detailed analysis
- Field-specific selector generation

**Output:** Schema file + analysis report with confidence scores, patterns found, warnings, and suggestions

### 2. `schema test`
Test schema against multiple URLs to validate extraction success.

**Usage:**
```bash
riptide schema test --schema schema.json --urls "url1,url2,url3" --report --output report.json
```

**Features:**
- Tests schema against multiple URLs
- Generates detailed validation reports
- Shows extraction success rates per field
- Displays average confidence and timing metrics
- Identifies most common errors
- Optional fail-fast mode

**Output:** Test results with pass/fail status, confidence scores, field success rates, and performance metrics

### 3. `schema diff`
Compare two schemas to identify differences and similarities.

**Usage:**
```bash
riptide schema diff --schema1 old.json --schema2 new.json --format table --only-diff
```

**Features:**
- Compares metadata (name, version, goal)
- Identifies added/removed fields
- Shows changed field definitions
- Multiple output formats (text, json, table)
- Option to show only differences

**Output:** Comparison report with differences, similarities, and field changes

### 4. `schema push`
Push schema to registry for reuse and sharing.

**Usage:**
```bash
riptide schema push --schema schema.json --name "article-extractor" --version 1.0.0 --public
```

**Features:**
- Uploads schema to central registry
- Version management
- Tag support for categorization
- Public/private visibility control
- Description and metadata updates

**Output:** Confirmation of schema push with registry URL

### 5. `schema list`
List available schemas from registry.

**Usage:**
```bash
riptide schema list --tag article --goal product --format table --limit 50
```

**Features:**
- Filters by tag and goal type
- Public/private filtering
- Multiple output formats (table, json, list)
- Shows usage counts and success rates
- Configurable result limits

**Output:** List of schemas with metadata, usage statistics, and success rates

### 6. `schema show`
Display detailed information about a schema.

**Usage:**
```bash
riptide schema show --schema article-extractor --selectors --validation --example
```

**Features:**
- Works with local files or registry names
- Shows field definitions
- Displays selector details (CSS, XPath, regex)
- Shows validation rules
- Provides example usage
- Multiple output formats (text, json, yaml)

**Output:** Complete schema details with fields, selectors, validation, and usage examples

### 7. `schema rm`
Remove schema from registry.

**Usage:**
```bash
riptide schema rm --name article-extractor --version 1.0.0 --force
```

**Features:**
- Removes specific version or all versions
- Confirmation prompt (unless --force)
- Prevents accidental deletions

**Output:** Confirmation of schema removal

## Data Structures

### ExtractionSchema
Main schema structure containing:
- `name`: Schema identifier
- `version`: Semantic version
- `goal`: Extraction goal type (article, product, etc.)
- `description`: Optional description
- `fields`: HashMap of field definitions
- `selectors`: HashMap of selector rules per field
- `validation`: Optional validation rules
- `metadata`: Usage tracking and metadata

### FieldSchema
Field definition with:
- `field_type`: Data type (string, number, array, etc.)
- `required`: Whether field is mandatory
- `description`: Optional field description
- `default`: Default value if not found
- `transform`: Optional transformation function

### SelectorRule
Selector definition with:
- `selector`: Actual selector string
- `selector_type`: Type (css, xpath, regex)
- `priority`: Selection priority (higher = preferred)
- `confidence`: Confidence score (0.0-1.0)
- `fallback`: Optional fallback selector

### ValidationRules
Schema validation configuration:
- `min_fields`: Minimum fields required
- `required_fields`: List of mandatory fields
- `min_confidence`: Minimum confidence threshold
- `custom_rules`: Custom validation expressions

### SchemaMetadata
Tracking and metadata:
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp
- `author`: Schema author
- `tags`: Categorization tags
- `is_public`: Public visibility flag
- `usage_count`: Number of times used
- `success_rate`: Overall extraction success rate

## API Endpoints
All commands interact with the RipTide API server:
- `POST /api/v1/schema/learn`: Learn schema from URL
- `POST /api/v1/schema/test`: Test schema against URLs
- `POST /api/v1/schema/push`: Push schema to registry
- `POST /api/v1/schema/list`: List available schemas
- `POST /api/v1/schema/show`: Get schema details
- `POST /api/v1/schema/remove`: Remove schema from registry

## Integration

### Commands Module
Added to `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`:
```rust
pub mod schema;
use schema::SchemaCommands;

pub enum Commands {
    // ... other commands ...
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
}
```

### Main CLI
Wired in `/workspaces/eventmesh/crates/riptide-cli/src/main.rs`:
```rust
Commands::Schema { command } => commands::schema::execute(client, command, &cli.output).await,
```

## Design Patterns
Follows consistent patterns with other RipTide commands:
- ✅ Async execution with RipTideClient
- ✅ Multiple output formats (text, json, table)
- ✅ Detailed error handling with anyhow::Result
- ✅ Progress indicators and status messages
- ✅ File I/O for local schema management
- ✅ API integration for registry operations
- ✅ Comprehensive help text with clap

## Future Enhancements
- YAML output support (currently shows JSON with warning)
- Headless browser integration for schema learning
- Advanced validation rules (regex patterns, value ranges)
- Schema versioning and migration tools
- Collaborative schema editing
- Performance benchmarking for schemas
- Auto-update schemas based on site changes

## Testing
Verified all subcommands with `--help`:
```bash
✅ riptide schema --help
✅ riptide schema learn --help
✅ riptide schema test --help
✅ riptide schema diff --help
✅ riptide schema push --help
✅ riptide schema list --help
✅ riptide schema show --help
✅ riptide schema rm --help
```

## Build Status
✅ Successfully compiles with `cargo build --release`
✅ All type checks pass
✅ No compilation warnings (except intentional unused parameter)

## Memory Storage
Implementation details stored in coordination memory at:
- `hive/coder/schema-implementation`
- `hive/coder/schema-status`

## Completion Date
2025-10-15
