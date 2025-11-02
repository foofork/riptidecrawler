# Authentication Audit Logging

## Overview

Comprehensive audit logging for all authentication events in the Riptide API. Every authentication attempt (success, failure, or blocked) is logged with structured data for security monitoring, incident response, and compliance.

## Features

- **Structured Logging**: JSON-compatible format for easy log aggregation
- **Security First**: Never logs full API keys (only first 8 characters)
- **Complete Coverage**: All authentication events logged
- **IP Tracking**: Extracts client IP from X-Forwarded-For or X-Real-IP headers
- **Configurable**: Enable/disable via environment variables

## Configuration

### Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `ENABLE_AUTH_AUDIT_LOGGING` | boolean | `true` | Enable/disable audit logging |
| `AUTH_AUDIT_LOG_LEVEL` | string | `"info"` | Log level (info, warn, error) |

### Example Configuration

```bash
# Enable audit logging (default)
export ENABLE_AUTH_AUDIT_LOGGING=true

# Set audit log level
export AUTH_AUDIT_LOG_LEVEL=info

# Disable audit logging (not recommended)
export ENABLE_AUTH_AUDIT_LOGGING=false
```

## Log Format

All audit logs use structured tracing with the following fields:

### Common Fields (All Events)

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `event` | string | Event type | `auth_success`, `auth_failure`, `auth_blocked` |
| `timestamp` | ISO 8601 | Event timestamp | `2025-11-02T12:34:56.789Z` |
| `ip` | string | Client IP address | `192.168.1.100` |
| `method` | string | HTTP method | `GET`, `POST`, `PUT`, `DELETE` |
| `path` | string | Request path | `/api/v1/extract` |

### Event-Specific Fields

#### Success Events (`auth_success`)

| Field | Description | Example |
|-------|-------------|---------|
| `key_prefix` | First 8 chars of API key | `sk-prod-` |

#### Failure Events (`auth_failure`)

| Field | Description | Example |
|-------|-------------|---------|
| `reason` | Failure reason | `invalid_key`, `missing_key`, `malformed` |

#### Blocked Events (`auth_blocked`)

| Field | Description | Example |
|-------|-------------|---------|
| `retry_after_secs` | Seconds until retry allowed | `60` |

## Event Types

### 1. Authentication Success (`auth_success`)

Logged when a valid API key is successfully authenticated.

**Example Log:**
```json
{
  "level": "INFO",
  "event": "auth_success",
  "timestamp": "2025-11-02T12:34:56.789Z",
  "ip": "192.168.1.100",
  "key_prefix": "sk-prod-",
  "method": "GET",
  "path": "/api/v1/extract",
  "message": "Authentication successful"
}
```

**Triggered By:**
- Valid API key in `X-API-Key` header
- Valid Bearer token in `Authorization` header

### 2. Authentication Failure (`auth_failure`)

Logged when authentication fails due to invalid, missing, or malformed credentials.

**Example Logs:**

**Invalid Key:**
```json
{
  "level": "WARN",
  "event": "auth_failure",
  "timestamp": "2025-11-02T12:35:00.123Z",
  "ip": "203.0.113.42",
  "reason": "invalid_key",
  "method": "POST",
  "path": "/api/v1/crawl",
  "message": "Authentication failed"
}
```

**Missing Key:**
```json
{
  "level": "WARN",
  "event": "auth_failure",
  "timestamp": "2025-11-02T12:35:15.456Z",
  "ip": "198.51.100.88",
  "reason": "missing_key",
  "method": "GET",
  "path": "/api/v1/health",
  "message": "Authentication failed"
}
```

**Triggered By:**
- Invalid API key provided
- Missing API key headers
- Malformed Authorization header

### 3. Authentication Blocked (`auth_blocked`)

Logged when a request is blocked due to rate limiting.

**Example Log:**
```json
{
  "level": "WARN",
  "event": "auth_blocked",
  "timestamp": "2025-11-02T12:36:00.789Z",
  "ip": "10.0.0.50",
  "method": "GET",
  "path": "/api/v1/test",
  "retry_after_secs": 60,
  "message": "Authentication blocked (rate limited)"
}
```

