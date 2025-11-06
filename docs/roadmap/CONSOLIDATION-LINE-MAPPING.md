# 📍 Roadmap Consolidation Line Mapping

**Purpose:** Detailed line-by-line mapping of consolidation opportunities
**File:** `/workspaces/eventmesh/docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md`
**Current Size:** 2,701 lines

---

## 🎨 Visual Breakdown (2,701 lines → 1,584 lines)

```
Lines 1-115:    Header & Status              [115 lines → 87 lines]   -28
Lines 116-150:  File Ops Reference           [35 lines → 35 lines]    0
Lines 151-173:  Timeline Overview            [23 lines → 23 lines]    0
Lines 174-175:  Phase 0 Header              [2 lines → 2 lines]      0

┌─ HIGH IMPACT: COMPLETED WORK ───────────────────────────────────┐
│ Lines 176-830:  Week 0-1 ✅ COMPLETE     [656 lines → 120 lines] │ -536 ★★★
│ Lines 831-1054: Week 1-2 Error System   [224 lines → 90 lines]  │ -134 ★★
│ Lines 1055-1226: Week 1.5-2 Config      [172 lines → 110 lines] │ -62 ★★
│ Lines 1227-1367: Week 2-2.5 TDD Guide   [140 lines → 65 lines]  │ -75 ★★
│ Lines 1370-1650: Spider ✅ COMPLETE     [280 lines → 100 lines] │ -180 ★★★
│ Lines 1651-1918: Trait Composition      [267 lines → 180 lines] │ -87 ★★
│ Lines 1919-1971: Facade ✅ COMPLETE     [52 lines → 40 lines]   │ -12 ★★★
└──────────────────────────────────────────────────────────────────┘
                                          SUBTOTAL: -1,086 lines

Lines 1973-2165: Python SDK              [192 lines → 120 lines]  -72 ★★
Lines 2166-2343: Events Schema           [177 lines → 100 lines]  -77 ★★
Lines 2347-2499: Testing & Launch        [152 lines → 85 lines]   -67 ★★
Lines 2500-2558: Success Metrics         [59 lines → 59 lines]    0
Lines 2559-2595: Critical Path           [37 lines → 37 lines]    0
Lines 2596-2679: v1.1 Planning           [84 lines → 44 lines]    -40 ★
Lines 2680-2701: Footer                  [22 lines → 22 lines]    0

                                          TOTAL: -1,117 lines
                                          FINAL: 1,584 lines ✅
```

**Legend:**
- ★★★ = High priority (completed work)
- ★★  = Medium priority (code examples)
- ★   = Low priority (future planning)

---

## 📋 Detailed Line-by-Line Breakdown

### **Section 1: Week 0-1 ✅ COMPLETE (Lines 176-830)**

**Target: Collapse from 656 lines to 120 lines (-536 lines)**

#### **Subsections:**

| Line Range | Component | Current | Target | Savings | Method |
|------------|-----------|---------|--------|---------|--------|
| 176-192 | Header | 17 | 10 | -7 | Simplify |
| 193-352 | **Redis Pooling** | 160 | 15 | **-145** | Summary table |
| 353-437 | **HTTP Client Factory** | 85 | 15 | **-70** | Summary table |
| 439-608 | **Retry Logic** | 170 | 20 | **-150** | Summary table |
| 609-616 | Time Utilities | 8 | 5 | -3 | Keep concise |
| 617-727 | Rate Limiting | 111 | 20 | -91 | Fold code |
| 728-794 | Simple Rate Limiting | 67 | 15 | -52 | Fold code |
| 795-830 | Feature Gates | 36 | 20 | -16 | Simplify |

#### **Consolidation Template:**

**BEFORE (Lines 193-352: Redis Pooling):**
```
160 lines:
  - Phase 1a header (5 lines)
  - Find existing implementations (10 lines)
  - Code example (60 lines)
  - TDD approach (20 lines)
  - Acceptance criteria (8 lines)
  - Phase 1b header (5 lines)
  - Verification commands (15 lines)
  - Migration commands (20 lines)
  - Verification after (10 lines)
  - Acceptance criteria (7 lines)
```

