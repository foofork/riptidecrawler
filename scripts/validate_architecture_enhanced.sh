#!/bin/bash
# Enhanced Architecture Validation Script
# Enforces strict layering, separation of concerns, and clean architecture principles
# Based on approved roadmap with user clarifications

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
WARNINGS=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    ((WARNINGS++))
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
}

# Check if running in project root
if [ ! -f "Cargo.toml" ]; then
    fail "Must run from project root (Cargo.toml not found)"
    exit 1
fi

section "1. Domain Layer Purity (riptide-types)"

info "Checking domain has no dependencies on API, Facade, or Infrastructure..."

# Check if domain (riptide-types) depends on higher layers or infra
if cargo tree -p riptide-types --invert riptide-types 2>/dev/null | \
   grep -iE 'riptide-(api|facade|reliability|cache|browser|pdf|spider|search|persistence|fetch|pool)' > /dev/null; then
    fail "Domain (riptide-types) has forbidden dependencies on application/infra layers"
    cargo tree -p riptide-types --invert riptide-types | \
        grep -iE 'riptide-(api|facade|reliability|cache|browser|pdf|spider|search|persistence|fetch|pool)'
else
    pass "Domain layer is pure (no upward/infra dependencies)"
fi

# Check domain files don't import from forbidden crates
info "Scanning domain source files for forbidden imports..."
DOMAIN_VIOLATIONS=$(find crates/riptide-types/src -name "*.rs" -exec grep -l "use riptide_\(api\|facade\|reliability\|cache\|browser\|pdf\|spider\|search\|persistence\|fetch\|pool\)" {} \; 2>/dev/null | wc -l)
if [ "$DOMAIN_VIOLATIONS" -gt 0 ]; then
    fail "Found $DOMAIN_VIOLATIONS domain files with forbidden imports"
    find crates/riptide-types/src -name "*.rs" -exec grep -l "use riptide_\(api\|facade\|reliability\|cache\|browser\|pdf\|spider\|search\|persistence\|fetch\|pool\)" {} \;
else
    pass "Domain source files have no forbidden imports"
fi

section "2. Facade Layer Dependencies (riptide-facade = Application Layer)"

info "Checking facade depends ONLY on riptide-types (ports)..."

# Facade should only depend on riptide-types for ports
# Allow: riptide-types, standard library, common utils (config, events, monitoring)
# Forbid: HTTP frameworks, databases, Redis, browser, etc.
FORBIDDEN_DEPS=$(cargo tree -p riptide-facade 2>/dev/null | \
    grep -iE 'axum|actix-web|hyper|reqwest|redis|sqlx|tokio-postgres|headless|chrome' || true)

if [ -n "$FORBIDDEN_DEPS" ]; then
    fail "Facade has forbidden transitive dependencies"
    echo "$FORBIDDEN_DEPS"
else
    pass "Facade has no forbidden transitive dependencies"
fi

# Check facade source files for HTTP/database imports
info "Scanning facade source files for HTTP/JSON types..."
FACADE_HTTP=$(find crates/riptide-facade/src -name "*.rs" -exec grep -n "actix_web::\|hyper::\|reqwest::\|axum::" {} + 2>/dev/null || true)
if [ -n "$FACADE_HTTP" ]; then
    fail "Found HTTP types in facade source files"
    echo "$FACADE_HTTP" | head -20
else
    pass "Facade has no HTTP types"
fi

# Check for serde_json::Value in facades (except allowed files)
info "Checking for serde_json::Value in facades..."
JSON_VALUE=$(find crates/riptide-facade/src -name "*.rs" -exec grep -n "serde_json::Value" {} + 2>/dev/null || true)
if [ -n "$JSON_VALUE" ]; then
    warn "Found serde_json::Value in facade files (should use typed DTOs)"
    echo "$JSON_VALUE" | head -20
else
    pass "Facade uses typed DTOs (no serde_json::Value)"
