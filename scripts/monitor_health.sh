#!/bin/bash
# EventMesh Health Monitoring Script
# Usage: ./scripts/monitor_health.sh [environment] [interval]

set -e

# Configuration
ENVIRONMENT=${1:-local}
INTERVAL=${2:-30}  # seconds
LOG_DIR="logs"
ALERT_THRESHOLD=3  # consecutive failures before alerting

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$LOG_DIR"

# Environment-specific configuration
case "$ENVIRONMENT" in
    local)
        HEALTH_URL="http://localhost:8080/healthz"
        API_URL="http://localhost:8080/api/v1"
        METRICS_URL="http://localhost:8080/metrics"
        ;;
    dev)
        HEALTH_URL="http://dev.eventmesh.internal:8080/healthz"
        API_URL="http://dev.eventmesh.internal:8080/api/v1"
        METRICS_URL="http://dev.eventmesh.internal:8080/metrics"
        ;;
    staging)
        HEALTH_URL="http://staging.eventmesh.internal:8080/healthz"
        API_URL="http://staging.eventmesh.internal:8080/api/v1"
        METRICS_URL="http://staging.eventmesh.internal:8080/metrics"
        ;;
    production)
        HEALTH_URL="https://api.eventmesh.com/healthz"
        API_URL="https://api.eventmesh.com/api/v1"
        METRICS_URL="https://api.eventmesh.com/metrics"
        ;;
    *)
        echo -e "${RED}Unknown environment: $ENVIRONMENT${NC}"
        echo "Usage: $0 [local|dev|staging|production] [interval_seconds]"
        exit 1
        ;;
esac

LOG_FILE="$LOG_DIR/health_monitor_${ENVIRONMENT}_$(date +%Y%m%d).log"
ALERT_LOG="$LOG_DIR/health_alerts_${ENVIRONMENT}.log"

log_message() {
    local level=$1
    shift
    local message="$*"
    local timestamp=$(date "+%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

check_health() {
    local url=$1
    local response_code=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")
    echo "$response_code"
}

get_response_time() {
    local url=$1
    local time=$(curl -s -o /dev/null -w "%{time_total}" "$url" 2>/dev/null || echo "0")
    echo "$time"
}

check_metrics() {
    local url=$1
    local metrics=$(curl -s "$url" 2>/dev/null || echo "")
    if [ -n "$metrics" ]; then
        echo "1"
    else
        echo "0"
    fi
}

send_alert() {
    local message="$1"
    local timestamp=$(date "+%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] ALERT: $message" | tee -a "$ALERT_LOG"

    # Customize alert mechanism (Slack, email, PagerDuty, etc.)
    # Example: Send to Slack webhook
    # curl -X POST -H 'Content-type: application/json' \
    #   --data '{"text":"'"$message"'"}' \
    #   "$SLACK_WEBHOOK_URL"

    # Example: Send email
    # echo "$message" | mail -s "EventMesh Alert: $ENVIRONMENT" ops-team@company.com

    # For now, just log it
    echo -e "${RED}ALERT:${NC} $message"
}

# Initialize
echo -e "${BLUE}Starting health monitoring for $ENVIRONMENT environment${NC}"
echo "Health URL: $HEALTH_URL"
echo "Interval: ${INTERVAL}s"
echo "Log file: $LOG_FILE"
echo ""

log_message "INFO" "Health monitoring started for $ENVIRONMENT environment"
consecutive_failures=0

# Main monitoring loop
while true; do
    TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")

    # Health check
    HTTP_CODE=$(check_health "$HEALTH_URL")
    RESPONSE_TIME=$(get_response_time "$HEALTH_URL")

    # Check metrics endpoint
    METRICS_AVAILABLE=$(check_metrics "$METRICS_URL")

    # Evaluate health status
    if [ "$HTTP_CODE" = "200" ]; then
        consecutive_failures=0

        # Response time warning threshold (2 seconds)
        if (( $(echo "$RESPONSE_TIME > 2.0" | bc -l) )); then
            echo -e "[${YELLOW}WARN${NC}] [$TIMESTAMP] Health check passed but slow (${RESPONSE_TIME}s)"
            log_message "WARN" "Health check slow response: ${RESPONSE_TIME}s"
        else
            echo -e "[${GREEN}OK${NC}] [$TIMESTAMP] Health check passed (${RESPONSE_TIME}s)"
            log_message "INFO" "Health check OK - Response time: ${RESPONSE_TIME}s"
        fi

        # Log metrics availability
        if [ "$METRICS_AVAILABLE" = "1" ]; then
            log_message "INFO" "Metrics endpoint available"
        else
            log_message "WARN" "Metrics endpoint not available"
        fi

    else
        consecutive_failures=$((consecutive_failures + 1))
        echo -e "[${RED}FAIL${NC}] [$TIMESTAMP] Health check failed (HTTP $HTTP_CODE)"
        log_message "ERROR" "Health check failed - HTTP $HTTP_CODE"

        # Send alert after threshold
        if [ $consecutive_failures -ge $ALERT_THRESHOLD ]; then
            send_alert "EventMesh $ENVIRONMENT is down! $consecutive_failures consecutive failures (HTTP $HTTP_CODE)"
        fi
    fi

    # Additional checks for production
    if [ "$ENVIRONMENT" = "production" ]; then
        # Check API endpoint responsiveness
        API_CODE=$(check_health "${API_URL}/status")
        if [ "$API_CODE" != "200" ]; then
            log_message "WARN" "API status endpoint returned HTTP $API_CODE"
        fi
    fi

    # Status summary every 10 checks
    CHECK_COUNT=$((CHECK_COUNT + 1))
    if [ $((CHECK_COUNT % 10)) -eq 0 ]; then
        echo ""
        echo -e "${BLUE}=== Status Summary (Last 10 checks) ===${NC}"
        tail -n 10 "$LOG_FILE" | grep -E "(OK|FAIL|WARN)" | \
            awk '{print $3}' | sort | uniq -c
        echo ""
    fi

    sleep "$INTERVAL"
done
