# Reliability Module Usage Guide

## Quick Start

The Reliability Module is automatically integrated into the RipTide API pipeline. No code changes are needed to benefit from retry logic and graceful degradation.

## Configuration

### Environment Variables

Set these variables to customize reliability behavior:

```bash
# Maximum retry attempts for HTTP requests (default: 3)
export RELIABILITY_MAX_RETRIES=3

# Timeout for operations in seconds (default: 10)
export RELIABILITY_TIMEOUT_SECS=10

# Enable graceful degradation fallback (default: true)
export RELIABILITY_GRACEFUL_DEGRADATION=true

# Quality threshold for fast extraction (0.0-1.0, default: 0.6)
export RELIABILITY_QUALITY_THRESHOLD=0.6
```

## How It Works

### Extraction Modes

The pipeline automatically selects the extraction mode based on gate analysis:

1. **Fast Mode** (Raw decision)
   - Uses WASM extraction with retry logic
   - Fast response, good for static content
   - Automatically retries on transient failures

2. **ProbesFirst Mode** (ProbesFirst decision)
   - Tries fast extraction first
   - Evaluates content quality
   - Falls back to headless if quality < threshold
   - Best balance of speed and quality

3. **Headless Mode** (Headless decision)
   - Uses headless browser rendering
   - Circuit breaker protection
   - Falls back to fast extraction on failure
   - Best for dynamic/SPA content

### Retry Behavior

- **HTTP Requests**: Up to 3 retry attempts with exponential backoff
- **Backoff Strategy**: 200ms → 300ms → 450ms (with jitter)
- **Idempotent Only**: Only retries GET requests
- **Fast Fail**: Headless service has no retries for quick recovery

### Circuit Breaker

Prevents cascading failures when services are degraded:

```
┌─────────┐  3 failures   ┌──────────┐  60s cooldown  ┌────────────┐
│ Closed  │ ──────────────▶│  Open    │ ──────────────▶│ Half-Open  │
│         │                │          │                │            │
│ Normal  │                │ Blocked  │                │ Testing    │
└─────────┘                └──────────┘                └────────────┘
     ▲                                                        │
     └────────────────── 2 successes ────────────────────────┘
```

- **Threshold**: 3 consecutive failures
- **Cooldown**: 60 seconds before retry
- **Recovery**: 2 successful requests to close

### Quality Evaluation

For ProbesFirst mode, content is scored on:

| Criteria | Weight | Scoring |
|----------|--------|---------|
| Title present | 20% | Has non-empty title |
| Content length | 40% | >1000 chars = full score, >200 = half |
| Markdown structure | 20% | Presence of headers, links, emphasis |
| Metadata | 20% | Byline, date, description, links |

**Threshold**: Default 0.6 (60%) - configurable via environment variable

## Monitoring

### Events Emitted

The reliability system emits events to the event bus:

#### Success Event
```json
{
  "type": "pipeline.extraction.reliable_success",
  "severity": "info",
  "metadata": {
    "url": "https://example.com/article",
    "decision": "ProbesFirst",
    "content_length": 15420
  }
}
```

#### Failure Event
```json
{
  "type": "pipeline.extraction.reliable_failure",
  "severity": "warning",
  "metadata": {
    "url": "https://example.com/article",
    "error": "Headless service timeout after 10s"
  }
}
```

### Metrics to Monitor

Query your event bus for these metrics:

1. **Retry Rate**: Count of retries / total requests
2. **Circuit Breaker State**: Open, closed, half-open
3. **Fallback Rate**: Fallback attempts / total requests
4. **Quality Score Distribution**: Histogram of quality scores

### Example Prometheus Queries

```promql
# Retry rate
rate(pipeline_extraction_retries_total[5m])

# Circuit breaker state (0=closed, 1=open, 2=half-open)
reliability_circuit_breaker_state

# Fallback success rate
rate(pipeline_extraction_fallback_success_total[5m])
/ rate(pipeline_extraction_fallback_total[5m])
```

## Troubleshooting

### High Retry Rate

**Symptom**: Many retry events in logs

**Possible Causes**:
- Network instability
- Target server rate limiting
- Timeouts too aggressive

**Solutions**:
1. Increase `RELIABILITY_TIMEOUT_SECS`
2. Add delays between requests
3. Check network connectivity
4. Review target server rate limits

### Circuit Breaker Always Open

**Symptom**: Circuit breaker constantly in open state

**Possible Causes**:
- Headless service down
- Headless service overloaded
- Configuration mismatch

**Solutions**:
1. Check headless service health
2. Increase headless service capacity
3. Verify `HEADLESS_URL` configuration
4. Review circuit breaker threshold settings

