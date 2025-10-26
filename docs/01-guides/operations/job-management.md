# RipTide Local Job Management System

The RipTide CLI includes a comprehensive local job management system that allows you to queue, track, and manage extraction jobs without requiring an external API server.

## Overview

The job management system provides:

- **Unique Job IDs**: Auto-generated unique identifiers for each job
- **Job Status Tracking**: Monitor jobs through their lifecycle (pending → running → completed/failed/cancelled)
- **Progress Monitoring**: Real-time progress tracking for batch extractions
- **Persistent Storage**: Jobs stored in `~/.riptide/jobs/` with metadata, logs, and results
- **Job History**: Full audit trail with timestamps and state transitions
- **Streaming Support**: Real-time log streaming for running jobs
- **Priority Queuing**: Support for job prioritization (low, medium, high, critical)
- **Metrics Integration**: Built-in integration with the CLI metrics system

## Architecture

### Directory Structure

```
~/.riptide/jobs/
├── job_1a2b3c4d_5e6f7g8h/
│   ├── metadata.json       # Job configuration and state
│   ├── logs.jsonl          # Job execution logs (JSON Lines format)
│   └── results.json        # Extraction results
├── job_9i0j1k2l_3m4n5o6p/
│   ├── metadata.json
│   ├── logs.jsonl
│   └── results.json
└── ...
```

### Components

1. **Job Types** (`src/job/types.rs`):
   - `Job`: Main job structure with metadata and state
   - `JobId`: Unique job identifier with timestamp and random component
   - `JobStatus`: Job lifecycle states
   - `JobPriority`: Priority levels for job scheduling
   - `JobProgress`: Progress tracking with completion percentage
   - `LogEntry`: Structured log entries with timestamps and levels

2. **Job Storage** (`src/job/storage.rs`):
   - Persistent storage backend using filesystem
   - JSON serialization for metadata and results
   - JSONL (JSON Lines) format for logs
   - Efficient querying and filtering
   - Storage cleanup utilities

3. **Job Manager** (`src/job/manager.rs`):
   - Job lifecycle orchestration
   - In-memory caching for active jobs
   - Progress tracking and updates
   - Log aggregation
   - Statistics and reporting

4. **CLI Commands** (`src/commands/job_local.rs`):
   - User-facing command interface
   - Interactive features (watch mode, log following)
   - Result formatting and export

## Commands

### Submit a Job

Submit URLs for extraction:

```bash
# Single URL
riptide job-local submit --url https://example.com --strategy auto

# Multiple URLs
riptide job-local submit \
  --url https://example.com \
  --url https://another.com \
  --strategy wasm \
  --name "my-extraction" \
  --priority high \
  --tags "production,important"

# With streaming output
riptide job-local submit \
  --url https://example.com \
  --strategy auto \
  --stream
```

**Options:**
- `--url`: URL(s) to extract (required, can specify multiple)
- `--strategy`: Extraction strategy (auto, wasm, css, llm, regex, etc.)
- `--name`: Optional job name for identification
- `--priority`: Priority level (low, medium, high, critical)
- `--tags`: Comma-separated tags for categorization
- `--stream`: Enable real-time log streaming

### List Jobs

View all jobs with optional filtering:

```bash
# List all jobs
riptide job-local list

# Filter by status
riptide job-local list --status running

# Filter by priority
riptide job-local list --priority high

# Filter by tag
riptide job-local list --tag production

# Limit results
riptide job-local list --limit 20

# JSON output
riptide job-local list -o json
```

**Output:**
```
Found 5 job(s)
┌──────────┬────────┬───────────┬──────────┬──────────┬─────────────────────┐
│ ID       │ Name   │ Status    │ Priority │ Progress │ Created             │
├──────────┼────────┼───────────┼──────────┼──────────┼─────────────────────┤
│ job_1a2b │ test   │ running   │ high     │ 5/10 (50%)│ 2024-10-16 10:30:00 │
│ job_3c4d │ -      │ completed │ medium   │ 8/8 (100%)│ 2024-10-16 09:15:00 │
└──────────┴────────┴───────────┴──────────┴──────────┴─────────────────────┘
```

### Check Job Status

Monitor job progress:

```bash
# View job status (supports short IDs)
riptide job-local status --id job_1a2b

# Detailed status with URLs
riptide job-local status --id job_1a2b --detailed

# Watch mode (auto-refresh every 2 seconds)
riptide job-local status --id job_1a2b --watch

# Custom refresh interval
riptide job-local status --id job_1a2b --watch --interval 5
```

