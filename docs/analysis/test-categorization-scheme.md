# Test Categorization Scheme for Extraction Methods

## Overview
This document defines comprehensive test categories for validating extraction methods across different content types, complexity levels, and failure modes.

## Test Dimension Matrix

### Dimension 1: Content Type
Different types of web content require specialized extraction approaches.

### Dimension 2: Complexity Level
From simple static HTML to complex JavaScript-rendered SPAs.

### Dimension 3: Extraction Challenge
Specific difficulties that test extraction robustness.

### Dimension 4: Expected Method Performance
Which extraction methods should excel for each category.

## Test Categories

### Category 1: Static HTML Sites

#### 1.1 Simple Blog Posts
**Characteristics:**
- Clean HTML structure
- Standard semantic tags (article, header, main)
- Minimal JavaScript
- Clear content boundaries

**Test URLs:**
- Personal blogs with article tags
- Medium articles (static fallback)
- DEV.to posts
- Hashnode articles
- Ghost blog posts

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐⭐⭐⭐ (95% confidence)
- **CSS Extraction**: ⭐⭐⭐⭐⭐ (90% confidence)
- **Regex Extraction**: ⭐⭐⭐ (60% confidence)
- **Headless**: ⭐⭐⭐ (unnecessary overhead)

**Test Scenarios:**
- Title extraction from `<title>`, `<h1>`, `og:title`
- Content extraction from `<article>`, `.post-content`
- Metadata extraction (author, date, tags)
- Image URLs and alt text extraction
- Internal and external link extraction

**Edge Cases:**
- Multiple h1 tags
- Nested article elements
- Comments sections (should be filtered)
- Related posts (should be separated)

#### 1.2 News Articles
**Characteristics:**
- Structured journalism format
- Bylines and datelines
- Quote extraction needed
- Category/section tags
- Often has paywalls or ads

**Test URLs:**
- BBC News articles
- The Guardian articles
- Reuters news items
- Local news sites
- Tech news (TechCrunch, Ars Technica)

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐⭐⭐ (85% confidence)
- **CSS Extraction**: ⭐⭐⭐⭐⭐ (95% confidence, with news selectors)
- **Regex Extraction**: ⭐⭐⭐⭐ (80% confidence for quotes, dates)
- **Headless**: ⭐⭐⭐ (for paywall bypass)

**Test Scenarios:**
- Byline extraction (author names)
- Dateline extraction (location + date)
- Pull quotes and blockquotes
- Image captions
- Article section/category
- Related articles vs main content

**Edge Cases:**
- Multiple author bylines
- Updated timestamps vs published dates
- Embedded tweets/social media
- Live updates (breaking news format)
- Paywalled content

#### 1.3 Documentation Sites
**Characteristics:**
- Hierarchical structure
- Code blocks and syntax highlighting
- Navigation breadcrumbs
- Version-specific content
- API references

**Test URLs:**
- Rust documentation (docs.rs)
- MDN Web Docs
- React documentation
- Python docs
- AWS documentation

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐⭐⭐ (85% confidence)
- **CSS Extraction**: ⭐⭐⭐⭐⭐ (95% confidence, with doc selectors)
- **Regex Extraction**: ⭐⭐⭐ (65% confidence)
- **Headless**: ⭐⭐ (unnecessary)

**Test Scenarios:**
- Code block extraction with language tags
- Inline code vs block code differentiation
- Nested section extraction
- Table of contents generation
- Parameter/argument lists
- Return type information

**Edge Cases:**
- Tabbed code examples (multiple languages)
- Interactive examples
- Version switchers
- Deprecated API markers

### Category 2: JavaScript-Heavy SPAs

#### 2.1 React/Next.js Sites
**Characteristics:**
- Client-side rendering
- Hydration markers
- Data in `__NEXT_DATA__` script
- Minimal initial HTML
- High script-to-content ratio

**Test URLs:**
- Next.js showcase sites
- Modern e-commerce (Vercel commerce)
- SaaS product pages
- React-based blogs

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐ (40% confidence, pre-render)
- **CSS Extraction**: ⭐⭐ (45% confidence, server-rendered)
- **Regex Extraction**: ⭐ (20% confidence)
- **Headless**: ⭐⭐⭐⭐⭐ (95% confidence)

**Test Scenarios:**
- Initial HTML extraction (SSR content)
- Hydrated content extraction (requires JS)
- JSON-LD structured data
- Meta tags from React Helmet
- Dynamic route handling

**Edge Cases:**
- Infinite scroll (pagination)
- Lazy-loaded images
- Skeleton screens
- Loading states
- Client-only content

