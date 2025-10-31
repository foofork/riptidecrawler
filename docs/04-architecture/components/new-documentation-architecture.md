# RipTide Documentation Architecture Design

**Version**: 2.0
**Date**: 2025-10-11
**Status**: Architecture Design Phase
**Analyst**: Hive Mind Swarm Agent

---

## Executive Summary

This document defines a modern, wiki-style documentation architecture for RipTide that follows best practices from leading projects like Retell AI and Crawl4AI. The new structure emphasizes:

- **Clear hierarchy** with logical navigation paths
- **User-centric organization** (by role and use case)
- **Discoverability** through consistent naming and structure
- **Maintainability** with DRY principles and single-source-of-truth
- **Scalability** to accommodate growth

---

## Current State Analysis

### Existing Documentation Structure

```
docs/
├── README.md (Overview)
├── DOCUMENTATION_SUMMARY_V1.md
├── API_TOOLING_QUICKSTART.md
├── CLI_COMPLETION_REPORT.md
├── PRODUCTION_READY_CHECKLIST.md
├── WASM_TDD_EXECUTION_REPORT.md
├── WASM_TDD_SUMMARY.md
├── WASM_BINDING_COMPLETION_GUIDE.md
├── cache-key-fix-summary.md
├── confidence-scoring-api.md
├── gap-fixes-summary.md
├── gap-fixes-review.md
├── strategy-composition.md
├── wasm-memory-improvements.md
├── google-vertex-auth.md
├── performance-monitoring.md
├── streaming-metrics-guide.md
├── profiling-integration-complete.md
├── suppression-analysis.md
├── test-underscore-fixes-summary.md
├── riptide-{component}-underscore-fixes.md (multiple)
├── provider-activation-analysis.md
├── analysis/ (5 files)
├── architecture/ (1 file)
├── development/ (4 files)
└── performance/ (2 files)
```

### Pain Points Identified

1. **Flat Structure**: Too many files at root level (25+ files)
2. **Inconsistent Naming**: Mix of UPPERCASE, lowercase, and kebab-case
3. **Duplicate Information**: Multiple summaries, reports, and guides covering overlapping topics
4. **Poor Discoverability**: Hard to find specific information without prior knowledge
5. **Mixed Audiences**: Developer notes, user guides, and internal reports all mixed together
6. **No Clear Entry Points**: No obvious starting point for different user types

---

## New Documentation Architecture

### Design Principles

1. **Audience-First Organization**: Structure by user role and journey
2. **Progressive Disclosure**: Start simple, add complexity as needed
3. **Single Source of Truth**: One canonical location per topic
4. **Clear Hierarchy**: Maximum 3 levels deep for easy navigation
5. **Consistent Naming**: kebab-case for files, Title Case for directories
6. **Self-Documenting**: File names clearly indicate content

### Top-Level Structure

