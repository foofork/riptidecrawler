# RipTide CLI - Deployment Checklist
**Version**: 2.0.0
**Date**: 2025-10-17

## Pre-Deployment Validation

### Code Quality
- [ ] All tests pass (188/188 expected)
- [ ] No critical compiler warnings (dead code warnings OK)
- [ ] Security audit complete
- [ ] Code review approved
- [ ] Performance benchmarks meet targets

### Build Artifacts
- [ ] Release binary built successfully
- [ ] Binary tested on target platform
- [ ] Dependencies verified (Rust 1.82+)
- [ ] WASM module available (if using direct mode)
- [ ] Checksums generated for artifacts

### Documentation
- [ ] README.md updated with current version
- [ ] CHANGELOG.md includes all changes
- [ ] API documentation current
- [ ] Configuration guide reviewed
- [ ] Migration guide available (if upgrading)

---

## Environment Configuration

### Required Environment Variables

#### Core Configuration
```bash
# Required for all deployments
export RIPTIDE_OUTPUT_DIR=/var/lib/riptide/output
export RIPTIDE_LOG_LEVEL=info  # or warn/error for production
export RIPTIDE_CLI_MODE=api_first  # or direct

# API Mode (if using API)
export RIPTIDE_API_URL=https://api.riptide.example.com
export RIPTIDE_API_KEY=<your-secure-api-key>

# Direct Mode (if not using API)
export RIPTIDE_WASM_PATH=/opt/riptide/riptide-extraction.wasm
```

#### Output Directories
```bash
# These default to ${RIPTIDE_OUTPUT_DIR}/<subdir> if not specified
export RIPTIDE_SCREENSHOTS_DIR=${RIPTIDE_OUTPUT_DIR}/screenshots
export RIPTIDE_HTML_DIR=${RIPTIDE_OUTPUT_DIR}/html
export RIPTIDE_PDF_DIR=${RIPTIDE_OUTPUT_DIR}/pdf
export RIPTIDE_DOM_DIR=${RIPTIDE_OUTPUT_DIR}/dom
export RIPTIDE_HAR_DIR=${RIPTIDE_OUTPUT_DIR}/har
export RIPTIDE_CACHE_DIR=${RIPTIDE_OUTPUT_DIR}/cache
export RIPTIDE_LOGS_DIR=${RIPTIDE_OUTPUT_DIR}/logs
```

#### Resource Limits
```bash
# Tune based on system resources
export RIPTIDE_MAX_CONCURRENT_RENDERS=10
export RIPTIDE_MAX_CONCURRENT_PDF=2
export RIPTIDE_MAX_CONCURRENT_WASM=4
export RIPTIDE_MEMORY_LIMIT_MB=2048
export RIPTIDE_MEMORY_MAX_PER_REQUEST_MB=256
```

#### Timeouts
```bash
# Tune based on workload
export RIPTIDE_RENDER_TIMEOUT=3
export RIPTIDE_PDF_TIMEOUT=30
export RIPTIDE_WASM_TIMEOUT=10
export RIPTIDE_HTTP_TIMEOUT=10
export RIPTIDE_GLOBAL_TIMEOUT=30
```

#### Rate Limiting
```bash
# Enable for production
export RIPTIDE_RATE_LIMIT_ENABLED=true
export RIPTIDE_RATE_LIMIT_RPS=1.5
export RIPTIDE_RATE_LIMIT_JITTER=0.1
export RIPTIDE_RATE_LIMIT_BURST_CAPACITY=3
```

#### Headless Browser Pool
```bash
# Tune based on workload and memory
export RIPTIDE_HEADLESS_POOL_SIZE=3
export RIPTIDE_HEADLESS_MIN_POOL_SIZE=1
export RIPTIDE_HEADLESS_IDLE_TIMEOUT=300
export RIPTIDE_HEADLESS_HEALTH_CHECK_INTERVAL=60
export RIPTIDE_HEADLESS_MAX_PAGES_PER_BROWSER=10
```

### Configuration Checklist
- [ ] All required variables set
- [ ] Output directories exist and writable
- [ ] Resource limits appropriate for system
- [ ] Timeouts tuned for workload
- [ ] Rate limiting enabled and configured
- [ ] API key secure (not in version control)
- [ ] Log level appropriate for environment

