# RipTide Deployment Documentation

Complete guide to deploying and distributing RipTide in production environments.

## 📚 Documentation Index

### Quick Start
- **[Quick Start Guide](../00-getting-started/QUICK_START.md)** - Get running in < 5 minutes (Binary, Docker, or Source)
- [**Docker Deployment Modes**](./docker-modes.md) - Minimal, Simple, Distributed

### Binary Distribution
- **[Distribution Summary](./DISTRIBUTION_SUMMARY.md)** - Executive overview of the distribution strategy
- **[Binary Distribution Architecture](./BINARY_DISTRIBUTION.md)** - Complete technical design and implementation
- **[Implementation Checklist](./IMPLEMENTATION_CHECKLIST.md)** - Step-by-step deployment guide

### Docker Deployment
- **[Docker Optimization](./DOCKER_OPTIMIZATION.md)** - Optimize builds, reduce image size, improve performance
- [**Docker Modes**](./docker-modes.md) - Minimal, Simple, Distributed configurations

### Version Management
- **[Version Management](./VERSION_MANAGEMENT.md)** - Versioning strategy, updates, and release process
- **[Release Template](../../.github/RELEASE_TEMPLATE.md)** - Template for creating new releases

### Configuration
- [Configuration Guide](../config/README.md) - Complete configuration reference
- [Environment Variables](./environment-variables.md) - All available env vars
- [TOML Configuration](./toml-configuration.md) - Config file reference

### Deployment Modes

#### 1. Minimal Mode
- **File**: `docker-compose.minimal.yml`
- **Resources**: ~440MB, 1 container
- **Use Case**: Development, CI/CD, testing
- [Full Documentation](./docker-modes.md#1-minimal-mode)

#### 2. Simple Mode
- **File**: `docker-compose.simple.yml`
- **Resources**: ~600MB, 2 containers
- **Use Case**: Development with cache, small production
- [Full Documentation](./docker-modes.md#2-simple-mode)

#### 3. Distributed Mode
- **File**: `docker-compose.yml`
- **Resources**: ~1.2GB, 3+ containers
- **Use Case**: Production, high-volume, JavaScript rendering
- [Full Documentation](./docker-modes.md#3-distributed-mode)

### Advanced Topics
- [Load Balancing](./load-balancing.md) - Scaling multiple instances
- [SSL/TLS Setup](./ssl-setup.md) - HTTPS configuration
- [Monitoring & Metrics](./monitoring.md) - Observability setup
- [Backup & Recovery](./backup-recovery.md) - Data protection
- [Security Best Practices](./security.md) - Hardening your deployment

### Platform-Specific Guides
- [Kubernetes Deployment](./kubernetes.md) - K8s manifests and Helm charts
- [AWS ECS](./aws-ecs.md) - Deploy on AWS ECS/Fargate
- [Google Cloud Run](./google-cloud-run.md) - Serverless deployment
- [Azure Container Instances](./azure-aci.md) - Deploy on Azure
- [DigitalOcean](./digitalocean.md) - Deploy on DO droplets/apps

### Troubleshooting
- [Common Issues](./troubleshooting.md) - Solutions to frequent problems
- [Performance Tuning](./performance-tuning.md) - Optimization guide
- [Debugging Guide](./debugging.md) - Debug containers and services

## 🚀 Quick Reference

### Start Commands
```bash
# Minimal (zero dependencies)
docker-compose -f docker-compose.minimal.yml up -d

# Simple (with Redis)
docker-compose -f docker-compose.simple.yml up -d

# Distributed (full production)
docker-compose up -d
```

### Health Checks
```bash
# API health
curl http://localhost:8080/health

# Redis health (simple/distributed)
docker-compose exec redis redis-cli ping

# All services
docker-compose ps
```

### Logs
```bash
# Follow all logs
docker-compose logs -f

# Specific service
docker-compose logs -f riptide-api

# Last 100 lines
docker-compose logs --tail=100
```

## 📊 Comparison Matrix

| Feature | Minimal | Simple | Distributed |
|---------|---------|--------|-------------|
| Redis | ❌ | ✅ | ✅ |
| Workers | ❌ | ❌ | ✅ |
| Browser | ❌ | ❌ | ✅ |
| Memory | ~440MB | ~600MB | ~1.2GB |
| Containers | 1 | 2 | 3+ |
| Use Case | Dev/Testing | Dev/Small Prod | Production |

## 🎯 Deployment Decision Tree

```
Start here: What's your use case?
│
├─ Development / Testing / CI/CD
│  └─ Use: docker-compose.minimal.yml
│     └─ Zero dependencies, fast startup
│
├─ Small production (< 1000 req/day)
│  ├─ Need cache persistence?
│  │  ├─ Yes → docker-compose.simple.yml
│  │  └─ No → docker-compose.minimal.yml
│  └─ Need JavaScript rendering?
│     └─ Yes → docker-compose.yml (distributed)
│
└─ Production / High-volume (> 1000 req/day)
   └─ Use: docker-compose.yml (distributed)
      ├─ Scale API instances: --scale riptide-api=N
      ├─ Add load balancer
      └─ Enable monitoring
```

## 🛡️ Security Checklist

Before deploying to production:

- [ ] Set strong API key (`RIPTIDE_API_KEY`)
- [ ] Enable authentication (`REQUIRE_AUTH=true`)
- [ ] Configure CORS origins (`CORS_ORIGINS`)
- [ ] Use HTTPS (reverse proxy)
- [ ] Restrict Redis access (firewall)
- [ ] Enable rate limiting
- [ ] Set up monitoring and alerts
- [ ] Configure backup strategy
- [ ] Review security headers
- [ ] Update to latest version

## 📞 Support

- **GitHub Issues**: https://github.com/ruvnet/riptide/issues
- **Documentation**: https://docs.riptide.dev
- **Community**: https://discord.gg/riptide
- **Security**: security@riptide.dev

---

**Last Updated**: 2025-11-12
**Version**: 2.0.0