```
docs/
├── README.md                           # Navigation hub & quick links
├── CHANGELOG.md                        # Version history
├── CONTRIBUTING.md                     # How to contribute to docs
│
├── Getting-Started/                    # New user onboarding
│   ├── README.md                       # Quick start guide
│   ├── installation.md                 # Setup instructions
│   ├── first-crawl.md                  # Hello World example
│   ├── core-concepts.md                # Key terminology
│   └── faq.md                          # Common questions
│
├── User-Guide/                         # End-user documentation
│   ├── README.md                       # User guide overview
│   ├── Crawling/                       # Web crawling features
│   │   ├── single-page.md             # Scraping one URL
│   │   ├── multi-page.md              # Crawling websites
│   │   ├── search-integration.md      # Search functionality
│   │   └── advanced-options.md        # Depth, filters, etc.
│   ├── Extraction/                     # Content extraction
│   │   ├── strategies.md              # CSS, WASM, LLM, Regex
│   │   ├── confidence-scoring.md      # Quality metrics
│   │   ├── strategy-composition.md    # Chaining strategies
│   │   ├── pdf-processing.md          # PDF extraction
│   │   └── table-extraction.md        # Structured data
│   ├── Streaming/                      # Real-time features
│   │   ├── protocols.md               # NDJSON, SSE, WebSocket
│   │   ├── progress-tracking.md       # Status updates
│   │   └── examples.md                # Code samples
│   ├── CLI/                            # Command-line tool
│   │   ├── installation.md            # CLI setup
│   │   ├── commands.md                # All commands reference
│   │   ├── examples.md                # Common workflows
│   │   └── configuration.md           # CLI config
│   └── API/                            # HTTP API usage
│       ├── authentication.md          # Auth methods
│       ├── endpoints.md               # All endpoints
│       ├── request-format.md          # Request structure
│       ├── response-format.md         # Response structure
│       └── rate-limiting.md           # Usage limits
│
├── Use-Cases/                          # Real-world scenarios
│   ├── README.md                       # Use case overview
│   ├── knowledge-base-building.md      # LLM training data
│   ├── price-monitoring.md             # E-commerce scraping
│   ├── content-aggregation.md          # News/blog aggregation
│   ├── seo-analysis.md                 # Sitemap generation
│   └── research-automation.md          # Academic/market research
│
├── Advanced/                           # Power user features
│   ├── README.md                       # Advanced guide overview
│   ├── Performance/                    # Optimization
│   │   ├── tuning.md                  # Config optimization
│   │   ├── caching.md                 # Cache strategies
│   │   ├── concurrency.md             # Parallel processing
│   │   └── profiling.md               # Memory/CPU profiling
│   ├── Configuration/                  # Advanced config
│   │   ├── environment-variables.md   # All env vars
│   │   ├── yaml-reference.md          # Config file format
│   │   ├── redis-tuning.md            # Redis optimization
│   │   └── wasm-settings.md           # WASM configuration
│   ├── Stealth/                        # Anti-detection
│   │   ├── overview.md                # Stealth features
│   │   ├── fingerprint-randomization.md
│   │   └── proxy-rotation.md
│   └── LLM-Integration/                # AI features
│       ├── providers.md               # OpenAI, Anthropic, Google
│       ├── extraction-prompts.md      # Prompt engineering
│       └── structured-output.md       # Schema-based extraction
│
├── Architecture/                       # System design
│   ├── README.md                       # Architecture overview
│   ├── system-overview.md              # High-level design
│   ├── dual-path-pipeline.md           # Pipeline architecture
│   ├── wasm-integration.md             # WebAssembly details
│   ├── headless-browser.md             # Chromiumoxide integration
│   ├── caching-layer.md                # Redis architecture
│   ├── streaming-architecture.md       # Real-time protocols
│   ├── session-management.md           # Session persistence
│   ├── worker-queue.md                 # Background jobs
│   ├── monitoring.md                   # Metrics & observability
│   └── decision-records/               # ADRs
│       ├── README.md
│       ├── 001-wasm-component-model.md
│       ├── 002-dual-path-pipeline.md
│       └── 003-redis-caching.md
│
├── API-Reference/                      # Complete API spec
│   ├── README.md                       # API reference overview
│   ├── openapi.yaml                    # OpenAPI 3.0 spec
│   ├── Endpoints/                      # By category
│   │   ├── health-metrics.md
│   │   ├── crawling.md
│   │   ├── search.md
│   │   ├── streaming.md
│   │   ├── spider.md
│   │   ├── strategies.md
│   │   ├── pdf-processing.md
│   │   ├── stealth.md
│   │   ├── table-extraction.md
│   │   ├── llm-providers.md
│   │   ├── sessions.md
│   │   ├── workers-jobs.md
│   │   └── monitoring.md
│   ├── Models/                         # Data schemas
│   │   ├── request-models.md
│   │   ├── response-models.md
│   │   └── error-codes.md
│   └── Examples/                       # API usage examples
│       ├── curl.md
│       ├── python.md
│       ├── javascript.md
│       └── postman-collection.json
│
├── Deployment/                         # Production deployment
│   ├── README.md                       # Deployment overview
│   ├── Quick-Start/                    # Fast deployment
│   │   ├── docker-compose.md          # Single-server setup
│   │   └── local-development.md       # Dev environment
│   ├── Production/                     # Enterprise deployment
│   │   ├── docker.md                  # Docker production
│   │   ├── kubernetes.md              # K8s deployment
│   │   ├── kong-gateway.md            # API gateway setup
│   │   ├── scaling.md                 # Horizontal scaling
│   │   ├── high-availability.md       # HA architecture
│   │   └── security.md                # Security hardening
│   ├── Monitoring/                     # Observability
│   │   ├── prometheus.md              # Metrics collection
│   │   ├── grafana-dashboards.md      # Visualization
│   │   ├── alerting.md                # Alert configuration
│   │   └── logging.md                 # Log aggregation
│   └── Troubleshooting/                # Ops guide
│       ├── common-issues.md           # Known problems
│       ├── debugging.md               # Debug techniques
│       └── performance-issues.md      # Performance tuning
│
├── Development/                        # Contributor docs
│   ├── README.md                       # Dev guide overview
│   ├── Getting-Started/                # Setup
│   │   ├── environment-setup.md       # Dev environment
│   │   ├── building-from-source.md    # Build instructions
│   │   └── project-structure.md       # Codebase layout
│   ├── Contributing/                   # Contribution guide
│   │   ├── code-of-conduct.md
│   │   ├── pull-requests.md
│   │   ├── issue-guidelines.md
│   │   └── style-guide.md
│   ├── Testing/                        # Test documentation
│   │   ├── overview.md                # Test philosophy
│   │   ├── unit-tests.md              # Unit testing
│   │   ├── integration-tests.md       # Integration testing
│   │   ├── tdd-workflow.md            # TDD process
│   │   └── coverage.md                # Coverage requirements
│   ├── Crates/                         # Crate documentation
│   │   ├── README.md                  # Workspace overview
│   │   ├── riptide-api.md
│   │   ├── riptide-core.md
│   │   ├── riptide-extraction.md
│   │   ├── riptide-search.md
│   │   ├── riptide-headless.md
│   │   ├── riptide-workers.md
│   │   ├── riptide-intelligence.md
│   │   ├── riptide-persistence.md
│   │   ├── riptide-streaming.md
│   │   ├── riptide-stealth.md
│   │   ├── riptide-pdf.md
│   │   ├── riptide-performance.md
│   │   └── riptide-extractor-wasm.md
│   └── Release-Process/                # Release docs
│       ├── versioning.md              # Semver policy
│       ├── changelog-guide.md         # Changelog format
│       └── release-checklist.md       # Release steps
│
├── Tools-SDKs/                         # Client tools
│   ├── README.md                       # Tools overview
│   ├── CLI/                            # Command-line tool
│   │   ├── installation.md
│   │   ├── usage.md
│   │   └── examples.md
│   ├── Python-SDK/                     # Python client
│   │   ├── installation.md
│   │   ├── quickstart.md
│   │   └── api-reference.md
│   └── Web-Playground/                 # Interactive UI
│       ├── getting-started.md
│       └── features.md
│
└── Internal/                           # Internal documentation
    ├── README.md                       # Internal docs note
    ├── Reports/                        # Status reports
    │   ├── gap-fixes-summary.md
    │   ├── wasm-tdd-report.md
    │   ├── cli-completion-report.md
    │   └── production-readiness.md
    ├── Technical-Debt/                 # Known issues
    │   ├── test-underscore-fixes.md
    │   ├── provider-activation.md
    │   └── suppression-analysis.md
    └── Analysis/                       # Deep dives
        ├── cache-key-optimization.md
        ├── wasm-memory-improvements.md
        ├── performance-bottlenecks.md
        └── resourcemanager-architecture.md
```

