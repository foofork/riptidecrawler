#!/bin/bash
# P2-F1 Day 4-5: Automated import migration script
# Migrates imports from riptide-core to riptide-reliability and riptide-types

set -e

echo "🔄 P2-F1 Migration: riptide-core → riptide-reliability + riptide-types"
echo "=================================================================="

# Target directories (exclude riptide-core itself)
TARGETS=(
    "crates/riptide-api"
    "crates/riptide-workers"
    "crates/riptide-search"
    "crates/riptide-pdf"
    "crates/riptide-persistence"
    "crates/riptide-streaming"
    "crates/riptide-cli"
    "crates/riptide-extraction"
    "crates/riptide-performance"
    "crates/riptide-intelligence"
    "tests"
)

# Migration patterns
echo "📝 Applying import transformations..."

for target in "${TARGETS[@]}"; do
    if [ ! -d "$target" ]; then
        echo "⚠️  Skipping $target (not found)"
        continue
    fi

    echo "✓ Processing $target..."

    # Find all Rust files
    find "$target" -name "*.rs" -type f | while read -r file; do
        # Skip if file doesn't contain riptide_core imports
        if ! grep -q "use riptide_core" "$file" 2>/dev/null; then
            continue
        fi

        echo "  → Updating $file"

        # Create backup
        cp "$file" "$file.bak"

        # 1. Reliability patterns: circuit, gate, reliability
        sed -i 's/use riptide_core::circuit::/use riptide_reliability::circuit::/g' "$file"
        sed -i 's/use riptide_core::circuit_breaker::/use riptide_reliability::circuit_breaker::/g' "$file"
        sed -i 's/use riptide_core::gate::/use riptide_reliability::gate::/g' "$file"
        sed -i 's/use riptide_core::reliability::/use riptide_reliability::reliability::/g' "$file"

        # 2. Types and errors
        sed -i 's/use riptide_core::types::/use riptide_types::/g' "$file"
        sed -i 's/use riptide_core::component::/use riptide_types::component::/g' "$file"
        sed -i 's/use riptide_core::conditional::/use riptide_types::conditional::/g' "$file"
        sed -i 's/use riptide_core::error::/use riptide_types::error::/g' "$file"

        # 3. WASM validation (moved to riptide-extraction)
        sed -i 's/use riptide_core::wasm_validation::/use riptide_extraction::validation::wasm::/g' "$file"

        # 4. Short-form imports (single items)
        sed -i 's/use riptide_core::{circuit, /use riptide_reliability::{circuit, /g' "$file"
        sed -i 's/use riptide_core::{gate, /use riptide_reliability::{gate, /g' "$file"
        sed -i 's/use riptide_core::{types, /use riptide_types::{/g' "$file"

        # Check if file was actually modified
        if ! diff "$file" "$file.bak" > /dev/null 2>&1; then
            echo "    ✅ Updated"
        else
            echo "    → No changes needed"
        fi

        # Clean up backup
        rm "$file.bak"
    done
done

echo ""
echo "📦 Updating Cargo.toml dependencies..."
echo "=================================================================="

# Update Cargo.toml files
for target in "${TARGETS[@]}"; do
    toml_file="$target/Cargo.toml"

    if [ ! -f "$toml_file" ]; then
        continue
    fi

    # Check if it depends on riptide-core
    if ! grep -q 'riptide-core = { path = "../riptide-core"' "$toml_file" 2>/dev/null; then
        continue
    fi

    echo "✓ Updating $toml_file..."

    # Create backup
    cp "$toml_file" "$toml_file.bak"

    # Replace riptide-core with riptide-reliability and riptide-types
    # This assumes dependencies are on separate lines
    sed -i '/riptide-core = { path = "..\/riptide-core"/c\
# P2-F1 Day 4-5: Migrated from riptide-core to riptide-reliability + riptide-types\
riptide-reliability = { path = "../riptide-reliability" }\
riptide-types = { path = "../riptide-types" }' "$toml_file"

    echo "  ✅ Dependencies updated"

    # Clean up backup
    rm "$toml_file.bak"
done

echo ""
echo "🎉 Migration complete!"
echo ""
echo "📊 Summary:"
echo "  - Updated imports in all target crates"
echo "  - Updated Cargo.toml dependencies"
echo "  - riptide-core → riptide-reliability (circuit, gate, reliability)"
echo "  - riptide-core → riptide-types (types, component, conditional, error)"
echo "  - riptide-core → riptide-extraction (wasm_validation)"
echo ""
echo "⚠️  Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Build workspace: cargo build --workspace"
echo "  3. Run tests: cargo test --workspace"
echo "  4. Fix any remaining issues manually"
echo ""
