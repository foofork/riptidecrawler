#!/usr/bin/env bash
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$TEST_DIR/results"

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Compare two session files
compare_sessions() {
    local session1=$1
    local session2=$2

    if [[ ! -f "$session1" ]] || [[ ! -f "$session2" ]]; then
        log_error "Session files not found"
        exit 1
    fi

    log_info "Comparing sessions:"
    log_info "  Session 1: $(basename "$session1")"
    log_info "  Session 2: $(basename "$session2")"

    local comparison_file="$RESULTS_DIR/comparison-$(date +%s).json"

    # Create comparison report
    jq -n \
        --slurpfile s1 "$session1" \
        --slurpfile s2 "$session2" \
        '{
            session1: $s1[0].session_id,
            session2: $s2[0].session_id,
            timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ")),
            summary: {
                session1_success_rate: $s1[0].success_rate,
                session2_success_rate: $s2[0].success_rate,
                rate_change: ($s2[0].success_rate - $s1[0].success_rate)
            },
            improvements: [
                $s1[0].results as $r1 | $s2[0].results as $r2 |
                $r1 | to_entries | .[] |
                select(.value.success == false) |
                .key as $idx |
                $r2[$idx] | select(.success == true) |
                {
                    test_id: .test_id,
                    method: .method,
                    change: "fixed"
                }
            ],
            regressions: [
                $s1[0].results as $r1 | $s2[0].results as $r2 |
                $r1 | to_entries | .[] |
                select(.value.success == true) |
                .key as $idx |
                $r2[$idx] | select(.success == false) |
                {
                    test_id: .test_id,
                    method: .method,
                    change: "broken"
                }
            ],
            performance_changes: [
                $s1[0].results as $r1 | $s2[0].results as $r2 |
                $r1 | to_entries | .[] |
                select(.value.success == true) |
                .key as $idx |
                .value as $v1 |
                $r2[$idx] | select(.success == true) |
                {
                    test_id: .test_id,
                    method: .method,
                    duration_change_ms: (.duration_ms - $v1.duration_ms),
                    duration_change_pct: ((.duration_ms - $v1.duration_ms) / $v1.duration_ms * 100)
                } | select(.duration_change_pct > 20 or .duration_change_pct < -20)
            ]
        }' > "$comparison_file"

    # Print comparison
    echo ""
    echo "================================"
    echo "Session Comparison Report"
    echo "================================"
    echo ""

    log_info "Success Rate Change: $(jq -r '.summary.rate_change' "$comparison_file")%"

    local improvements=$(jq '.improvements | length' "$comparison_file")
    local regressions=$(jq '.regressions | length' "$comparison_file")

    if [[ $improvements -gt 0 ]]; then
        echo ""
        log_success "Improvements ($improvements):"
        jq -r '.improvements[] | "  ✅ \(.test_id) [\(.method)] - Now succeeds"' "$comparison_file"
    fi

    if [[ $regressions -gt 0 ]]; then
        echo ""
        log_error "Regressions ($regressions):"
        jq -r '.regressions[] | "  ❌ \(.test_id) [\(.method)] - Now fails"' "$comparison_file"
    fi

    local perf_changes=$(jq '.performance_changes | length' "$comparison_file")
    if [[ $perf_changes -gt 0 ]]; then
        echo ""
        log_warning "Significant Performance Changes ($perf_changes):"
        jq -r '.performance_changes[] |
            if .duration_change_pct < 0 then
                "  ⚡ \(.test_id) [\(.method)] - \(.duration_change_pct | fabs | floor)% faster"
            else
                "  🐌 \(.test_id) [\(.method)] - \(.duration_change_pct | floor)% slower"
            end' "$comparison_file"
    fi

    echo ""
    log_success "Comparison saved to: $comparison_file"
}

# Compare methods within a session
compare_methods() {
    local session=$1

    if [[ ! -f "$session" ]]; then
        log_error "Session file not found: $session"
        exit 1
    fi

    log_info "Comparing methods in session: $(basename "$session")"

    local methods=$(jq -r '.methods[]' "$session")

    echo ""
    echo "================================"
    echo "Method Comparison"
    echo "================================"
    echo ""

    # For each method, calculate statistics
    while IFS= read -r method; do
        local success=$(jq "[.results[] | select(.method == \"$method\" and .success == true)] | length" "$session")
        local total=$(jq "[.results[] | select(.method == \"$method\")] | length" "$session")
        local rate=$(echo "scale=1; $success * 100 / $total" | bc)
        local avg_duration=$(jq "[.results[] | select(.method == \"$method\" and .success == true) | .duration_ms] | add / length // 0" "$session")
        local avg_content=$(jq "[.results[] | select(.method == \"$method\" and .success == true) | .content_length] | add / length // 0" "$session")

        echo "📊 $method"
        echo "   Success Rate: $success/$total ($rate%)"
        echo "   Avg Duration: ${avg_duration}ms"
        echo "   Avg Content:  ${avg_content} bytes"
        echo ""
    done <<< "$methods"

    # Find best method
    local best_method=$(jq -r '
        .methods as $methods |
        $methods | map(. as $method |
            {
                method: $method,
                success: ([.results[] | select(.method == $method and .success == true)] | length),
                total: ([.results[] | select(.method == $method)] | length),
                avg_duration: ([.results[] | select(.method == $method and .success == true) | .duration_ms] | add / length // 999999)
            }
        ) | max_by(.success / .total * 1000 - .avg_duration) | .method
    ' "$session")

    echo ""
    log_success "🏆 Best Overall Method: $best_method"
}

# Main
COMMAND="${1:-help}"

case "$COMMAND" in
    sessions)
        SESSION1="${2:-}"
        SESSION2="${3:-}"

        if [[ -z "$SESSION1" ]] || [[ -z "$SESSION2" ]]; then
            # List available sessions
            log_info "Available sessions:"
            ls -t "$RESULTS_DIR"/session-*.json | head -n 10 | while read -r session; do
                echo "  $(basename "$session")"
            done
            echo ""
            echo "Usage: $0 sessions <session1> <session2>"
            exit 1
        fi

        compare_sessions "$RESULTS_DIR/$SESSION1" "$RESULTS_DIR/$SESSION2"
        ;;
    methods)
        SESSION="${2:-$(ls -t "$RESULTS_DIR"/session-*.json | head -n1)}"
        compare_methods "$SESSION"
        ;;
    diff)
        # Quick diff between two most recent sessions
        SESSIONS=($(ls -t "$RESULTS_DIR"/session-*.json | head -n 2))
        if [[ ${#SESSIONS[@]} -lt 2 ]]; then
            log_error "Need at least 2 sessions to compare"
            exit 1
        fi
        compare_sessions "${SESSIONS[0]}" "${SESSIONS[1]}"
        ;;
    *)
        echo "Usage: $0 {sessions|methods|diff}"
        echo ""
        echo "Commands:"
        echo "  sessions <s1> <s2> - Compare two test sessions"
        echo "  methods [session]  - Compare methods within a session"
        echo "  diff               - Quick diff of two most recent sessions"
        exit 1
        ;;
esac