---

## File Naming Conventions

### Directory Names
- **Title Case with Hyphens**: `Getting-Started/`, `User-Guide/`, `API-Reference/`
- **Descriptive**: Clearly indicates content area
- **Consistent**: Same pattern throughout

### File Names
- **kebab-case**: `installation.md`, `strategy-composition.md`, `api-reference.md`
- **Descriptive**: `multi-provider-llm-integration.md` not `llm.md`
- **Hierarchical**: Use subdirectories instead of long names

### Special Files
- **README.md**: Overview/index for each directory
- **CHANGELOG.md**: Version history (root only)
- **CONTRIBUTING.md**: Contribution guidelines (root only)

---

## Content Migration Mapping

### Old → New Location Mapping

| Old Path | New Path | Action |
|----------|----------|--------|
| `docs/README.md` | `docs/README.md` | **Rewrite** as navigation hub |
| `docs/API_TOOLING_QUICKSTART.md` | `docs/Tools-SDKs/README.md` | **Move & Rename** |
| `docs/CLI_COMPLETION_REPORT.md` | `docs/Internal/Reports/cli-completion-report.md` | **Move** |
| `docs/PRODUCTION_READY_CHECKLIST.md` | `docs/Internal/Reports/production-readiness.md` | **Move** |
| `docs/WASM_TDD_*.md` | `docs/Internal/Reports/wasm-tdd-report.md` | **Merge & Move** |
| `docs/WASM_BINDING_COMPLETION_GUIDE.md` | `docs/Architecture/wasm-integration.md` | **Move & Enhance** |
| `docs/cache-key-fix-summary.md` | `docs/Internal/Analysis/cache-key-optimization.md` | **Move** |
| `docs/confidence-scoring-api.md` | `docs/User-Guide/Extraction/confidence-scoring.md` | **Move & Enhance** |
| `docs/gap-fixes-*.md` | `docs/Internal/Reports/gap-fixes-summary.md` | **Merge & Move** |
| `docs/strategy-composition.md` | `docs/User-Guide/Extraction/strategy-composition.md` | **Move & Enhance** |
| `docs/wasm-memory-improvements.md` | `docs/Internal/Analysis/wasm-memory-improvements.md` | **Move** |
| `docs/google-vertex-auth.md` | `docs/Advanced/LLM-Integration/providers.md` | **Merge** |
| `docs/performance-monitoring.md` | `docs/Deployment/Monitoring/prometheus.md` | **Move & Enhance** |
| `docs/streaming-metrics-guide.md` | `docs/User-Guide/Streaming/progress-tracking.md` | **Move & Enhance** |
| `docs/profiling-integration-complete.md` | `docs/Advanced/Performance/profiling.md` | **Move & Enhance** |
| `docs/*-underscore-fixes.md` | `docs/Internal/Technical-Debt/` | **Consolidate & Move** |
| `docs/suppression-analysis.md` | `docs/Internal/Analysis/` | **Move** |
| `docs/provider-activation-analysis.md` | `docs/Internal/Analysis/` | **Move** |
| `docs/analysis/*` | `docs/Internal/Analysis/` | **Move** |
| `docs/architecture/*` | `docs/Architecture/` | **Keep & Enhance** |
| `docs/development/*` | `docs/Development/Getting-Started/` | **Reorganize** |
| `docs/performance/*` | `docs/Advanced/Performance/` | **Move & Enhance** |

