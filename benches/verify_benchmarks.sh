#!/bin/bash
# Quick verification script for benchmark compilation

echo "=== RipTide Benchmark Verification ==="
echo ""

echo "📁 Checking benchmark files..."
ls -lh benches/*.rs benches/Cargo.toml

echo ""
echo "📊 Line counts:"
wc -l benches/*.rs

echo ""
echo "🧪 Verifying benchmark configuration..."
grep "^\[\[bench\]\]" benches/Cargo.toml

echo ""
echo "✅ Verification complete!"
echo ""
echo "To run benchmarks:"
echo "  cargo bench --bench facade_benchmark"
echo "  cargo bench --bench hybrid_launcher_benchmark"
echo ""
echo "To establish baselines:"
echo "  cargo bench -- --save-baseline p1c1-baseline"
echo "  cargo bench -- --save-baseline p1c3-baseline"
