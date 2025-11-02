# Disk Space Management Scripts

Quick reference for the disk space management tools added to EventMesh.

## 📚 Complete Documentation

See [/workspaces/eventmesh/docs/DISK-SPACE-MANAGEMENT.md](/workspaces/eventmesh/docs/DISK-SPACE-MANAGEMENT.md) for comprehensive documentation.

## 🚀 Quick Start

### Check Current Disk Status
```bash
./scripts/disk-monitor.sh
```

### Run Standard Cleanup (13-14GB savings)
```bash
./scripts/cleanup-disk.sh
```

### Preview What Will Be Cleaned
```bash
./scripts/cleanup-disk.sh --dry-run
```

### Aggressive Cleanup (14.5GB+ savings)
```bash
./scripts/cleanup-disk.sh --aggressive
```

### Smart Cleanup (Auto-selects strategy)
```bash
./scripts/smart-cleanup.sh
```

### Pre-Build Check
```bash
./scripts/pre-build-check.sh
```

## 📋 Available Scripts

| Script | Purpose | Expected Savings |
|--------|---------|------------------|
| `cleanup-disk.sh` | Comprehensive cleanup | 13-14GB (standard), 14.5GB+ (aggressive) |
| `pre-build-check.sh` | Validates disk space before builds | N/A (validation only) |
| `smart-cleanup.sh` | Auto-selects cleanup strategy | Varies by disk usage |
| `disk-monitor.sh` | Reports disk usage status | N/A (monitoring only) |

## 💡 NPM Commands

From the workspace root:
```bash
npm run clean              # Standard cleanup
npm run clean:aggressive   # Aggressive cleanup
npm run clean:preview      # Preview cleanup (dry-run)
npm run disk:check         # Check disk status
npm run disk:report        # Full disk report
npm run disk:smart         # Smart cleanup
```

## 🎯 Current Disk State

- **Total:** 63GB
- **Used:** 44GB (74%)
- **Free:** 16GB

### Critical Directories

- `target/debug/`: 14GB
  - `deps/`: 9.5GB
  - `incremental/`: 4GB
  - `build/`: 876MB
- `cli/node_modules/`: 89MB
- `playground/node_modules/`: 202MB

## ⚡ Quick Actions

**Before starting work:**
```bash
./scripts/disk-monitor.sh
```

**Before major builds:**
```bash
./scripts/pre-build-check.sh && cargo build
```

**End of day:**
```bash
./scripts/smart-cleanup.sh
```

**Emergency (low space):**
```bash
./scripts/cleanup-disk.sh --aggressive
```

## 🤖 Automation

- GitHub Actions workflow: `.github/workflows/disk-cleanup.yml`
- Runs daily at 2 AM UTC
- Manual trigger available via workflow_dispatch

## 📊 Exit Codes

| Code | Status | Meaning |
|------|--------|---------|
| 0 | ✅ Healthy | < 75% disk usage |
| 1 | ⚠️ Warning | 75-85% disk usage |
| 2 | ❌ Critical | > 85% disk usage |

## 🔗 See Also

- Full documentation: `/workspaces/eventmesh/docs/DISK-SPACE-MANAGEMENT.md`
- GitHub Actions: `.github/workflows/disk-cleanup.yml`
- Package scripts: `package.json`, `cli/package.json`, `playground/package.json`