### Content Consolidation Plan

**Merge Candidates** (reduce duplication):
1. **WASM Documentation**: Merge 3 WASM files into comprehensive guides
   - `WASM_TDD_EXECUTION_REPORT.md` + `WASM_TDD_SUMMARY.md` → `wasm-tdd-report.md`
   - `WASM_BINDING_COMPLETION_GUIDE.md` → enhance `Architecture/wasm-integration.md`

2. **Gap Fixes**: Consolidate into single report
   - `gap-fixes-summary.md` + `gap-fixes-review.md` → `Internal/Reports/gap-fixes-summary.md`

3. **Underscore Fixes**: Merge component-specific fixes
   - All `*-underscore-fixes.md` → `Internal/Technical-Debt/test-underscore-fixes.md`

4. **LLM Provider Docs**: Merge into comprehensive guide
   - `google-vertex-auth.md` → part of `Advanced/LLM-Integration/providers.md`

---

## Navigation Hierarchy

### Primary Entry Points by User Type

**New Users** → `Getting-Started/README.md`
- Installation → `Getting-Started/installation.md`
- First Crawl → `Getting-Started/first-crawl.md`
- Core Concepts → `Getting-Started/core-concepts.md`

**End Users** → `User-Guide/README.md`
- Crawling → `User-Guide/Crawling/`
- Extraction → `User-Guide/Extraction/`
- API → `User-Guide/API/`
- CLI → `User-Guide/CLI/`

**DevOps Engineers** → `Deployment/README.md`
- Quick Start → `Deployment/Quick-Start/`
- Production → `Deployment/Production/`
- Monitoring → `Deployment/Monitoring/`
- Troubleshooting → `Deployment/Troubleshooting/`

