# Quick Reference: Extraction Methods & Test Categories

## Extraction Methods At-a-Glance

### Trek (WASM) Extraction
```
Best For: Static HTML, clean content
Speed:    ⚡⚡⚡⚡⚡ <500ms
Accuracy: ⭐⭐⭐⭐⭐ 90-95%
Status:   ⚠️  WASM binding incomplete (mock data)
```

### CSS Selector Extraction
```
Best For: Structured content with known patterns
Speed:    ⚡⚡⚡⚡ <300ms
Accuracy: ⭐⭐⭐⭐⭐ 90-95% (with good selectors)
Features: 12 transformers, :has-text() filter, merge policy
```

### Regex Pattern Extraction
```
Best For: Structured data (emails, phones, prices)
Speed:    ⚡⚡⚡⚡⚡ <200ms
Accuracy: ⭐⭐⭐ 60-85% (pattern-dependent)
Features: Pre-built pattern sets (news, contact, financial)
```

### Headless Browser Extraction
```
Best For: JavaScript-heavy SPAs, dynamic content
Speed:    ⚡⚡ <3s (expensive)
Accuracy: ⭐⭐⭐⭐⭐ 90-95% on SPAs
Cost:     💰💰💰 High resource usage
```

## When to Use Which Method

### Use Trek (WASM)
- ✅ Static HTML blogs
- ✅ News articles
- ✅ Documentation sites
- ✅ Server-rendered pages
- ❌ SPAs (React/Vue/Angular)
- ❌ Dynamic content

### Use CSS Extraction
- ✅ Known site structures
- ✅ E-commerce product pages
- ✅ Recipe sites
- ✅ Job listings
- ✅ News articles (with news selectors)
- ⚠️  Requires selector configuration

### Use Regex Extraction
- ✅ Email addresses
- ✅ Phone numbers
- ✅ Prices and currencies
- ✅ Dates
- ✅ URLs
- ❌ Unstructured text
- ❌ Complex layouts

### Use Headless Browser
- ✅ React/Next.js sites
- ✅ Vue/Nuxt sites
- ✅ Angular applications
- ✅ Infinite scroll content
- ✅ JavaScript-gated content
- ❌ Static HTML (waste of resources)
- ⚠️  High cost, use as last resort

## Test Category Quick Reference

### Priority 1: Static HTML (Easy Wins)
```
Category 1.1: Simple Blog Posts
  URLs:     Personal blogs, Medium, DEV.to
  Methods:  Trek ⭐⭐⭐⭐⭐, CSS ⭐⭐⭐⭐⭐
  Tests:    Title, content, metadata extraction
  Expected: >90% accuracy, <500ms

Category 1.2: News Articles
  URLs:     BBC, Guardian, Reuters
  Methods:  CSS ⭐⭐⭐⭐⭐, Trek ⭐⭐⭐⭐
  Tests:    Byline, dateline, quotes
  Expected: >90% accuracy, <500ms

Category 1.3: Documentation
  URLs:     docs.rs, MDN, React docs
  Methods:  CSS ⭐⭐⭐⭐⭐, Trek ⭐⭐⭐⭐
  Tests:    Code blocks, sections, TOC
  Expected: >85% accuracy, <800ms
```

### Priority 2: E-commerce (High Value)
```
Category 3.1: Product Pages
  URLs:     Amazon, Shopify, Etsy
  Methods:  CSS ⭐⭐⭐⭐⭐, Regex ⭐⭐⭐⭐
  Tests:    Price, SKU, specs, images
  Expected: >90% accuracy, <1s

Category 3.2: SaaS Pricing
  URLs:     GitHub, Notion, Stripe pricing
  Methods:  CSS ⭐⭐⭐⭐⭐, Regex ⭐⭐⭐⭐
  Tests:    Pricing tiers, features
  Expected: >90% accuracy, <1s
```

### Priority 3: SPAs (Headless Required)
```
Category 2.1: React/Next.js
  URLs:     Modern SaaS, e-commerce
  Methods:  Headless ⭐⭐⭐⭐⭐, Trek ⭐⭐
  Tests:    Hydrated content, dynamic data
  Expected: >90% accuracy, <3s

Category 2.2: Vue/Nuxt
  URLs:     Vue-based sites
  Methods:  Headless ⭐⭐⭐⭐⭐
  Tests:    SSR vs CSR content
  Expected: >90% accuracy, <3s
```

### Priority 4: Dynamic Content (Challenging)
```
Category 4.1: Social Media
  URLs:     Twitter, LinkedIn, Reddit
  Methods:  Headless ⭐⭐⭐⭐, Regex ⭐⭐
  Tests:    Posts, hashtags, mentions
  Expected: >80% accuracy, <3s

Category 4.2: Search Results
  URLs:     Google, DuckDuckGo, Bing
  Methods:  CSS ⭐⭐⭐⭐, Headless ⭐⭐⭐⭐
  Tests:    Result extraction, pagination
  Expected: >85% accuracy, <2s
```

### Priority 5: Edge Cases (Robustness)
```
Category 8.1: Invalid HTML
  Tests:    Parser recovery, error handling
  Expected: No crashes, graceful degradation

Category 8.2: Minimal Pages
  Tests:    Empty pages, error pages
  Expected: 100% error detection

Category 8.3: Huge Pages
  Tests:    Resource limits, timeouts
  Expected: No memory leaks, proper limits
```

