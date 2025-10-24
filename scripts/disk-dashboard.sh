#!/bin/bash
# Comprehensive Disk Space Dashboard
# Provides detailed view of disk usage for CI/CD monitoring

WORKSPACE="/workspaces/eventmesh"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║         DISK SPACE MONITORING DASHBOARD                       ║"
echo "║         EventMesh CI/CD Build Process                         ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Overall disk usage
echo "📊 FILESYSTEM OVERVIEW"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
df -h / | awk 'NR==1 {print "  "$0} NR==2 {print "  "$0}'
echo ""

# Available space with color coding
AVAIL_GB=$(df -BG / | awk 'NR==2 {print $4}' | sed 's/G//')
if [ "$AVAIL_GB" -lt 5 ]; then
    STATUS="🚨 CRITICAL"
    COLOR="RED"
elif [ "$AVAIL_GB" -lt 10 ]; then
    STATUS="⚠️  WARNING"
    COLOR="YELLOW"
else
    STATUS="✅ HEALTHY"
    COLOR="GREEN"
fi

echo "🎯 STATUS: $STATUS"
echo "   Available: ${AVAIL_GB}GB"
echo ""

# Build artifacts breakdown
echo "📦 BUILD ARTIFACTS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -d "${WORKSPACE}/target" ]; then
    echo "  target/            $(du -sh ${WORKSPACE}/target 2>/dev/null | cut -f1)"
fi
if [ -d "${WORKSPACE}/.sccache" ]; then
    echo "  .sccache/          $(du -sh ${WORKSPACE}/.sccache 2>/dev/null | cut -f1)"
fi
if [ -d "${WORKSPACE}/wasm" ]; then
    echo "  wasm/              $(du -sh ${WORKSPACE}/wasm 2>/dev/null | cut -f1)"
fi
echo ""

# Docker usage
echo "🐳 DOCKER RESOURCES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if command -v docker &> /dev/null; then
    docker system df 2>/dev/null | tail -n +2 | awk '{print "  "$0}'
else
    echo "  Docker not available"
fi
echo ""

# Top space consumers
echo "💾 TOP 5 SPACE CONSUMERS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
du -h --max-depth=2 "${WORKSPACE}" 2>/dev/null | sort -rh | head -5 | awk '{print "  "$0}'
echo ""

# Cleanup recommendations
echo "🧹 CLEANUP OPTIONS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ "$AVAIL_GB" -lt 5 ]; then
    echo "  🚨 IMMEDIATE ACTION REQUIRED:"
    echo "     cargo clean --profile ci        # Clean CI artifacts (~2-4GB)"
    echo "     rm -rf .sccache                 # Clear cache (~10GB)"
    echo "     docker system prune -af         # Remove Docker (~1GB)"
elif [ "$AVAIL_GB" -lt 10 ]; then
    echo "  ⚠️  RECOMMENDED:"
    echo "     cargo clean --profile ci        # Clean CI artifacts"
    echo "     rm -rf .sccache/*/f*            # Partial cache cleanup"
else
    echo "  ✅ No cleanup needed currently"
    echo "     Monitoring active - will alert if space drops below 10GB"
fi
echo ""

# Monitoring info
echo "🔍 MONITORING STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Script: /workspaces/eventmesh/scripts/monitor-disk-space.sh"
echo "  Metrics: /workspaces/eventmesh/.swarm/disk-space-metrics.json"
echo "  Check frequency: Every 5 minutes during builds"
echo "  Critical threshold: < 5GB"
echo "  Warning threshold: < 10GB"
echo ""

echo "╚════════════════════════════════════════════════════════════════╝"
echo "  Last updated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
echo "╚════════════════════════════════════════════════════════════════╝"
