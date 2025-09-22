# Production Deployment Guide

This comprehensive guide covers deploying RipTide Crawler in production environments, including cloud platforms, infrastructure setup, security hardening, and operational best practices.

## Production Architecture Overview

```
Internet
    ↓
┌─────────────────┐
│  Load Balancer  │ (SSL Termination, Rate Limiting)
│   (nginx/HAProxy)│
└─────────────────┘
    ↓
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  API Instance 1 │    │  API Instance 2 │    │  API Instance N │
│   (Container)   │    │   (Container)   │    │   (Container)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
    ↓                      ↓                      ↓
┌─────────────────┐    ┌─────────────────┐
│ Headless Svc 1  │    │ Headless Svc 2  │
│   (Container)   │    │   (Container)   │
└─────────────────┘    └─────────────────┘
    ↓
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Redis Primary   │    │ Redis Replica 1 │    │ Redis Replica 2 │
│   (Cluster)     │    │   (Cluster)     │    │   (Cluster)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
    ↓
┌─────────────────┐
│ Shared Storage  │ (NFS/S3/GCS for artifacts)
│   (Persistent)  │
└─────────────────┘
```

## Cloud Platform Deployments

### AWS Deployment

#### ECS with Fargate

```yaml
# ecs-task-definition.json
{
  "family": "riptide-crawler",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "2048",
  "memory": "4096",
  "executionRoleArn": "arn:aws:iam::ACCOUNT:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::ACCOUNT:role/riptideTaskRole",
  "containerDefinitions": [
    {
      "name": "riptide-api",
      "image": "riptide/api:v0.1.0",
      "cpu": 1024,
      "memory": 2048,
      "essential": true,
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "REDIS_URL",
          "value": "redis://riptide-redis.elasticache.amazonaws.com:6379"
        },
        {
          "name": "RIPTIDE_HEADLESS_SERVICE_URL",
          "value": "http://headless.riptide.local:9123"
        }
      ],
      "secrets": [
        {
          "name": "SERPER_API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:riptide/serper-api-key"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/riptide-crawler",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "api"
        }
      },
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:8080/health || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 60
      }
    },
    {
      "name": "riptide-headless",
      "image": "riptide/headless:v0.1.0",
      "cpu": 1024,
      "memory": 2048,
      "essential": true,
      "portMappings": [
        {
          "containerPort": 9123,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "CHROME_FLAGS",
          "value": "--no-sandbox --disable-dev-shm-usage --disable-gpu"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/riptide-crawler",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "headless"
        }
      }
    }
  ]
}
```

#### Terraform Infrastructure

