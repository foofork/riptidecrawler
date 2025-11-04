# 🎯 RipTide v1 API - User Experience Design
## Making Web Extraction as Simple as crawl4ai & firecrawl

---

## Vision: Progressive Complexity API

The API should be **dead simple for basic use** but **progressively powerful** for advanced scenarios.

```python
# As simple as crawl4ai for basic extraction
result = riptide.extract("https://example.com")

# As powerful as you need with schema-driven extraction
result = riptide.pipeline(
    search="events in Amsterdam",
    schema="events.v1",
    output_format="calendar"
)
```

---

## 📊 Comparison with Existing Tools

### Current State of Popular Tools

| Tool | Simple API | Schema Support | Auto Pipeline | Search→Extract |
|------|------------|----------------|---------------|----------------|
| **crawl4ai** | ✅ `crawl(url)` | ❌ | ❌ | ❌ |
| **firecrawl** | ✅ `scrape(url)` | ✅ Limited | ❌ | ❌ |
| **RipTide v0.9** | ❌ Complex | ❌ | ❌ | ❌ |
| **RipTide v1.0** | ✅ | ✅ Full | ✅ | ✅ |

---

## 🚀 Core API Design - Progressive Enhancement

### Level 1: Dead Simple (80% of users)

```bash
# Just extract content from a URL
curl -X POST https://api.riptide.io/v1/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Response: Smart extraction with auto-detected schema
{
  "success": true,
  "data": {
    "title": "Example Page",
    "content": "...",
    "metadata": {...},
    "schema_detected": "article"
  }
}
```

**Python SDK Example:**
```python
from riptide import RipTide

client = RipTide()

# Simplest possible - just like crawl4ai
result = client.extract("https://example.com")
print(result.content)

# Or with options
result = client.extract(
    "https://example.com",
    headless=True,  # Use browser if needed
    clean=True      # Clean HTML
)
```

### Level 2: Schema-Aware (15% of users)

```bash
# Extract with specific schema
curl -X POST https://api.riptide.io/v1/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://eventsite.com/calendar",
    "schema": "events.v1",
    "output_format": "icalendar"
  }'

# Response: Structured data matching schema
{
  "success": true,
  "data": {
    "events": [
      {
        "title": "Tech Conference",
        "date": "2025-12-01",
        "location": "Amsterdam",
        "url": "..."
      }
    ],
    "format": "events.v1",
    "output_url": "https://api.riptide.io/exports/abc123.ics"
  }
}
```

**Python SDK:**
```python
# Extract events from any website
events = client.extract(
    "https://eventsite.com",
    schema="events",  # Auto-selects events.v1
    output_format="icalendar"
)

# Or extract jobs
jobs = client.extract(
    "https://careers.example.com",
    schema="jobs",
    filters={"location": "remote"}
)
```

### Level 3: Automated Pipeline (5% of users)

```bash
# Complete pipeline: Search → Discover → Crawl → Extract
curl -X POST https://api.riptide.io/v1/pipeline \
  -H "Content-Type: application/json" \
  -d '{
    "search": "tech events Amsterdam December 2025",
    "schema": "events.v1",
    "max_sources": 10,
    "output_format": "calendar",
    "webhook_url": "https://myapp.com/webhook"
  }'

# Response: Pipeline job started
{
  "job_id": "pipe_abc123",
  "status": "running",
  "stages": {
    "search": "completed",
    "discovery": "running",
    "extraction": "pending"
  },
  "progress_url": "https://api.riptide.io/v1/jobs/pipe_abc123"
}
```

**Python SDK:**
```python
# Full automated pipeline
pipeline = client.pipeline(
    search="tech conferences Europe 2025",
    schema="events",
    options={
        "dedupe": True,
        "min_confidence": 0.8,
        "output_format": "google_calendar"
    }
)

# Stream results as they come
for event in pipeline.stream():
    print(f"Found: {event.title} on {event.date}")

# Or wait for all
results = pipeline.wait()
```

---

## 🎨 API Patterns - Best of Both Worlds

### Simple Defaults, Progressive Options

