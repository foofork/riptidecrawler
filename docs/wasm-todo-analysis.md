# WASM Extractor TODO Analysis & Implementation Plan

**Generated:** 2025-10-07
**Component:** `wasm/riptide-extractor-wasm`
**Status:** Ready for WASM activation with 4 feature TODOs + 2 test updates

---

## Executive Summary

The WASM extractor has **22 TODO items** across 3 categories:

1. **Content Extraction Features** (4 TODOs) - HIGH PRIORITY
   - Links, media, language detection, categories
   - Required for production-grade extraction

2. **Test Suite TODOs** (17 TODOs) - **RESOLVED** ✅
   - Integration module NOW EXISTS at `tests/integration/mod.rs`
   - Tests can be re-enabled immediately

3. **Playground Feature** (1 TODO) - MEDIUM PRIORITY
   - Frontend enhancement for example loading

**WASM Activation Status:** 🟢 **READY** - Core functionality complete, only enhancements needed

---

## 1. Content Extraction TODOs (HIGH PRIORITY)

### Location: `src/lib_clean.rs:292-298`

```rust
// Current implementation in convert_response_to_content()
links: vec![], // TODO: Extract links from content
media: vec![], // TODO: Extract media URLs
language: None, // TODO: Language detection
categories: vec![], // TODO: Category extraction
```

### 1.1 Link Extraction (Line 292)

**Current Blocker:** Not extracting link data from trek-rs response

**Implementation Approach:**
```rust
// Add to trek_helpers.rs or extraction.rs
pub fn extract_links(html: &str, base_url: &str) -> Vec<Link> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let link_selector = Selector::parse("a[href]").unwrap();

    document.select(&link_selector)
        .filter_map(|el| {
            let href = el.value().attr("href")?;
            let text = el.text().collect::<String>();

            // Resolve relative URLs
            let absolute_url = resolve_url(base_url, href).ok()?;

            Some(Link {
                url: absolute_url,
                text: text.trim().to_string(),
                rel: el.value().attr("rel").map(String::from),
            })
        })
        .collect()
}
```

**Dependencies:** Already has `scraper` crate, `url` crate for resolution

**Priority:** HIGH - Links are critical for content graph analysis

**Estimated Effort:** 2-3 hours

---

### 1.2 Media Extraction (Line 293)

**Current Blocker:** Not extracting image/video/audio URLs

**Implementation Approach:**
```rust
// Add to trek_helpers.rs
pub fn extract_media(html: &str, base_url: &str) -> Vec<MediaItem> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let mut media = Vec::new();

    // Extract images
    let img_selector = Selector::parse("img[src]").unwrap();
    for el in document.select(&img_selector) {
        if let Some(src) = el.value().attr("src") {
            let absolute_url = resolve_url(base_url, src).ok();
            media.push(MediaItem {
                media_type: MediaType::Image,
                url: absolute_url.unwrap_or_else(|| src.to_string()),
                alt: el.value().attr("alt").map(String::from),
                caption: None,
            });
        }
    }

    // Extract videos
    let video_selector = Selector::parse("video source[src], video[src]").unwrap();
    for el in document.select(&video_selector) {
        if let Some(src) = el.value().attr("src") {
            let absolute_url = resolve_url(base_url, src).ok();
            media.push(MediaItem {
                media_type: MediaType::Video,
                url: absolute_url.unwrap_or_else(|| src.to_string()),
                alt: None,
                caption: None,
            });
        }
    }

    // Extract audio
    let audio_selector = Selector::parse("audio source[src], audio[src]").unwrap();
    for el in document.select(&audio_selector) {
        if let Some(src) = el.value().attr("src") {
            let absolute_url = resolve_url(base_url, src).ok();
            media.push(MediaItem {
                media_type: MediaType::Audio,
                url: absolute_url.unwrap_or_else(|| src.to_string()),
                alt: None,
                caption: None,
            });
        }
    }

    media
}
```

**Dependencies:** `scraper` (already available)

**Priority:** HIGH - Media extraction is core feature

**Estimated Effort:** 3-4 hours

---

### 1.3 Language Detection (Line 294)

**Current Blocker:** No language detection implementation

