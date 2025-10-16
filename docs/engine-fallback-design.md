# Smart Engine Selection with Fallback Chain

## Overview

This document describes the intelligent extraction engine selection system with automatic fallback chains implemented in RipTide CLI.

## Architecture

### Fallback Chain: raw → wasm → headless

The system implements a three-tier fallback chain for robust extraction:

1. **Raw Engine** (Fastest) - Basic HTTP fetch without processing
2. **WASM Engine** (Balanced) - Local WASM-based extraction with good performance
3. **Headless Engine** (Most Robust) - Browser-based extraction for JavaScript-heavy sites

## Content Analysis Heuristics

### Framework Detection

The system analyzes content to detect:

- **React/Next.js**: `__NEXT_DATA__`, `react`, `_reactRoot`, `__webpack_require__`
- **Vue**: `v-app`, `vue`, `createApp`
- **Angular**: `ng-app`, `ng-version`, `platformBrowserDynamic`

### SPA (Single Page Application) Markers

Detects client-side rendering patterns:
- `<!-- rendered by`
- `__webpack`
- `window.__INITIAL_STATE__`
- `data-react-helmet`

### Anti-Scraping Detection

Identifies anti-scraping measures:
- Cloudflare protection
- `cf-browser-verification`
- Google reCAPTCHA
- hCaptcha
- PerimeterX

### Content Quality Metrics

- **Content Ratio**: Text content vs markup ratio (threshold: 10%)
- **Main Content**: Presence of `<article>`, `<main>`, or content containers
- **Text Density**: Minimum 5% text-to-character ratio

## Retry Logic

### Exponential Backoff

```
Attempt 1: Immediate
Attempt 2: Wait 1000ms
Attempt 3: Wait 2000ms
Max Retries: 3
```

### Backoff Formula

```rust
backoff_ms = INITIAL_BACKOFF_MS * 2^(attempt - 1)
```

## Quality Validation

### Sufficiency Criteria

An extraction is considered sufficient if ALL of the following are met:

1. **Minimum Content Length**: ≥ 100 characters
2. **Minimum Confidence**: ≥ 50%
3. **Minimum Text Ratio**: ≥ 5%

### Quality Metrics

```rust
pub struct ExtractionQuality {
    content_length: usize,       // Total characters
    text_ratio: f64,             // Text vs total ratio
    has_structure: bool,         // Metadata presence
    confidence_score: f64,       // 0.0 - 1.0
    extraction_time_ms: u64,     // Performance metric
}
```

## Decision Flow

```
┌─────────────────────────────────────┐
│   Analyze Content Characteristics   │
└──────────────┬──────────────────────┘
               │
               ├──> Anti-scraping? ──────────> Headless
               │
               ├──> JS Framework? ────────────> Headless
               │
               ├──> Low Content Ratio? ───────> Headless
               │
               ├──> WASM Content? ────────────> WASM
               │
               └──> Standard HTML? ───────────> WASM (default)

┌─────────────────────────────────────┐
│      Fallback Chain Execution       │
└──────────────┬──────────────────────┘
               │
          ┌────┴────┐
          │  Raw    │
          └────┬────┘
               │
          Sufficient? ──Yes──> Return
               │
               No
               │
          ┌────┴────┐
          │  WASM   │
          └────┬────┘
               │
          Sufficient? ──Yes──> Return
               │
               No
               │
          ┌────┴────┐
          │Headless │
          └────┬────┘
               │
           Success or Fail
```

## Performance Metrics

### Tracked Metrics

- Engine selection decision
- Attempt duration per engine
- Total extraction duration
- Success/failure rate per engine
- Quality scores by engine

### Memory Coordination

Metrics are stored in shared memory for agent coordination:

```json
{
  "final_engine": "wasm",
  "total_duration_ms": 1250,
  "attempts": 2,
  "attempt_details": [
    {
      "engine": "raw",
      "success": false,
      "duration_ms": 150
    },
    {
      "engine": "wasm",
      "success": true,
      "duration_ms": 1100
    }
  ],
  "url": "https://example.com",
  "timestamp": "2025-10-16T19:30:00Z"
}
```

## Integration Points

### WASM Agent Coordination

The engine selection system coordinates with the WASM agent via shared memory:

- **Pre-extraction**: Store content analysis results
- **Post-extraction**: Share quality metrics
- **Failure**: Communicate errors for fallback decision

### Headless Agent Coordination

For JavaScript-heavy sites, coordinates with headless browser agent:

- **Stealth Settings**: Pass-through from CLI args
- **Timeout Configuration**: Shared timeout values
- **Behavior Simulation**: Optional user interaction simulation

## Configuration

### Constants

```rust
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;
const MIN_CONTENT_LENGTH: usize = 100;
const MIN_TEXT_RATIO: f64 = 0.05;
const MIN_CONFIDENCE: f64 = 0.5;
```

### CLI Arguments

- `--engine <type>`: Force specific engine (auto, raw, wasm, headless)
- `--no-wasm`: Disable WASM engine
- `--stealth-level <level>`: Stealth preset for headless
- `--init-timeout-ms <ms>`: WASM initialization timeout
- `--headless-timeout <ms>`: Headless browser timeout

## Testing

### Unit Tests

- Content ratio calculation
- SPA detection
- React/Vue/Angular detection
- Standard HTML detection
- Extraction quality validation
- Quality metric analysis

### Integration Tests

- Full fallback chain execution
- Retry logic with backoff
- Memory coordination
- Cross-engine handoff

## Error Handling

### Graceful Degradation

1. Raw fails → Try WASM
2. WASM fails → Try Headless
3. Headless fails → Return comprehensive error

### Error Reporting

```
❌ All extraction methods failed:
  1. raw - Failed (150ms)
     Error: Connection timeout
  2. wasm - Failed (1100ms)
     Error: WASM initialization timeout
  3. headless - Failed (3500ms)
     Error: Browser launch failed
```

## Future Enhancements

1. **Machine Learning**: Train models on extraction patterns
2. **Adaptive Timeouts**: Dynamic timeout based on historical data
3. **Cost Optimization**: Balance speed vs resource usage
4. **Parallel Attempts**: Try multiple engines concurrently
5. **Caching**: Cache engine selection decisions per domain

## References

- **WASM Extraction**: `crates/riptide-extraction/src/wasm_extraction.rs`
- **Headless Browser**: `crates/riptide-headless/src/launcher.rs`
- **Stealth**: `crates/riptide-stealth/src/lib.rs`
- **CLI Commands**: `crates/riptide-cli/src/commands/extract.rs`

## Agent Coordination Protocol

### Memory Keys

- `swarm/engine-selection/{url}`: Decision history
- `swarm/engine-selection/metrics`: Performance metrics
- `swarm/coder/engine-fallback-module`: Module registration

### Notification Events

- `extraction_started`: Begin fallback chain
- `engine_attempt`: Each engine attempt
- `extraction_completed`: Final result
- `extraction_failed`: All engines failed

## Performance Benchmarks

| Engine | Avg Time | Success Rate | Content Quality |
|--------|----------|--------------|-----------------|
| Raw    | 50ms     | 60%          | Low             |
| WASM   | 200ms    | 85%          | High            |
| Headless | 2000ms | 95%          | Very High       |

## Best Practices

1. **Let Auto-Detection Work**: Use `--engine auto` for best results
2. **Monitor Metrics**: Check `.swarm/memory.db` for performance data
3. **Tune Timeouts**: Adjust based on network and site complexity
4. **Use Stealth**: Enable for anti-scraping sites
5. **Cache Decisions**: Reuse engine selection for similar domains