**Developers** → `Development/README.md`
- Getting Started → `Development/Getting-Started/`
- Contributing → `Development/Contributing/`
- Testing → `Development/Testing/`
- Crates → `Development/Crates/`

**Architects** → `Architecture/README.md`
- System Overview → `Architecture/system-overview.md`
- Pipeline → `Architecture/dual-path-pipeline.md`
- WASM → `Architecture/wasm-integration.md`

**Integrators** → `Tools-SDKs/README.md`
- CLI → `Tools-SDKs/CLI/`
- Python SDK (riptide-sdk) → `Tools-SDKs/Python-SDK/`
- Playground → `Tools-SDKs/Web-Playground/`

---

## Documentation Standards

### Markdown Style Guide

**Headers**
```markdown
# H1: Page Title (one per file)
## H2: Major Section
### H3: Subsection
#### H4: Detail (avoid H5+)
```

**Code Blocks**
```markdown
```bash
# Shell commands with language tag
```

```rust
// Rust code with syntax highlighting
```
```

**Links**
```markdown
[Relative Link](../other-section/page.md)
[Absolute Link](/docs/User-Guide/API/endpoints.md)
[External Link](https://example.com)
```

**Lists**
```markdown
- Unordered list
- With consistent bullets

1. Ordered list
2. For sequential steps
```

**Callouts**
```markdown
> **Note**: Use blockquotes for important notes

> **Warning**: Highlight risks or cautions

> **Tip**: Provide helpful suggestions
```

**Tables**
```markdown
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Data     | Data     | Data     |
```

### Content Guidelines

**Structure**
1. **Summary**: Brief overview (2-3 sentences)
2. **Prerequisites**: What's needed (if applicable)
3. **Main Content**: Detailed information
4. **Examples**: Code samples or use cases
5. **Related**: Links to related docs

**Writing Style**
- **Clarity**: Use simple, direct language
- **Brevity**: Keep paragraphs to 3-5 sentences
- **Action-Oriented**: Use imperative mood for instructions
- **Inclusive**: Avoid jargon, define terms
- **Consistent**: Use standard terminology

**Code Examples**
- **Complete**: Runnable, not snippets
- **Annotated**: Explain key parts with comments
- **Realistic**: Use practical examples
- **Tested**: Verify all examples work

---

## Template Files

### README Template (Directory Index)

```markdown
# [Section Name]

Brief overview of this section (2-3 sentences).

## Contents

- [Page 1](page-1.md) - Short description
- [Page 2](page-2.md) - Short description
- [Page 3](page-3.md) - Short description

## Quick Links

- Related section 1
- Related section 2

## Prerequisites

List any prerequisites for this section (if applicable).
```

### Page Template (Individual Document)

