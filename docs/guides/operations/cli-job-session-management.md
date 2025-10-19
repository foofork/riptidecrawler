# RipTide CLI: Job and Session Management

## Overview

RipTide CLI now includes comprehensive job and session management capabilities for orchestrating async extraction workflows and maintaining stateful sessions across requests.

## Job Management

Job management enables asynchronous batch processing of extraction tasks with progress tracking, logging, and result retrieval.

### Commands

#### `riptide job submit`

Submit a new extraction job for asynchronous processing.

```bash
# Submit single URL
riptide job submit \
  --url https://example.com \
  --method wasm \
  --name "Example Extraction" \
  --priority high

# Submit multiple URLs in batch mode
riptide job submit \
  --url https://example1.com \
  --url https://example2.com \
  --url https://example3.com \
  --batch \
  --max-concurrent 10 \
  --output-dir ./results \
  --tags "production,high-priority"

# Submit with custom configuration
riptide job submit \
  --url https://example.com \
  --config ./job-config.json \
  --priority critical
```

**Options:**
- `--url`: URLs to extract (can specify multiple)
- `--method`: Extraction method (wasm, css, llm, regex, auto)
- `--name`: Job name for identification
- `--priority`: Job priority (low, medium, high, critical)
- `--output-dir`: Directory to save results
- `--batch`: Enable batch mode for parallel processing
- `--max-concurrent`: Maximum concurrent extractions (default: 5)
- `--tags`: Comma-separated tags for categorization
- `--config`: JSON file with additional configuration

#### `riptide job list`

List all jobs with optional filtering.

```bash
# List all jobs
riptide job list

# Filter by status
riptide job list --status running

# Filter by priority and tag
riptide job list --priority high --tag production

# Show recent jobs from last 24 hours
riptide job list --recent 24 --limit 100
```

**Options:**
- `--status`: Filter by status (pending, running, completed, failed, cancelled)
- `--priority`: Filter by priority (low, medium, high, critical)
- `--tag`: Filter by tag
- `--limit`: Maximum number of jobs to list (default: 50)
- `--recent`: Show jobs from last N hours

#### `riptide job status`

Check status of a specific job with optional watch mode.

```bash
# Get job status
riptide job status --job-id abc123 --detailed

# Watch mode (continuous updates)
riptide job status --job-id abc123 --watch --interval 2
```

**Options:**
- `--job-id`: Job ID to check
- `--detailed`: Show detailed information
- `--watch`: Continuously update status
- `--interval`: Update interval in seconds for watch mode (default: 2)

#### `riptide job logs`

View logs for a job with optional streaming.

```bash
# View recent logs
riptide job logs --job-id abc123 --lines 100

# Follow logs (tail -f style)
riptide job logs --job-id abc123 --follow

# Filter by log level
riptide job logs --job-id abc123 --level error

# Search logs
riptide job logs --job-id abc123 --grep "timeout"
```

**Options:**
- `--job-id`: Job ID
- `--follow`: Follow log output (tail -f style)
- `--lines`: Number of log lines to show (default: 100)
- `--level`: Filter by level (debug, info, warn, error)
- `--grep`: Search pattern in logs

#### `riptide job cancel`

Cancel a running job.

```bash
# Cancel specific job
riptide job cancel --job-id abc123

# Force cancel without cleanup
riptide job cancel --job-id abc123 --force

# Cancel all jobs with specific tag
riptide job cancel --tag "experimental" --force
```

**Options:**
- `--job-id`: Job ID to cancel
- `--force`: Force cancellation without cleanup
- `--tag`: Cancel multiple jobs by tag

#### `riptide job results`

Retrieve results from a completed job.

```bash
# Get results as JSON
riptide job results --job-id abc123 --format json

# Save results to file
riptide job results --job-id abc123 \
  --output ./results.json \
  --include-html

# Get results as markdown
riptide job results --job-id abc123 --format markdown
```