### Low Quality Scores

**Symptom**: Frequent fallback to headless mode

**Possible Causes**:
- Quality threshold too high
- Content genuinely poor quality
- WASM extraction not working well

**Solutions**:
1. Lower `RELIABILITY_QUALITY_THRESHOLD` (e.g., 0.4-0.5)
2. Verify WASM extractor configuration
3. Check if target sites need headless rendering
4. Review quality evaluation weights

### All Extractions Failing

**Symptom**: No successful extractions

**Possible Causes**:
- WASM extractor not loaded
- All services down
- Configuration errors

**Solutions**:
1. Check logs for initialization errors
2. Verify `WASM_EXTRACTOR_PATH` is correct
3. Test WASM extractor independently
4. Check circuit breaker states
5. Verify network connectivity

## Best Practices

### 1. Start with Defaults

The default configuration works well for most use cases:
- 3 retries balances reliability and latency
- 10s timeout prevents hanging requests
- 0.6 quality threshold is a good middle ground

### 2. Monitor and Tune

After deployment:
1. Watch retry and fallback rates
2. Adjust timeouts based on real latencies
3. Tune quality threshold for your content types
4. Monitor circuit breaker state changes

### 3. Use ProbesFirst for Unknown Content

When crawling diverse content:
- ProbesFirst mode provides best balance
- Automatically adapts to content complexity
- Minimizes unnecessary headless rendering

### 4. Handle Events

Register event handlers for:
- Alerting on high failure rates
- Logging retry patterns
- Tracking quality score trends
- Circuit breaker state changes

### 5. Test Failure Scenarios

Regularly test:
- Service outages (simulate with network rules)
- Slow responses (add artificial delays)
- Partial failures (random errors)
- Quality degradation (poor content)

## Performance Impact

### Latency

- **Fast extraction**: 50-200ms (no change)
- **With retries**: +200-600ms on failures (exponential backoff)
- **Fallback**: +2-5s when falling back to headless
- **Circuit breaker**: Instant fail when open (saves time)

### Resource Usage

- **Memory**: +2MB per ReliableExtractor instance (negligible)
- **CPU**: Minimal overhead from retry logic
- **Network**: Retries increase traffic on failures (3x max)

### Throughput

- **No failures**: Same as before (~100-500 req/s)
- **With retries**: Reduced by retry overhead (~80-400 req/s)
- **Circuit breaker open**: Higher throughput (fast fails)

## Advanced Configuration

### Custom Quality Evaluation

To implement custom quality evaluation:

```rust
// In your code, implement a custom evaluator
impl QualityEvaluator for CustomEvaluator {
    fn evaluate(&self, doc: &ExtractedDoc) -> f32 {
        // Your custom logic here
        // Return score between 0.0 and 1.0
    }
}
```

### Custom Retry Strategy

Modify `ReliabilityConfig` in code:

```rust
ReliabilityConfig {
    http_retry: RetryConfig {
        max_attempts: 5,  // More aggressive
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,  // Faster backoff
        jitter: true,
    },
    // ... other settings
}
```

### Per-Domain Configuration

For different behavior per domain:

```rust
// Example: Different timeouts for different domains
let timeout = match url.host_str() {
    Some("slow-site.com") => Duration::from_secs(30),
    Some("fast-site.com") => Duration::from_secs(5),
    _ => Duration::from_secs(10),
};
```

## Migration Guide

### From Direct WasmExtractor

**Before**:
```rust
let doc = wasm_extractor.extract(html, url, "article")?;
```

**After**:
```rust
// Automatically handled by pipeline
// No changes needed - ReliableExtractor wraps existing logic
```

### From Custom Retry Logic

**Before**:
```rust
for attempt in 0..3 {
    match extract(url).await {
        Ok(doc) => return Ok(doc),
        Err(e) if attempt < 2 => continue,
        Err(e) => return Err(e),
    }
}
```

**After**:
```rust
// Remove custom retry logic
// ReliableExtractor handles this automatically
let doc = extract_content(html, url, decision).await?;
```

## Support

For issues or questions:
1. Check logs for reliability events
2. Review this guide's troubleshooting section
3. Examine event bus for detailed metrics
4. File an issue with:
   - Configuration settings
   - Error messages
   - Event logs
   - Reproduction steps

## Related Documentation

- [Reliability Module Integration Summary](./RELIABILITY_MODULE_INTEGRATION_SUMMARY.md)
- [Event System Documentation](../crates/riptide-core/src/events/README.md)
- [Circuit Breaker Pattern](../crates/riptide-core/src/circuit_breaker.rs)
- [Pipeline Architecture](../crates/riptide-api/src/pipeline.rs)