#### 2.2 Vue.js/Nuxt Sites
**Characteristics:**
- Similar to React but different markers
- `data-server-rendered` attribute
- Vue-specific directives
- Potentially SSR or CSR

**Test URLs:**
- Nuxt.js examples
- Vue-powered documentation
- Vue e-commerce implementations

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐ (45% confidence)
- **CSS Extraction**: ⭐⭐⭐ (60% confidence)
- **Regex Extraction**: ⭐⭐ (30% confidence)
- **Headless**: ⭐⭐⭐⭐⭐ (95% confidence)

#### 2.3 Angular Sites
**Characteristics:**
- App-root component
- TypeScript-generated content
- Often enterprise applications
- Heavy framework overhead

**Test URLs:**
- Angular Material documentation
- Enterprise dashboards
- Angular showcase apps

**Expected Performance:**
- **Trek (WASM)**: ⭐ (30% confidence)
- **CSS Extraction**: ⭐⭐ (40% confidence)
- **Regex Extraction**: ⭐ (25% confidence)
- **Headless**: ⭐⭐⭐⭐⭐ (90% confidence)

### Category 3: E-commerce Product Pages

#### 3.1 Standard Product Pages
**Characteristics:**
- Product title and description
- Price and availability
- Multiple images/gallery
- Specifications table
- Reviews section
- Related products

**Test URLs:**
- Amazon product pages
- Shopify store products
- WooCommerce products
- Etsy listings
- eBay auctions

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐⭐ (70% confidence)
- **CSS Extraction**: ⭐⭐⭐⭐⭐ (95% confidence, with product selectors)
- **Regex Extraction**: ⭐⭐⭐⭐ (85% confidence for prices, SKUs)
- **Headless**: ⭐⭐⭐⭐ (80% confidence, for dynamic pricing)

**Test Scenarios:**
- Product title extraction
- Price extraction (currency normalization)
- SKU/model number extraction
- Availability status
- Image gallery URLs
- Specification table parsing
- Rating/review scores
- Brand information

**Edge Cases:**
- Dynamic pricing (sales, discounts)
- Out of stock vs in stock
- Multiple variants (size, color)
- Subscription pricing models
- Bundle deals
- Limited time offers

#### 3.2 Service/SaaS Pricing Pages
**Characteristics:**
- Pricing tiers
- Feature comparison tables
- Monthly vs annual pricing
- Call-to-action buttons
- Trial information

**Test URLs:**
- GitHub pricing
- Notion pricing
- Stripe pricing
- AWS pricing calculator
- SaaS company pricing pages

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐⭐ (65% confidence)
- **CSS Extraction**: ⭐⭐⭐⭐⭐ (90% confidence)
- **Regex Extraction**: ⭐⭐⭐⭐ (80% confidence for prices)
- **Headless**: ⭐⭐⭐ (60% confidence)

**Test Scenarios:**
- Pricing tier extraction
- Feature list per tier
- Price comparison (monthly vs annual)
- Currency handling
- Feature matrix table extraction

### Category 4: Dynamic Content Sites

#### 4.1 Social Media Feeds
**Characteristics:**
- Infinite scroll
- User-generated content
- Real-time updates
- Authentication required
- Heavy JavaScript

**Test URLs:**
- Twitter/X posts (public)
- LinkedIn articles (public)
- Reddit threads
- Facebook posts (public pages)

