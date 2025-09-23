# Phase 3 API Changes and Integration Guide

## Overview

This document details the API changes and new endpoints introduced in Phase 3 to support advanced crawling features while maintaining full backward compatibility with existing implementations.

## Backward Compatibility Statement

**All existing API endpoints and request formats will continue to work unchanged.** Phase 3 introduces only additive changes - new optional fields and new endpoints that extend functionality without breaking existing integrations.

## Enhanced Existing Endpoints

### 1. POST /crawl - Enhanced Batch Crawling

#### Current (Phase 1-2) - Fully Supported
```json
{
  "urls": ["https://example.com"],
  "options": {
    "concurrency": 16,
    "cache_mode": "read_through",
    "dynamic_wait_for": null,
    "scroll_steps": 8,
    "token_chunk_max": 1200,
    "token_overlap": 120
  }
}
```

#### Phase 3 Enhancements - Additive Only
```json
{
  "urls": ["https://example.com"],
  "options": {
    // All existing options supported
    "concurrency": 16,
    "cache_mode": "read_through",
    "dynamic_wait_for": "selector:.content",
    "scroll_steps": 8,
    "token_chunk_max": 1200,
    "token_overlap": 120,

    // NEW: Stealth options
    "stealth": {
      "rotate_user_agents": true,
      "randomize_headers": true,
      "use_proxies": false,
      "add_jitter": true,
      "stealth_headless": true
    },

    // NEW: Enhanced extraction
    "extraction": {
      "mode": "article", // "article" | "full" | "metadata"
      "include_chunks": true,
      "include_structured_data": true,
      "include_outlinks": true,
      "byline_extraction": true,
      "date_extraction": true
    }
  },

  // NEW: Deep crawling capabilities
  "deep_crawl": {
    "enabled": true,
    "max_depth": 2,
    "max_pages": 50,
    "same_domain_only": true,
    "respect_robots": true,
    "crawl_budget_seconds": 300,
    "follow_sitemaps": true,
    "url_patterns": {
      "include": ["*/articles/*", "*/blog/*"],
      "exclude": ["*/admin/*", "*/login"]
    }
  }
}
```

#### Enhanced Response Format
```json
{
  "total_urls": 25,  // May be higher with deep crawl
  "successful": 23,
  "failed": 2,
  "from_cache": 8,
  "results": [
    {
      "url": "https://example.com",
      "status": 200,
      "from_cache": false,
      "gate_decision": "headless",
      "quality_score": 0.85,
      "processing_time_ms": 1234,
      "cache_key": "riptide:v1:read_through:abc123",
      "document": {
        // Enhanced document structure
        "url": "https://example.com",
        "title": "Article Title",
        "byline": "John Doe",                    // NEW
        "published_iso": "2024-01-15T10:30:00Z", // NEW
        "markdown": "# Article Title\n\n...",
        "text": "Article content...",
        "links": ["https://related.com"],
        "media": ["https://img.example.com/pic.jpg"],
        "language": "en",                        // NEW
        "reading_time": 5,                       // NEW
        "quality_score": 85,                     // NEW
        "word_count": 1200,                      // NEW
        "categories": ["Technology", "AI"],      // NEW
        "site_name": "Example Blog",             // NEW
        "description": "Meta description...",   // NEW

        // NEW: Content chunking
        "content_chunks": [
          {
            "text": "First paragraph...",
            "tokens": 45,
            "chunk_index": 0,
            "metadata": {
              "source_url": "https://example.com",
              "start_offset": 0,
              "end_offset": 180,
              "preserve_boundaries": true
            }
          }
        ],

        // NEW: Structured data
        "structured_data": {
          "@context": "https://schema.org",
          "@type": "Article",
          "headline": "Article Title",
          "author": "John Doe"
        },

        // NEW: Enhanced links with context
        "outlinks": [
          {
            "url": "https://related.com",
            "text": "Related Article",
            "context": "mentioned in paragraph 3"
          }
        ],

        // NEW: Images with metadata
        "images_with_context": [
          {
            "url": "https://img.example.com/pic.jpg",
            "alt": "Descriptive text",
            "caption": "Image caption",
            "position": "after_paragraph_2"
          }
        ]
      },
      "error": null,

      // NEW: Deep crawl metadata (if applicable)
      "crawl_metadata": {
        "depth": 1,
        "parent_url": "https://example.com",
        "discovered_via": "sitemap", // "links" | "sitemap" | "search"
        "discovery_time": "2024-01-15T10:35:00Z"
      }
    }
  ],
  "statistics": {
    "total_processing_time_ms": 15000,
    "avg_processing_time_ms": 650.0,
    "gate_decisions": {
      "raw": 10,
      "probes_first": 8,
      "headless": 5,
      "cached": 2
    },
    "cache_hit_rate": 0.32,

    // NEW: Deep crawl statistics
    "deep_crawl_stats": {
      "seeds_provided": 1,
      "total_discovered": 24,
      "sitemap_urls": 15,
      "link_discovered": 9,
      "depth_breakdown": {
        "0": 1,
        "1": 12,
        "2": 12
      },
      "robots_blocked": 3,
      "budget_exhausted": false
    }
  }
}
```