```hcl
# main.tf
provider "aws" {
  region = var.aws_region
}

# VPC and Networking
resource "aws_vpc" "riptide_vpc" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "riptide-vpc"
  }
}

resource "aws_subnet" "private_subnets" {
  count             = 2
  vpc_id            = aws_vpc.riptide_vpc.id
  cidr_block        = "10.0.${count.index + 1}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name = "riptide-private-subnet-${count.index + 1}"
  }
}

resource "aws_subnet" "public_subnets" {
  count                   = 2
  vpc_id                  = aws_vpc.riptide_vpc.id
  cidr_block              = "10.0.${count.index + 10}.0/24"
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name = "riptide-public-subnet-${count.index + 1}"
  }
}

# Application Load Balancer
resource "aws_lb" "riptide_alb" {
  name               = "riptide-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb_sg.id]
  subnets            = aws_subnet.public_subnets[*].id

  enable_deletion_protection = false

  tags = {
    Name = "riptide-alb"
  }
}

# ECS Cluster
resource "aws_ecs_cluster" "riptide_cluster" {
  name = "riptide-cluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

# ElastiCache Redis
resource "aws_elasticache_subnet_group" "riptide_redis" {
  name       = "riptide-redis-subnet-group"
  subnet_ids = aws_subnet.private_subnets[*].id
}

resource "aws_elasticache_replication_group" "riptide_redis" {
  replication_group_id         = "riptide-redis"
  description                  = "Redis cluster for RipTide Crawler"

  node_type                    = "cache.r6g.large"
  port                         = 6379
  parameter_group_name         = "default.redis7"

  num_cache_clusters           = 2
  automatic_failover_enabled   = true
  multi_az_enabled            = true

  subnet_group_name            = aws_elasticache_subnet_group.riptide_redis.name
  security_group_ids           = [aws_security_group.redis_sg.id]

  at_rest_encryption_enabled   = true
  transit_encryption_enabled   = true

  tags = {
    Name = "riptide-redis"
  }
}

# RDS for metadata (optional)
resource "aws_db_instance" "riptide_postgres" {
  identifier = "riptide-postgres"

  engine              = "postgres"
  engine_version      = "15.4"
  instance_class      = "db.t3.medium"
  allocated_storage   = 100
  storage_encrypted   = true

  db_name  = "riptide"
  username = "riptide"
  password = var.db_password

  vpc_security_group_ids = [aws_security_group.rds_sg.id]
  db_subnet_group_name   = aws_db_subnet_group.riptide_db.name

  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"

  skip_final_snapshot = false
  final_snapshot_identifier = "riptide-postgres-final-snapshot"

  tags = {
    Name = "riptide-postgres"
  }
}

# S3 bucket for artifacts
resource "aws_s3_bucket" "riptide_artifacts" {
  bucket = "riptide-artifacts-${random_string.bucket_suffix.result}"
}

resource "aws_s3_bucket_versioning" "riptide_artifacts" {
  bucket = aws_s3_bucket.riptide_artifacts.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_encryption" "riptide_artifacts" {
  bucket = aws_s3_bucket.riptide_artifacts.id

  server_side_encryption_configuration {
    rule {
      apply_server_side_encryption_by_default {
        sse_algorithm = "AES256"
      }
    }
  }
}

# ECS Service
resource "aws_ecs_service" "riptide_service" {
  name            = "riptide-service"
  cluster         = aws_ecs_cluster.riptide_cluster.id
  task_definition = aws_ecs_task_definition.riptide_task.arn
  desired_count   = 3
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = aws_subnet.private_subnets[*].id
    security_groups  = [aws_security_group.ecs_sg.id]
    assign_public_ip = false
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.riptide_api.arn
    container_name   = "riptide-api"
    container_port   = 8080
  }

  depends_on = [aws_lb_listener.riptide_api]
}
```

### Google Cloud Platform (GKE)

#### Kubernetes Deployment

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: riptide-crawler

---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: riptide-config
  namespace: riptide-crawler
data:
  riptide.yml: |
    search:
      provider: serper
      api_key_env: SERPER_API_KEY
      country: us
      locale: en
    crawl:
      concurrency: 16
      timeout_ms: 20000
    redis:
      url: "redis://redis-service:6379"

---
# k8s/secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: riptide-secrets
  namespace: riptide-crawler
type: Opaque
data:
  serper-api-key: <base64-encoded-api-key>

---
# k8s/api-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riptide-api
  namespace: riptide-crawler
spec:
  replicas: 3
  selector:
    matchLabels:
      app: riptide-api
  template:
    metadata:
      labels:
        app: riptide-api
    spec:
      containers:
      - name: riptide-api
        image: riptide/api:v0.1.0
        ports:
        - containerPort: 8080
        env:
        - name: SERPER_API_KEY
          valueFrom:
            secretKeyRef:
              name: riptide-secrets
              key: serper-api-key
        - name: RIPTIDE_CONFIG_FILE
          value: /etc/riptide/riptide.yml
        - name: REDIS_URL
          value: redis://redis-service:6379
        volumeMounts:
        - name: config-volume
          mountPath: /etc/riptide
        - name: artifacts-volume
          mountPath: /data
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config-volume
        configMap:
          name: riptide-config
      - name: artifacts-volume
        persistentVolumeClaim:
          claimName: riptide-artifacts-pvc

---
# k8s/api-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: riptide-api-service
  namespace: riptide-crawler
spec:
  selector:
    app: riptide-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP

---
# k8s/headless-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riptide-headless
  namespace: riptide-crawler
spec:
  replicas: 2
  selector:
    matchLabels:
      app: riptide-headless
  template:
    metadata:
      labels:
        app: riptide-headless
    spec:
      containers:
      - name: riptide-headless
        image: riptide/headless:v0.1.0
        ports:
        - containerPort: 9123
        env:
        - name: CHROME_FLAGS
          value: "--no-sandbox --disable-dev-shm-usage --disable-gpu"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
        volumeMounts:
        - name: shm-volume
          mountPath: /dev/shm
      volumes:
      - name: shm-volume
        emptyDir:
          medium: Memory
          sizeLimit: 2Gi