**Options:**
- `--job-id`: Job ID
- `--format`: Output format (json, text, markdown)
- `--output`: Output directory to save results
- `--include-html`: Include raw HTML in results

#### `riptide job retry`

Retry a failed job.

```bash
# Retry with default settings
riptide job retry --job-id abc123

# Retry with max attempts
riptide job retry --job-id abc123 --max-retries 5
```

**Options:**
- `--job-id`: Job ID to retry
- `--max-retries`: Maximum retry attempts (default: 3)

#### `riptide job stats`

Show job statistics with aggregations.

```bash
# Show stats for last 24 hours
riptide job stats

# Show stats for last 7 days
riptide job stats --range 7d

# Group by status
riptide job stats --range 30d --group-by status
```

**Options:**
- `--range`: Time range (1h, 24h, 7d, 30d, all)
- `--group-by`: Group by field (status, priority, method)

---

## Session Management

Session management enables stateful HTTP requests with persistent cookies, headers, and configuration across multiple extraction operations.

### Commands

#### `riptide session new`

Create a new session with optional initialization.

```bash
# Create basic session
riptide session new --name production

# Create with cookies and headers
riptide session new \
  --name authenticated \
  --description "Session with auth cookies" \
  --cookies ./cookies.json \
  --headers ./headers.json \
  --user-agent "Mozilla/5.0..." \
  --timeout 60

# Create with tags
riptide session new \
  --name testing \
  --tags "test,dev,temporary"
```

**Options:**
- `--name`: Session name (required)
- `--description`: Session description
- `--cookies`: JSON file with initial cookies
- `--headers`: JSON file with initial headers
- `--tags`: Comma-separated tags
- `--user-agent`: Custom user agent string
- `--timeout`: Session timeout in minutes (0 = no timeout)

#### `riptide session list`

List all sessions with filtering.

```bash
# List all sessions
riptide session list

# Show detailed information
riptide session list --detailed

# Filter by tag
riptide session list --tag production

# Show only active sessions
riptide session list --active
```

**Options:**
- `--tag`: Filter by tag
- `--active`: Show only active sessions
- `--detailed`: Show detailed information

#### `riptide session use`

Switch to a different session.

```bash
# Switch to existing session
riptide session use --name production

# Switch and create if doesn't exist
riptide session use --name testing --create
```

**Options:**
- `--name`: Session name to switch to
- `--create`: Create session if it doesn't exist

#### `riptide session current`

Show current active session.

```bash
# Show current session
riptide session current

# Show with details
riptide session current --detailed
```

**Options:**
- `--detailed`: Show detailed information

#### `riptide session export`

Export session state to file.

```bash
# Export to JSON
riptide session export \
  --name production \
  --output ./session-backup.json

# Export to YAML without cookies
riptide session export \
  --name production \
  --output ./session-backup.yaml \
  --format yaml \
  --include-cookies false

# Export headers only
riptide session export \
  --name production \
  --output ./headers.json \
  --include-cookies false \
  --include-headers true
```

**Options:**
- `--name`: Session name to export
- `--output`: Output file path
- `--format`: Export format (json, yaml)
- `--include-cookies`: Include cookies (default: true)
- `--include-headers`: Include headers (default: true)

#### `riptide session import`

Import session state from file.

```bash
# Import session
riptide session import --input ./session-backup.json

# Import with new name
riptide session import \
  --input ./session-backup.json \
  --name production-clone

# Overwrite existing
riptide session import \
  --input ./session-backup.json \
  --overwrite
```

**Options:**
- `--input`: Input file path
- `--name`: Override session name
- `--overwrite`: Overwrite existing session

#### `riptide session rm`

Remove a session.

```bash
# Remove session with confirmation
riptide session rm --name testing

# Force remove without confirmation
riptide session rm --name testing --force

# Remove all sessions with tag
riptide session rm --tag temporary --force
```

