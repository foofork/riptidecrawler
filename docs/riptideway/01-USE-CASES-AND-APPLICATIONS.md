# RipTide Use Cases & Applications

**Comprehensive guide to real-world applications and use cases**

---

## Table of Contents
- [Enterprise Use Cases](#enterprise-use-cases)
- [Data Intelligence](#data-intelligence)
- [Content Aggregation](#content-aggregation)
- [Research & Analysis](#research--analysis)
- [Business Applications](#business-applications)
- [Technical Applications](#technical-applications)
- [Industry-Specific Solutions](#industry-specific-solutions)

---

## Enterprise Use Cases

### 1. Competitive Intelligence & Market Research

**Problem**: Companies need real-time competitive intelligence across multiple sources.

**RipTide Solution**:
```json
{
  "workflow": "Competitive Monitoring",
  "components": [
    "Spider deep crawling for competitor websites",
    "PDF extraction for whitepapers and reports",
    "Table extraction for pricing comparisons",
    "LLM summarization for insights",
    "Scheduled jobs for daily updates"
  ],
  "endpoints_used": [
    "POST /spider/crawl",
    "POST /pdf/process",
    "POST /api/v1/tables/extract",
    "POST /workers/schedule"
  ]
}
```

**Key Features**:
- ✅ Deep crawl competitor sites with respect for robots.txt
- ✅ Extract pricing tables and product comparisons
- ✅ Process PDF reports and whitepapers
- ✅ Schedule daily automated crawls
- ✅ LLM-powered summarization of findings
- ✅ Session management for authenticated access

**Benefits**:
- Automated daily monitoring
- Structured data extraction (tables, prices)
- Historical trend analysis
- Real-time alerts via streaming

---

### 2. Lead Generation & Contact Discovery

**Problem**: Sales teams need to discover and qualify leads from multiple sources.

**RipTide Solution**:
```json
{
  "workflow": "Lead Discovery",
  "components": [
    "Search integration (Serper) for company discovery",
    "Spider crawling for contact pages",
    "Regex extraction for emails and phone numbers",
    "Table extraction for team directories",
    "Worker queue for batch processing"
  ],
  "endpoints_used": [
    "POST /deepsearch",
    "POST /spider/crawl",
    "POST /strategies/crawl (regex mode)",
    "POST /workers/jobs"
  ]
}
```

**Implementation**:
```bash
# Step 1: Search for companies in target industry
POST /deepsearch
{
  "query": "fintech startups Series A funding",
  "num_results": 50,
  "extract_content": true
}

# Step 2: Spider crawl each company website
POST /spider/crawl
{
  "seed_urls": ["https://company.com"],
  "max_depth": 3,
  "strategy": "breadth_first",
  "respect_robots": true
}

# Step 3: Extract contact information
POST /strategies/crawl?strategy=regex
{
  "url": "https://company.com/contact",
  "regex_patterns": [
    {"name": "email", "pattern": "[a-z0-9._%+-]+@[a-z0-9.-]+\\.[a-z]{2,}"},
    {"name": "phone", "pattern": "\\+?[1-9]\\d{1,14}"}
  ]
}
```

**Benefits**:
- Automated lead discovery
- Contact validation
- Team hierarchy extraction from org charts
- Batch processing for scale

---

### 3. Document Intelligence & Knowledge Management

**Problem**: Organizations have PDFs scattered across systems and need structured extraction.

**RipTide Solution**:
```json
{
  "workflow": "Document Processing Pipeline",
  "components": [
    "PDF extraction with metadata",
    "Table extraction from documents",
    "Content chunking for vector DB ingestion",
    "LLM summarization and classification"
  ],
  "endpoints_used": [
    "POST /pdf/process",
    "POST /api/v1/tables/extract",
    "POST /strategies/crawl (chunking enabled)"
  ]
}
```

**Implementation**:
```bash
# Process PDF with comprehensive extraction
POST /pdf/process
{
  "pdf_data": "base64_encoded_pdf",
  "stream_progress": true
}

# Extract tables for structured data
POST /api/v1/tables/extract
{
  "html_content": "...",
  "extract_options": {
    "include_nested": true,
    "detect_data_types": true
  }
}

# Export to formats
GET /api/v1/tables/{table_id}/export?format=csv
GET /api/v1/tables/{table_id}/export?format=markdown
```

**Use Cases**:
- Contract analysis (extract terms, dates, parties)
- Financial report processing
- Legal document review
- Regulatory compliance documentation

---

## Data Intelligence

### 4. Real-Time News Monitoring & Aggregation

**Problem**: Media companies need to aggregate news from multiple sources in real-time.

**RipTide Solution**:
```json
{
  "workflow": "News Aggregation",
  "components": [
    "NDJSON streaming for real-time updates",
    "Spider crawling for article discovery",
    "LLM classification for topic categorization",
    "Session management for paywalled content"
  ],
  "endpoints_used": [
    "POST /crawl/stream",
    "POST /spider/crawl",
    "POST /sessions (for authenticated access)"
  ]
}
```

**Implementation**:
```bash
# Real-time streaming crawl
POST /crawl/stream
{
  "urls": [
    "https://news-site-1.com/latest",
    "https://news-site-2.com/breaking"
  ],
  "options": {
    "use_spider": true,
    "cache_mode": "read_write"
  }
}

# Response: NDJSON stream
{"status": "processing", "url": "https://news-site-1.com/latest"}
{"status": "success", "url": "https://news-site-1.com/latest", "document": {...}}
{"status": "processing", "url": "https://news-site-2.com/breaking"}
```

**Benefits**:
- Real-time article discovery
- Live streaming updates
- Paywall handling via sessions
- Multi-source aggregation

---

### 5. E-commerce Price Monitoring

**Problem**: Retailers need to monitor competitor pricing across thousands of products.

**RipTide Solution**:
```json
{
  "workflow": "Price Intelligence",
  "components": [
    "Scheduled jobs for hourly price checks",
    "CSS extraction for structured data",
    "Table extraction for price comparisons",
    "Stealth mode to avoid blocks"
  ],
  "endpoints_used": [
    "POST /workers/schedule",
    "POST /strategies/crawl (css_json mode)",
    "POST /stealth/configure"
  ]
}
```

**Implementation**:
```bash
# Configure stealth for e-commerce sites
POST /stealth/configure
{
  "preset": "aggressive",
  "custom_settings": {
    "rotate_user_agents": true,
    "randomize_timing": true
  }
}

# Extract pricing data
POST /strategies/crawl?strategy=css_json
{
  "url": "https://competitor.com/product/123",
  "css_selectors": {
    "price": ".price-display",
    "original_price": ".was-price",
    "availability": ".stock-status",
    "rating": ".product-rating"
  }
}

# Schedule hourly checks
POST /workers/schedule
{
  "job_type": "batch_crawl",
  "cron_expression": "0 * * * *",
  "urls": ["https://competitor.com/products"],
  "priority": "normal"
}
```

**Benefits**:
- Hourly price updates
- Historical price tracking
- Availability monitoring
- Competitive intelligence

---

### 6. Financial Data Aggregation

**Problem**: Financial analysts need structured data from earnings reports, filings, and tables.

**RipTide Solution**:
```json
{
  "workflow": "Financial Data Extraction",
  "components": [
    "PDF extraction for SEC filings",
    "Table extraction for financial statements",
    "Regex extraction for key metrics",
    "Content chunking for LLM analysis"
  ],
  "endpoints_used": [
    "POST /pdf/process",
    "POST /api/v1/tables/extract",
    "POST /strategies/crawl (regex mode)"
  ]
}
```

**Implementation**:
```bash
# Extract financial tables from PDF
POST /pdf/process
{
  "pdf_data": "base64_encoded_10k_report",
  "filename": "AAPL-10K-2024.pdf"
}

# Extract financial tables
POST /api/v1/tables/extract
{
  "html_content": "...",
  "extract_options": {
    "detect_data_types": true,
    "min_size": [3, 3]
  }
}

# Export to CSV for analysis
GET /api/v1/tables/{id}/export?format=csv
```

**Use Cases**:
- Earnings report analysis
- SEC filing monitoring
- Financial statement comparison
- Key metrics extraction (EPS, revenue, margins)

---

## Content Aggregation

### 7. Academic Research Paper Aggregation

**Problem**: Researchers need to discover and extract content from academic papers across journals.

**RipTide Solution**:
```json
{
  "workflow": "Research Paper Processing",
  "components": [
    "Search integration for paper discovery",
    "PDF extraction for full-text",
    "Table/figure extraction",
    "Citation extraction via regex",
    "Content chunking for embeddings"
  ]
}
```

**Benefits**:
- Automated paper discovery
- Full-text extraction
- Citation graph building
- Abstract and summary generation

---

### 8. Job Board Aggregation

**Problem**: Job seekers and recruiters need unified view of postings across multiple sites.

**RipTide Solution**:
```json
{
  "workflow": "Job Listing Aggregation",
  "components": [
    "Spider crawling for job discovery",
    "CSS extraction for structured data",
    "Session management for login-required sites",
    "Worker queue for batch processing"
  ]
}
```

**Implementation**:
```bash
# Spider crawl job boards
POST /spider/crawl
{
  "seed_urls": [
    "https://jobs-site-1.com/engineering",
    "https://jobs-site-2.com/remote"
  ],
  "max_pages": 100,
  "strategy": "breadth_first"
}

# Extract job details
POST /strategies/crawl?strategy=css_json
{
  "url": "https://jobs-site.com/posting/123",
  "css_selectors": {
    "title": "h1.job-title",
    "company": ".company-name",
    "location": ".job-location",
    "salary": ".salary-range",
    "description": ".job-description"
  }
}
```

---

## Research & Analysis

### 9. SEO & Content Analysis

**Problem**: SEO professionals need to analyze competitor content and structure.

**RipTide Solution**:
```json
{
  "workflow": "SEO Intelligence",
  "components": [
    "Spider crawling for site structure",
    "HTML analysis for meta tags",
    "Table extraction for content audits",
    "Link extraction for backlink analysis"
  ]
}
```

**Use Cases**:
- Site structure mapping
- Meta tag analysis
- Content gap identification
- Backlink profile building

---

### 10. Market Sentiment Analysis

**Problem**: Traders need sentiment data from news, social media, and forums.

**RipTide Solution**:
```json
{
  "workflow": "Sentiment Monitoring",
  "components": [
    "Real-time streaming for live updates",
    "LLM sentiment classification",
    "Content chunking for topic extraction",
    "Scheduled crawls for periodic updates"
  ]
}
```

**Benefits**:
- Real-time sentiment tracking
- Multi-source aggregation
- Topic trend analysis
- Historical sentiment data

---

## Business Applications

### 11. Real Estate Listing Aggregation

**RipTide Solution**:
- Spider crawling of real estate sites
- Table extraction for property features
- Image extraction from listings
- Scheduled updates for new listings

---

### 12. Customer Review Monitoring

**RipTide Solution**:
- Multi-site review aggregation
- Sentiment analysis via LLM
- Table extraction for rating summaries
- Real-time alerts for negative reviews

---

### 13. Legal Case Research

**RipTide Solution**:
- PDF extraction for case documents
- Table extraction for precedents
- Citation extraction via regex
- Content chunking for similarity search

---

## Technical Applications

### 14. API Documentation Aggregation

**RipTide Solution**:
- Spider crawling of API docs
- Code example extraction
- Table extraction for endpoints
- Version comparison across releases

---

### 15. Software Vulnerability Monitoring

**RipTide Solution**:
- CVE database crawling
- PDF extraction for advisories
- Table extraction for affected versions
- Scheduled daily security updates

---

## Industry-Specific Solutions

### Healthcare
- Medical journal paper extraction
- Clinical trial data aggregation
- Drug information monitoring
- Regulatory update tracking

### Finance
- SEC filing monitoring
- Earnings report analysis
- Market data aggregation
- Regulatory compliance tracking

### Real Estate
- Property listing aggregation
- Market trend analysis
- Zoning regulation monitoring
- Historical sales data

### E-commerce
- Product catalog aggregation
- Price comparison
- Review aggregation
- Inventory monitoring

### Media & Publishing
- News aggregation
- Content syndication
- Copyright monitoring
- Trending topic discovery

### Legal
- Case law research
- Contract analysis
- Regulatory compliance
- Document discovery

### Education
- Course catalog aggregation
- Research paper discovery
- Academic job postings
- Grant opportunity monitoring

---

## Common Patterns & Best Practices

### Pattern 1: Authenticated Crawling
```bash
# Create session with login
POST /sessions
{
  "ttl_seconds": 3600,
  "metadata": {"user": "analysis-bot"}
}

# Set cookies
POST /sessions/{session_id}/cookies
{
  "domain": "site.com",
  "name": "session_token",
  "value": "...",
  "secure": true
}

# Crawl with session
POST /crawl
{
  "urls": ["https://site.com/protected"],
  "options": {
    "session_id": "{session_id}"
  }
}
```

### Pattern 2: Batch Processing with Workers
```bash
# Submit batch job
POST /workers/jobs
{
  "job_type": "batch_crawl",
  "urls": ["url1", "url2", ...],
  "priority": "high"
}

# Monitor progress
GET /workers/jobs/{job_id}

# Retrieve results
GET /workers/jobs/{job_id}/result
```

### Pattern 3: Real-Time Monitoring
```bash
# Start streaming crawl
POST /crawl/stream
{
  "urls": ["https://news-site.com/latest"],
  "options": {"use_spider": true}
}

# Receive NDJSON stream
{"status": "processing", "url": "..."}
{"status": "success", "document": {...}}
{"status": "completed", "total": 10}
```

### Pattern 4: Stealth Crawling
```bash
# Configure stealth
POST /stealth/configure
{
  "preset": "aggressive",
  "rotate_fingerprints": true
}

# Crawl with stealth
POST /crawl
{
  "urls": ["https://protected-site.com"],
  "options": {
    "stealth_enabled": true
  }
}
```

---

## Performance Considerations

### High-Volume Crawling
- Use worker queue for batch processing
- Implement rate limiting per domain
- Enable caching for repeated requests
- Use scheduled jobs for periodic crawls

### Real-Time Requirements
- Use streaming protocols (NDJSON, SSE, WebSocket)
- Enable fast-path CSS extraction
- Optimize cache hit rate
- Monitor queue depth

### Large-Scale Operations
- Distribute via multiple instances
- Use Redis cluster for caching
- Implement circuit breakers
- Monitor resource utilization

---

## Conclusion

RipTide's comprehensive feature set enables a wide range of use cases across industries:

**Strengths**:
- ✅ Multi-strategy extraction (CSS, WASM, LLM, Regex)
- ✅ Real-time streaming for live updates
- ✅ PDF and table processing for documents
- ✅ Stealth capabilities for protected sites
- ✅ Worker queue for batch processing
- ✅ Session management for authentication
- ✅ LLM abstraction for AI enhancement

**Ideal For**:
- Data intelligence and aggregation
- Competitive monitoring
- Content research and analysis
- Financial data extraction
- Lead generation
- Real-time news monitoring

**Not Ideal For**:
- Simple one-off scraping (use simpler tools)
- Non-structured content (without LLM)
- Sites with strict anti-bot measures (respect robots.txt)

**Next Steps**:
- Identify your specific use case
- Review relevant endpoint documentation
- Set up appropriate monitoring
- Configure stealth and rate limiting
- Test with small batches before scaling
