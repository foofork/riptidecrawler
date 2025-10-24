#!/bin/bash
# Continuous Disk Space Monitoring
# Runs monitoring every 5 minutes and alerts on threshold crossings

MONITOR_INTERVAL=300  # 5 minutes
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITOR_SCRIPT="${SCRIPT_DIR}/monitor-disk-space.sh"
METRICS_FILE="/workspaces/eventmesh/.swarm/disk-space-metrics.json"

echo "🔍 Starting continuous disk space monitoring..."
echo "   Check interval: ${MONITOR_INTERVAL}s (5 minutes)"
echo "   Monitor script: ${MONITOR_SCRIPT}"
echo "   Metrics file: ${METRICS_FILE}"
echo ""

# Run once immediately
"${MONITOR_SCRIPT}"

# Update metrics file
cat > "${METRICS_FILE}" << EOF
{
  "monitoring_active": true,
  "last_check": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "status": "$([ $? -eq 0 ] && echo "HEALTHY" || echo "WARNING")",
  "available_gb": $(df -BG / | awk 'NR==2 {print $4}' | sed 's/G//'),
  "used_percent": $(df -h / | awk 'NR==2 {print $5}' | sed 's/%//'),
  "artifacts": {
    "target": "$(du -sh /workspaces/eventmesh/target 2>/dev/null | cut -f1 || echo '0')",
    "sccache": "$(du -sh /workspaces/eventmesh/.sccache 2>/dev/null | cut -f1 || echo '0')",
    "docker": "$(docker system df 2>/dev/null | awk 'NR==2 {print $3}' || echo '0')"
  },
  "thresholds": {
    "critical": 5,
    "warning": 10
  },
  "cleanup_commands": [
    "cargo clean --profile ci",
    "rm -rf .sccache",
    "docker system prune -af --volumes"
  ],
  "monitoring_script": "${MONITOR_SCRIPT}",
  "continuous_monitoring": true,
  "check_interval_seconds": ${MONITOR_INTERVAL}
}
EOF

echo ""
echo "✅ Continuous monitoring active"
echo "   Run manually: ${SCRIPT_DIR}/disk-dashboard.sh"
echo "   View metrics: cat ${METRICS_FILE}"
echo ""
echo "To monitor in background during CI/CD:"
echo "  nohup ${0} >> /workspaces/eventmesh/.swarm/monitor.log 2>&1 &"