---
# k8s/redis-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: riptide-crawler
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
        command:
        - redis-server
        - --appendonly
        - "yes"
        - --maxmemory
        - 1gb
        - --maxmemory-policy
        - allkeys-lru
        volumeMounts:
        - name: redis-data
          mountPath: /data
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
      volumes:
      - name: redis-data
        persistentVolumeClaim:
          claimName: redis-pvc

---
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: riptide-ingress
  namespace: riptide-crawler
  annotations:
    kubernetes.io/ingress.class: "gce"
    kubernetes.io/ingress.global-static-ip-name: "riptide-ip"
    networking.gke.io/managed-certificates: "riptide-ssl-cert"
    kubernetes.io/ingress.allow-http: "false"
spec:
  rules:
  - host: api.riptide.yourdomain.com
    http:
      paths:
      - path: /*
        pathType: ImplementationSpecific
        backend:
          service:
            name: riptide-api-service
            port:
              number: 80

---
# k8s/managed-certificate.yaml
apiVersion: networking.gke.io/v1
kind: ManagedCertificate
metadata:
  name: riptide-ssl-cert
  namespace: riptide-crawler
spec:
  domains:
    - api.riptide.yourdomain.com
```

### Azure Container Instances

#### ARM Template

```json
{
  "$schema": "https://schema.management.azure.com/schemas/2019-04-01/deploymentTemplate.json#",
  "contentVersion": "1.0.0.0",
  "parameters": {
    "containerGroupName": {
      "type": "string",
      "defaultValue": "riptide-crawler",
      "metadata": {
        "description": "Name for the container group"
      }
    },
    "serperApiKey": {
      "type": "securestring",
      "metadata": {
        "description": "Serper API key for search functionality"
      }
    }
  },
  "resources": [
    {
      "type": "Microsoft.ContainerInstance/containerGroups",
      "apiVersion": "2021-03-01",
      "name": "[parameters('containerGroupName')]",
      "location": "[resourceGroup().location]",
      "properties": {
        "containers": [
          {
            "name": "riptide-api",
            "properties": {
              "image": "riptide/api:v0.1.0",
              "ports": [
                {
                  "port": 8080,
                  "protocol": "TCP"
                }
              ],
              "environmentVariables": [
                {
                  "name": "SERPER_API_KEY",
                  "secureValue": "[parameters('serperApiKey')]"
                },
                {
                  "name": "REDIS_URL",
                  "value": "redis://localhost:6379"
                }
              ],
              "resources": {
                "requests": {
                  "cpu": 2,
                  "memoryInGB": 4
                }
              }
            }
          },
          {
            "name": "riptide-headless",
            "properties": {
              "image": "riptide/headless:v0.1.0",
              "ports": [
                {
                  "port": 9123,
                  "protocol": "TCP"
                }
              ],
              "environmentVariables": [
                {
                  "name": "CHROME_FLAGS",
                  "value": "--no-sandbox --disable-dev-shm-usage --disable-gpu"
                }
              ],
              "resources": {
                "requests": {
                  "cpu": 2,
                  "memoryInGB": 4
                }
              }
            }
          },
          {
            "name": "redis",
            "properties": {
              "image": "redis:7-alpine",
              "ports": [
                {
                  "port": 6379,
                  "protocol": "TCP"
                }
              ],
              "resources": {
                "requests": {
                  "cpu": 1,
                  "memoryInGB": 2
                }
              }
            }
          }
        ],
        "osType": "Linux",
        "ipAddress": {
          "type": "Public",
          "ports": [
            {
              "port": 8080,
              "protocol": "TCP"
            }
          ]
        },
        "restartPolicy": "Always"
      }
    }
  ],
  "outputs": {
    "containerIPv4Address": {
      "type": "string",
      "value": "[reference(resourceId('Microsoft.ContainerInstance/containerGroups', parameters('containerGroupName'))).ipAddress.ip]"
    }
  }
}
```

## Security Hardening

### SSL/TLS Configuration

#### Nginx SSL Proxy

```nginx
# /etc/nginx/sites-available/riptide
server {
    listen 80;
    server_name api.riptide.yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.riptide.yourdomain.com;

    # SSL Configuration
    ssl_certificate /etc/ssl/certs/riptide.crt;
    ssl_certificate_key /etc/ssl/private/riptide.key;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;

    # Modern configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;

    # HSTS
    add_header Strict-Transport-Security "max-age=63072000" always;

    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy "strict-origin-when-cross-origin";

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    # Proxy configuration
    location / {
        proxy_pass http://riptide_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;

        # Buffer settings
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
        proxy_busy_buffers_size 256k;
    }

    # Health check endpoint (no auth required)
    location /health {
        proxy_pass http://riptide_backend/health;
        access_log off;
    }
}

upstream riptide_backend {
    least_conn;
    server 127.0.0.1:8080 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:8081 max_fails=3 fail_timeout=30s;
    server 127.0.0.1:8082 max_fails=3 fail_timeout=30s;
}
```

### Authentication and Authorization

#### API Key Management

```yaml
# riptide.yml security configuration
api:
  auth:
    enabled: true
    method: api_key

    # API keys with permissions
    api_keys:
      - key: "${API_KEY_PRODUCTION}"
        name: "Production Client"
        permissions: ["crawl", "search", "metrics"]
        rate_limit: 10000  # requests per hour
        ip_whitelist:
          - "10.0.0.0/8"
          - "172.16.0.0/12"

      - key: "${API_KEY_MONITORING}"
        name: "Monitoring System"
        permissions: ["health", "metrics"]
        rate_limit: 1000

  # Rate limiting
  rate_limiting:
    enabled: true
    storage: redis

    global:
      requests_per_minute: 1000
      burst: 100

    per_ip:
      requests_per_minute: 100
      burst: 10

    per_key: true

  # CORS configuration
  cors:
    enabled: true
    origins:
      - "https://dashboard.yourdomain.com"
      - "https://app.yourdomain.com"
    methods: ["GET", "POST", "OPTIONS"]
    headers: ["Content-Type", "Authorization"]
    max_age: 86400
```

#### JWT Authentication (Alternative)

```yaml
api:
  auth:
    enabled: true
    method: jwt

    jwt:
      secret: "${JWT_SECRET}"
      algorithm: "HS256"
      expiration: 3600  # 1 hour
      issuer: "riptide-crawler"

      # Public key for verification (if using RS256)
      public_key_file: "/etc/ssl/jwt-public.pem"

  # OAuth2 integration (optional)
  oauth2:
    enabled: true
    provider: "auth0"  # or "google", "github"
    client_id: "${OAUTH2_CLIENT_ID}"
    client_secret: "${OAUTH2_CLIENT_SECRET}"
    redirect_uri: "https://api.riptide.yourdomain.com/auth/callback"
    scopes: ["openid", "profile", "email"]
```

### Network Security

#### Firewall Configuration

```bash
# UFW firewall rules
sudo ufw default deny incoming
sudo ufw default allow outgoing

# SSH access (change default port)
sudo ufw allow 2222/tcp

# HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Internal services (restrict to private networks)
sudo ufw allow from 10.0.0.0/8 to any port 6379  # Redis
sudo ufw allow from 10.0.0.0/8 to any port 9123  # Headless service

# Docker networks
sudo ufw allow in on docker0
sudo ufw allow in on br-+

sudo ufw --force enable
```

#### VPC Security Groups (AWS Example)

```hcl
# Security group for load balancer
resource "aws_security_group" "alb_sg" {
  name_prefix = "riptide-alb-"
  vpc_id      = aws_vpc.riptide_vpc.id

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# Security group for ECS tasks
resource "aws_security_group" "ecs_sg" {
  name_prefix = "riptide-ecs-"
  vpc_id      = aws_vpc.riptide_vpc.id

  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.alb_sg.id]
  }

  ingress {
    from_port = 9123
    to_port   = 9123
    protocol  = "tcp"
    self      = true
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# Security group for Redis
resource "aws_security_group" "redis_sg" {
  name_prefix = "riptide-redis-"
  vpc_id      = aws_vpc.riptide_vpc.id

  ingress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [aws_security_group.ecs_sg.id]
  }
}
```

## High Availability and Disaster Recovery

### Multi-Region Deployment

```yaml
# Global load balancer configuration (Cloudflare example)
global_load_balancer:
  pools:
    - name: "us-west"
      origins:
        - address: "us-west.api.riptide.yourdomain.com"
          weight: 1
          health_check:
            method: "GET"
            path: "/health"
            expected_codes: "200"

    - name: "us-east"
      origins:
        - address: "us-east.api.riptide.yourdomain.com"
          weight: 1
          health_check:
            method: "GET"
            path: "/health"
            expected_codes: "200"

  # Failover rules
  rules:
    - condition: "pool.us-west.healthy"
      action: "pool.us-west"
    - condition: "pool.us-east.healthy"
      action: "pool.us-east"
    - action: "maintenance_page"
```

### Backup Strategy

```bash
#!/bin/bash
# Production backup script

BACKUP_TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_BASE="/backups"
S3_BUCKET="s3://riptide-backups"

# Create backup directory
BACKUP_DIR="$BACKUP_BASE/$BACKUP_TIMESTAMP"
mkdir -p "$BACKUP_DIR"

# 1. Redis backup
echo "Backing up Redis..."
redis-cli -h $REDIS_HOST BGSAVE
redis-cli -h $REDIS_HOST LASTSAVE > "$BACKUP_DIR/redis_lastsave.txt"
redis-cli -h $REDIS_HOST --rdb "$BACKUP_DIR/redis_dump.rdb"

# 2. Application data backup
echo "Backing up application data..."
tar -czf "$BACKUP_DIR/artifacts.tar.gz" /data/artifacts/

# 3. Configuration backup
echo "Backing up configuration..."
cp -r /etc/riptide "$BACKUP_DIR/config"

# 4. Database backup (if using PostgreSQL)
if [ ! -z "$DATABASE_URL" ]; then
    echo "Backing up database..."
    pg_dump "$DATABASE_URL" | gzip > "$BACKUP_DIR/database.sql.gz"
fi

# 5. Upload to S3
echo "Uploading to S3..."
aws s3 sync "$BACKUP_DIR" "$S3_BUCKET/$BACKUP_TIMESTAMP/"

# 6. Cleanup old local backups (keep last 7 days)
find "$BACKUP_BASE" -type d -mtime +7 -exec rm -rf {} \;

# 7. Cleanup old S3 backups (keep last 30 days)
aws s3 ls "$S3_BUCKET/" | awk '$1 < "'$(date -d '30 days ago' '+%Y-%m-%d')'" {print $4}' | \
    xargs -I {} aws s3 rm --recursive "$S3_BUCKET/{}"

echo "Backup completed: $BACKUP_TIMESTAMP"
```

## Monitoring and Observability

### Comprehensive Monitoring Stack

```yaml
# docker-compose.monitoring.yml
version: '3.8'

services:
  # Prometheus
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - ./monitoring/alerts.yml:/etc/prometheus/alerts.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--web.enable-lifecycle'
      - '--web.enable-admin-api'

  # Grafana
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
      - GF_USERS_ALLOW_SIGN_UP=false
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro

  # Alertmanager
  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./monitoring/alertmanager.yml:/etc/alertmanager/alertmanager.yml:ro
      - alertmanager_data:/alertmanager

  # Loki for logs
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki-config.yml:/etc/loki/local-config.yaml:ro
      - loki_data:/tmp/loki

  # Promtail for log collection
  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/log:/var/log:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - ./monitoring/promtail-config.yml:/etc/promtail/config.yml:ro

  # Jaeger for distributed tracing
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"
      - "14268:14268"
    environment:
      - COLLECTOR_OTLP_ENABLED=true

volumes:
  prometheus_data:
  grafana_data:
  alertmanager_data:
  loki_data:
```

### Alerting Configuration

```yaml
# monitoring/alerts.yml
groups:
  - name: riptide_alerts
    rules:
      - alert: RipTideAPIDown
        expr: up{job="riptide-api"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "RipTide API is down"
          description: "RipTide API has been down for more than 1 minute"

      - alert: HighErrorRate
        expr: rate(riptide_http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} requests per second"

      - alert: HighMemoryUsage
        expr: (container_memory_usage_bytes / container_spec_memory_limit_bytes) > 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is above 80%"

      - alert: RedisDown
        expr: redis_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Redis is down"
          description: "Redis has been unreachable for more than 1 minute"

      - alert: SlowResponseTime
        expr: histogram_quantile(0.95, rate(riptide_http_request_duration_seconds_bucket[5m])) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow response times"
          description: "95th percentile response time is {{ $value }} seconds"
```

## Performance Optimization

### Load Testing

```bash
#!/bin/bash
# Load testing script using K6

cat > load_test.js << 'EOF'
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  scenarios: {
    constant_load: {
      executor: 'constant-vus',
      vus: 50,
      duration: '5m',
    },
    ramp_up: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 100 },
        { duration: '5m', target: 100 },
        { duration: '2m', target: 200 },
        { duration: '5m', target: 200 },
        { duration: '2m', target: 0 },
      ],
    },
  },
};

export default function () {
  let payload = JSON.stringify({
    urls: ['https://httpbin.org/html'],
    options: {
      concurrency: 5,
      cache_mode: 'read_through'
    }
  });

  let params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer ' + __ENV.API_KEY,
    },
  };

  let response = http.post(__ENV.API_URL + '/crawl', payload, params);

  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 10s': (r) => r.timings.duration < 10000,
  });

  sleep(1);
}
EOF

# Run load test
API_URL="https://api.riptide.yourdomain.com" \
API_KEY="your-api-key" \
k6 run load_test.js
```

### Performance Tuning

```yaml
# High-performance configuration
api:
  # Connection pooling
  http_client:
    pool_size: 100
    keep_alive_timeout: 30s
    connection_timeout: 5s

  # Request handling
  workers: 8  # CPU cores
  max_concurrent_requests: 1000
  request_timeout: 30s

crawl:
  # Optimal concurrency based on load testing
  concurrency: 64
  timeout_ms: 15000

  # Connection reuse
  keep_alive: true
  keep_alive_timeout: 30s

  # Resource limits
  max_response_mb: 10
  max_redirects: 3

extraction:
  # WASM optimization
  wasm_pool_size: 16
  wasm_timeout_seconds: 20

  # Content processing
  parallel_extraction: true
  max_parallel_extractions: 8

redis:
  # Connection optimization
  pool_size: 50
  pool_timeout_seconds: 1

  # Pipeline operations
  pipeline_size: 100

  # Compression for large values
  compression: true
  compression_threshold: 1024

dynamic:
  # Chrome optimization
  max_concurrent_sessions: 4
  session_pool_size: 8
  page_pool_enabled: true

  # Resource management
  chrome_memory_limit: "2GB"
  chrome_cpu_limit: "2.0"
```

## Cost Optimization

### Resource Right-Sizing

```yaml
# Cost-optimized AWS ECS task definition
{
  "family": "riptide-cost-optimized",
  "cpu": "1024",  # Reduced from 2048
  "memory": "2048",  # Reduced from 4096
  "containerDefinitions": [
    {
      "name": "riptide-api",
      "cpu": 512,
      "memory": 1024,
      "memoryReservation": 512
    },
    {
      "name": "riptide-headless",
      "cpu": 512,
      "memory": 1024,
      "memoryReservation": 512
    }
  ]
}
```

### Auto-Scaling Configuration

```hcl
# Auto-scaling for ECS service
resource "aws_appautoscaling_target" "riptide_target" {
  max_capacity       = 10
  min_capacity       = 2
  resource_id        = "service/${aws_ecs_cluster.riptide_cluster.name}/${aws_ecs_service.riptide_service.name}"
  scalable_dimension = "ecs:service:DesiredCount"
  service_namespace  = "ecs"
}

resource "aws_appautoscaling_policy" "riptide_up" {
  name               = "riptide-scale-up"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.riptide_target.resource_id
  scalable_dimension = aws_appautoscaling_target.riptide_target.scalable_dimension
  service_namespace  = aws_appautoscaling_target.riptide_target.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "ECSServiceAverageCPUUtilization"
    }
    target_value = 70.0
  }
}
```

This production deployment guide provides a comprehensive foundation for deploying RipTide Crawler at scale with proper security, monitoring, and operational practices.