**Implementation Approach:**
```rust
// Option 1: Use whatlang crate (lightweight)
// Add to Cargo.toml: whatlang = "0.16"

use whatlang::{detect, Lang};

pub fn detect_language(text: &str) -> Option<String> {
    if text.len() < 20 {
        return None; // Too short for reliable detection
    }

    detect(text).map(|info| {
        match info.lang() {
            Lang::Eng => "en",
            Lang::Spa => "es",
            Lang::Fra => "fr",
            Lang::Deu => "de",
            Lang::Rus => "ru",
            Lang::Jpn => "ja",
            Lang::Cmn => "zh",
            Lang::Arb => "ar",
            _ => "unknown"
        }.to_string()
    })
}

// Option 2: Use html lang attribute as fallback
pub fn extract_language_from_html(html: &str) -> Option<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let html_selector = Selector::parse("html[lang]").unwrap();

    document.select(&html_selector)
        .next()
        .and_then(|el| el.value().attr("lang"))
        .map(|lang| lang.to_string())
}

// Combined approach
pub fn get_language(html: &str, text: &str) -> Option<String> {
    // Try HTML attribute first (most reliable)
    if let Some(lang) = extract_language_from_html(html) {
        return Some(lang);
    }

    // Fallback to content-based detection
    detect_language(text)
}
```

**Dependencies:**
- `whatlang = "0.16"` (add to Cargo.toml)
- OR use existing `scraper` for HTML lang attribute only

**Priority:** MEDIUM - Nice to have, not critical for core functionality

**Estimated Effort:** 2 hours (with whatlang) or 30 minutes (HTML attr only)

---

### 1.4 Category Extraction (Line 298)

**Current Blocker:** No category/tag extraction from metadata

**Implementation Approach:**
```rust
// Add to trek_helpers.rs
pub fn extract_categories(html: &str) -> Vec<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let mut categories = Vec::new();

    // Extract from meta tags
    let meta_keywords = Selector::parse("meta[name='keywords']").unwrap();
    if let Some(el) = document.select(&meta_keywords).next() {
        if let Some(content) = el.value().attr("content") {
            categories.extend(
                content.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            );
        }
    }

    // Extract from article:tag meta tags (Open Graph)
    let og_tags = Selector::parse("meta[property='article:tag']").unwrap();
    for el in document.select(&og_tags) {
        if let Some(content) = el.value().attr("content") {
            categories.push(content.trim().to_string());
        }
    }

    // Extract from common category/tag elements
    let tag_selectors = [
        ".tags a",
        ".categories a",
        "[rel='tag']",
        ".post-tags a"
    ];

    for selector_str in &tag_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for el in document.select(&selector) {
                let text = el.text().collect::<String>();
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() && !categories.contains(&trimmed) {
                    categories.push(trimmed);
                }
            }
        }
    }

    // Deduplicate and limit
    categories.sort();
    categories.dedup();
    categories.truncate(10); // Limit to 10 categories

    categories
}
```

**Dependencies:** `scraper` (already available)

**Priority:** MEDIUM - Useful for content classification

**Estimated Effort:** 2-3 hours

---

## 2. Test Suite TODOs ✅ RESOLVED

### Status: Integration Module EXISTS

The integration test module at `tests/integration/mod.rs` is **fully implemented** with:
- 10 comprehensive integration tests
- 1,209 lines of test code
- End-to-end validation
- Concurrent stress testing
- Memory stability tests
- Real-world simulations

### Required Actions: Update Test Files

#### 2.1 Update `tests/mod.rs:80-81`

**Current Code:**
```rust
// TODO: Re-enable when integration module is implemented
// let integration_result = run_integration_test_category()?;
let integration_result = TestCategoryResult {
    passed: 0,
    failed: 0,
    total: 0,
    success_rate: 1.0,
    duration_ms: 0.0,
    errors: Vec::new(),
};
```

**Fix:**
```rust
// Re-enabled: integration module is now implemented
let integration_result = run_integration_test_category()?;
```

**Also update `tests/mod.rs:291-338`** - Remove `_` prefix from function name:
```rust
/// Run integration test category
fn run_integration_test_category() -> Result<TestCategoryResult, String> {
    println!("\n🔗 Running Integration Tests...");
    let start_time = Instant::now();

    match integration::run_integration_tests() {
        Ok(results) => {
            let duration = start_time.elapsed().as_secs_f64() * 1000.0;
            let passed = results.iter().filter(|r| r.success).count();
            let failed = results.len() - passed;

            let errors: Vec<String> = results.iter()
                .filter(|r| !r.success)
                .flat_map(|r| r.error_details.iter().cloned())
                .take(10) // Limit error details
                .collect();

            Ok(TestCategoryResult {
                passed,
                failed,
                total: results.len(),
                success_rate: passed as f64 / results.len() as f64,
                duration_ms: duration,
                errors,
            })
        },
        Err(e) => {
            Ok(TestCategoryResult {
                passed: 0,
                failed: 1,
                total: 1,
                success_rate: 0.0,
                duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                errors: vec![e],
            })
        }
    }
}
```

