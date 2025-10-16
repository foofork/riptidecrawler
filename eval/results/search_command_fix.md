# Quick Fix: RipTide Search Command Parameter Mismatch

## Problem
CLI search command fails with `400 Bad Request: missing field 'q'`

## Root Cause
CLI sends `query=` but API expects `q=`

## Fix

**File:** `/workspaces/eventmesh/crates/riptide-cli/src/commands/search.rs`

**Line 28:** Change parameter name in URL construction

### Before (Broken)
```rust
let mut url = format!(
    "/api/v1/search?query={}&limit={}",  // ❌ Wrong parameter name
    urlencoding::encode(&args.query),
    args.limit
);
```

### After (Fixed)
```rust
let mut url = format!(
    "/api/v1/search?q={}&limit={}",  // ✅ Correct parameter name
    urlencoding::encode(&args.query),
    args.limit
);
```

## Verification

### Test the fix:
```bash
# After rebuilding:
cargo build --release

# Test CLI command:
riptide search --query "rust programming" --limit 5

# Expected output:
✅ Found 1 results in 0ms

Result #1
Title: Result for: rust programming
URL: https://example.com/result-1
Snippet: This is a search result snippet...
```

### Test with curl to verify:
```bash
# This should work (and currently does):
curl "http://localhost:8080/api/v1/search?q=rust+programming&limit=5"
```

## Related Files
- **CLI:** `/workspaces/eventmesh/crates/riptide-cli/src/commands/search.rs` (needs fix)
- **API:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/search.rs` (correct)

## Impact
- **Breaking:** No (only fixes broken functionality)
- **API Changes:** None
- **CLI Changes:** Parameter name in internal URL construction only
- **User Facing:** Fixes search command that was previously broken

## Priority
**CRITICAL** - Search command is non-functional without this fix
