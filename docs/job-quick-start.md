# Job Management - Quick Start Guide

## Installation

The job management system is built into the RipTide CLI. No additional setup required!

```bash
# Build the CLI
cargo build -p riptide-cli --release

# Or use in development
cargo run -p riptide-cli -- job-local --help
```

## 5-Minute Tutorial

### 1. Submit Your First Job

```bash
# Extract a single URL
riptide job-local submit --url https://example.com --strategy auto

# Output:
# Job submitted successfully: job_18b9c4d5e6f_a1b2c3d4
# View status: riptide job-local status --id job_18b9c4d5e6f
# View logs: riptide job-local logs --id job_18b9c4d5e6f
```

### 2. Check Job Status

```bash
# Using short ID (first 8 chars)
riptide job-local status --id job_18b9

# Output:
# Job: job_18b9c4d5e6f_a1b2c3d4
# Status: completed
# Progress: 1/1 (100%)
# Duration: 2.34s
```

### 3. View Results

```bash
riptide job-local results --id job_18b9

# Save to file
riptide job-local results --id job_18b9 --output results.json
```

### 4. Monitor Logs

```bash
# Follow logs in real-time
riptide job-local logs --id job_18b9 --follow

# View errors only
riptide job-local logs --id job_18b9 --level error
```

## Common Commands

### Submit Jobs

```bash
# Single URL
riptide job-local submit --url https://example.com

# Multiple URLs
riptide job-local submit \
  --url https://site1.com \
  --url https://site2.com \
  --url https://site3.com

# With metadata
riptide job-local submit \
  --url https://example.com \
  --name "my-extraction" \
  --priority high \
  --tags "production,important"

# Different strategies
riptide job-local submit --url https://example.com --strategy wasm
riptide job-local submit --url https://example.com --strategy css
riptide job-local submit --url https://example.com --strategy llm
```

### List Jobs

```bash
# All jobs
riptide job-local list

# Running jobs only
riptide job-local list --status running

# High priority jobs
riptide job-local list --priority high

# Tagged jobs
riptide job-local list --tag production

# Last 10 jobs
riptide job-local list --limit 10
```

### Monitor Progress

```bash
# One-time status check
riptide job-local status --id job_xxx

# Watch mode (auto-refresh)
riptide job-local status --id job_xxx --watch

# Detailed view
riptide job-local status --id job_xxx --detailed
```

### View Logs

```bash
# Last 100 lines
riptide job-local logs --id job_xxx

# Follow mode (tail -f)
riptide job-local logs --id job_xxx --follow

# Errors only
riptide job-local logs --id job_xxx --level error

# Search logs
riptide job-local logs --id job_xxx --grep "extraction"

# Last 200 lines
riptide job-local logs --id job_xxx --lines 200
```

### Manage Jobs

```bash
# Cancel running job
riptide job-local cancel --id job_xxx

# View statistics
riptide job-local stats

# Clean up old jobs (30+ days)
riptide job-local cleanup --days 30

# Preview cleanup (dry run)
riptide job-local cleanup --days 30 --dry-run

# Storage info
riptide job-local storage
```

## Output Formats

All commands support JSON output:

```bash
# JSON output
riptide job-local list -o json
riptide job-local status --id job_xxx -o json
riptide job-local stats -o json
```

## Job Storage

All jobs are stored locally:

```
~/.riptide/jobs/
├── job_18b9c4d5e6f_a1b2c3d4/
│   ├── metadata.json    # Job configuration
│   ├── logs.jsonl       # Execution logs
│   └── results.json     # Extraction results
└── ...
```

You can inspect these files directly:

```bash
# View job metadata
cat ~/.riptide/jobs/job_xxx/metadata.json | jq

# View logs
cat ~/.riptide/jobs/job_xxx/logs.jsonl

# View results
cat ~/.riptide/jobs/job_xxx/results.json | jq
```

## Best Practices

1. **Use Short IDs**: You only need the first 8 characters
   ```bash
   riptide job-local status --id job_18b9  # Instead of full ID
   ```

2. **Tag Important Jobs**: Makes filtering easier
   ```bash
   riptide job-local submit --url ... --tags "production,critical"
   riptide job-local list --tag production
   ```

3. **Set Priorities**: Helps organize your workload
   ```bash
   riptide job-local submit --url ... --priority high
   ```

4. **Regular Cleanup**: Keep storage manageable
   ```bash
   riptide job-local cleanup --days 30  # Monthly cleanup
   ```

5. **Watch Long Jobs**: Monitor progress in real-time
   ```bash
   riptide job-local status --id job_xxx --watch
   ```

## Troubleshooting

### Job Not Found

```bash
# List all jobs to find the right ID
riptide job-local list

# Use full ID if short ID is ambiguous
riptide job-local status --id job_18b9c4d5e6f_a1b2c3d4
```

### View Job Errors

```bash
# Check status for error message
riptide job-local status --id job_xxx --detailed

# View error logs
riptide job-local logs --id job_xxx --level error
```

### Clear All Jobs

```bash
# CAUTION: This deletes all job data
rm -rf ~/.riptide/jobs/*

# Or use cleanup with 0 days
riptide job-local cleanup --days 0
```

## Advanced Usage

### Batch Processing

```bash
# Submit batch job
riptide job-local submit \
  --url https://site1.com \
  --url https://site2.com \
  --url https://site3.com \
  --name "batch-extraction" \
  --priority high

# Monitor in watch mode
riptide job-local status --id job_xxx --watch
```

### Custom Workflows

```bash
# 1. Submit job
JOB_ID=$(riptide job-local submit --url https://example.com -o json | jq -r '.job_id')

# 2. Wait for completion
while [ $(riptide job-local status --id $JOB_ID -o json | jq -r '.status') != "completed" ]; do
  sleep 2
done

# 3. Get results
riptide job-local results --id $JOB_ID --output results.json
```

### Integration with Other Tools

```bash
# Export job stats to CSV
riptide job-local stats -o json | \
  jq -r '.by_status | to_entries[] | [.key, .value] | @csv'

# Find failed jobs
riptide job-local list --status failed -o json | \
  jq -r '.[] | .id'

# Monitor storage usage
watch -n 5 'riptide job-local storage'
```

## Next Steps

- Read the [full documentation](./job-management.md)
- Check out [extraction strategies](./extraction-strategies.md)
- Learn about [metrics integration](./metrics.md)
- Explore the [CLI overview](./cli-overview.md)

## Quick Reference

| Command | Description |
|---------|-------------|
| `submit` | Create new job |
| `list` | View all jobs |
| `status` | Check job progress |
| `logs` | View execution logs |
| `cancel` | Stop running job |
| `results` | Get extraction results |
| `stats` | View statistics |
| `cleanup` | Remove old jobs |
| `storage` | Storage information |

## Need Help?

```bash
# Command help
riptide job-local --help
riptide job-local submit --help
riptide job-local status --help

# Show all subcommands
riptide job-local help
```