**Output:**
```
Job: job_1a2b3c4d_5e6f7g8h
Name: my-extraction
Status: running
Priority: high
Strategy: wasm
Progress: 5/10 (50%)
Current: https://example.com/page5
Created: 2024-10-16 10:30:00
Started: 2024-10-16 10:30:05
```

### View Job Logs

Stream or view job execution logs:

```bash
# View last 100 log lines
riptide job-local logs --id job_1a2b

# Follow logs in real-time
riptide job-local logs --id job_1a2b --follow

# Filter by log level
riptide job-local logs --id job_1a2b --level error

# Search logs
riptide job-local logs --id job_1a2b --grep "extraction failed"

# Custom line limit
riptide job-local logs --id job_1a2b --lines 200
```

**Output:**
```
2024-10-16 10:30:05 INFO Job started
2024-10-16 10:30:06 INFO Processing URL: https://example.com
2024-10-16 10:30:08 INFO Extracted 1500 characters
2024-10-16 10:30:09 WARN Slow response time: 2.5s
```

### Cancel a Job

Stop a running job:

```bash
# Cancel job
riptide job-local cancel --id job_1a2b
```

### Get Job Results

Retrieve extraction results:

```bash
# View results in terminal
riptide job-local results --id job_1a2b

# Save to file
riptide job-local results --id job_1a2b --output results.json

# JSON output format
riptide job-local results --id job_1a2b -o json
```

### Job Statistics

View aggregated job statistics:

```bash
# Overall statistics
riptide job-local stats

# JSON output
riptide job-local stats -o json
```

**Output:**
```
Job Statistics

Total Jobs: 42
Average Duration: 12.34s
Success Rate: 95.2%

By Status:
┌───────────┬───────┐
│ Status    │ Count │
├───────────┼───────┤
│ completed │ 40    │
│ failed    │ 2     │
│ running   │ 0     │
└───────────┴───────┘

By Priority:
┌──────────┬───────┐
│ Priority │ Count │
├──────────┼───────┤
│ high     │ 15    │
│ medium   │ 20    │
│ low      │ 7     │
└──────────┴───────┘
```

### Clean Up Old Jobs

Delete completed jobs older than a specified period:

```bash
# Preview what would be deleted (dry run)
riptide job-local cleanup --days 30 --dry-run

# Delete jobs older than 30 days
riptide job-local cleanup --days 30

# Custom retention period
riptide job-local cleanup --days 7
```

### Storage Information

View storage usage:

```bash
# Storage statistics
riptide job-local storage
```

**Output:**
```
Job Storage Information

Base Directory: "/home/user/.riptide/jobs"
Total Jobs: 42
Total Size: 15.23 MB
```

## Job Lifecycle

```
┌─────────┐
│ Pending │ ──submit──> Job created and stored
└────┬────┘
     │
     ▼
┌─────────┐
│ Running │ ──execute─> Processing URLs
└────┬────┘
     │
     ├──success──> ┌───────────┐
     │             │ Completed │
     │             └───────────┘
     │
     ├──error───> ┌────────┐
     │            │ Failed │
     │            └────────┘
     │
     └──cancel──> ┌───────────┐
                  │ Cancelled │
                  └───────────┘
```

## Job Data Structures

### Job Metadata (metadata.json)

```json
{
  "id": "job_1a2b3c4d_5e6f7g8h",
  "name": "my-extraction",
  "status": "running",
  "priority": "high",
  "urls": [
    "https://example.com",
    "https://another.com"
  ],
  "strategy": "wasm",
  "created_at": "2024-10-16T10:30:00Z",
  "updated_at": "2024-10-16T10:30:10Z",
  "started_at": "2024-10-16T10:30:05Z",
  "completed_at": null,
  "progress": {
    "total": 2,
    "completed": 1,
    "failed": 0,
    "percentage": 50.0,
    "current_item": "https://another.com"
  },
  "tags": ["production", "important"],
  "error": null,
  "results_path": "/home/user/.riptide/jobs/job_1a2b3c4d/results.json",
  "log_path": "/home/user/.riptide/jobs/job_1a2b3c4d/logs.jsonl",
  "stream": true
}
```

### Log Entry (logs.jsonl)

Each line is a separate JSON object:

```json
{"timestamp":"2024-10-16T10:30:05Z","level":"Info","message":"Job started","url":null}
{"timestamp":"2024-10-16T10:30:06Z","level":"Info","message":"Processing URL","url":"https://example.com"}
{"timestamp":"2024-10-16T10:30:08Z","level":"Warn","message":"Slow response","url":"https://example.com"}
```

### Results (results.json)

```json
{
  "job_id": "job_1a2b3c4d_5e6f7g8h",
  "total_urls": 2,
  "successful": 2,
  "failed": 0,
  "extractions": [
    {
      "url": "https://example.com",
      "status": "success",
      "content": "Extracted content...",
      "metadata": {
        "title": "Example Page",
        "extraction_time_ms": 1234
      }
    }
  ]
}
```

## Integration with Metrics System

The job system integrates with the CLI's metrics collection:

- Job submission events
- Job completion metrics
- Success/failure rates
- Average job duration
- Resource usage tracking

Access metrics via:

```bash
# View metrics
riptide metrics show

# Live metrics monitoring
riptide metrics tail

# Export metrics
riptide metrics export --format json --output metrics.json
```

## Best Practices

1. **Use Descriptive Names**: Tag jobs with meaningful names for easy identification
2. **Set Appropriate Priorities**: Use priority levels to manage important extractions
3. **Monitor Progress**: Use watch mode for long-running jobs
4. **Regular Cleanup**: Periodically clean up old completed jobs
5. **Check Storage**: Monitor storage usage with large extraction batches
6. **Use Tags**: Tag jobs for easy filtering and organization
7. **Review Logs**: Check logs for errors and performance issues

## Advanced Usage

### Batch Processing

Process multiple URLs efficiently:

```bash
# Submit batch job
riptide job-local submit \
  --url https://site1.com \
  --url https://site2.com \
  --url https://site3.com \
  --strategy auto \
  --name "batch-extraction" \
  --priority high

# Monitor progress
riptide job-local status --id job_xxx --watch
```

### Error Handling

Handle failed extractions:

```bash
# Find failed jobs
riptide job-local list --status failed

# Check error details
riptide job-local status --id job_xxx --detailed

# Review error logs
riptide job-local logs --id job_xxx --level error
```

### Performance Monitoring

Track job performance:

```bash
# View statistics
riptide job-local stats

# Check individual job timing
riptide job-local status --id job_xxx

# Export metrics for analysis
riptide metrics export --output metrics.json
```

## API vs Local Jobs

RipTide supports two job management modes:

| Feature | API Jobs (`job`) | Local Jobs (`job-local`) |
|---------|------------------|--------------------------|
| Server Required | Yes | No |
| Execution | Server-side | Local CLI |
| Storage | Server database | `~/.riptide/jobs/` |
| Scalability | High | Limited by local resources |
| Use Case | Production, shared teams | Development, personal use |

Choose `job-local` for:
- Local development and testing
- Single-user scenarios
- Offline operation
- Full control over job data

Choose `job` (API) for:
- Production deployments
- Team collaboration
- Centralized job management
- High-volume processing

## Troubleshooting

### Job Not Found

If you get "Job not found" errors:

```bash
# List all jobs to find the correct ID
riptide job-local list

# Use the full job ID
riptide job-local status --id job_1a2b3c4d_5e6f7g8h
```

### Storage Issues

If you encounter storage problems:

```bash
# Check storage status
riptide job-local storage

# Clean up old jobs
riptide job-local cleanup --days 30

# Manually clear all jobs (caution!)
rm -rf ~/.riptide/jobs/*
```

### Corrupted Job Data

If job metadata is corrupted:

```bash
# Delete the specific job directory
rm -rf ~/.riptide/jobs/job_xxx

# Or use the cleanup command
riptide job-local cleanup --days 0
```

## Future Enhancements

Planned features:

- [ ] Job retry mechanism for failed extractions
- [ ] Job templates for common extraction patterns
- [ ] Job scheduling (cron-like)
- [ ] Job dependencies and workflows
- [ ] Resource limits and throttling
- [ ] Export/import job configurations
- [ ] Job notifications (webhooks, email)
- [ ] Distributed job execution
- [ ] Job result caching
- [ ] Advanced filtering and search

## See Also

- [CLI Overview](./cli-overview.md)
- [Metrics System](./metrics.md)
- [Extraction Strategies](./extraction-strategies.md)
- [API Documentation](./api.md)
