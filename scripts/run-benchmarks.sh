#!/bin/bash
# Comprehensive benchmark execution with baseline comparison

BENCHMARKS=(
    "performance_benches"
    "persistence_benchmarks"
    "pool_benchmark"
    "strategies_bench"
    "stratified_pool_bench"
)

BASELINE_NAME="${1:-week2-start}"
OUTPUT_DIR="./benchmarks"
mkdir -p "$OUTPUT_DIR"

echo "🚀 Running ${#BENCHMARKS[@]} benchmark suites..."
echo "📊 Baseline: $BASELINE_NAME"
echo "📁 Output directory: $OUTPUT_DIR"
echo ""

# Run each benchmark
for bench in "${BENCHMARKS[@]}"; do
    echo ""
    echo "⏱️  Running $bench..."

    cargo bench \
        --bench "$bench" \
        -- --save-baseline "$BASELINE_NAME" \
        2>&1 | tee "$OUTPUT_DIR/$bench-$BASELINE_NAME.log"

    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        echo "✅ $bench complete"
    else
        echo "❌ $bench failed"
    fi
done

# Generate summary
echo ""
echo "📋 Benchmark Summary:"
echo "===================="
grep -h "time:" $OUTPUT_DIR/*-$BASELINE_NAME.log 2>/dev/null | head -20

echo ""
echo "✅ Benchmarks saved to baseline: $BASELINE_NAME"
echo "📁 Logs saved to: $OUTPUT_DIR/"
echo ""
echo "To compare with this baseline later:"
echo "  cargo bench --bench <benchmark_name> -- --baseline $BASELINE_NAME"