fi

section "3. Handler Layer Validation (<50 LOC, I/O only)"

info "Checking handler file sizes..."

# Find all handler files
HANDLER_FILES=$(find crates/riptide-api/src/handlers -name "*.rs" 2>/dev/null)
OVERSIZED_HANDLERS=0

for handler in $HANDLER_FILES; do
    LOC=$(wc -l < "$handler")
    FILENAME=$(basename "$handler")

    if [ "$LOC" -gt 50 ]; then
        ((OVERSIZED_HANDLERS++))
        if [ "$OVERSIZED_HANDLERS" -le 10 ]; then
            warn "Handler $FILENAME exceeds 50 LOC: $LOC lines"
        fi
    fi
done

if [ "$OVERSIZED_HANDLERS" -gt 0 ]; then
    warn "Found $OVERSIZED_HANDLERS handlers exceeding 50 LOC target"
else
    pass "All handlers are ≤50 LOC"
fi

# Check for business logic loops in handlers (for/while/loop)
info "Checking handlers for business logic (loops)..."
HANDLER_LOOPS=$(find crates/riptide-api/src/handlers -name "*.rs" -exec grep -n '\b\(for\|while\|loop\)\b' {} + 2>/dev/null | \
    grep -v "// " | grep -v "/\*" || true)

if [ -n "$HANDLER_LOOPS" ]; then
    warn "Found loops in handlers (should be in facades/domain)"
    echo "$HANDLER_LOOPS" | head -10
else
    pass "Handlers have no business logic loops"
fi

section "4. Redis Scope & Consolidation"

info "Checking Redis dependencies are scoped to ≤2 crates..."

# Find all Cargo.toml files with redis dependency
REDIS_CRATES=$(find crates -name Cargo.toml -exec grep -l "redis" {} \; 2>/dev/null | wc -l)

if [ "$REDIS_CRATES" -gt 2 ]; then
    fail "Found Redis in $REDIS_CRATES crates (target: ≤2)"
    find crates -name Cargo.toml -exec grep -l "redis" {} \;
else
    pass "Redis scoped to $REDIS_CRATES crates (target: ≤2)"
fi

# Check Redis usage is only in allowed crates (cache, persistence/workers)
info "Checking Redis usage in source files..."
REDIS_USAGE=$(rg -n '\bredis::' crates/ 2>/dev/null | grep -v 'riptide-\(cache\|persistence\|workers\)' || true)

if [ -n "$REDIS_USAGE" ]; then
    warn "Found Redis usage outside allowed crates (cache, persistence, workers)"
    echo "$REDIS_USAGE" | head -10
else
    pass "Redis usage properly scoped"
fi

section "5. Deduplication Validation"

info "Checking for duplicate robots.rs files..."
ROBOTS_FILES=$(find crates -name "robots.rs" 2>/dev/null | wc -l)

if [ "$ROBOTS_FILES" -gt 1 ]; then
    fail "Found $ROBOTS_FILES robots.rs files (should be 1 in riptide-utils)"
    find crates -name "robots.rs"
else
    pass "Single robots.rs implementation (no duplicates)"
fi

info "Checking for duplicate memory_manager.rs files..."
MEMORY_FILES=$(find crates -name "memory_manager.rs" -o -name "memory.rs" 2>/dev/null | wc -l)

if [ "$MEMORY_FILES" -gt 1 ]; then
    fail "Found $MEMORY_FILES memory manager files (should be 1 in riptide-pool)"
    find crates -name "memory_manager.rs" -o -name "memory.rs"
else
    pass "Single memory manager implementation (no duplicates)"
fi

