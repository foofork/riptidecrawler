#!/bin/bash

# Working Examples for Riptide Extract Command
# Only RAW engine is currently functional
# Date: 2025-10-16

RIPTIDE="/workspaces/eventmesh/target/x86_64-unknown-linux-gnu/release/riptide"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  RIPTIDE EXTRACT - WORKING EXAMPLES (Raw Engine Only)          ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Example 1: Basic extraction
echo "Example 1: Basic HTML extraction from simple site"
echo "Command: riptide extract --url 'https://example.com' --engine raw --local"
echo ""
$RIPTIDE extract --url "https://example.com" --engine raw --local
echo ""
echo "────────────────────────────────────────────────────────────────"
echo ""

# Example 2: Large page extraction
echo "Example 2: Extract large Wikipedia article (fast!)"
echo "Command: riptide extract --url 'https://en.wikipedia.org/wiki/Rust_(programming_language)' --engine raw --local"
echo ""
$RIPTIDE extract --url "https://en.wikipedia.org/wiki/Rust_(programming_language)" --engine raw --local | head -100
echo "... (truncated, 581KB total)"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo ""

# Example 3: Save to file
echo "Example 3: Save extracted content to file"
echo "Command: riptide extract --url 'https://www.rust-lang.org/' --engine raw --local -f output.html"
echo ""
$RIPTIDE extract --url "https://www.rust-lang.org/" --engine raw --local -f /workspaces/eventmesh/eval/results/rust_lang_output.html
echo "Saved to: /workspaces/eventmesh/eval/results/rust_lang_output.html"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo ""

# Example 4: Extract from file
echo "Example 4: Extract from local HTML file"
echo "Command: riptide extract --input-file output.html --engine raw --local"
echo ""
$RIPTIDE extract --input-file /workspaces/eventmesh/eval/results/rust_lang_output.html --engine raw --local | head -50
echo "... (truncated)"
echo ""
echo "────────────────────────────────────────────────────────────────"
echo ""

# Non-working examples (for documentation)
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  CURRENTLY BROKEN FEATURES (WASM Interface Issue)              ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "These will NOT work until WASM module is fixed:"
echo ""
echo "  ✗ --engine auto"
echo "  ✗ --engine wasm"
echo "  ✗ --engine headless"
echo "  ✗ --method wasm|css|llm|regex"
echo "  ✗ --strategy chain|parallel|fallback"
echo "  ✗ --show-confidence"
echo "  ✗ --metadata (with non-raw engines)"
echo ""
echo "Error: 'expected field extractor-version, found trek-version'"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "For more details, see:"
echo "  - CSV Report: /workspaces/eventmesh/eval/results/extract_command_tests.csv"
echo "  - Analysis: /workspaces/eventmesh/eval/results/extract_command_analysis.md"
echo "  - Test Matrix: /workspaces/eventmesh/eval/results/test_matrix.txt"