**AFTER (15 lines):**
```markdown
| **Redis Pooling** | REFACTOR + MIGRATE | ~150 | 8 | ✅ Complete |

**Achievement:** RedisPool with health checks, 10+ concurrent connections
**Files:** `crates/riptide-utils/src/redis.rs`
**Tests:** 8 passing
**Details:** See `docs/phase0/PHASE-0-COMPLETION-REPORT.md`

<details>
<summary>Implementation Details (140 lines) - Click to expand</summary>
[moved code here]
</details>
```

**Savings:** 160 - 15 = 145 lines

---

### **Section 2: Spider Decoupling ✅ COMPLETE (Lines 1370-1650)**

**Target: Collapse from 280 lines to 100 lines (-180 lines)**

#### **Subsections:**

| Line Range | Component | Current | Target | Savings |
|------------|-----------|---------|--------|---------|
| 1370-1400 | Problem Description | 31 | 5 | -26 |
| 1401-1455 | Robots Policy Toggle | 55 | 10 | -45 |
| 1456-1518 | ContentExtractor Trait | 63 | 15 | -48 |
| 1519-1548 | Result Types | 30 | 10 | -20 |
| 1549-1608 | Refactor Spider | 60 | 15 | -45 |
| 1609-1640 | Update Facades | 32 | 10 | -22 |
| 1641-1650 | Acceptance Criteria | 10 | 35 | +25 (table) |

#### **Consolidation Template:**

**AFTER (100 lines):**
```markdown
### Week 2.5-5.5: Spider Decoupling ✅ COMPLETE (2025-11-04)

**Report:** `docs/phase1/PHASE-1-SPIDER-DECOUPLING-COMPLETION-REPORT.md`
**Tests:** 88/88 passing (22 unit + 66 integration)
**Quality:** Zero clippy warnings

<details>
<summary>📊 Completion Summary - Click to expand</summary>

| Component | Lines | Tests | Status |
|-----------|-------|-------|--------|
| ContentExtractor Trait | +120 | 22 | ✅ |
| BasicExtractor | +80 | 15 | ✅ |
| NoOpExtractor | +30 | 10 | ✅ |
| Result Types | +50 | 8 | ✅ |
| Facade Updates | +100 | 33 | ✅ |

**Key Achievements:**
- Spider works without extraction (pure URL discovery)
- Modular extractor plugins (ICS, JSON-LD, LLM ready)
- ~200 lines removed from spider core
- Robots.txt toggle with ethical warnings

</details>

<details>
<summary>Code Examples - Click to expand</summary>

[ContentExtractor trait definition]
[BasicExtractor implementation]
[Result types]

</details>

**Known Issues:** 23 riptide-api errors (browser/llm features) - scheduled Week 1.5
```

---

### **Section 3: Facade Unification ✅ COMPLETE (Lines 1919-1971)**

**Target: Collapse from 52 lines to 40 lines (-12 lines)**

#### **Consolidation Template:**

**BEFORE (52 lines):**
- Header (3 lines)
- Verified line counts (5 lines)
- Code example (25 lines)
- Acceptance criteria (10 lines)
- Status footer (9 lines)

**AFTER (40 lines):**
```markdown
### Week 9: Facade Unification ✅ COMPLETE (2025-11-05)

**Report:** `docs/phase1/PHASE-1-WEEK-9-FACADE-UNIFICATION-COMPLETION-REPORT.md`
**Achievement:** CrawlFacade wraps 1,640 lines (NOT rebuilt)
**Tests:** 23/23 passing (11 integration + 12 unit)
**Modes:** Standard (PipelineOrchestrator) + Enhanced (StrategiesPipelineOrchestrator)

**Key Design:** Arc-wrapped delegation with zero code duplication

<details>
<summary>CrawlFacade Implementation - Click to expand</summary>

[code example]

</details>
```

---

## 🔧 Code Folding Strategy

### **Pattern 1: Completed Code Examples**

**Criteria for Folding:**
- ✅ Completed work (not pending)
- ✅ Code blocks > 30 lines
- ✅ Detailed implementation examples

**Template:**
```markdown
<details>
<summary><strong>Component Name (X lines)</strong> - Click to expand</summary>

```rust
// Full code here
```

**Acceptance Criteria:**
- [x] Item 1 ✅
- [x] Item 2 ✅

</details>
```