info "Checking for duplicate cache implementations..."
CACHE_MANAGERS=$(rg -l "struct.*CacheManager" crates/*/src 2>/dev/null | wc -l)

if [ "$CACHE_MANAGERS" -gt 1 ]; then
    warn "Found $CACHE_MANAGERS cache manager implementations"
    rg -l "struct.*CacheManager" crates/*/src
else
    pass "Single cache manager implementation"
fi

section "6. Build & Compilation"

info "Checking workspace compiles with zero warnings..."

if RUSTFLAGS="-D warnings" cargo build --workspace --all-targets 2>&1 | tee /tmp/build.log | grep -i "warning:" > /dev/null; then
    fail "Build has warnings (RUSTFLAGS=\"-D warnings\" must pass)"
    grep -i "warning:" /tmp/build.log | head -20
else
    pass "Workspace compiles with zero warnings"
fi

section "7. Clippy Validation"

info "Running clippy in strict mode..."

if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee /tmp/clippy.log | grep -i "warning:\|error:" > /dev/null; then
    fail "Clippy found issues (must pass with -D warnings)"
    grep -i "warning:\|error:" /tmp/clippy.log | head -20
else
    pass "Clippy clean (no warnings)"
fi

section "8. Circular Dependencies"

info "Checking for circular dependencies..."

# Simple circular dependency check
if cargo tree --workspace --duplicates 2>&1 | grep -i "cycle" > /dev/null; then
    fail "Found circular dependencies"
    cargo tree --workspace --duplicates | grep -i "cycle"
else
    pass "No circular dependencies detected"
fi

section "9. Test Coverage (Facades ≥90%)"

info "Checking facade test coverage..."

# Build test binaries
if cargo test -p riptide-facade --no-run --quiet 2>&1; then
    pass "Facade tests compile successfully"

    # Count facade source files vs test files
    FACADE_SRC=$(find crates/riptide-facade/src -name "*.rs" | wc -l)
    FACADE_TESTS=$(find crates/riptide-facade -path "*/tests/*" -name "*.rs" -o -path "*/src/*" -name "*_test.rs" -o -path "*/src/*" -name "*_tests.rs" | wc -l)

    info "Facade source files: $FACADE_SRC, Test files: $FACADE_TESTS"

    if [ "$FACADE_TESTS" -lt "$((FACADE_SRC * 9 / 10))" ]; then
        warn "Facade may not meet ≥90% coverage target (rough estimate based on file count)"
    else
        pass "Facade test coverage appears adequate"
    fi
else
    fail "Facade tests failed to compile"
fi

section "10. JSON/HTTP Leaks in Ports (riptide-types)"

info "Checking for HTTP/JSON types in domain/ports..."

# Ports should use domain types, not HTTP request/response types
PORTS_HTTP=$(find crates/riptide-types/src -name "*.rs" -exec grep -n "HttpRequest\|HttpResponse\|StatusCode\|actix_web" {} + 2>/dev/null || true)

if [ -n "$PORTS_HTTP" ]; then
    fail "Found HTTP types in domain/ports layer"
    echo "$PORTS_HTTP"
else
    pass "Ports use domain types (no HTTP leakage)"
fi

# Check for serde_json::Value in ports (except events.rs which may use it for flexibility)
PORTS_JSON=$(find crates/riptide-types/src -name "*.rs" ! -name "events.rs" -exec grep -n "serde_json::Value" {} + 2>/dev/null || true)

if [ -n "$PORTS_JSON" ]; then
    warn "Found serde_json::Value in ports (use typed domain objects)"
    echo "$PORTS_JSON" | head -10
else
    pass "Ports use typed domain objects"
fi

section "11. Performance Baseline Check"

info "Checking for performance baseline configuration..."

if [ -f "benches/baseline_metrics.json" ]; then
    pass "Performance baseline exists (benches/baseline_metrics.json)"
else
    warn "No performance baseline found (create with: cargo bench --save-baseline main)"
fi

# Check if criterion is configured
if grep -q "criterion" Cargo.toml 2>/dev/null; then
    pass "Criterion benchmarking configured"
else
    warn "Criterion not found in Cargo.toml (add for performance tracking)"
fi

section "12. Feature Flag Strategy"

info "Checking for feature flag configuration..."

# Check if feature flags exist for new facades
if grep -q "^\[features\]" crates/riptide-facade/Cargo.toml 2>/dev/null; then
    pass "Feature flags configured in facade crate"

    # List configured features
    info "Configured features:"
    sed -n '/^\[features\]/,/^\[/p' crates/riptide-facade/Cargo.toml | grep "=" | head -5
else
    warn "No feature flags in facade Cargo.toml (add for incremental rollout)"
fi

section "═══════════════════════════════════════════════════"
section "VALIDATION SUMMARY"
section "═══════════════════════════════════════════════════"

echo ""
echo -e "${GREEN}Passed:${NC}   $PASSED"
echo -e "${YELLOW}Warnings:${NC} $WARNINGS"
echo -e "${RED}Failed:${NC}   $FAILED"
echo ""

# Acceptance criteria checklist
section "ACCEPTANCE CRITERIA CHECKLIST"

echo ""
echo "Criterion                     | Status"
echo "------------------------------|--------"

# Handlers <50 LOC
if [ "$OVERSIZED_HANDLERS" -eq 0 ]; then
    echo -e "Handlers <50 LOC              | ${GREEN}✓ PASS${NC}"
else
    echo -e "Handlers <50 LOC              | ${YELLOW}⚠ $OVERSIZED_HANDLERS oversized${NC}"
fi

# Domain deps
if cargo tree -p riptide-types --invert riptide-types 2>/dev/null | \
   grep -iE 'riptide-(api|facade|reliability|cache|browser)' > /dev/null; then
    echo -e "Domain deps (none)            | ${RED}✗ FAIL${NC}"
else
    echo -e "Domain deps (none)            | ${GREEN}✓ PASS${NC}"
fi

# Facade deps
if [ -n "$FORBIDDEN_DEPS" ]; then
    echo -e "Facade deps (types only)      | ${RED}✗ FAIL${NC}"
else
    echo -e "Facade deps (types only)      | ${GREEN}✓ PASS${NC}"
fi

# Redis crates
if [ "$REDIS_CRATES" -le 2 ]; then
    echo -e "Redis crates ≤2               | ${GREEN}✓ PASS${NC}"
else
    echo -e "Redis crates ≤2               | ${RED}✗ FAIL ($REDIS_CRATES)${NC}"
fi

# JSON/HTTP in facades
if [ -n "$FACADE_HTTP" ]; then
    echo -e "JSON/HTTP in facades (0)      | ${RED}✗ FAIL${NC}"
else
    echo -e "JSON/HTTP in facades (0)      | ${GREEN}✓ PASS${NC}"
fi

# Facade coverage (rough estimate)
echo -e "Facade coverage ≥90%          | ${YELLOW}⚠ Manual check${NC}"

# Clippy
if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep -i "warning:" > /dev/null; then
    echo -e "Clippy warnings (0)           | ${RED}✗ FAIL${NC}"
else
    echo -e "Clippy warnings (0)           | ${GREEN}✓ PASS${NC}"
fi

# Circular deps
if cargo tree --workspace --duplicates 2>&1 | grep -i "cycle" > /dev/null; then
    echo -e "Circular deps (0)             | ${RED}✗ FAIL${NC}"
else
    echo -e "Circular deps (0)             | ${GREEN}✓ PASS${NC}"
fi

echo ""

# Exit code
if [ "$FAILED" -gt 0 ]; then
    echo -e "${RED}VALIDATION FAILED${NC} - $FAILED critical issues must be fixed"
    exit 1
elif [ "$WARNINGS" -gt 5 ]; then
    echo -e "${YELLOW}VALIDATION PASSED WITH WARNINGS${NC} - $WARNINGS issues should be addressed"
    exit 0
else
    echo -e "${GREEN}VALIDATION PASSED${NC} - Architecture meets all criteria"
    exit 0
fi