### 2. POST /deepsearch - Enhanced Search-Driven Crawling

#### Phase 3 Enhancements
```json
{
  "query": "artificial intelligence news",
  "limit": 20,
  "include_content": true,
  "crawl_options": {
    // All /crawl options supported
    "stealth": { "rotate_user_agents": true },
    "extraction": { "include_structured_data": true }
  },

  // NEW: Search refinement
  "search_options": {
    "country": "us",
    "language": "en",
    "time_range": "week", // "day" | "week" | "month" | "year"
    "result_types": ["web"], // "web" | "news" | "academic"
    "exclude_domains": ["spam.com"],
    "require_content_type": "article"
  },

  // NEW: Follow discovered links
  "follow_links": {
    "enabled": true,
    "max_depth": 1,
    "same_domain_only": false,
    "max_additional_pages": 10
  }
}
```

### 3. POST /render - Enhanced Dynamic Content

#### Current (Phase 2) - Fully Supported
```json
{
  "url": "https://spa-example.com",
  "wait_for": "domcontentloaded",
  "scroll_steps": 3
}
```

#### Phase 3 Enhancements
```json
{
  "url": "https://spa-example.com",

  // Enhanced wait conditions
  "wait_for": {
    "type": "selector",        // "timeout" | "selector" | "custom_js" | "network_idle"
    "value": ".article-content",
    "timeout": 10000
  },

  // Advanced scrolling
  "scroll_config": {
    "steps": 5,
    "step_px": 300,
    "delay_ms": 500,
    "trigger_lazy_loading": true
  },

  // Page interactions
  "actions": [
    {
      "type": "click",
      "selector": ".load-more-button",
      "wait_after": 2000
    },
    {
      "type": "type",
      "selector": "input[name='search']",
      "text": "search term",
      "wait_after": 1000
    },
    {
      "type": "evaluate",
      "script": "document.querySelector('.modal').style.display = 'none';"
    }
  ],

  // Artifact capture
  "capture": {
    "screenshot": true,
    "mhtml": false,
    "full_page": true,
    "format": "png" // "png" | "jpeg" | "webp"
  },

  // Stealth options for headless
  "stealth": {
    "disable_automation_flags": true,
    "randomize_viewport": true,
    "custom_user_agent": "Mozilla/5.0...",
    "block_resources": ["images", "stylesheets"], // Performance optimization
    "custom_headers": {
      "Accept-Language": "en-US,en;q=0.9"
    }
  }
}
```

#### Enhanced Response
```json
{
  "url": "https://spa-example.com",
  "rendered_html": "<html>...</html>",
  "final_url": "https://spa-example.com#loaded", // After redirects/SPA navigation
  "load_time_ms": 3500,
  "screenshot_url": "https://artifacts.riptide.com/abc123.png", // If captured
  "mhtml_url": "https://artifacts.riptide.com/abc123.mhtml",     // If captured
  "page_metadata": {
    "title": "Final Page Title",
    "viewport": { "width": 1920, "height": 1080 },
    "performance": {
      "dom_content_loaded": 1200,
      "load_complete": 2800,
      "network_requests": 45,
      "javascript_errors": 0
    }
  },
  "actions_executed": [
    {
      "action": "click .load-more-button",
      "success": true,
      "execution_time": 1500
    }
  ]
}
```

## New Endpoints