```python
# Minimum viable call (crawl4ai simplicity)
client.extract(url)

# Progressive enhancement
client.extract(
    url,
    # Basic options
    headless=True,
    timeout=30,
    
    # Advanced extraction
    schema="events.v1",
    strategies=["llm", "rules"],
    
    # Output control
    output_format="csv",
    include_metadata=True,
    
    # Performance
    cache_ttl=3600,
    parallel=True
)
```

### Smart Schema Detection

```python
# Let RipTide figure out the schema
result = client.extract("https://indeed.com/jobs?q=python")
print(result.schema_detected)  # "jobs.v1"

# Or be explicit
result = client.extract(
    "https://indeed.com/jobs?q=python",
    schema="jobs.v1"
)
```

### Streaming for Large Operations

```python
# Stream results as they're extracted
for item in client.crawl_stream(url, max_pages=100):
    process(item)  # Don't wait for all 100 pages

# Or batch
results = client.crawl(url, max_pages=100)  # Waits for all
```

---

## 🔧 Configuration Profiles - Sensible Defaults

### Lite Profile (Default)
```yaml
# Fastest, cheapest extraction
profile: lite
strategies:
  - ics       # Fast calendar extraction
  - json_ld   # Structured data
  - rules     # Pattern matching
headless: false  # No browser overhead
llm: false       # No AI costs
```

### Full Profile
```yaml
# Maximum extraction quality
profile: full
strategies:
  - ics
  - json_ld
  - rules
  - wasm      # Custom extractors
  - llm       # AI extraction
headless: true   # Full browser
llm: true        # Use AI when needed
```

### Custom Profiles
```python
# Users can create custom profiles
client = RipTide(profile={
    "strategies": ["llm"],  # LLM-only extraction
    "headless": True,
    "llm_model": "gpt-4",
    "max_cost": 0.50  # Cost cap per request
})
```

---

## 📡 Webhooks & Async Operations

### Fire and Forget
```python
# Start extraction, get results via webhook
job = client.extract_async(
    url="https://example.com",
    webhook_url="https://myapp.com/webhook"
)
print(job.id)  # "job_xyz789"
```

### Webhook Payload
```json
{
  "job_id": "job_xyz789",
  "status": "completed",
  "result": {
    "schema": "events.v1",
    "data": [...],
    "metadata": {
      "extraction_time": 1.23,
      "strategies_used": ["ics", "rules"],
      "confidence": 0.95
    }
  }
}
```

---

## 🎯 Schema-Driven Extraction Examples

### Events Schema
```python
# Extract from any event website
events = client.extract(
    "https://meetup.com/tech-amsterdam",
    schema="events"
)

# Automatic output formats
events.to_icalendar("events.ics")
events.to_google_calendar()
events.to_csv("events.csv")
```

### Jobs Schema
```python
# Extract from any job board
jobs = client.extract(
    "https://careers.spotify.com",
    schema="jobs",
    filters={
        "location": "Stockholm",
        "type": "engineering"
    }
)

# Rich filtering on extracted data
remote_jobs = jobs.filter(remote=True)
senior_jobs = jobs.filter(level="senior")
```

### Custom Schema
```python
# Define your own schema
my_schema = {
    "name": "products",
    "fields": {
        "name": "string",
        "price": "number",
        "availability": "boolean"
    }
}

products = client.extract(
    "https://shop.example.com",
    schema=my_schema
)
```

---

## 🔄 Migration from Other Tools

### From crawl4ai
```python
# crawl4ai style
from crawl4ai import crawl
result = crawl("https://example.com")

# RipTide v1 (compatible)
from riptide import crawl  # Compatibility import
result = crawl("https://example.com")

# Or modern style
from riptide import RipTide
client = RipTide()
result = client.extract("https://example.com")
```

### From firecrawl
```python
# firecrawl style
from firecrawl import FirecrawlApp
app = FirecrawlApp(api_key="...")
result = app.scrape_url("https://example.com")

# RipTide v1 (similar)
from riptide import RipTide
client = RipTide(api_key="...")
result = client.extract("https://example.com")
```

---

## 📊 Performance & Pricing Model

