# Extract Command Input Enhancement

## Summary

Enhanced the `extract` command in RipTide CLI to support multiple input sources: file, stdin, and URL. This provides flexibility for different use cases and integration scenarios.

## Implementation Details

### Changes Made

#### 1. ExtractArgs Structure (`src/commands/mod.rs`)
- **Modified `url`**: Changed from `String` to `Option<String>` (no longer required)
- **Added `input_file`**: `Option<String>` - Path to HTML file
- **Added `stdin`**: `bool` - Flag to read from stdin

```rust
pub struct ExtractArgs {
    /// URL to extract content from
    #[arg(long)]
    pub url: Option<String>,

    /// Read HTML from a file
    #[arg(long)]
    pub input_file: Option<String>,

    /// Read HTML from stdin
    #[arg(long)]
    pub stdin: bool,

    // ... other fields remain unchanged
}
```

#### 2. Execute Function (`src/commands/extract.rs`)

**Input Priority Order:**
1. **stdin** (highest priority)
2. **input-file**
3. **url** (fallback)
4. Error if none provided

**New Function: `execute_direct_extraction()`**
- Handles HTML content from file or stdin
- Bypasses HTTP fetching
- Uses WASM extraction directly on provided content
- Supports all engine modes (auto, raw, wasm)

**Modified Function: `execute_local_extraction()`**
- Now validates URL presence
- Maintains all existing functionality for URL-based extraction

**Modified Function: `output_extraction_result()`**
- Added `source` parameter to display input source
- Shows "stdin", file path, or URL in output

## Usage Examples

### 1. Extract from File
```bash
riptide extract --input-file page.html --local
riptide extract --input-file page.html --engine wasm
```

### 2. Extract from Stdin
```bash
cat page.html | riptide extract --stdin --local
curl https://example.com | riptide extract --stdin --engine raw
```

### 3. Extract from URL (existing functionality)
```bash
riptide extract --url https://example.com --local
```

### 4. Priority Testing
```bash
# stdin takes priority even if other sources provided
echo '<html>...</html>' | riptide extract --stdin --url https://example.com
```

## Engine Modes

All three input sources support the same engine modes:

- **auto**: Automatically detect and select engine
- **raw**: Return HTML without extraction
- **wasm**: Use WASM-based extraction (requires WASM module)
- **headless**: Browser-based (not yet implemented)

## Validation

- At least one input source must be provided
- Error message clearly indicates available options
- Backwards compatible with existing URL-only usage

## Test Results

### ✅ Stdin Input
```bash
echo '<html>...</html>' | riptide extract --stdin --engine raw
# Output: Successfully extracts content
```

### ✅ File Input
```bash
riptide extract --input-file tests/test-extract-input.html --engine raw
# Output: Successfully reads and extracts content
```

### ✅ URL Input (Backwards Compatibility)
```bash
riptide extract --url https://example.com --local --engine raw
# Output: Successfully fetches and extracts
```

### ✅ Input Validation
```bash
riptide extract --engine raw
# Error: At least one input source is required: --url, --input-file, or --stdin
```

## Benefits

1. **Pipeline Integration**: Easy to integrate with shell pipelines and scripts
2. **Batch Processing**: Can process local HTML files without HTTP overhead
3. **Testing**: Simplifies testing with local HTML fixtures
4. **Flexibility**: Choose the most convenient input method for each use case
5. **Performance**: Skip HTTP fetching when content is already available

## Architecture

```
Input Priority Flow:
    stdin? ──yes──> execute_direct_extraction()
      │
      no
      │
    file? ──yes──> execute_direct_extraction()
      │
      no
      │
    url? ──yes──> execute_local_extraction() OR API extraction
      │
      no
      │
    ERROR: No input source
```

## Engine Gating

For file and stdin inputs:
- Auto-detection still works based on HTML content analysis
- All extraction features remain available
- Source is tracked in metadata for debugging

## Metadata Tracking

Enhanced metadata includes:
- `source`: Shows input source ("stdin", file path, or URL)
- All existing metadata fields preserved
- Engine information retained

## Future Enhancements

1. Support for multiple file inputs (batch mode)
2. Watch mode for file changes
3. Integration with other commands (tables, render)
4. Custom input source plugins

## Files Modified

- `/workspaces/eventmesh/crates/riptide-cli/src/commands/mod.rs`
- `/workspaces/eventmesh/crates/riptide-cli/src/commands/extract.rs`

## Memory Storage

Implementation stored in: `hive/coder/extract-input-enhancement`

## Status

✅ **COMPLETE** - All functionality implemented and tested successfully
