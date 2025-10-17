#!/bin/bash
# Monitor system resources during build/test operations
# Usage: ./scripts/monitor_resources.sh [duration_seconds] [interval_seconds]

set -e

DURATION=${1:-300}  # Default 5 minutes
INTERVAL=${2:-5}    # Default 5 seconds
OUTPUT_DIR="logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$OUTPUT_DIR"

echo "Starting resource monitoring for ${DURATION}s (interval: ${INTERVAL}s)"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Log file names
MEMORY_LOG="$OUTPUT_DIR/memory_usage_${TIMESTAMP}.log"
CPU_LOG="$OUTPUT_DIR/cpu_usage_${TIMESTAMP}.log"
DISK_LOG="$OUTPUT_DIR/disk_io_${TIMESTAMP}.log"
PROCESS_LOG="$OUTPUT_DIR/process_usage_${TIMESTAMP}.log"
SUMMARY_LOG="$OUTPUT_DIR/resource_summary_${TIMESTAMP}.log"

# Initialize logs
echo "Resource Monitoring Started: $(date)" | tee "$SUMMARY_LOG"
echo "Duration: ${DURATION}s, Interval: ${INTERVAL}s" | tee -a "$SUMMARY_LOG"
echo "" | tee -a "$SUMMARY_LOG"

# Function to monitor memory
monitor_memory() {
    echo "=== Memory Usage at $(date) ===" >> "$MEMORY_LOG"
    free -h >> "$MEMORY_LOG"
    echo "" >> "$MEMORY_LOG"
}

# Function to monitor CPU
monitor_cpu() {
    echo "=== CPU Usage at $(date) ===" >> "$CPU_LOG"
    if command -v mpstat >/dev/null 2>&1; then
        mpstat 1 1 >> "$CPU_LOG"
    else
        # Fallback to top
        top -bn1 | grep "Cpu(s)" >> "$CPU_LOG"
    fi
    echo "" >> "$CPU_LOG"
}

# Function to monitor disk I/O
monitor_disk() {
    echo "=== Disk I/O at $(date) ===" >> "$DISK_LOG"
    if command -v iostat >/dev/null 2>&1; then
        iostat -x 1 1 >> "$DISK_LOG"
    else
        # Fallback to basic disk usage
        df -h | grep -E "(Filesystem|/dev/)" >> "$DISK_LOG"
    fi
    echo "" >> "$DISK_LOG"
}

# Function to monitor top processes
monitor_processes() {
    echo "=== Top Processes at $(date) ===" >> "$PROCESS_LOG"
    ps aux --sort=-%cpu | head -n 11 >> "$PROCESS_LOG"
    echo "" >> "$PROCESS_LOG"
    echo "=== Top Memory Consumers ===" >> "$PROCESS_LOG"
    ps aux --sort=-%mem | head -n 11 >> "$PROCESS_LOG"
    echo "" >> "$PROCESS_LOG"
}

# Monitor for specified duration
START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION))

echo "Monitoring resources... (Ctrl+C to stop early)"

while [ $(date +%s) -lt $END_TIME ]; do
    monitor_memory
    monitor_cpu
    monitor_disk
    monitor_processes
    sleep "$INTERVAL"
done

# Generate summary
echo ""
echo "Monitoring complete. Generating summary..."
echo ""

echo "=== Resource Monitoring Summary ===" | tee -a "$SUMMARY_LOG"
echo "End Time: $(date)" | tee -a "$SUMMARY_LOG"
echo "" | tee -a "$SUMMARY_LOG"

# Memory summary
echo "## Memory Usage Summary" | tee -a "$SUMMARY_LOG"
echo "### Peak Memory Usage:" | tee -a "$SUMMARY_LOG"
if [ -f "$MEMORY_LOG" ]; then
    grep -A 1 "Mem:" "$MEMORY_LOG" | tail -n 1 | tee -a "$SUMMARY_LOG"
fi
echo "" | tee -a "$SUMMARY_LOG"

# CPU summary
echo "## CPU Usage Summary" | tee -a "$SUMMARY_LOG"
if [ -f "$CPU_LOG" ]; then
    echo "### Average CPU utilization:" | tee -a "$SUMMARY_LOG"
    if grep -q "mpstat" "$CPU_LOG"; then
        grep "Average:" "$CPU_LOG" | tail -n 1 | tee -a "$SUMMARY_LOG"
    else
        echo "See $CPU_LOG for details" | tee -a "$SUMMARY_LOG"
    fi
fi
echo "" | tee -a "$SUMMARY_LOG"

# Disk summary
echo "## Disk I/O Summary" | tee -a "$SUMMARY_LOG"
if [ -f "$DISK_LOG" ]; then
    echo "### Final disk usage:" | tee -a "$SUMMARY_LOG"
    df -h | grep -E "(Filesystem|/dev/root)" | tee -a "$SUMMARY_LOG"
fi
echo "" | tee -a "$SUMMARY_LOG"

# Process summary
echo "## Top Resource Consumers" | tee -a "$SUMMARY_LOG"
if [ -f "$PROCESS_LOG" ]; then
    echo "### Most CPU-intensive process:" | tee -a "$SUMMARY_LOG"
    grep -A 1 "USER" "$PROCESS_LOG" | tail -n 1 | awk '{print $11, "-", $3 "% CPU"}' | tee -a "$SUMMARY_LOG"
    echo "### Most memory-intensive process:" | tee -a "$SUMMARY_LOG"
    grep -A 1 "=== Top Memory" "$PROCESS_LOG" | grep -A 1 "USER" | tail -n 1 | awk '{print $11, "-", $4 "% Memory"}' | tee -a "$SUMMARY_LOG"
fi
echo "" | tee -a "$SUMMARY_LOG"

# Target directory size (if exists)
if [ -d "target" ]; then
    echo "## Build Artifacts" | tee -a "$SUMMARY_LOG"
    echo "Target directory size: $(du -sh target/ | cut -f1)" | tee -a "$SUMMARY_LOG"
    echo "" | tee -a "$SUMMARY_LOG"
fi

echo "=== Log Files ===" | tee -a "$SUMMARY_LOG"
echo "- Memory: $MEMORY_LOG" | tee -a "$SUMMARY_LOG"
echo "- CPU: $CPU_LOG" | tee -a "$SUMMARY_LOG"
echo "- Disk: $DISK_LOG" | tee -a "$SUMMARY_LOG"
echo "- Processes: $PROCESS_LOG" | tee -a "$SUMMARY_LOG"
echo "- Summary: $SUMMARY_LOG" | tee -a "$SUMMARY_LOG"
echo ""

echo "Resource monitoring complete!"
echo "Summary saved to: $SUMMARY_LOG"
