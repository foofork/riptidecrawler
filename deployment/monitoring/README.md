# RipTide Monitoring Infrastructure

Production-ready monitoring stack for RipTide EventMesh using Prometheus, Grafana, and AlertManager.

## Components

- **Prometheus** (port 9090): Metrics collection and storage
- **Grafana** (port 3000): Visualization and dashboards
- **AlertManager** (port 9093): Alert routing and management
- **Node Exporter** (port 9100): System-level metrics

## Quick Start

### Prerequisites
- Docker and Docker Compose installed
- RipTide API running on port 8080 (or update `prometheus.yml`)
- At least 2GB available memory

### Starting the Stack

```bash
# From the deployment/monitoring directory
docker-compose -f docker-compose.monitoring.yml up -d

# Check service status
docker-compose -f docker-compose.monitoring.yml ps

# View logs
docker-compose -f docker-compose.monitoring.yml logs -f
```

### Accessing Services

- **Grafana**: http://localhost:3000
  - Username: `admin`
  - Password: `riptide_admin_change_me` (⚠️ CHANGE IN PRODUCTION!)

- **Prometheus**: http://localhost:9090
- **AlertManager**: http://localhost:9093

### Stopping the Stack

```bash
# Stop services
docker-compose -f docker-compose.monitoring.yml down

# Stop and remove volumes (deletes all metrics data)
docker-compose -f docker-compose.monitoring.yml down -v
```

## Configuration

### Prometheus Configuration

Edit `/workspaces/eventmesh/deployment/monitoring/prometheus/prometheus.yml` to:
- Add/remove scrape targets
- Adjust scrape intervals
- Configure service discovery

**Key scrape targets:**
- `riptide-api`: Main API server metrics (port 8080)
- `riptide-wasm`: WASM runtime metrics (port 8081)
- `node-exporter`: System metrics

### Alert Rules

Edit `/workspaces/eventmesh/deployment/monitoring/prometheus/alerts.yml` to configure:
- Alert thresholds
- Evaluation intervals
- Alert labels and annotations

**Default alerts:**
- Service down detection
- High memory/CPU usage
- Low disk space
- High response times
- Error rate monitoring
- WASM-specific alerts

### AlertManager Configuration

Edit `/workspaces/eventmesh/deployment/monitoring/alertmanager/alertmanager.yml` to:
- Configure notification channels (email, Slack, webhooks)
- Set up routing rules
- Define inhibit rules

**Notification setup (TODO):**
1. Configure SMTP settings for email alerts
2. Add Slack webhook URLs
3. Set up PagerDuty integration (if needed)

## Grafana Dashboards

### Pre-configured Dashboard
- **RipTide Overview**: System health, resource usage, request rates

### Creating Custom Dashboards
1. Log in to Grafana (http://localhost:3000)
2. Click "+" → "Dashboard"
3. Add panels with Prometheus queries
4. Save dashboard to `/workspaces/eventmesh/deployment/monitoring/grafana/dashboards/`

### Useful PromQL Queries

```promql
# API Request rate
rate(http_requests_total{job="riptide-api"}[5m])

# Memory usage percentage
(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100

# CPU usage percentage
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Active WASM instances
wasm_active_instances

# P95 response time
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

## Data Retention

- **Prometheus**: 30 days (configurable in `docker-compose.monitoring.yml`)
- **Grafana**: Persistent across restarts
- **AlertManager**: Persistent across restarts

### Backup Volumes

```bash
# Backup all monitoring data
docker run --rm -v riptide_prometheus-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/prometheus-backup.tar.gz -C /data .

docker run --rm -v riptide_grafana-data:/data -v $(pwd):/backup \
  alpine tar czf /backup/grafana-backup.tar.gz -C /data .
```

## Troubleshooting

### Prometheus Can't Reach RipTide API

**Problem**: `up{job="riptide-api"} == 0`

**Solutions:**
1. Ensure RipTide API is running: `curl http://localhost:8080/health`
2. Check Docker network connectivity: `docker network inspect monitoring_monitoring`
3. Verify `host.docker.internal` resolves (Linux may need `--add-host`)
4. Update `prometheus.yml` with correct target addresses

### Grafana Shows No Data

**Problem**: Dashboards are empty

**Solutions:**
1. Check Prometheus data source: Grafana → Configuration → Data Sources
2. Verify Prometheus is scraping: http://localhost:9090/targets
3. Check time range in dashboard (last 5 minutes vs last 24 hours)
4. Verify metrics exist: http://localhost:9090/graph

### Alerts Not Firing

**Problem**: No alerts in AlertManager

**Solutions:**
1. Check alert rules loaded: http://localhost:9090/rules
2. Verify AlertManager connectivity: http://localhost:9090/config
3. Check AlertManager logs: `docker logs riptide-alertmanager`
4. Test alert manually: Prometheus → Alerts → trigger condition

### High Memory Usage

**Problem**: Docker containers consuming too much memory

**Solutions:**
1. Reduce Prometheus retention: change `--storage.tsdb.retention.time=30d` to `7d`
2. Decrease scrape frequency in `prometheus.yml`
3. Limit container memory in `docker-compose.monitoring.yml`:
   ```yaml
   deploy:
     resources:
       limits:
         memory: 1G
   ```

## Security Hardening (Production)

### Critical Changes Before Production:

1. **Change Grafana password**:
   ```yaml
   environment:
     - GF_SECURITY_ADMIN_PASSWORD=STRONG_PASSWORD_HERE
   ```

2. **Enable authentication** for Prometheus and AlertManager:
   - Add reverse proxy with basic auth (nginx/traefik)
   - Use TLS certificates
   - Restrict network access

3. **Configure real notification channels**:
   - Set up SMTP for email alerts
   - Add Slack/PagerDuty webhooks
   - Test notification delivery

4. **Network isolation**:
   - Move to internal network only
   - Use reverse proxy for external access
   - Enable firewall rules

5. **Secrets management**:
   - Use Docker secrets or environment files
   - Never commit credentials to git
   - Rotate passwords regularly

## Next Steps

1. ✅ Infrastructure files created
2. ⏳ Wait for test fixes before deployment
3. 🔄 Deploy monitoring stack
4. 📊 Validate metrics collection
5. 🔔 Test alert notifications
6. 🎨 Create custom dashboards

## Support

For issues or questions:
- Check logs: `docker-compose logs -f [service]`
- Prometheus docs: https://prometheus.io/docs/
- Grafana docs: https://grafana.com/docs/
- AlertManager docs: https://prometheus.io/docs/alerting/latest/alertmanager/
