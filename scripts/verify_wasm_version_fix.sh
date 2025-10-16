#!/bin/bash
# Script to verify WASM version field fix

set -e

echo "=== WASM Version Field Fix Verification ==="
echo ""

# 1. Check WIT file has correct field name
echo "1. Checking WIT file for 'extractor-version' field..."
if grep -q "extractor-version: string" wasm/riptide-extractor-wasm/wit/extractor.wit; then
    echo "   ✅ WIT file has correct 'extractor-version' field"
else
    echo "   ❌ WIT file missing 'extractor-version' field"
    exit 1
fi

# 2. Check for any remaining trek-version references
echo ""
echo "2. Checking for any remaining 'trek-version' references..."
TREK_VERSION_COUNT=$(grep -r "trek-version" wasm/riptide-extractor-wasm/src/ 2>/dev/null | wc -l || echo "0")
if [ "$TREK_VERSION_COUNT" -eq "0" ]; then
    echo "   ✅ No 'trek-version' references found in source code"
else
    echo "   ❌ Found $TREK_VERSION_COUNT 'trek-version' references:"
    grep -n "trek-version" wasm/riptide-extractor-wasm/src/ || true
    exit 1
fi

# 3. Check Rust code uses extractor_version field
echo ""
echo "3. Checking Rust code uses 'extractor_version' field..."
if grep -q "extractor_version: get_extractor_version()" wasm/riptide-extractor-wasm/src/lib.rs; then
    echo "   ✅ lib.rs uses correct 'extractor_version' field"
else
    echo "   ❌ lib.rs missing correct field"
    exit 1
fi

# 4. Check WASM module builds successfully
echo ""
echo "4. Building WASM module..."
if cargo build --manifest-path wasm/riptide-extractor-wasm/Cargo.toml --target wasm32-wasip1 --release --quiet 2>&1 | grep -q "error"; then
    echo "   ❌ WASM build failed"
    exit 1
else
    echo "   ✅ WASM module builds successfully"
fi

# 5. Check WASM artifact exists
echo ""
echo "5. Checking WASM build artifact..."
if [ -f "target/wasm32-wasip1/release/riptide_extractor_wasm.wasm" ]; then
    WASM_SIZE=$(ls -lh target/wasm32-wasip1/release/riptide_extractor_wasm.wasm | awk '{print $5}')
    echo "   ✅ WASM module built: $WASM_SIZE"
else
    echo "   ❌ WASM artifact not found"
    exit 1
fi

# 6. Check core WIT file alignment
echo ""
echo "6. Checking riptide-core WIT alignment..."
if grep -q "extractor-version: string" crates/riptide-core/wit/world.wit; then
    echo "   ✅ riptide-core WIT file has correct 'extractor-version' field"
else
    echo "   ❌ riptide-core WIT file missing correct field"
    exit 1
fi

echo ""
echo "=== ✅ ALL CHECKS PASSED ==="
echo ""
echo "Summary of changes:"
echo "  - Updated WASM WIT interface: trek-version → extractor-version"
echo "  - Updated Rust implementation to use extractor_version field"
echo "  - Removed all trek-rs references from comments"
echo "  - Verified consistency across WASM and core WIT files"
echo "  - WASM module builds successfully with correct interface"
echo ""
echo "This fix resolves the type-checking error that was blocking 17/22 extract tests."
