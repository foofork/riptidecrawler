#!/bin/bash
# ============================================================================
# RipTide API Health Check Script
# ============================================================================
# Used by Docker HEALTHCHECK to verify container health
# ============================================================================

set -e

# Health check endpoint (note: /healthz not /health)
HEALTH_URL="${RIPTIDE_API_HOST:-localhost}:${RIPTIDE_API_PORT:-8080}/healthz"

# Perform health check with timeout
if curl -f -s --max-time 5 "http://${HEALTH_URL}" >/dev/null 2>&1; then
    exit 0
else
    exit 1
fi