**Options:**
- `--name`: Session name to remove
- `--force`: Force removal without confirmation
- `--tag`: Remove all sessions matching tag

#### `riptide session update`

Update session metadata.

```bash
# Update description
riptide session update \
  --name production \
  --description "Updated production session"

# Add tags
riptide session update \
  --name production \
  --add-tags "stable,verified"

# Remove tags
riptide session update \
  --name production \
  --remove-tags "experimental"

# Update user agent and timeout
riptide session update \
  --name production \
  --user-agent "CustomBot/1.0" \
  --timeout 120
```

**Options:**
- `--name`: Session name
- `--description`: New description
- `--add-tags`: Add tags (comma-separated)
- `--remove-tags`: Remove tags (comma-separated)
- `--user-agent`: Update user agent
- `--timeout`: Update timeout in minutes

#### `riptide session add-cookies`

Add cookies to a session.

```bash
# Add basic cookie
riptide session add-cookies \
  --name production \
  --cookie-name session_id \
  --cookie-value "abc123"

# Add secure cookie with expiration
riptide session add-cookies \
  --name production \
  --cookie-name auth_token \
  --cookie-value "token123" \
  --domain example.com \
  --path "/" \
  --secure \
  --http-only \
  --expires "2025-12-31T23:59:59Z"
```

**Options:**
- `--name`: Session name
- `--cookie-name`: Cookie name
- `--cookie-value`: Cookie value
- `--domain`: Cookie domain
- `--path`: Cookie path (default: "/")
- `--secure`: Cookie is secure
- `--http-only`: Cookie is HTTP-only
- `--expires`: Cookie expiration (RFC3339 format)

#### `riptide session add-headers`

Add headers to a session.

```bash
# Add authorization header
riptide session add-headers \
  --name production \
  --header-name Authorization \
  --header-value "Bearer token123"

# Add custom headers
riptide session add-headers \
  --name production \
  --header-name X-Custom-Header \
  --header-value "custom-value"
```

**Options:**
- `--name`: Session name
- `--header-name`: Header name
- `--header-value`: Header value

#### `riptide session clone`

Clone an existing session.

```bash
# Clone session with all data
riptide session clone --from production --to staging

# Clone without cookies
riptide session clone \
  --from production \
  --to staging \
  --cookies false

# Clone headers only
riptide session clone \
  --from production \
  --to staging \
  --cookies false \
  --headers true
```

**Options:**
- `--from`: Source session name
- `--to`: New session name
- `--cookies`: Clone cookies (default: true)
- `--headers`: Clone headers (default: true)

#### `riptide session clear`

Clear session data.

```bash
# Clear cookies only
riptide session clear --name production --cookies

# Clear headers only
riptide session clear --name production --headers

# Clear all data
riptide session clear --name production --all
```

**Options:**
- `--name`: Session name
- `--cookies`: Clear cookies
- `--headers`: Clear headers
- `--all`: Clear all data

#### `riptide session stats`

Show session statistics.

```bash
# Show stats for specific session
riptide session stats --name production

# Show overall stats for all sessions
riptide session stats
```

**Options:**
- `--name`: Session name (optional, shows all if not specified)

---

## Architecture