**Expected Performance:**
- **Trek (WASM)**: ⭐ (20% confidence)
- **CSS Extraction**: ⭐⭐ (35% confidence)
- **Regex Extraction**: ⭐⭐ (40% confidence for @mentions, #hashtags)
- **Headless**: ⭐⭐⭐⭐ (85% confidence)

**Test Scenarios:**
- Post content extraction
- Username/handle extraction
- Hashtag extraction
- Mention extraction
- Timestamp extraction
- Like/share counts
- Comment threads

**Edge Cases:**
- Deleted posts
- Private content
- Rate limiting
- Infinite scroll pagination
- Embedded media

#### 4.2 Search Results Pages
**Characteristics:**
- Structured result lists
- Pagination or infinite scroll
- Ads mixed with results
- Filters and facets
- Dynamic ranking

**Test URLs:**
- Google search results
- DuckDuckGo results
- Bing search results
- Amazon search results
- E-commerce site search

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐ (45% confidence)
- **CSS Extraction**: ⭐⭐⭐⭐ (85% confidence)
- **Regex Extraction**: ⭐⭐⭐ (60% confidence)
- **Headless**: ⭐⭐⭐⭐ (80% confidence)

### Category 5: Authentication & Paywalled Sites

#### 5.1 Soft Paywalls
**Characteristics:**
- Limited preview content
- "Continue reading" prompts
- Metered access (X articles/month)
- Blur effects on text
- Login prompts

**Test URLs:**
- Medium (metered paywall)
- NYTimes (soft paywall)
- Financial Times
- Local news sites

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐ (40% confidence, preview only)
- **CSS Extraction**: ⭐⭐⭐ (60% confidence, preview only)
- **Regex Extraction**: ⭐⭐ (35% confidence)
- **Headless**: ⭐⭐⭐⭐ (75% confidence, with techniques)

**Test Scenarios:**
- Preview content extraction
- Detecting paywall presence
- Extracting available metadata
- Identifying subscription requirements

#### 5.2 Hard Paywalls
**Characteristics:**
- No content without login
- Redirect to login page
- Authentication required
- Session-based access

**Test URLs:**
- WSJ articles (hard paywall)
- Premium research papers
- Member-only forums
- Subscription services

**Expected Performance:**
- **Trek (WASM)**: ⭐ (10% confidence, metadata only)
- **CSS Extraction**: ⭐ (15% confidence, metadata only)
- **Regex Extraction**: ⭐ (10% confidence)
- **Headless**: ⭐ (20% confidence, without credentials)

### Category 6: Anti-Scraping Measures

#### 6.1 Rate Limiting
**Characteristics:**
- 429 status codes
- IP-based blocking
- Request frequency limits
- Captcha challenges

**Test Scenarios:**
- Retry logic testing
- Backoff strategy validation
- Error handling for 429s
- Cache utilization

#### 6.2 Bot Detection
**Characteristics:**
- User-agent checking
- JavaScript challenges
- Cloudflare protection
- Behavioral analysis

**Test URLs:**
- Cloudflare-protected sites
- Sites with Distil Networks
- reCAPTCHA v3 sites

**Expected Performance:**
- **Trek (WASM)**: ⭐ (blocked)
- **CSS Extraction**: ⭐ (blocked)
- **Regex Extraction**: ⭐ (blocked)
- **Headless**: ⭐⭐⭐ (65% confidence, with proper headers)

#### 6.3 Dynamic Obfuscation
**Characteristics:**
- Randomized class names
- Email/phone obfuscation
- Content encoding
- Dynamic CSS generation

**Test Scenarios:**
- Obfuscated email extraction
- Encoded phone number extraction
- Dynamic selector handling

### Category 7: Rich Media Content

#### 7.1 Video Platforms
**Characteristics:**
- Embedded video players
- Transcript availability
- Metadata-rich
- Thumbnail extraction

**Test URLs:**
- YouTube video pages
- Vimeo video pages
- TikTok (web version)
- Twitch streams

**Expected Performance:**
- **Trek (WASM)**: ⭐⭐ (metadata only)
- **CSS Extraction**: ⭐⭐⭐⭐ (metadata + transcripts)
- **Regex Extraction**: ⭐⭐⭐ (URLs, IDs)
- **Headless**: ⭐⭐⭐⭐ (dynamic content)

**Test Scenarios:**
- Video title and description
- Channel/author information
- View count and engagement metrics
- Transcript extraction (if available)
- Comment extraction
- Thumbnail URLs

#### 7.2 Image Galleries
**Characteristics:**
- Multiple images
- Lightbox/modal viewers
- Lazy loading
- Pagination

**Test URLs:**
- Flickr albums
- Google Photos (public albums)
- Photography portfolio sites
- Product image galleries

**Test Scenarios:**
- All image URL extraction
- Alt text preservation
- Caption extraction
- EXIF metadata (if available)
- Gallery structure preservation

### Category 8: Malformed/Edge Case HTML

#### 8.1 Invalid HTML
**Characteristics:**
- Unclosed tags
- Malformed nesting
- Invalid attributes
- Mixed encodings

**Test Scenarios:**
- Parser recovery
- Best-effort extraction
- Error reporting
- Confidence scoring degradation

#### 8.2 Minimal HTML
**Characteristics:**
- Nearly empty pages
- Redirect pages
- Error pages (404, 500)
- Maintenance pages

**Test Scenarios:**
- Graceful failure
- Error detection
- Status code handling
- Minimal content extraction

#### 8.3 Extremely Large Pages
**Characteristics:**
- Multi-megabyte HTML
- Thousands of DOM nodes
- Performance stress test
- Memory constraints

**Test Scenarios:**
- Resource limit enforcement
- Timeout handling
- Partial extraction strategies
- Performance degradation curves

### Category 9: Internationalization

#### 9.1 Non-English Content
**Languages to Test:**
- Spanish (Romance language)
- Mandarin Chinese (logographic)
- Arabic (RTL script)
- Japanese (mixed scripts)
- Russian (Cyrillic)
- Hindi (Devanagari)

**Test Scenarios:**
- UTF-8 encoding handling
- Character preservation
- RTL text handling
- Mixed-script content
- Language detection

#### 9.2 Multi-Language Pages
**Characteristics:**
- Language switchers
- Localized content
- Translation metadata
- Region-specific content

**Test Scenarios:**
- Language selection detection
- Content extraction per language
- Metadata language tags
- Translation completeness

### Category 10: Specialized Content Types

#### 10.1 Academic Papers
**Characteristics:**
- Abstract, sections, references
- Author affiliations
- Citation metadata
- DOI/ISBN identifiers

**Test URLs:**
- arXiv papers
- PubMed articles
- IEEE Xplore papers
- Open access journals

**Test Scenarios:**
- Abstract extraction
- Author and affiliation extraction
- Citation extraction
- Section hierarchy preservation
- Figure and table extraction
- Bibliography parsing

#### 10.2 Recipe Sites
**Characteristics:**
- Ingredients lists
- Step-by-step instructions
- Cooking times
- Nutrition information
- Reviews and ratings

**Test URLs:**
- AllRecipes
- Food Network recipes
- Blog-based recipes
- Recipe aggregators

**Test Scenarios:**
- Ingredient list extraction (with quantities)
- Instruction step extraction (ordered)
- Prep/cook time extraction
- Servings information
- Nutrition facts table
- Filtering ads and stories

#### 10.3 Job Listings
**Characteristics:**
- Job title and description
- Requirements and qualifications
- Salary range
- Location information
- Application instructions

**Test URLs:**
- LinkedIn jobs
- Indeed listings
- Company career pages
- Job board aggregators

**Test Scenarios:**
- Structured field extraction
- Requirement list parsing
- Salary range extraction
- Location normalization
- Contact information extraction

## Test Execution Matrix

### Performance Comparison Tests
For each category, run all extraction methods and compare:

| Metric | Target | Critical |
|--------|--------|----------|
| Accuracy | >90% | >70% |
| Speed | <1s | <5s |
| Confidence | >0.8 | >0.5 |
| Memory | <100MB | <500MB |
| Cache Hit Rate | >80% | >50% |

### Failure Mode Tests

1. **Network Failures**
   - Timeout scenarios
   - Connection drops
   - Partial responses
   - DNS failures

2. **Content Errors**
   - Invalid HTML
   - Missing expected elements
   - Empty responses
   - Redirect loops

3. **Resource Exhaustion**
   - Memory limits exceeded
   - CPU timeout
   - Disk space issues
   - Cache overflow

4. **Integration Failures**
   - WASM component unavailable
   - Headless service down
   - Cache service unreachable
   - PDF processor failure

## Test Data Generation

### Synthetic Test Pages
Create controlled test pages for:
- Minimal HTML (various valid structures)
- Maximal HTML (stress testing)
- Edge cases (malformed, unusual structures)
- Performance benchmarks

### Real-World URL Sets
Curate collections of:
- 100 blog posts (various platforms)
- 50 news articles (various sources)
- 50 product pages (e-commerce variety)
- 30 documentation pages
- 30 SPA applications
- 20 social media pages
- 20 paywalled sites

### Ground Truth Dataset
For each test URL:
- Manual extraction of expected content
- Confidence score expectations
- Performance baseline
- Known failure modes
- Edge case documentation

## Success Criteria by Category

### Static HTML (Categories 1.1-1.3)
- Trek: >90% accuracy, <500ms
- CSS: >95% accuracy, <300ms
- Regex: >60% accuracy, <200ms

### SPA Content (Categories 2.1-2.3)
- Headless: >90% accuracy, <3s
- Trek: >40% accuracy (SSR only)
- CSS: >50% accuracy (SSR only)

### E-commerce (Categories 3.1-3.2)
- CSS with product selectors: >90% accuracy
- Regex for structured data: >85% accuracy
- Combined approach: >95% accuracy

### Dynamic Content (Categories 4.1-4.2)
- Headless: >80% accuracy
- CSS: >70% accuracy (static portions)
- Combined: >85% accuracy

### Protected Content (Categories 5.1-6.3)
- Detection rate: >95%
- Graceful degradation: 100%
- Metadata extraction: >70%

### Rich Media (Categories 7.1-7.2)
- Metadata extraction: >90%
- Transcript/caption: >80%
- URL extraction: >95%

### Edge Cases (Categories 8.1-8.3)
- Parser resilience: >95%
- Error reporting: 100%
- No crashes: 100%

### Internationalization (Category 9)
- Character preservation: >99%
- Language detection: >90%
- RTL support: >95%

### Specialized (Category 10)
- Domain-specific accuracy: >85%
- Structured data extraction: >90%
- Format preservation: >80%
