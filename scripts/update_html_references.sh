#!/bin/bash

# Script to update all references from riptide-extraction to riptide-extraction

echo "Updating references from riptide-extraction to riptide-extraction..."

# List of files to update (excluding binary files and .git)
files_to_update=(
    "CHANGELOG.md"
    "RUNTIME_VERIFICATION_REPORT.md"
    "deploy.sh"
    "crates/riptide-core/benches/strategies_bench.rs"
    "crates/riptide-core/examples/trait_system_demo.rs"
    "crates/riptide-core/examples/strategies_demo.rs"
    "crates/riptide-core/src/spider/core.rs"
    "CLI_TEST_SUMMARY.md"
    "crates/riptide-api/src/strategies_pipeline.rs"
    "crates/riptide-api/src/handlers/strategies.rs"
    "crates/riptide-api/src/handlers/tables.rs"
    "crates/riptide-core/src/types.rs"
    "crates/riptide-core/src/strategies/manager.rs"
    "crates/riptide-core/src/strategies/performance.rs"
    "crates/riptide-core/src/strategies/implementations.rs"
    "crates/riptide-core/src/strategies/traits.rs"
    "crates/riptide-core/src/component.rs"
    "crates/riptide-core/README.md"
    "tests/wasm-memory/Cargo.toml"
    "tests/week3/README.md"
    "tests/fix_topic_chunker.rs"
    "tests/lib.rs"
    "tests/README.md"
    "tests/trek_removal_validation_report.md"
    "crates/riptide-streaming/README.md"
    "tests/TEST_URL_RESEARCH.md"
    "tests/integration_test.rs"
    "crates/riptide-extraction/tests/extraction_tests.rs"
    "crates/riptide-extraction/tests/integration_tests.rs"
    "crates/riptide-extraction/tests/html_extraction_tests.rs"
    "crates/riptide-extraction/CHUNKING_IMPLEMENTATION.md"
    "crates/riptide-extraction/src/strategy_implementations.rs"
    "crates/riptide-extraction/README.md"
    "deny.toml"
    "scripts/detailed_coverage.sh"
)

# Update each file
for file in "${files_to_update[@]}"; do
    if [ -f "$file" ]; then
        echo "Updating $file..."
        sed -i 's/riptide-extraction/riptide-extraction/g' "$file"
    else
        echo "Warning: $file not found, skipping..."
    fi
done

echo "Done! All references have been updated."
echo ""
echo "Please review the changes and run the tests to ensure everything works correctly:"
echo "  cargo test --all"
echo "  cargo build --all"