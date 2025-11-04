# Skip Extraction API Parameter

## Overview

The `skip_extraction` parameter allows you to retrieve raw HTML content without running content extraction. This is useful as a workaround while the native parser is being implemented.

## API Usage

### Request Format

```json
{
  "urls": ["https://example.com"],
  "options": {
    "skip_extraction": true,
    "render_mode": "headless"
  }
}
```

### Parameters

- `skip_extraction` (optional, boolean): When set to `true`, skips content extraction and returns only raw HTML
  - Default: `false` (extraction enabled)
  - When `true`: Returns HTML in the `html` field, other fields (text, links, metadata) are empty

### Example Request

```bash
curl -X POST http://localhost:3000/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "options": {
      "skip_extraction": true,
      "render_mode": "headless"
    }
  }'
```

### Response Format

When `skip_extraction: true`:

```json
{
  "total_urls": 1,
  "successful": 1,
  "failed": 0,
  "from_cache": 0,
  "results": [
    {
      "url": "https://example.com",
      "status": 200,
      "from_cache": false,
      "gate_decision": "raw",
      "quality_score": 0.0,
      "processing_time_ms": 234,
      "document": {
        "url": "https://example.com",
        "html": "<!DOCTYPE html>...",  // Full rendered HTML
        "title": null,
        "text": "",                     // Empty when skipping extraction
        "links": [],                    // Empty when skipping extraction
        "media": [],                    // Empty when skipping extraction
        "quality_score": null,
        "byline": null,
        "published_iso": null,
        "markdown": null,
        "language": null,
        "reading_time": null,
        "word_count": null,
        "categories": [],
        "site_name": null,
        "description": null
      },
      "error": null,
      "cache_key": "riptide:v1:read_through:abc123"
    }
  ],
  "statistics": {
    "total_processing_time_ms": 234,
    "avg_processing_time_ms": 234.0,
    "gate_decisions": {
      "raw": 1,
      "probes_first": 0,
      "headless": 0,
      "cached": 0
    },
    "cache_hit_rate": 0.0
  }
}
```

## Use Cases

1. **Custom Extraction**: Retrieve raw HTML and use your own extraction logic
2. **Workaround**: Bypass WASM extraction crashes while native parser is being developed
3. **Debugging**: Get the actual HTML to debug extraction issues
4. **Archival**: Store raw HTML for later processing

## Performance Considerations

- Skipping extraction is faster as it bypasses the WASM extraction step
- Still processes through the gate analysis to determine fetch strategy
- HTML content can be large, so be mindful of network and storage costs
- Consider using compression when storing or transmitting raw HTML

## Notes

- The `html` field is only populated when `skip_extraction: true`
- During normal extraction (when `skip_extraction` is `false` or not provided), the `html` field is `null`
- All render modes (`static`, `dynamic`, `adaptive`) work with `skip_extraction`
- Caching still works normally with `skip_extraction` enabled