**Triggered By:**
- Rate limit exceeded for IP address
- Too many failed authentication attempts

## Security Considerations

### 1. API Key Protection

**CRITICAL**: Audit logs NEVER contain full API keys. Only the first 8 characters are logged.

**Examples:**
- Full key: `sk-prod-1234567890abcdef` → Logged: `sk-prod-`
- Full key: `my-secret-api-key-xyz` → Logged: `my-secre`
- Short key: `test` → Logged: `test` (full key if < 8 chars)

### 2. IP Address Sanitization

All IP addresses are sanitized before logging:
- Control characters removed
- Limited to 45 characters (max IPv6 length)
- Invalid characters filtered out

### 3. No Sensitive Data in Logs

Audit logs NEVER contain:
- Full API keys
- User credentials
- Request bodies
- Response data
- Stack traces with secrets

## Log Analysis

### Using grep

**Find all failed auth attempts from specific IP:**
```bash
grep 'auth_failure' application.log | grep 'ip=203.0.113.42'
```

**Find all successful auth attempts:**
```bash
grep 'auth_success' application.log
```

**Find all auth events in last hour:**
```bash
grep 'event=auth_' application.log | grep "$(date -u -d '1 hour ago' '+%Y-%m-%d')"
```

### Using jq

**Parse JSON logs and filter by event type:**
```bash
cat application.log | jq 'select(.event == "auth_failure")'
```

**Count failed auth attempts by IP:**
```bash
cat application.log | jq -r 'select(.event == "auth_failure") | .ip' | sort | uniq -c | sort -rn
```

**Find all auth failures with reason:**
```bash
cat application.log | jq 'select(.event == "auth_failure") | {timestamp, ip, reason, path}'
```

**Get auth events in time range:**
```bash
cat application.log | jq 'select(.timestamp >= "2025-11-02T12:00:00Z" and .timestamp <= "2025-11-02T13:00:00Z") | select(.event | startswith("auth_"))'
```

**Count auth events by type:**
```bash
cat application.log | jq -r 'select(.event | startswith("auth_")) | .event' | sort | uniq -c
```

### Using awk

**Extract IP addresses from failed attempts:**
```bash
awk '/auth_failure/ && /ip=/ {match($0, /ip=([^ ]+)/, arr); print arr[1]}' application.log
```

**Count requests by path:**
```bash
awk '/auth_success/ && /path=/ {match($0, /path=([^ ]+)/, arr); print arr[1]}' application.log | sort | uniq -c
```

## Common Queries

### Security Monitoring

**1. Find brute force attacks (multiple failures from same IP):**
```bash
cat application.log | jq -r 'select(.event == "auth_failure") | .ip' | sort | uniq -c | awk '$1 > 10 {print $2 " - " $1 " failures"}'
```

**2. Detect unusual access patterns (new IPs):**
```bash
cat application.log | jq -r 'select(.event == "auth_success") | .ip' | sort -u > current_ips.txt
diff previous_ips.txt current_ips.txt
```

**3. Monitor failed auth attempts over time:**
```bash
cat application.log | jq -r 'select(.event == "auth_failure") | .timestamp' | cut -d'T' -f1 | sort | uniq -c
```

### Incident Response

**1. Investigate specific incident (IP + time range):**
```bash
cat application.log | jq 'select(.ip == "203.0.113.42" and .timestamp >= "2025-11-02T12:00:00Z" and .timestamp <= "2025-11-02T13:00:00Z")'
```

**2. Find all events related to specific API key prefix:**
```bash
cat application.log | jq 'select(.key_prefix == "sk-prod-")'
```

**3. Analyze blocked requests:**
```bash
cat application.log | jq 'select(.event == "auth_blocked") | {timestamp, ip, path, retry_after_secs}'
```

### Performance Analysis

**1. Track authentication volume:**
```bash
cat application.log | jq -r 'select(.event == "auth_success") | .timestamp' | cut -d'T' -f2 | cut -d':' -f1 | sort | uniq -c
```

**2. Find most accessed endpoints:**
```bash
cat application.log | jq -r 'select(.event == "auth_success") | .path' | sort | uniq -c | sort -rn | head -10
```