```markdown
# [Page Title]

**Brief description** (1-2 sentences).

## Overview

Summary of what this page covers.

## [Main Section 1]

Content...

### [Subsection]

Content...

## Examples

```language
code example
```

## Related Documentation

- [Related Page 1](../path/to/page.md)
- [Related Page 2](../path/to/page.md)

## Next Steps

- What to read next
- What to try next
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Create new directory structure
- [ ] Write all README.md files (navigation)
- [ ] Migrate critical user-facing docs
  - Getting Started
  - User Guide essentials
  - API Reference

### Phase 2: Core Content (Week 2)
- [ ] Migrate and enhance user documentation
- [ ] Consolidate internal reports
- [ ] Create Use Cases section
- [ ] Update all cross-references

### Phase 3: Advanced & Technical (Week 3)
- [ ] Migrate Architecture documentation
- [ ] Create Development documentation
- [ ] Migrate Deployment guides
- [ ] Add Architecture Decision Records

### Phase 4: Polish & Tooling (Week 4)
- [ ] Create Tools & SDKs documentation
- [ ] Add comprehensive examples
- [ ] Build documentation site (optional: MkDocs/Docusaurus)
- [ ] Setup automated link checking
- [ ] Create contribution guidelines

### Phase 5: Validation & Launch
- [ ] Internal review
- [ ] User testing
- [ ] Fix broken links
- [ ] Generate API reference from OpenAPI
- [ ] Archive old documentation

---

## Success Metrics

### Quantitative Metrics
- **Time to First Success**: Users complete first crawl < 5 minutes
- **Documentation Coverage**: 100% of API endpoints documented
- **Search Effectiveness**: Users find answers in < 3 clicks
- **Link Health**: 0 broken internal links
- **Update Frequency**: All docs updated within 1 week of code changes

### Qualitative Metrics
- **User Feedback**: Positive sentiment from user surveys
- **Support Tickets**: Reduced documentation-related issues
- **Community Engagement**: More contributions to docs
- **Onboarding Success**: New users successfully self-serve

---

## Maintenance Guidelines

### Ownership
- **User Guides**: Technical Writers + Product Team
- **API Reference**: Backend Engineers
- **Architecture**: Architecture Team
- **Development**: Engineering Team
- **Deployment**: DevOps Team

### Update Triggers
1. **Code Changes**: Update docs in same PR
2. **API Changes**: Update OpenAPI spec + examples
3. **Bug Fixes**: Update troubleshooting guides
4. **New Features**: Add to User Guide + changelog
5. **Deprecations**: Add warnings + migration guides

### Review Process
1. **Technical Accuracy**: Engineers review
2. **Clarity**: Technical writers review
3. **Completeness**: Product team reviews
4. **Examples**: QA team validates

### Tools
- **Linting**: `markdownlint` for style consistency
- **Link Checking**: `markdown-link-check` for broken links
- **Spell Checking**: `cspell` for typos
- **Version Control**: Git with conventional commits
- **Site Generation**: MkDocs Material or Docusaurus (optional)

---

## Documentation Site (Optional)

### Recommended: MkDocs Material

**Benefits**:
- Beautiful, responsive UI
- Built-in search
- Version selector
- Code highlighting
- Markdown-native
- Easy GitHub Pages deployment

**Configuration**: `mkdocs.yml`
```yaml
site_name: RipTide Documentation
site_url: https://riptide.dev/docs
theme:
  name: material
  palette:
    primary: blue
    accent: indigo
  features:
    - navigation.instant
    - navigation.sections
    - navigation.top
    - search.suggest
    - toc.integrate

nav:
  - Home: index.md
  - Getting Started: Getting-Started/
  - User Guide: User-Guide/
  - Use Cases: Use-Cases/
  - Advanced: Advanced/
  - Architecture: Architecture/
  - API Reference: API-Reference/
  - Deployment: Deployment/
  - Development: Development/
  - Tools & SDKs: Tools-SDKs/
```

---

## Recommendations

### High Priority
1. **Create README.md hub**: Central navigation for all audiences
2. **Migrate Getting Started**: Critical for user onboarding
3. **Consolidate internal docs**: Move reports out of main docs
4. **Fix naming inconsistency**: Standardize kebab-case
5. **Add missing use cases**: Real-world examples attract users

### Medium Priority
6. **Generate API reference**: Automate from OpenAPI spec
7. **Add Architecture Decision Records**: Document design choices
8. **Create contribution guide**: Lower barrier for contributors
9. **Build documentation site**: Better UX with search and navigation
10. **Add troubleshooting guides**: Reduce support burden

### Low Priority
11. **Create video tutorials**: Complement text documentation
12. **Add interactive examples**: In-browser playground
13. **Translate documentation**: International users
14. **API changelog**: Detailed API version history

---

## Conclusion

This architecture provides:

✅ **Clear structure** organized by user role and journey
✅ **Easy navigation** with intuitive hierarchy
✅ **Reduced duplication** through consolidation
✅ **Better discoverability** with consistent naming
✅ **Scalability** to accommodate future growth
✅ **Maintainability** with ownership and processes

The proposed structure follows industry best practices from leading documentation sites while addressing RipTide's specific needs. Implementation will occur in 5 phases over approximately 4-5 weeks, with immediate focus on user-facing documentation and gradual enhancement of technical content.

---

**Next Steps**:
1. Review and approve architecture design
2. Create directory structure (Phase 1)
3. Begin content migration following mapping plan
4. Setup automated tooling (linting, link checking)
5. Deploy documentation site (optional)

**Owner**: Documentation Team
**Reviewer**: Engineering Leadership
**Timeline**: 4-5 weeks for full implementation