---

## System Requirements

### Hardware Requirements
- [ ] **CPU**: 4+ cores recommended
- [ ] **Memory**: 4GB+ RAM (8GB recommended)
- [ ] **Disk**: 10GB+ free space for cache and outputs
- [ ] **Network**: Stable internet connection

### Software Requirements
- [ ] **OS**: Linux (Ubuntu 20.04+ or equivalent)
- [ ] **Rust**: 1.82+ installed (for building)
- [ ] **Chrome/Chromium**: Latest version (for headless mode)
- [ ] **Git**: For version control

### Network Requirements
- [ ] Outbound HTTPS (443) allowed
- [ ] Access to target websites (if extracting)
- [ ] Access to API server (if using API mode)
- [ ] DNS resolution working

---

## Installation Steps

### 1. System Preparation
```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y build-essential pkg-config libssl-dev

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
```

### 2. Clone Repository
```bash
# Clone the repository
git clone <repository-url> /opt/riptide
cd /opt/riptide

# Checkout production tag
git checkout v2.0.0
```

### 3. Build Application
```bash
# Clean build
cargo clean

# Build release binary
cargo build --release

# Verify binary
ls -lh target/release/riptide-cli
./target/release/riptide-cli --version
```

### 4. Install Binary
```bash
# Option 1: System-wide installation
sudo cp target/release/riptide-cli /usr/local/bin/riptide-cli
sudo chmod +x /usr/local/bin/riptide-cli

# Option 2: User installation
mkdir -p ~/.local/bin
cp target/release/riptide-cli ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

### 5. Configure Application
```bash
# Create configuration from template
cp .env.example .env

# Edit configuration
nano .env  # or vim, etc.

# Create necessary directories
sudo mkdir -p /var/lib/riptide/output/{screenshots,html,pdf,dom,har,cache,logs}
sudo chown -R $USER:$USER /var/lib/riptide
```

### Installation Checklist
- [ ] System packages updated
- [ ] Dependencies installed
- [ ] Rust toolchain available
- [ ] Repository cloned
- [ ] Binary built successfully
- [ ] Binary installed to PATH
- [ ] Configuration file created
- [ ] Output directories created
- [ ] Permissions set correctly

---

## Post-Installation Validation

### 1. Basic Functionality
```bash
# Test 1: Version check
riptide-cli --version
# Expected: RipTide CLI v2.0.0

# Test 2: Help command
riptide-cli --help
# Expected: Usage information displayed

# Test 3: Health check
riptide-cli health
# Expected: Health status displayed
```

### 2. Direct Mode Test
```bash
# Test extraction in direct mode
riptide-cli extract --url https://example.com --direct
# Expected: Extracted content displayed
```

### 3. API Mode Test (if configured)
```bash
# Test extraction via API
riptide-cli extract --url https://example.com
# Expected: Extracted content via API
```

### 4. Cache Validation
```bash
# Test cache hit
riptide-cli extract --url https://example.com --direct
riptide-cli extract --url https://example.com --direct
# Expected: Second request faster (cache hit)
```

### 5. Output Validation
```bash
# Check output directories
ls -la $RIPTIDE_OUTPUT_DIR/
# Expected: Directories exist with correct permissions

# Check logs
cat $RIPTIDE_LOGS_DIR/riptide.log
# Expected: Log entries present, no errors
```

### Validation Checklist
- [ ] Version check passes
- [ ] Help command works
- [ ] Health check reports healthy
- [ ] Direct mode extraction works
- [ ] API mode extraction works (if enabled)
- [ ] Cache hit observed
- [ ] Output files created
- [ ] Log files created
- [ ] No errors in logs

---

## Monitoring Setup

### 1. Health Check Endpoint
```bash
# Configure health check monitoring
# Endpoint: GET /health
# Expected: 200 OK with JSON health status

# Example using systemd timer
sudo tee /etc/systemd/system/riptide-health-check.service << EOF
[Unit]
Description=RipTide Health Check
After=network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/riptide-cli health
User=riptide
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

sudo tee /etc/systemd/system/riptide-health-check.timer << EOF
[Unit]
Description=RipTide Health Check Timer