**3. Monitor HTTP method distribution:**
```bash
cat application.log | jq -r 'select(.event | startswith("auth_")) | .method' | sort | uniq -c
```

## Log Retention

### Recommendations

- **Development**: 7-30 days
- **Staging**: 30-90 days
- **Production**: 90-365 days (or per compliance requirements)

### Compliance

Audit logs may be required for:
- SOC 2 compliance
- GDPR compliance (with IP anonymization)
- PCI DSS compliance
- HIPAA compliance

Consult with your legal and compliance teams for specific retention requirements.

## Integration with Log Aggregation Systems

### Elasticsearch

```bash
# Send logs to Elasticsearch
cat application.log | jq 'select(.event | startswith("auth_"))' | \
  while read line; do
    curl -X POST "http://localhost:9200/auth-logs/_doc" \
         -H 'Content-Type: application/json' \
         -d "$line"
  done
```

### Splunk

```bash
# Configure Splunk forwarder to monitor audit logs
[monitor:///var/log/riptide/application.log]
sourcetype = riptide:auth:audit
index = security
```

### CloudWatch Logs

```bash
# Use AWS CLI to push logs
aws logs put-log-events \
  --log-group-name /riptide/auth \
  --log-stream-name audit \
  --log-events "$(cat application.log | jq -r 'select(.event | startswith("auth_"))')"
```

## Troubleshooting

### No Audit Logs Appearing

**Check configuration:**
```bash
echo $ENABLE_AUTH_AUDIT_LOGGING  # Should be "true"
echo $AUTH_AUDIT_LOG_LEVEL       # Should be "info" or "warn"
```

**Check log level:**
```bash
# Ensure RUST_LOG includes tracing
export RUST_LOG=riptide_api=info,tower_http=debug
```

**Verify middleware is active:**
```bash
# Look for "Authentication check" debug logs
grep "Authentication check" application.log
```

### Missing IP Addresses

**Problem**: IP shows as "unknown" in logs.

**Solutions:**
1. Ensure `X-Forwarded-For` or `X-Real-IP` headers are set by load balancer/proxy
2. Configure nginx/Apache to add proper headers:

```nginx
# Nginx configuration
location / {
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Real-IP $remote_addr;
}
```

### Key Prefixes Too Short

**Problem**: Need more than 8 characters for audit trail.

**Solution**: Modify `get_key_prefix()` in `/workspaces/eventmesh/crates/riptide-api/src/middleware/auth.rs`:

```rust
fn get_key_prefix(key: &str) -> String {
    key.chars().take(12).collect()  // Change 8 to 12 or desired length
}
```

**Security Warning**: Longer prefixes increase risk of key leakage. Never log full keys.

## Best Practices

1. **Enable by Default**: Always enable audit logging in production
2. **Monitor Regularly**: Set up alerts for unusual patterns
3. **Rotate Logs**: Implement log rotation to manage disk space
4. **Encrypt at Rest**: Encrypt audit logs if they contain sensitive data
5. **Access Control**: Restrict access to audit logs to authorized personnel only
6. **Real-time Alerts**: Set up alerts for:
   - High failure rates from single IP
   - Unusual access patterns
   - Repeated blocked requests
7. **Regular Audits**: Review logs weekly for security incidents
8. **Backup Logs**: Maintain offsite backups for disaster recovery

## Example Alerting Rules

### Prometheus AlertManager

```yaml
groups:
  - name: auth_alerts
    rules:
      - alert: HighAuthFailureRate
        expr: rate(auth_failure_total[5m]) > 10
        annotations:
          summary: "High authentication failure rate detected"

      - alert: SuspiciousIPActivity
        expr: count by (ip) (auth_failure_total) > 50
        annotations:
          summary: "Suspicious activity from IP {{ $labels.ip }}"
```

### AWS CloudWatch Alarms

```bash
aws cloudwatch put-metric-alarm \
  --alarm-name auth-failure-spike \
  --metric-name AuthFailures \
  --namespace Riptide/Auth \
  --statistic Sum \
  --period 300 \
  --threshold 100 \
  --comparison-operator GreaterThanThreshold
```

## References

- [Riptide API Authentication](./authentication.md)
- [Security Best Practices](./security.md)
- [Tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