### Job Management Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Job Management                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐         │
│  │  Submit  │───▶│  Queue   │───▶│ Process  │         │
│  └──────────┘    └──────────┘    └──────────┘         │
│       │               │                │                │
│       │               │                │                │
│  ┌────▼────┐    ┌────▼────┐    ┌─────▼─────┐         │
│  │  Store  │    │ Monitor │    │  Results  │         │
│  │ Metadata│    │Progress │    │  Storage  │         │
│  └─────────┘    └─────────┘    └───────────┘         │
│       │               │                │                │
│       └───────────────┴────────────────┘                │
│                       │                                  │
│                  ┌────▼────┐                            │
│                  │  Logs   │                            │
│                  └─────────┘                            │
└─────────────────────────────────────────────────────────┘
```

**Key Features:**
- Async job submission with priority queuing
- Batch processing with configurable concurrency
- Real-time progress tracking
- Structured logging with streaming
- Result persistence and retrieval
- Job lifecycle management (pending → running → completed/failed)

### Session Management Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Session Management                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐      ┌──────────────┐                │
│  │   Cookies    │      │   Headers    │                │
│  │   Storage    │      │   Storage    │                │
│  └──────┬───────┘      └──────┬───────┘                │
│         │                     │                          │
│         └──────────┬──────────┘                          │
│                    │                                     │
│              ┌─────▼──────┐                             │
│              │  Session   │                             │
│              │  Manager   │                             │
│              └─────┬──────┘                             │
│                    │                                     │
│         ┌──────────┼──────────┐                         │
│         │          │          │                         │
│    ┌────▼───┐ ┌───▼────┐ ┌──▼─────┐                   │
│    │ Persist│ │ Export │ │ Import │                   │
│    │  .json │ │ YAML   │ │ JSON   │                   │
│    └────────┘ └────────┘ └────────┘                   │
└─────────────────────────────────────────────────────────┘
```

**Key Features:**
- Persistent cookie and header storage
- Session lifecycle management
- Import/Export for backup and sharing
- Session switching and cloning
- Metadata tracking (requests, success rate)
- Timeout-based session expiration
- Tag-based organization

### Data Models

#### Job Model

```rust
pub struct Job {
    pub id: String,
    pub name: Option<String>,
    pub status: JobStatus,           // pending, running, completed, failed, cancelled
    pub priority: JobPriority,       // low, medium, high, critical
    pub urls: Vec<String>,
    pub method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: JobProgress,
    pub tags: Vec<String>,
    pub error: Option<String>,
    pub results_path: Option<String>,
}

pub struct JobProgress {
    pub total: u32,
    pub completed: u32,
    pub failed: u32,
    pub percentage: f32,
    pub current_url: Option<String>,
}
```

#### Session Model

```rust
pub struct Session {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub cookies: Vec<Cookie>,
    pub headers: HashMap<String, String>,
    pub tags: Vec<String>,
    pub user_agent: Option<String>,
    pub timeout_minutes: u64,
    pub metadata: SessionMetadata,
}

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<DateTime<Utc>>,
}

pub struct SessionMetadata {
    pub requests_count: u64,
    pub last_request_url: Option<String>,
    pub success_count: u64,
    pub error_count: u64,
}
```

---

## Usage Examples

### Example 1: Batch URL Extraction with Job Management

```bash
# 1. Submit batch job
riptide job submit \
  --url https://news1.com \
  --url https://news2.com \
  --url https://news3.com \
  --batch \
  --max-concurrent 5 \
  --name "News Extraction" \
  --priority high \
  --tags "news,batch" \
  --output-dir ./results

# 2. Monitor progress
riptide job status --job-id <job-id> --watch

# 3. View logs
riptide job logs --job-id <job-id> --follow

# 4. Get results
riptide job results --job-id <job-id> --output ./results.json
```

### Example 2: Authenticated Session Management

```bash
# 1. Create authenticated session
riptide session new \
  --name auth-session \
  --description "Session with authentication"

# 2. Add authentication cookie
riptide session add-cookies \
  --name auth-session \
  --cookie-name session_token \
  --cookie-value "abc123xyz" \
  --domain example.com \
  --secure \
  --http-only

# 3. Add authorization header
riptide session add-headers \
  --name auth-session \
  --header-name Authorization \
  --header-value "Bearer token123"

# 4. Switch to session
riptide session use --name auth-session

# 5. Use session for extraction
riptide extract --url https://example.com/protected

# 6. Export session for backup
riptide session export \
  --name auth-session \
  --output ./session-backup.json
```