### 1. Streaming Endpoints

#### POST /crawl/stream - Real-time Crawl Progress
```bash
curl -X POST https://api.riptide.com/crawl/stream \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{"urls": ["https://example.com"], "deep_crawl": {"enabled": true}}'
```

**Response Format (Server-Sent Events):**
```
event: progress
data: {"type": "started", "total_urls": 1, "timestamp": "2024-01-15T10:30:00Z"}

event: url_discovered
data: {"url": "https://example.com/article1", "depth": 1, "parent": "https://example.com"}

event: url_completed
data: {"url": "https://example.com", "status": 200, "processing_time_ms": 1234, "quality_score": 0.85}

event: extraction_complete
data: {"url": "https://example.com", "document": {...}}

event: batch_complete
data: {"total_processed": 25, "successful": 23, "failed": 2, "total_time_ms": 15000}
```

#### POST /deepsearch/stream - Real-time Search and Crawl
Similar to `/crawl/stream` but includes search progress events:
```
event: search_started
data: {"query": "AI news", "search_engine": "serper"}

event: search_complete
data: {"urls_found": 20, "search_time_ms": 500}

event: url_completed
data: {"url": "https://news.com/ai-article", "rank": 1, "document": {...}}
```

### 2. PDF-Specific Endpoints

#### POST /pdf/extract - Direct PDF Processing
```json
{
  "pdf_url": "https://example.com/document.pdf",
  "options": {
    "extract_images": true,
    "extract_metadata": true,
    "ocr_if_needed": false,
    "page_range": {"start": 1, "end": 10}
  }
}
```

**Response:**
```json
{
  "url": "https://example.com/document.pdf",
  "text": "Extracted text content...",
  "metadata": {
    "title": "Document Title",
    "author": "Author Name",
    "creation_date": "2024-01-01T00:00:00Z",
    "modification_date": "2024-01-15T10:30:00Z",
    "page_count": 25,
    "creator": "Microsoft Word",
    "producer": "PDF Creator"
  },
  "images": [
    {
      "page": 3,
      "position": {"x": 100, "y": 200, "width": 300, "height": 200},
      "extracted_url": "https://artifacts.riptide.com/img123.jpg",
      "description": "Chart showing quarterly results"
    }
  ],
  "processing_time_ms": 2500
}
```

### 3. Bulk Operations

#### POST /bulk/queue - Queue Large Crawl Jobs
```json
{
  "job_name": "quarterly-crawl-2024",
  "urls": ["https://site1.com", "https://site2.com"], // Up to 10,000 URLs
  "options": {
    "priority": "normal", // "low" | "normal" | "high"
    "schedule": {
      "start_time": "2024-01-15T02:00:00Z",
      "rate_limit": "100_per_hour"
    },
    "callbacks": {
      "webhook_url": "https://yoursite.com/webhook",
      "notification_email": "admin@yoursite.com"
    }
  },
  "crawl_options": {
    // All standard crawl options
    "deep_crawl": {"enabled": true, "max_depth": 3}
  }
}
```

#### GET /bulk/status/{job_id} - Check Job Progress
```json
{
  "job_id": "qc2024-abc123",
  "job_name": "quarterly-crawl-2024",
  "status": "running", // "queued" | "running" | "completed" | "failed"
  "progress": {
    "total_urls": 50000,
    "completed": 12500,
    "failed": 125,
    "in_progress": 25,
    "estimated_completion": "2024-01-16T08:30:00Z"
  },
  "created_at": "2024-01-15T10:00:00Z",
  "started_at": "2024-01-15T10:05:00Z",
  "statistics": {
    "avg_processing_time_ms": 850,
    "cache_hit_rate": 0.15,
    "gate_decisions": {"raw": 8000, "headless": 4500}
  }
}
```

## Error Handling Enhancements

### New Error Types
```json
{
  "error": {
    "type": "deep_crawl_budget_exceeded",
    "message": "Crawl budget of 300 seconds exceeded",
    "retryable": false,
    "details": {
      "budget_seconds": 300,
      "elapsed_seconds": 315,
      "urls_processed": 45,
      "urls_remaining": 12
    }
  }
}

{
  "error": {
    "type": "pdf_processing_failed",
    "message": "PDF appears to be corrupted or password protected",
    "retryable": false,
    "details": {
      "pdf_size_bytes": 15000000,
      "error_code": "INVALID_PDF_STRUCTURE"
    }
  }
}

{
  "error": {
    "type": "stealth_detection",
    "message": "Bot detection triggered, consider adjusting stealth settings",
    "retryable": true,
    "details": {
      "detection_type": "browser_automation",
      "suggested_action": "enable_stealth_headless"
    }
  }
}
```