[Timer]
OnBootSec=5min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable riptide-health-check.timer
sudo systemctl start riptide-health-check.timer
```

### 2. Log Monitoring
```bash
# Configure log aggregation (example: rsyslog)
sudo tee /etc/rsyslog.d/50-riptide.conf << EOF
if \$programname == 'riptide-cli' then /var/log/riptide/application.log
& stop
EOF

sudo systemctl restart rsyslog
```

### 3. Performance Metrics
```bash
# Monitor key metrics:
# - Request latency
# - Cache hit rate
# - Error rate
# - Memory usage
# - CPU usage

# Example: Prometheus exporter (if available)
# Or use: journalctl -u riptide-cli -f
```

### 4. Alerting
Configure alerts for:
- [ ] Health check failures
- [ ] High error rates (>1%)
- [ ] Memory usage >80%
- [ ] Disk space <10% free
- [ ] API authentication failures

### Monitoring Checklist
- [ ] Health check endpoint configured
- [ ] Health check monitoring enabled
- [ ] Log aggregation configured
- [ ] Log rotation configured
- [ ] Performance metrics collection enabled
- [ ] Alert thresholds defined
- [ ] Alert notifications configured
- [ ] Dashboard created (if applicable)

---

## Security Hardening

### 1. File Permissions
```bash
# Secure configuration files
chmod 600 .env
chmod 600 /etc/riptide/config.toml  # if using config file

# Secure output directories
chmod 750 /var/lib/riptide
chmod 750 /var/lib/riptide/output
```

### 2. API Key Security
```bash
# Store API key in secure location
# DO NOT commit to version control
# Use environment variables or secret management

# Example: AWS Secrets Manager
export RIPTIDE_API_KEY=$(aws secretsmanager get-secret-value \
  --secret-id riptide-api-key --query SecretString --output text)

# Example: HashiCorp Vault
export RIPTIDE_API_KEY=$(vault kv get -field=api_key secret/riptide)
```

### 3. Network Security
```bash
# Enable firewall rules
sudo ufw allow 8080/tcp  # if running API server
sudo ufw allow 443/tcp   # outbound HTTPS
sudo ufw enable

# Configure TLS (if running API server)
export RIPTIDE_ENABLE_TLS=true
export RIPTIDE_TLS_CERT_PATH=/etc/ssl/certs/riptide.crt
export RIPTIDE_TLS_KEY_PATH=/etc/ssl/private/riptide.key
```

### 4. User Permissions
```bash
# Run as non-root user
sudo useradd -r -s /bin/false riptide
sudo chown -R riptide:riptide /var/lib/riptide
sudo chown -R riptide:riptide /opt/riptide

# Update systemd service (if using)
[Service]
User=riptide
Group=riptide
```

### Security Checklist
- [ ] Configuration files secured (600)
- [ ] Output directories secured (750)
- [ ] API key not in version control
- [ ] API key stored securely
- [ ] Firewall rules configured
- [ ] TLS enabled (if running API)
- [ ] Non-root user configured
- [ ] File ownership correct
- [ ] Network access restricted

---

## Rollback Plan

### 1. Pre-Deployment Backup
```bash
# Backup current binary
cp /usr/local/bin/riptide-cli /usr/local/bin/riptide-cli.backup.$(date +%Y%m%d)

# Backup configuration
cp .env .env.backup.$(date +%Y%m%d)

# Backup cache (optional)
tar -czf riptide-cache-backup-$(date +%Y%m%d).tar.gz $RIPTIDE_CACHE_DIR
```

### 2. Rollback Procedure
```bash
# Stop application (if running as service)
sudo systemctl stop riptide-cli

# Restore previous binary
sudo cp /usr/local/bin/riptide-cli.backup.YYYYMMDD /usr/local/bin/riptide-cli

# Restore configuration
cp .env.backup.YYYYMMDD .env

# Restart application
sudo systemctl start riptide-cli