### Example 3: Production Workflow

```bash
# 1. Create production session
riptide session new \
  --name production \
  --description "Production crawling session" \
  --tags "production,stable" \
  --timeout 120

# 2. Configure session
riptide session add-headers \
  --name production \
  --header-name X-API-Key \
  --header-value "prod-key-123"

# 3. Submit large batch job
riptide job submit \
  --url https://site1.com \
  --url https://site2.com \
  --url https://site3.com \
  --batch \
  --max-concurrent 10 \
  --priority high \
  --tags "production,batch-1" \
  --output-dir ./production-results

# 4. Monitor all production jobs
riptide job list --tag production --status running

# 5. View statistics
riptide job stats --range 24h --group-by status

# 6. Check session stats
riptide session stats --name production
```

---

## Storage

### Job Storage
Jobs are stored server-side via the RipTide API at `/api/v1/jobs/*`

### Session Storage
Sessions are stored locally at: `~/.riptide/sessions.json`

**Session File Format:**
```json
{
  "current_session": "production",
  "sessions": {
    "production": {
      "name": "production",
      "description": "Production session",
      "created_at": "2025-10-15T12:00:00Z",
      "updated_at": "2025-10-15T14:30:00Z",
      "last_used_at": "2025-10-15T14:30:00Z",
      "cookies": [
        {
          "name": "session_id",
          "value": "abc123",
          "domain": "example.com",
          "path": "/",
          "secure": true,
          "http_only": true,
          "expires": "2025-12-31T23:59:59Z"
        }
      ],
      "headers": {
        "Authorization": "Bearer token123",
        "X-Custom-Header": "value"
      },
      "tags": ["production", "stable"],
      "user_agent": "Mozilla/5.0...",
      "timeout_minutes": 120,
      "metadata": {
        "requests_count": 1250,
        "last_request_url": "https://example.com",
        "success_count": 1200,
        "error_count": 50
      }
    }
  }
}
```

---

## Best Practices

### Job Management
1. **Use Tags**: Organize jobs with descriptive tags for easy filtering
2. **Set Priorities**: Use priority levels to manage queue order
3. **Monitor Progress**: Use watch mode for long-running jobs
4. **Batch Processing**: Enable batch mode for processing multiple URLs
5. **Error Handling**: Check logs for failed jobs and use retry command
6. **Result Storage**: Specify output directories for organized results

### Session Management
1. **Session Naming**: Use descriptive names for easy identification
2. **Regular Backups**: Export sessions regularly for backup
3. **Timeout Management**: Set appropriate timeouts for auto-cleanup
4. **Security**: Be cautious when exporting sessions with sensitive cookies
5. **Tag Organization**: Use tags to group related sessions
6. **Session Cloning**: Clone sessions for testing before modifications

---

## API Endpoints

### Job Management Endpoints
- `POST /api/v1/jobs/submit` - Submit new job
- `GET /api/v1/jobs` - List jobs
- `GET /api/v1/jobs/{id}` - Get job status
- `GET /api/v1/jobs/{id}/logs` - Get job logs
- `POST /api/v1/jobs/{id}/cancel` - Cancel job
- `GET /api/v1/jobs/{id}/results` - Get job results
- `POST /api/v1/jobs/{id}/retry` - Retry job
- `GET /api/v1/jobs/stats` - Get job statistics

### Session Management
Session management is client-side only and does not require API endpoints.

---

## Future Enhancements

### Job Management
- [ ] Job scheduling and cron-like execution
- [ ] Job dependencies and workflows
- [ ] Job templates and presets
- [ ] Job result caching
- [ ] Job performance analytics
- [ ] Job notification webhooks

### Session Management
- [ ] Session encryption for sensitive data
- [ ] Session sharing across CLI instances
- [ ] Session versioning and history
- [ ] Session templates
- [ ] Cloud sync for sessions
- [ ] Session analytics and insights