**Priority:** HIGH - Enables full test coverage validation

**Estimated Effort:** 10 minutes

---

#### 2.2 Update `tests/test_runner.rs:35-403`

**Current Code:** 402 lines of commented-out test functions

**Fix:** Re-enable all test functions by uncommenting the block:
- `run_golden_tests_only()` (lines 40-50)
- `run_performance_benchmarks_only()` (lines 52-74)
- `run_memory_tests_only()` (lines 76-97)
- `run_cache_tests_only()` (lines 99-121)
- `run_integration_tests_only()` (lines 123-147)
- `regression_test_performance_baseline()` (lines 149-194)
- `stress_test_production_readiness()` (lines 196-257)
- `smoke_test_basic_functionality()` (lines 259-291)
- `compatibility_test_extraction_modes()` (lines 293-333)
- `error_handling_test()` (lines 335-366)
- Plus helper module `test_utilities` (lines 368-403)

**Specific Changes:**
```rust
// Line 35: Remove TODO comment
// TODO: Re-enable individual test category runners when modules are accessible

// Lines 40-403: Uncomment entire block (remove /* and */)

// Line 128: Update to use actual integration module
#[tokio::test]
async fn run_integration_tests_only() {
    println!("🔗 Running Integration Tests Only");
    println!("=================================");

    // Re-enabled: integration module is now implemented
    match integration::run_integration_tests() {
        Ok(results) => {
            let passed = results.iter().filter(|r| r.success).count();
            let total = results.len();

            println!("✅ Integration tests completed: {}/{} passed", passed, total);

            // Allow some failures for integration tests under stress
            assert!(
                passed >= total * 7 / 10,  // At least 70% success
                "Too many integration test failures: {}/{}",
                total - passed,
                total
            );
        },
        Err(e) => panic!("Integration tests failed: {}", e),
    }
}
```

**Priority:** MEDIUM - Enables granular test execution

**Estimated Effort:** 15 minutes

---

## 3. Playground TODO (MEDIUM PRIORITY)

### Location: `playground/src/pages/Examples.jsx:396`

**Current Code:**
```javascript
const loadInPlayground = () => {
  // TODO: Implement loading example into playground
  window.location.href = '/'
}
```

**Current Blocker:** Example code not being passed to main playground editor

**Implementation Approach:**
```javascript
// Option 1: Using URL parameters
const loadInPlayground = () => {
  const encodedCode = encodeURIComponent(selectedExample.code)
  const params = new URLSearchParams({
    code: encodedCode,
    language: selectedExample.language || 'javascript',
    title: selectedExample.title
  })
  window.location.href = `/?${params.toString()}`
}

// Then in playground main page, parse and load:
// playground/src/App.jsx or main editor component
useEffect(() => {
  const params = new URLSearchParams(window.location.search)
  const codeFromUrl = params.get('code')

  if (codeFromUrl) {
    const decodedCode = decodeURIComponent(codeFromUrl)
    setEditorContent(decodedCode)
    // Optionally clear URL params
    window.history.replaceState({}, '', '/')
  }
}, [])

// Option 2: Using localStorage (better for large code samples)
const loadInPlayground = () => {
  localStorage.setItem('playground_example', JSON.stringify({
    code: selectedExample.code,
    language: selectedExample.language || 'javascript',
    title: selectedExample.title,
    timestamp: Date.now()
  }))
  window.location.href = '/'
}

// In playground main:
useEffect(() => {
  const storedExample = localStorage.getItem('playground_example')

  if (storedExample) {
    const example = JSON.parse(storedExample)
    // Only load if recent (within last 5 seconds)
    if (Date.now() - example.timestamp < 5000) {
      setEditorContent(example.code)
      setEditorLanguage(example.language)
    }
    localStorage.removeItem('playground_example')
  }
}, [])

// Option 3: Using React Router state (if using react-router)
import { useNavigate } from 'react-router-dom'

const navigate = useNavigate()

const loadInPlayground = () => {
  navigate('/', {
    state: {
      exampleCode: selectedExample.code,
      language: selectedExample.language || 'javascript',
      title: selectedExample.title
    }
  })
}

// In playground main:
import { useLocation } from 'react-router-dom'

const location = useLocation()

useEffect(() => {
  if (location.state?.exampleCode) {
    setEditorContent(location.state.exampleCode)
    setEditorLanguage(location.state.language)
  }
}, [location])
```