### Transparent Pricing
```python
# Check costs before running
estimate = client.estimate_cost(
    url="https://example.com",
    options={"headless": True, "llm": True}
)
print(f"Estimated cost: ${estimate.cost}")
print(f"Estimated time: {estimate.duration}s")

# Set budget limits
client = RipTide(
    daily_budget=10.00,  # $10/day max
    per_request_limit=0.50  # $0.50/request max
)
```

### Performance Tiers
```yaml
# Response times by tier
lite:
  latency_p50: 200ms
  latency_p95: 500ms
  cost: $0.001

standard:
  latency_p50: 500ms
  latency_p95: 1500ms
  cost: $0.01

full:
  latency_p50: 1500ms
  latency_p95: 3500ms
  cost: $0.05
```

---

## 🚀 Quick Start Examples

### 1. Extract Events from a City
```python
from riptide import RipTide

client = RipTide()

# One-liner to get all events
events = client.pipeline(
    search="events in Berlin this week",
    schema="events"
).wait()

print(f"Found {len(events)} events")
for event in events[:5]:
    print(f"- {event.title} on {event.date}")
```

### 2. Monitor Job Board
```python
# Set up monitoring for new jobs
monitor = client.monitor(
    url="https://news.ycombinator.com/jobs",
    schema="jobs",
    schedule="daily",
    webhook="https://myapp.com/new-jobs"
)
```

### 3. Bulk Extraction
```python
# Extract from multiple URLs efficiently
urls = [
    "https://site1.com",
    "https://site2.com",
    "https://site3.com"
]

results = client.extract_batch(
    urls,
    schema="products",
    parallel=True
)
```

---

## 📝 CLI Experience

### Simple Commands
```bash
# Extract from URL
riptide extract https://example.com

# With schema
riptide extract https://eventsite.com --schema events

# Full pipeline
riptide pipeline "tech events Amsterdam" --schema events --output events.ics

# Monitor for changes
riptide monitor https://example.com/jobs --schema jobs --webhook http://localhost:3000
```

### Interactive Mode
```bash
$ riptide interactive

RipTide> extract https://example.com
✓ Extracted successfully (0.3s)

RipTide> set schema events
✓ Schema set to events.v1

RipTide> set output icalendar
✓ Output format set to icalendar

RipTide> extract https://meetup.com/events
✓ Extracted 15 events (1.2s)
✓ Saved to events.ics
```

---

## 🎯 Success Metrics for v1.0

### Developer Experience
- [ ] Single-line extraction works
- [ ] Schema auto-detection accuracy >80%
- [ ] Migration from crawl4ai <5 min
- [ ] SDK available for Python, Node, Go

### Performance
- [ ] Simple extraction <500ms p95
- [ ] Schema extraction <1500ms p95
- [ ] Pipeline completion <60s for 10 sources

### Adoption
- [ ] 100+ developers in first month
- [ ] 1000+ API calls daily
- [ ] 5+ production integrations

---

## 📚 Documentation Requirements

### Getting Started (5 minutes)
```markdown
1. Install: pip install riptide
2. Get API key: https://riptide.io/keys
3. First extraction:
   ```python
   from riptide import RipTide
   client = RipTide(api_key="...")
   result = client.extract("https://example.com")
   print(result.content)
   ```
```

### Examples Gallery
- Extract events → Google Calendar
- Job board → Email alerts
- Product prices → Monitoring
- News articles → RSS feed
- Social media → Analytics

### API Reference
- Complete OpenAPI spec
- Interactive playground
- Language-specific docs
- Schema library

---

## 🔮 Future Vision

### v1.1 - Intelligent Pipelines
- Auto-detect best extraction strategy
- Self-healing selectors
- Automatic schema mapping

### v1.2 - Ecosystem
- Community schemas
- Custom extractor marketplace
- Integrations (Zapier, n8n, etc)

### v2.0 - AI-Native
- Natural language pipelines
- Visual programming interface
- Automatic optimization

---

**The goal:** Make RipTide the "Stripe of web extraction" - simple for basic use, powerful when needed, with progressive disclosure of complexity.