## Integration Points Map

```
                    URL Request
                         ↓
            ┌────────────────────────┐
            │  Cache Check           │
            │  (check_cache)         │
            └────────┬───────────────┘
                     ↓
            ┌────────────────────────┐
            │  Content Fetch         │
            │  (fetch::get)          │
            └────────┬───────────────┘
                     ↓
            ┌────────────────────────┐
            │  Gate Analysis         │
            │  (score, decide)       │
            └────────┬───────────────┘
                     ↓
         ┌───────────┴───────────┐
         │     Decision Tree     │
         └───────────┬───────────┘
                     ↓
    ┌────────────────┼────────────────┐
    ↓                ↓                ↓
┌───────┐      ┌────────┐      ┌─────────┐
│  Raw  │      │ Probes │      │Headless │
└───┬───┘      └───┬────┘      └────┬────┘
    │              │                 │
    └──────────────┴─────────────────┘
                   ↓
        ┌──────────────────────┐
        │  StrategyManager     │
        │  (extract_content)   │
        └──────────┬───────────┘
                   ↓
    ┌──────────────┴──────────────┐
    │  Strategy Selection         │
    │  (Trek/CSS/Regex/Multi)    │
    └──────────────┬──────────────┘
                   ↓
         ┌─────────┴─────────┐
         │    Extractors     │
         ├───────────────────┤
         │ Trek    │ CSS     │
         │ Regex   │ Multi   │
         └─────────┬─────────┘
                   ↓
        ┌──────────────────────┐
        │  ProcessedContent    │
        │  (with metadata)     │
        └──────────┬───────────┘
                   ↓
        ┌──────────────────────┐
        │  Cache Storage       │
        │  (store_in_cache)    │
        └──────────────────────┘
```

## Critical Gaps Checklist

### High Priority (Blocking Production)
- [ ] Complete WASM component binding (currently returns mock data)
- [ ] Implement unified confidence scoring
- [ ] Standardize error handling across all extractors
- [ ] Fix cache key generation consistency

### Medium Priority (Quality Issues)
- [ ] Add strategy composition framework
- [ ] Implement performance monitoring dashboard
- [ ] Create smart strategy selection based on content type
- [ ] Add quality feedback loop

### Low Priority (Technical Debt)
- [ ] Refactor common utilities (title extraction, HTML parsing)
- [ ] Reduce content transformation duplication
- [ ] Unify metadata extraction approaches

## Test Execution Priority

### Week 1: Foundation Tests
1. Static HTML extraction (Categories 1.1-1.3)
2. Basic CSS selector tests
3. Trek fallback chain validation
4. Error handling tests

### Week 2: Core Functionality
5. E-commerce extraction (Categories 3.1-3.2)
6. Regex pattern extraction tests
7. Multi-strategy merge policy tests
8. Performance benchmarks (static content)

### Week 3: Advanced Features
9. SPA extraction with headless (Categories 2.1-2.3)
10. Dynamic content tests (Categories 4.1-4.2)
11. Confidence scoring validation
12. Cache consistency tests

### Week 4: Edge Cases & Polish
13. Malformed HTML tests (Category 8)
14. Internationalization tests (Category 9)
15. Anti-scraping resilience (Category 6)
16. Specialized content types (Category 10)

## Success Metrics Summary

| Category | Target Accuracy | Target Speed | Critical |
|----------|----------------|--------------|----------|
| Static HTML | >90% | <500ms | ✅ MUST PASS |
| SPAs | >90% | <3s | ⚠️  HEADLESS |
| E-commerce | >95% | <1s | ✅ HIGH VALUE |
| News | >90% | <500ms | ✅ MUST PASS |
| Documentation | >85% | <800ms | ✅ MUST PASS |
| Social Media | >80% | <3s | ⚠️  CHALLENGING |
| Edge Cases | No Crashes | N/A | ✅ CRITICAL |
| Paywalled | Metadata Only | <1s | ⚠️  LIMITED |

## Files Reference

- **Workflow Analysis**: `/workspaces/eventmesh/docs/analysis/extraction-workflow-integration.md`
- **Test Categories**: `/workspaces/eventmesh/docs/analysis/test-categorization-scheme.md`
- **Summary Report**: `/workspaces/eventmesh/docs/analysis/analyst-summary.md`
- **This Reference**: `/workspaces/eventmesh/docs/analysis/quick-reference.md`

## Memory Keys

- `hive/analysis/workflow-map` - Complete integration analysis
- `hive/analysis/test-categories` - Test categorization scheme
- Session ID: `swarm-1760176542369-15dzn86xo`

## Next Steps

### For Coder Agent
1. Read `/workspaces/eventmesh/docs/analysis/extraction-workflow-integration.md`
2. Implement tests following `/workspaces/eventmesh/docs/analysis/test-categorization-scheme.md`
3. Start with Priority 1 tests (Static HTML)
4. Address high-priority gaps (WASM binding, confidence scoring)

### For Tester Agent
1. Use test categorization as test plan
2. Execute tests in priority order
3. Validate performance expectations
4. Document failures and edge cases

### For Reviewer Agent
1. Verify integration points
2. Check for identified conflicts
3. Validate error handling
4. Review confidence scoring implementation