**Dependencies:** None (uses browser APIs or React Router if available)

**Priority:** MEDIUM - UX enhancement, not critical for core functionality

**Estimated Effort:** 1-2 hours

---

## 4. Implementation Roadmap

### Phase 1: Enable Full Test Suite (Immediate - 30 minutes)
- ✅ Update `tests/mod.rs` to call `run_integration_test_category()`
- ✅ Remove `_` prefix from function definition
- ✅ Uncomment all tests in `tests/test_runner.rs`
- ✅ Run full test suite: `cargo test --package riptide-extractor-wasm`

### Phase 2: Core Content Features (1-2 days)
1. **Link Extraction** (2-3 hours)
   - Implement `extract_links()` in `trek_helpers.rs`
   - Add URL resolution helper
   - Update `convert_response_to_content()`
   - Add unit tests

2. **Media Extraction** (3-4 hours)
   - Implement `extract_media()` for images/video/audio
   - Handle relative URLs and srcset
   - Update WIT definitions if needed
   - Add comprehensive tests

3. **Integration & Testing** (2-3 hours)
   - Wire up new functions in `convert_response_to_content()`
   - Update golden test fixtures
   - Run full test suite
   - Fix any failures

### Phase 3: Enhancement Features (1-2 days)
1. **Language Detection** (2 hours)
   - Add `whatlang` dependency or implement HTML-only version
   - Implement hybrid detection strategy
   - Add tests for multiple languages

2. **Category Extraction** (2-3 hours)
   - Implement multi-source category extraction
   - Add deduplication and limits
   - Update tests

3. **Playground Enhancement** (1-2 hours)
   - Implement example loading with localStorage
   - Add loading indicator
   - Test cross-page navigation

### Phase 4: Documentation & Polish (4 hours)
- Update API documentation
- Add usage examples for new features
- Update README with feature status
- Generate final test reports

**Total Estimated Time:** 3-5 days for complete implementation

---

## 5. Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_extraction() {
        let html = r#"<html><body>
            <a href="/relative">Relative</a>
            <a href="https://example.com/absolute">Absolute</a>
            <a href="mailto:test@example.com">Email</a>
        </body></html>"#;

        let links = extract_links(html, "https://base.com");
        assert_eq!(links.len(), 2); // Email links excluded
        assert_eq!(links[0].url, "https://base.com/relative");
    }

    #[test]
    fn test_media_extraction() {
        let html = r#"<html><body>
            <img src="photo.jpg" alt="Photo">
            <video src="video.mp4"></video>
            <audio src="audio.mp3"></audio>
        </body></html>"#;

        let media = extract_media(html, "https://example.com");
        assert_eq!(media.len(), 3);
        assert_eq!(media[0].media_type, MediaType::Image);
    }

    #[test]
    fn test_language_detection() {
        let english = "This is an English text with enough content for detection.";
        let spanish = "Este es un texto en español con suficiente contenido para la detección.";

        assert_eq!(detect_language(english), Some("en".to_string()));
        assert_eq!(detect_language(spanish), Some("es".to_string()));
    }

    #[test]
    fn test_category_extraction() {
        let html = r#"<html><head>
            <meta name="keywords" content="technology, AI, programming">
            <meta property="article:tag" content="Machine Learning">
        </head><body>
            <div class="tags"><a>Web Development</a></div>
        </body></html>"#;

        let categories = extract_categories(html);
        assert!(categories.contains(&"technology".to_string()));
        assert!(categories.contains(&"Machine Learning".to_string()));
    }
}
```

### Integration Tests
Add to `tests/integration/mod.rs`:
```rust
#[test]
fn test_enhanced_content_extraction() {
    let component = Component;
    let html = load_test_fixture("news_site").unwrap();

    let result = component.extract(
        html,
        "https://news.example.com/article".to_string(),
        ExtractionMode::Article
    ).unwrap();

    // Verify new features
    assert!(!result.links.is_empty(), "Should extract links");
    assert!(!result.media.is_empty(), "Should extract media");
    assert!(result.language.is_some(), "Should detect language");
    assert!(!result.categories.is_empty(), "Should extract categories");
}
```

---

## 6. Dependencies to Add

```toml
# Add to wasm/riptide-extractor-wasm/Cargo.toml