**Estimated Savings:** ~70% visual reduction (folded, not deleted)

### **Pattern 2: Completed Sections**

**Template:**
```markdown
### Section Name ✅ COMPLETE (Date)

**Report:** Link to detailed completion report
**Status:** X/X tests passing
**Achievement:** 1-2 sentence summary

<details>
<summary>📊 Completion Summary - Click to expand</summary>

| Component | Metric 1 | Metric 2 | Status |
|-----------|----------|----------|--------|
| Item 1 | Value | Value | ✅ |

**Key Achievements:**
- Bullet 1
- Bullet 2

</details>

<details>
<summary>Implementation Details - Click to expand</summary>

[Full code examples]
[Migration commands]
[Test results]

</details>
```

---

## 📊 Impact by Category

### **Category 1: Completed Work (3 sections)**

| Section | Lines | Target | Savings | Priority |
|---------|-------|--------|---------|----------|
| Week 0-1 | 656 | 120 | -536 | ★★★ HIGH |
| Spider Decoupling | 280 | 100 | -180 | ★★★ HIGH |
| Facade Unification | 52 | 40 | -12 | ★★★ HIGH |
| **TOTAL** | **988** | **260** | **-728** | |

**Method:** Summary tables + `<details>` blocks
**Risk:** LOW (completed work, safe to archive)

### **Category 2: Code Examples (6 sections)**

| Section | Lines | Savings | Priority |
|---------|-------|---------|----------|
| Error System | 224 | -134 | ★★ MEDIUM |
| Configuration | 172 | -62 | ★★ MEDIUM |
| TDD Guide | 140 | -75 | ★★ MEDIUM |
| Trait Composition | 267 | -87 | ★★ MEDIUM |
| Python SDK | 192 | -72 | ★★ MEDIUM |
| Events Schema | 177 | -77 | ★★ MEDIUM |
| **TOTAL** | **1,172** | **-507** | |

**Method:** `<details>` blocks for verbose code
**Risk:** LOW (preserved, just folded)

### **Category 3: Future Planning (1 section)**

| Section | Lines | Savings | Priority |
|---------|-------|---------|----------|
| v1.1 Planning | 84 | -40 | ★ LOW |
| **TOTAL** | **84** | **-40** | |

**Method:** Collapse details, keep summary
**Risk:** LOW (future work)

---

## ✅ Validation Checkpoints

### **After Priority 1 (HIGH IMPACT):**
```bash
# Expected: ~1,973 lines (2,701 - 728)
wc -l docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md
```

**Check:**
- [ ] Week 0-1 section collapsed to ~120 lines
- [ ] Spider section collapsed to ~100 lines
- [ ] Facade section collapsed to ~40 lines
- [ ] All completion reports linked
- [ ] All `<details>` blocks render correctly

### **After Priority 2 (MEDIUM IMPACT):**
```bash
# Expected: ~1,666 lines (1,973 - 307)
wc -l docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md
```

**Check:**
- [ ] Error system code folded
- [ ] Configuration examples folded
- [ ] TDD guide replaced with link
- [ ] Trait examples folded
- [ ] Python SDK code folded
- [ ] Events schema code folded

### **After Priority 3 (LOW IMPACT):**
```bash
# Expected: ~1,584 lines (1,666 - 82 including formatting improvements)
wc -l docs/roadmap/RIPTIDE-V1-DEFINITIVE-ROADMAP.md
```

**Check:**
- [ ] v1.1 planning collapsed
- [ ] Final line count: 1,500-1,800 ✅
- [ ] All pending items (⏳) preserved
- [ ] All critical sections intact

---

## 🎯 Success Metrics

**Quantitative:**
- Target: 1,500-1,800 lines
- Estimated: 1,584 lines ✅
- Reduction: 1,117 lines (-41%)

**Qualitative:**
- Zero information loss ✅
- Improved scannability ✅
- Faster reference lookups ✅
- All pending work visible ✅

---

**Ready for execution. See ROADMAP-CONSOLIDATION-STRATEGY.md for implementation steps.**