## Rate Limiting and Quotas

### Enhanced Rate Limiting Headers
```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 750
X-RateLimit-Reset: 1642234800
X-RateLimit-Scope: crawl_requests

# New headers for different resource types
X-RateLimit-PDF-Limit: 100
X-RateLimit-PDF-Remaining: 85
X-RateLimit-Headless-Limit: 500
X-RateLimit-Headless-Remaining: 420
```

## Authentication and API Keys

### Enhanced API Key Scopes
```http
Authorization: Bearer <api_key>
X-API-Scope: crawl,deepsearch,pdf,streaming,bulk
```

### New API Key Scopes
- `crawl`: Basic crawling endpoints
- `deepsearch`: Search-driven crawling
- `pdf`: PDF processing capabilities
- `streaming`: Real-time streaming endpoints
- `bulk`: Large-scale batch operations
- `premium`: Advanced stealth and unlimited headless

## Migration Guide

### For Existing Integrations

1. **No Changes Required**: All existing code will continue working
2. **Gradual Enhancement**: Add new features incrementally
3. **Feature Detection**: Check API version in responses

### Version Detection
```json
// All responses include API version
{
  "api_version": "3.0",
  "features_available": [
    "deep_crawl",
    "stealth_mode",
    "pdf_processing",
    "streaming",
    "content_chunking"
  ],
  // ... rest of response
}
```

### Recommended Migration Path

1. **Phase 3.0**: Test existing integrations (should work unchanged)
2. **Phase 3.1**: Add stealth options for sensitive sites
3. **Phase 3.2**: Enable deep crawling for site discovery
4. **Phase 3.3**: Implement streaming for large batches
5. **Phase 3.4**: Add PDF processing for document sites

## Performance Considerations

### Request Optimization
- **Deep crawling**: May significantly increase processing time
- **PDF processing**: Limit concurrent PDFs (recommended: max 2 per request)
- **Streaming**: Reduces memory usage for large batches
- **Stealth mode**: Adds 100-300ms per request for randomization

### Recommended Limits
```json
{
  "limits": {
    "max_urls_per_request": 100,    // Standard batch
    "max_deep_crawl_pages": 1000,   // Per seed URL
    "max_pdf_size_mb": 50,          // PDF processing
    "max_concurrent_requests": 10,   // Per API key
    "streaming_buffer_size": 64     // KB per stream
  }
}
```

## Example Integration Scenarios

### 1. Content Migration - Deep Site Crawl
```bash
# Discover and crawl entire site
curl -X POST https://api.riptide.com/crawl \
  -H "Authorization: Bearer your_api_key" \
  -d '{
    "urls": ["https://oldsite.com"],
    "deep_crawl": {
      "enabled": true,
      "max_depth": 3,
      "max_pages": 500,
      "follow_sitemaps": true
    },
    "options": {
      "extraction": {
        "include_structured_data": true,
        "include_outlinks": true
      }
    }
  }'
```

### 2. News Monitoring - Real-time Search
```bash
# Monitor news with streaming
curl -X POST https://api.riptide.com/deepsearch/stream \
  -H "Accept: text/event-stream" \
  -d '{
    "query": "AI breakthrough 2024",
    "search_options": {"time_range": "day"},
    "follow_links": {"enabled": true, "max_depth": 1}
  }'
```

### 3. Document Processing - PDF Research
```bash
# Process academic papers
curl -X POST https://api.riptide.com/crawl \
  -d '{
    "urls": [
      "https://arxiv.org/pdf/2401.12345.pdf",
      "https://research.org/paper.pdf"
    ],
    "options": {
      "extraction": {
        "mode": "full",
        "include_chunks": true
      }
    }
  }'
```

This comprehensive API documentation ensures smooth integration of Phase 3 features while maintaining full backward compatibility and providing clear migration paths for existing users.