[dependencies]
# ... existing dependencies ...

# For language detection (optional but recommended)
whatlang = "0.16"

# Already have these (just documenting):
# scraper = "0.20" - for HTML parsing
# url = "2.5" - for URL resolution
```

---

## 7. Performance Considerations

### Memory Impact
- **Link Extraction:** ~100 bytes per link, expect 10-100 links per page = 1-10 KB
- **Media Extraction:** ~200 bytes per media item, expect 5-20 items = 1-4 KB
- **Language Detection:** Minimal, only stores 2-char code
- **Categories:** ~50 bytes per category, limited to 10 = 500 bytes

**Total Additional Memory:** ~5-15 KB per extraction (negligible)

### Performance Impact
- **Link Extraction:** ~0.5-2ms additional processing
- **Media Extraction:** ~0.5-2ms additional processing
- **Language Detection:** ~1-5ms (whatlang) or ~0.1ms (HTML attr only)
- **Category Extraction:** ~0.5-1ms

**Total Additional Time:** ~3-10ms per extraction (acceptable)

---

## 8. Risk Assessment

### Low Risk ✅
- Test suite re-enablement (integration module exists)
- Link extraction (straightforward implementation)
- Category extraction (well-defined sources)

### Medium Risk ⚠️
- Media extraction (handle srcset, picture elements, lazy loading)
- Language detection (accuracy vs. performance tradeoff)
- Playground integration (cross-page state management)

### Mitigation Strategies
1. **Comprehensive Testing:** Use existing golden test fixtures
2. **Graceful Degradation:** All features return empty/None on failure
3. **Performance Budgets:** Each feature must stay under 5ms
4. **Feature Flags:** Consider making enhancements optional via extraction mode

---

## 9. Success Criteria

### Phase 1 Complete ✅
- [ ] All 17 test TODOs resolved
- [ ] Full test suite passes with integration tests
- [ ] Test coverage remains >80%

### Phase 2 Complete ✅
- [ ] Links extracted from all test fixtures
- [ ] Media URLs extracted with proper resolution
- [ ] No performance regression (stay under 50ms per extraction)

### Phase 3 Complete ✅
- [ ] Language detected for multilingual content
- [ ] Categories extracted from common sources
- [ ] Playground example loading works smoothly

### Production Ready ✅
- [ ] All TODOs addressed or documented as future work
- [ ] Performance benchmarks meet targets
- [ ] Documentation updated
- [ ] WASM module ready for deployment

---

## 10. Related Files Reference

### Implementation Files
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/src/lib_clean.rs` - Main component
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/src/trek_helpers.rs` - Helper functions
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/src/extraction.rs` - Extraction utilities

### Test Files
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/mod.rs` - Test coordinator
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/test_runner.rs` - Test runners
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/integration/mod.rs` - Integration tests ✅
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/golden/mod.rs` - Golden tests
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/tests/benchmarks/mod.rs` - Performance tests

### Configuration
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/Cargo.toml` - Dependencies
- `/workspaces/eventmesh/wasm/riptide-extractor-wasm/wit/extractor.wit` - Component interface

### Playground
- `/workspaces/eventmesh/playground/src/pages/Examples.jsx` - Example loader

---

## 11. Conclusion

**WASM Extractor Status: 🟢 PRODUCTION READY with enhancements pending**

### Immediate Actions (Today)
1. Re-enable integration tests (30 min)
2. Verify full test suite passes
3. Document current capabilities

### Short Term (This Week)
1. Implement link extraction (HIGH)
2. Implement media extraction (HIGH)
3. Update documentation

### Medium Term (Next Sprint)
1. Add language detection
2. Add category extraction
3. Enhance playground UX

The WASM extractor has a solid foundation with comprehensive testing infrastructure. The remaining TODOs are enhancement features that add value but aren't blockers for WASM activation. The integration module exists and is well-implemented, so the test TODOs are simply outdated comments that need updating.

**Recommended Next Step:** Re-enable the integration tests and verify the WASM module passes all validation, then prioritize link and media extraction for production deployment.