# Verify rollback
riptide-cli --version
riptide-cli health
```

### 3. Database/Cache Rollback
```bash
# If cache corruption or issues
rm -rf $RIPTIDE_CACHE_DIR/*
# Cache will rebuild automatically

# If database issues (if applicable)
# Restore from backup
tar -xzf riptide-cache-backup-YYYYMMDD.tar.gz -C $RIPTIDE_CACHE_DIR
```

### Rollback Checklist
- [ ] Previous binary backed up
- [ ] Configuration backed up
- [ ] Cache backed up (optional)
- [ ] Rollback procedure tested
- [ ] Rollback communication plan ready
- [ ] Health check post-rollback
- [ ] Smoke tests post-rollback

---

## Production Deployment

### 1. Deployment Window
- [ ] Deployment scheduled during maintenance window
- [ ] Stakeholders notified
- [ ] Monitoring team alerted
- [ ] Support team briefed

### 2. Deployment Steps
```bash
# 1. Put application in maintenance mode (if applicable)
# touch /var/lib/riptide/MAINTENANCE

# 2. Stop application (if running)
# sudo systemctl stop riptide-cli

# 3. Deploy new binary
sudo cp target/release/riptide-cli /usr/local/bin/riptide-cli
sudo chmod +x /usr/local/bin/riptide-cli

# 4. Update configuration (if needed)
# Review and update .env

# 5. Run database migrations (if applicable)
# Not applicable for v2.0.0

# 6. Clear cache (if needed)
# rm -rf $RIPTIDE_CACHE_DIR/*

# 7. Start application
# sudo systemctl start riptide-cli

# 8. Remove maintenance mode
# rm /var/lib/riptide/MAINTENANCE

# 9. Verify deployment
riptide-cli --version
riptide-cli health
```

### 3. Post-Deployment Validation
```bash
# Smoke tests
./scripts/smoke-tests.sh  # Run comprehensive smoke tests

# Performance baseline
./scripts/performance-baseline.sh  # Verify performance

# Monitor logs
tail -f $RIPTIDE_LOGS_DIR/riptide.log

# Check metrics
# Monitor dashboard for anomalies
```

### Deployment Checklist
- [ ] Maintenance window scheduled
- [ ] Stakeholders notified
- [ ] Backup completed
- [ ] Application stopped
- [ ] New binary deployed
- [ ] Configuration updated
- [ ] Cache cleared (if needed)
- [ ] Application started
- [ ] Health check passes
- [ ] Smoke tests pass
- [ ] Performance baseline met
- [ ] Logs reviewed
- [ ] Metrics normal
- [ ] Monitoring active

---

## Post-Deployment

### 1. Validation Period
- [ ] Monitor for 1 hour post-deployment
- [ ] Check error rates
- [ ] Validate performance metrics
- [ ] Review user feedback (if applicable)

### 2. Communication
- [ ] Deployment success announced
- [ ] Known issues communicated
- [ ] Support documentation updated
- [ ] Monitoring team informed

### 3. Documentation
- [ ] Deployment log updated
- [ ] Runbook updated (if needed)
- [ ] Lessons learned documented
- [ ] Next deployment planned

### Post-Deployment Checklist
- [ ] 1-hour monitoring complete
- [ ] Error rates normal
- [ ] Performance metrics acceptable
- [ ] No critical issues reported
- [ ] Deployment success communicated
- [ ] Documentation updated
- [ ] Backup retention verified
- [ ] Rollback plan updated

---

## Emergency Contacts

### On-Call Team
- **Primary**: [Name] - [Phone] - [Email]
- **Secondary**: [Name] - [Phone] - [Email]
- **Manager**: [Name] - [Phone] - [Email]

### Escalation Path
1. On-call engineer (0-15 min)
2. Team lead (15-30 min)
3. Engineering manager (30-60 min)
4. CTO (60+ min)

---

## Approval

### Deployment Authorization
- [ ] **Tech Lead**: _________________________ Date: _______
- [ ] **DevOps Lead**: ______________________ Date: _______
- [ ] **Security Lead**: _____________________ Date: _______
- [ ] **Product Manager**: ___________________ Date: _______

### Post-Deployment Sign-Off
- [ ] **Deployment Engineer**: _______________ Date: _______
- [ ] **QA Lead**: __________________________ Date: _______
- [ ] **Operations Lead**: ___________________ Date: _______

---

**Document Version**: 1.0
**Last Updated**: 2025-10-17
**Next Review**: 2025-11-17
