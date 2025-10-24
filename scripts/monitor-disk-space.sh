#!/bin/bash
# Disk Space Monitoring Script for CI/CD Build Process
# Monitors disk space and alerts if thresholds are crossed

CRITICAL_THRESHOLD_GB=5
WARNING_THRESHOLD_GB=10
WORKSPACE="/workspaces/eventmesh"

# Get available space in GB
get_available_space() {
    df -BG / | awk 'NR==2 {print $4}' | sed 's/G//'
}

# Get usage percentage
get_usage_percent() {
    df -h / | awk 'NR==2 {print $5}' | sed 's/%//'
}

# Get artifact sizes
get_artifact_sizes() {
    echo "Target: $(du -sh ${WORKSPACE}/target 2>/dev/null | cut -f1 || echo '0')"
    echo "Sccache: $(du -sh ${WORKSPACE}/.sccache 2>/dev/null | cut -f1 || echo '0')"
}

# Main monitoring logic
main() {
    AVAILABLE=$(get_available_space)
    USAGE=$(get_usage_percent)
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    echo "=== Disk Space Report @ $TIMESTAMP ==="
    echo "Available: ${AVAILABLE}GB"
    echo "Usage: ${USAGE}%"
    echo "Artifacts:"
    get_artifact_sizes

    # Check thresholds
    if [ "$AVAILABLE" -lt "$CRITICAL_THRESHOLD_GB" ]; then
        echo ""
        echo "🚨 CRITICAL: Only ${AVAILABLE}GB available!"
        echo "RECOMMENDED ACTIONS:"
        echo "  1. cargo clean --profile ci"
        echo "  2. rm -rf .sccache"
        echo "  3. docker system prune -af"

        # Notify swarm
        npx claude-flow@alpha hooks notify --message "🚨 CRITICAL: Disk space at ${AVAILABLE}GB (< ${CRITICAL_THRESHOLD_GB}GB). Immediate cleanup required!" 2>/dev/null || true

        return 2
    elif [ "$AVAILABLE" -lt "$WARNING_THRESHOLD_GB" ]; then
        echo ""
        echo "⚠️  WARNING: Only ${AVAILABLE}GB available"
        echo "Consider cleanup soon"

        # Notify swarm
        npx claude-flow@alpha hooks notify --message "⚠️ WARNING: Disk space at ${AVAILABLE}GB (< ${WARNING_THRESHOLD_GB}GB). Monitor closely." 2>/dev/null || true

        return 1
    else
        echo ""
        echo "✅ Status: HEALTHY (${AVAILABLE}GB available)"
        return 0
    fi
}

main "$@"
