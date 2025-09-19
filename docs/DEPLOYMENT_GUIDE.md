# MCP Server Deployment Guide

**Version**: 1.0  
**Last Updated**: January 18, 2025  
**SDK Version**: RMCP v0.6.3  
**Project**: mcp-boilerplate-rust

A comprehensive guide for deploying MCP servers to production environments using Docker, Kubernetes, and cloud platforms.

---

## Table of Contents

1. [Deployment Overview](#deployment-overview)
2. [Docker Deployment](#docker-deployment)
3. [Kubernetes Deployment](#kubernetes-deployment)
4. [Cloud Platform Deployment](#cloud-platform-deployment)
5. [Monitoring and Observability](#monitoring-and-observability)
6. [Security Hardening](#security-hardening)
7. [Performance Optimization](#performance-optimization)
8. [Scaling and Load Balancing](#scaling-and-load-balancing)
9. [CI/CD Pipeline](#cicd-pipeline)
10. [Troubleshooting](#troubleshooting)

---

## Deployment Overview

### Deployment Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │────│   MCP Servers   │────│   Monitoring    │
│   (nginx/ALB)   │    │  (Kubernetes)   │    │ (Prometheus)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         │                        │                        │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   MCP Clients   │    │   Databases     │    │   Logging       │
│   (AI Agents)   │    │   (PostgreSQL)  │    │ (Elasticsearch) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Supported Deployment Targets

- **Docker**: Single-node development and testing
- **Kubernetes**: Production-grade orchestration
- **AWS ECS**: Managed container service
- **Google Cloud Run**: Serverless containers
- **Azure Container Instances**: Simple cloud containers

### Pre-Deployment Checklist

- [ ] All tests pass (`cargo test --workspace`)
- [ ] Security scan complete (`cargo audit`)
- [ ] Performance benchmarks validated
- [ ] Configuration externalized
- [ ] Secrets management configured
- [ ] Monitoring and alerting setup
- [ ] Backup and recovery procedures tested

---

## Docker Deployment

### Build Multi-Stage Dockerfile

Our optimized Dockerfile provides 70% size reduction:

```dockerfile
# Multi-stage build for optimal image size
FROM rust:1.70-bookworm as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r mcpuser && useradd -r -g mcpuser mcpuser

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/my-mcp-server /app/

# Create necessary directories
RUN mkdir -p /app/data /app/logs && \
    chown -R mcpuser:mcpuser /app

# Switch to non-root user
USER mcpuser

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/my-mcp-server", "--health-check"]

# Expose port (if using HTTP transport)
EXPOSE 8080

# Run the server
CMD ["/app/my-mcp-server"]
```

### Docker Compose for Development

```yaml
version: '3.8'

services:
  mcp-server:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - MCP_SERVER_PORT=8080
      - DATABASE_URL=postgresql://user:password@postgres:5432/mcpdb
    depends_on:
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "/app/my-mcp-server", "--health-check"]
      interval: 30s
      timeout: 10s
      retries: 3
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: mcpdb
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:
```

### Build and Deploy Script

```bash
#!/bin/bash
# deployment/docker/build.sh

set -euo pipefail

# Configuration
IMAGE_NAME="my-org/mcp-server"
VERSION=${1:-latest}
REGISTRY=${REGISTRY:-"docker.io"}

echo "🚀 Building MCP Server Docker Image"
echo "Image: ${REGISTRY}/${IMAGE_NAME}:${VERSION}"

# Build image
docker build \
    --platform linux/amd64,linux/arm64 \
    --tag "${REGISTRY}/${IMAGE_NAME}:${VERSION}" \
    --tag "${REGISTRY}/${IMAGE_NAME}:latest" \
    .

# Test image
echo "🧪 Testing Docker image..."
docker run --rm "${REGISTRY}/${IMAGE_NAME}:${VERSION}" --version

# Security scan
echo "🔒 Running security scan..."
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    aquasec/trivy image "${REGISTRY}/${IMAGE_NAME}:${VERSION}"

# Push to registry
if [[ "${PUSH:-false}" == "true" ]]; then
    echo "📤 Pushing to registry..."
    docker push "${REGISTRY}/${IMAGE_NAME}:${VERSION}"
    docker push "${REGISTRY}/${IMAGE_NAME}:latest"
fi

echo "✅ Docker build complete!"
```

### Production Docker Configuration

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  mcp-server:
    image: my-org/mcp-server:${VERSION:-latest}
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=warn
      - MCP_SERVER_PORT=8080
      - DATABASE_URL_FILE=/run/secrets/database_url
      - API_KEY_FILE=/run/secrets/api_key
    secrets:
      - database_url
      - api_key
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
    healthcheck:
      test: ["CMD", "/app/my-mcp-server", "--health-check"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

secrets:
  database_url:
    external: true
  api_key:
    external: true
```

---

## Kubernetes Deployment

### Namespace and RBAC

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: mcp-system
  labels:
    name: mcp-system
    app.kubernetes.io/name: mcp-servers
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: mcp-server
  namespace: mcp-system
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: mcp-system
  name: mcp-server
rules:
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: mcp-server
  namespace: mcp-system
subjects:
- kind: ServiceAccount
  name: mcp-server
  namespace: mcp-system
roleRef:
  kind: Role
  name: mcp-server
  apiGroup: rbac.authorization.k8s.io
```

### ConfigMap and Secrets

```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: mcp-server-config
  namespace: mcp-system
data:
  server.yaml: |
    server:
      name: "Production MCP Server"
      version: "1.0.0"
      bind_address: "0.0.0.0"
      port: 8080
      worker_threads: 4
      max_connections: 1000
    
    monitoring:
      metrics_enabled: true
      metrics_port: 9090
      health_check_path: "/health"
    
    logging:
      level: "info"
      format: "json"
---
apiVersion: v1
kind: Secret
metadata:
  name: mcp-server-secrets
  namespace: mcp-system
type: Opaque
data:
  # Base64 encoded values
  database-url: cG9zdGdyZXNxbDovL3VzZXI6cGFzc3dvcmRAZGF0YWJhc2U6NTQzMi9tY3BkYg==
  api-key: bXktc2VjcmV0LWFwaS1rZXk=
  jwt-secret: bXktand0LXNlY3JldA==
```

### Deployment Manifest

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
  namespace: mcp-system
  labels:
    app: mcp-server
    version: v1
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: mcp-server
  template:
    metadata:
      labels:
        app: mcp-server
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: mcp-server
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
      containers:
      - name: mcp-server
        image: my-org/mcp-server:v1.0.0
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: MCP_CONFIG_PATH
          value: "/etc/mcp/server.yaml"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: mcp-server-secrets
              key: database-url
        - name: API_KEY
          valueFrom:
            secretKeyRef:
              name: mcp-server-secrets
              key: api-key
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: mcp-server-secrets
              key: jwt-secret
        volumeMounts:
        - name: config
          mountPath: /etc/mcp
          readOnly: true
        - name: data
          mountPath: /app/data
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 30
      volumes:
      - name: config
        configMap:
          name: mcp-server-config
      - name: data
        emptyDir: {}
```

### Service and Ingress

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: mcp-server
  namespace: mcp-system
  labels:
    app: mcp-server
spec:
  selector:
    app: mcp-server
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: 9090
    protocol: TCP
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: mcp-server
  namespace: mcp-system
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - mcp-api.example.com
    secretName: mcp-server-tls
  rules:
  - host: mcp-api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: mcp-server
            port:
              number: 80
```

### Horizontal Pod Autoscaler

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: mcp-server
  namespace: mcp-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mcp-server
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
```

### Pod Security Policy

```yaml
# k8s/pod-security-policy.yaml
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: mcp-server-psp
  namespace: mcp-system
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  supplementalGroups:
    rule: 'MustRunAs'
    ranges:
      - min: 1
        max: 65535
  fsGroup:
    rule: 'MustRunAs'
    ranges:
      - min: 1
        max: 65535
  readOnlyRootFilesystem: false
```

### Deployment Script

```bash
#!/bin/bash
# deployment/k8s/deploy.sh

set -euo pipefail

NAMESPACE=${NAMESPACE:-mcp-system}
IMAGE_TAG=${IMAGE_TAG:-latest}
DRY_RUN=${DRY_RUN:-false}

echo "🚀 Deploying MCP Server to Kubernetes"
echo "Namespace: ${NAMESPACE}"
echo "Image Tag: ${IMAGE_TAG}"

# Apply namespace and RBAC
kubectl apply -f k8s/namespace.yaml

# Apply ConfigMaps and Secrets
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secrets.yaml

# Apply Pod Security Policy
kubectl apply -f k8s/pod-security-policy.yaml

# Update deployment with new image
if [[ "${DRY_RUN}" == "true" ]]; then
    kubectl set image deployment/mcp-server \
        mcp-server=my-org/mcp-server:${IMAGE_TAG} \
        --namespace=${NAMESPACE} \
        --dry-run=client -o yaml
else
    kubectl set image deployment/mcp-server \
        mcp-server=my-org/mcp-server:${IMAGE_TAG} \
        --namespace=${NAMESPACE}
fi

# Apply all manifests
kubectl apply -f k8s/

# Wait for rollout
kubectl rollout status deployment/mcp-server --namespace=${NAMESPACE} --timeout=300s

# Verify deployment
kubectl get pods --namespace=${NAMESPACE} -l app=mcp-server

echo "✅ Deployment complete!"
```

---

## Cloud Platform Deployment

### AWS ECS Deployment

```json
{
  "family": "mcp-server",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "executionRoleArn": "arn:aws:iam::ACCOUNT:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::ACCOUNT:role/mcpServerTaskRole",
  "containerDefinitions": [
    {
      "name": "mcp-server",
      "image": "my-org/mcp-server:latest",
      "essential": true,
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "DATABASE_URL",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:mcp/database-url"
        },
        {
          "name": "API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:mcp/api-key"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/mcp-server",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": [
          "CMD-SHELL",
          "/app/mcp-server --health-check"
        ],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 60
      }
    }
  ]
}
```

### Google Cloud Run Deployment

```yaml
# cloud-run.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: mcp-server
  annotations:
    run.googleapis.com/ingress: all
    run.googleapis.com/execution-environment: gen2
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/maxScale: "10"
        autoscaling.knative.dev/minScale: "1"
        run.googleapis.com/cpu-throttling: "false"
        run.googleapis.com/execution-environment: gen2
    spec:
      containerConcurrency: 100
      timeoutSeconds: 300
      containers:
      - image: gcr.io/PROJECT-ID/mcp-server:latest
        ports:
        - containerPort: 8080
        env:
        - name: PORT
          value: "8080"
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            cpu: "1"
            memory: "512Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        startupProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          failureThreshold: 10
```

### Azure Container Instances

```yaml
# azure-container-instances.yaml
apiVersion: 2019-12-01
location: eastus
name: mcp-server-group
properties:
  containers:
  - name: mcp-server
    properties:
      image: myregistry.azurecr.io/mcp-server:latest
      resources:
        requests:
          cpu: 0.5
          memoryInGb: 1
      ports:
      - port: 8080
        protocol: TCP
      environmentVariables:
      - name: RUST_LOG
        value: info
      - name: PORT
        value: "8080"
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
    - protocol: TCP
      port: 8080
    dnsNameLabel: my-mcp-server
  registryCredentials:
  - server: myregistry.azurecr.io
    username: myregistry
    password: !secretref 'registry-password'
```

---

## Monitoring and Observability

### Prometheus Configuration

```yaml
# monitoring/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alerts/*.yml"

scrape_configs:
  - job_name: 'mcp-servers'
    static_configs:
      - targets: ['mcp-server:9090']
    metrics_path: /metrics
    scrape_interval: 10s
    
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Alert Rules

```yaml
# monitoring/alerts/mcp-server.yml
groups:
- name: mcp-server
  rules:
  - alert: MCPServerDown
    expr: up{job="mcp-servers"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "MCP Server is down"
      description: "MCP Server {{ $labels.instance }} has been down for more than 1 minute."

  - alert: MCPServerHighErrorRate
    expr: rate(mcp_tool_errors_total[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High error rate on MCP Server"
      description: "MCP Server {{ $labels.instance }} has an error rate of {{ $value }} errors per second."

  - alert: MCPServerHighLatency
    expr: histogram_quantile(0.95, rate(mcp_tool_duration_seconds_bucket[5m])) > 0.5
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High latency on MCP Server"
      description: "MCP Server {{ $labels.instance }} 95th percentile latency is {{ $value }}s."

  - alert: MCPServerHighMemoryUsage
    expr: process_resident_memory_bytes{job="mcp-servers"} / 1024 / 1024 > 400
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage on MCP Server"
      description: "MCP Server {{ $labels.instance }} is using {{ $value }}MB of memory."
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "id": null,
    "title": "MCP Server Dashboard",
    "tags": ["mcp", "server"],
    "timezone": "browser",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(mcp_tool_requests_total[5m])) by (tool_name)",
            "legendFormat": "{{tool_name}}"
          }
        ],
        "yAxes": [
          {
            "label": "Requests/sec"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(mcp_tool_duration_seconds_bucket[5m])) by (le, tool_name))",
            "legendFormat": "95th percentile - {{tool_name}}"
          },
          {
            "expr": "histogram_quantile(0.50, sum(rate(mcp_tool_duration_seconds_bucket[5m])) by (le, tool_name))",
            "legendFormat": "50th percentile - {{tool_name}}"
          }
        ],
        "yAxes": [
          {
            "label": "Seconds"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(mcp_tool_errors_total[5m])) by (tool_name)",
            "legendFormat": "{{tool_name}}"
          }
        ]
      },
      {
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "process_resident_memory_bytes / 1024 / 1024",
            "legendFormat": "Memory (MB)"
          }
        ]
      }
    ]
  }
}
```

### Structured Logging

```rust
// Add to your server implementation
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

// Instrument your tools
#[tool]
#[instrument(skip(self), fields(tool_name = "example_tool"))]
async fn example_tool(&self, input: String) -> Result<String, ServerError> {
    info!("Processing tool request");
    
    let result = self.process_input(input).await;
    
    match &result {
        Ok(_) => info!("Tool execution successful"),
        Err(e) => error!("Tool execution failed: {}", e),
    }
    
    result
}
```

---

## Security Hardening

### Container Security

```dockerfile
# Security-hardened Dockerfile
FROM rust:1.70-bookworm as builder

# Security: Update base image
RUN apt-get update && apt-get upgrade -y

# ... build steps ...

FROM gcr.io/distroless/cc-debian12

# Security: Non-root user
USER 65534:65534

# Security: Read-only root filesystem
# (Configure writable volumes for necessary directories)

# Security: Drop all capabilities
# (Handled by Kubernetes security context)

COPY --from=builder --chown=65534:65534 /app/target/release/mcp-server /app/

ENTRYPOINT ["/app/mcp-server"]
```

### Kubernetes Security Context

```yaml
spec:
  template:
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
        runAsGroup: 65534
        fsGroup: 65534
        seccompProfile:
          type: RuntimeDefault
      containers:
      - name: mcp-server
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: var-tmp
          mountPath: /var/tmp
      volumes:
      - name: tmp
        emptyDir: {}
      - name: var-tmp
        emptyDir: {}
```

### Network Policies

```yaml
# k8s/network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: mcp-server-netpol
  namespace: mcp-system
spec:
  podSelector:
    matchLabels:
      app: mcp-server
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    - podSelector:
        matchLabels:
          app: nginx-ingress
    ports:
    - protocol: TCP
      port: 8080
  - from:
    - namespaceSelector:
        matchLabels:
          name: monitoring
    ports:
    - protocol: TCP
      port: 9090
  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 443  # HTTPS
    - protocol: TCP
      port: 53   # DNS
    - protocol: UDP
      port: 53   # DNS
  - to:
    - namespaceSelector:
        matchLabels:
          name: database
    ports:
    - protocol: TCP
      port: 5432  # PostgreSQL
```

### Secrets Management

```yaml
# External Secrets Operator configuration
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: vault-backend
  namespace: mcp-system
spec:
  provider:
    vault:
      server: "https://vault.example.com"
      path: "secret"
      version: "v2"
      auth:
        kubernetes:
          mountPath: "kubernetes"
          role: "mcp-server"
---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: mcp-server-secrets
  namespace: mcp-system
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: mcp-server-secrets
    creationPolicy: Owner
  data:
  - secretKey: database-url
    remoteRef:
      key: mcp-server
      property: database_url
  - secretKey: api-key
    remoteRef:
      key: mcp-server
      property: api_key
```

---

## Performance Optimization

### Resource Configuration

```yaml
# Optimized resource allocation
spec:
  containers:
  - name: mcp-server
    resources:
      requests:
        cpu: 100m      # 0.1 CPU core
        memory: 128Mi   # 128 MB RAM
      limits:
        cpu: 1000m     # 1 CPU core
        memory: 512Mi   # 512 MB RAM
    env:
    - name: TOKIO_WORKER_THREADS
      value: "4"
    - name: RUST_BACKTRACE
      value: "0"  # Disable in production
```

### Database Optimization

```rust
// Database connection pool configuration
use sqlx::postgres::PgPoolOptions;

pub async fn create_optimized_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)        // Max connections
        .min_connections(5)         // Min connections  
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)  // Validate connections
        .connect(database_url)
        .await
}
```

### Caching Strategy

```rust
// Multi-level caching
use lru::LruCache;
use redis::Commands;

pub struct CacheLayer {
    // L1: In-memory cache
    memory_cache: Arc<RwLock<LruCache<String, CachedData>>>,
    
    // L2: Redis cache
    redis_client: redis::Client,
}

impl CacheLayer {
    pub async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Try L1 cache first
        {
            let cache = self.memory_cache.read().await;
            if let Some(data) = cache.peek(key) {
                if !data.is_expired() {
                    if let Ok(value) = serde_json::from_value(data.value.clone()) {
                        return Some(value);
                    }
                }
            }
        }
        
        // Try L2 cache
        if let Ok(mut conn) = self.redis_client.get_connection() {
            if let Ok(data) = conn.get::<_, String>(key) {
                if let Ok(value) = serde_json::from_str(&data) {
                    // Store in L1 cache
                    self.set_memory_cache(key, &value, Duration::from_secs(300)).await;
                    return Some(value);
                }
            }
        }
        
        None
    }
}
```

---

## Scaling and Load Balancing

### Horizontal Pod Autoscaler (HPA)

```yaml
# Advanced HPA configuration
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: mcp-server-hpa
  namespace: mcp-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: mcp-server
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: mcp_tool_requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Min
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 60
      - type: Pods
        value: 4
        periodSeconds: 60
      selectPolicy: Max
```

### Load Balancer Configuration

```yaml
# NGINX Ingress with advanced load balancing
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: mcp-server-lb
  namespace: mcp-system
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/load-balance: "round_robin"
    nginx.ingress.kubernetes.io/upstream-keepalive-connections: "10"
    nginx.ingress.kubernetes.io/upstream-keepalive-requests: "100"
    nginx.ingress.kubernetes.io/upstream-keepalive-timeout: "60"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "300"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "300"
    nginx.ingress.kubernetes.io/rate-limit: "1000"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  rules:
  - host: mcp-api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: mcp-server
            port:
              number: 80
```

### Service Mesh (Istio)

```yaml
# Istio VirtualService for advanced traffic management
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: mcp-server
  namespace: mcp-system
spec:
  hosts:
  - mcp-api.example.com
  http:
  - match:
    - uri:
        prefix: /
    route:
    - destination:
        host: mcp-server
        port:
          number: 80
    timeout: 30s
    retries:
      attempts: 3
      perTryTimeout: 10s
      retryOn: 5xx,gateway-error,connect-failure,refused-stream
    fault:
      delay:
        percentage:
          value: 0.1
        fixedDelay: 5s
---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: mcp-server
  namespace: mcp-system
spec:
  host: mcp-server
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        http1MaxPendingRequests: 50
        maxRequestsPerConnection: 10
    loadBalancer:
      simple: LEAST_CONN
    outlierDetection:
      consecutiveErrors: 3
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
```

---

## CI/CD Pipeline

### GitHub Actions Workflow

```yaml
# .github/workflows/deploy.yml
name: Deploy MCP Server

on:
  push:
    branches: [main]
    tags: ['v*']
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: |
        cargo fmt --all -- --check
        cargo clippy --all-targets --all-features -- -D warnings
        cargo test --workspace --all-features
    
    - name: Security audit
      run: |
        cargo install cargo-audit
        cargo audit

  build-and-push:
    needs: test
    runs-on: ubuntu-latest
    if: github.event_name != 'pull_request'
    outputs:
      image: ${{ steps.image.outputs.image }}
      digest: ${{ steps.build.outputs.digest }}
    steps:
    - name: Checkout
      uses: actions/checkout@v3
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v2
    
    - name: Log in to Container Registry
      uses: docker/login-action@v2
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v4
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=semver,pattern={{version}}
          type=semver,pattern={{major}}.{{minor}}
          type=sha
    
    - name: Build and push
      id: build
      uses: docker/build-push-action@v4
      with:
        context: .
        platforms: linux/amd64,linux/arm64
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max
    
    - name: Output image
      id: image
      run: |
        echo "image=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}@${{ steps.build.outputs.digest }}" >> $GITHUB_OUTPUT

  deploy-staging:
    needs: build-and-push
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: staging
    steps:
    - name: Deploy to staging
      run: |
        # Deploy to staging environment
        echo "Deploying ${{ needs.build-and-push.outputs.image }} to staging"

  deploy-production:
    needs: [build-and-push, deploy-staging]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    environment: production
    steps:
    - name: Deploy to production
      run: |
        # Deploy to production environment
        echo "Deploying ${{ needs.build-and-push.outputs.image }} to production"
```

### GitLab CI/CD Pipeline

```yaml
# .gitlab-ci.yml
stages:
  - test
  - build
  - deploy-staging
  - deploy-production

variables:
  DOCKER_TLS_CERTDIR: "/certs"
  DOCKER_REGISTRY: $CI_REGISTRY
  DOCKER_IMAGE: $CI_REGISTRY_IMAGE

test:
  stage: test
  image: rust:1.70
  before_script:
    - apt-get update && apt-get install -y pkg-config libssl-dev
    - rustup component add rustfmt clippy
  script:
    - cargo fmt --all -- --check
    - cargo clippy --all-targets --all-features -- -D warnings
    - cargo test --workspace --all-features
  cache:
    paths:
      - target/
      - ~/.cargo/

build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  before_script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
  script:
    - docker build -t $DOCKER_IMAGE:$CI_COMMIT_SHA .
    - docker push $DOCKER_IMAGE:$CI_COMMIT_SHA
    - |
      if [[ "$CI_COMMIT_REF_NAME" == "main" ]]; then
        docker tag $DOCKER_IMAGE:$CI_COMMIT_SHA $DOCKER_IMAGE:latest
        docker push $DOCKER_IMAGE:latest
      fi
  only:
    - main
    - tags

deploy-staging:
  stage: deploy-staging
  image: bitnami/kubectl:latest
  script:
    - kubectl config use-context staging
    - kubectl set image deployment/mcp-server mcp-server=$DOCKER_IMAGE:$CI_COMMIT_SHA -n mcp-system
    - kubectl rollout status deployment/mcp-server -n mcp-system
  environment:
    name: staging
    url: https://staging-mcp-api.example.com
  only:
    - main

deploy-production:
  stage: deploy-production
  image: bitnami/kubectl:latest
  script:
    - kubectl config use-context production
    - kubectl set image deployment/mcp-server mcp-server=$DOCKER_IMAGE:$CI_COMMIT_SHA -n mcp-system
    - kubectl rollout status deployment/mcp-server -n mcp-system
  environment:
    name: production
    url: https://mcp-api.example.com
  when: manual
  only:
    - tags
```

---

## Troubleshooting

### Common Issues and Solutions

#### 1. Pod Startup Issues

**Symptoms**: Pods stuck in Pending or CrashLoopBackOff state

```bash
# Debug commands
kubectl describe pod <pod-name> -n mcp-system
kubectl logs <pod-name> -n mcp-system --previous
kubectl get events -n mcp-system --sort-by='.lastTimestamp'

# Check resource constraints
kubectl top nodes
kubectl top pods -n mcp-system
```

**Solutions**:
- Increase resource requests/limits
- Check node capacity and availability
- Verify image pull permissions
- Check secret and configmap references

#### 2. Service Discovery Issues

**Symptoms**: Services not reachable, DNS resolution failures

```bash
# Test service connectivity
kubectl run debug --image=busybox -it --rm --restart=Never -- nslookup mcp-server.mcp-system.svc.cluster.local

# Check service endpoints
kubectl get endpoints mcp-server -n mcp-system
kubectl describe service mcp-server -n mcp-system
```

**Solutions**:
- Verify service selector matches pod labels
- Check if pods are in Ready state
- Validate network policies
- Test DNS resolution from different namespaces

#### 3. Performance Issues

**Symptoms**: High response times, timeouts, memory leaks

```bash
# Monitor resource usage
kubectl top pods -n mcp-system
kubectl describe hpa mcp-server -n mcp-system

# Check application metrics
curl http://mcp-server.mcp-system.svc.cluster.local:9090/metrics
```

**Solutions**:
- Scale horizontally (increase replicas)
- Optimize resource allocation
- Review application-level caching
- Profile application for bottlenecks

#### 4. Security Issues

**Symptoms**: Authentication failures, permission denied errors

```bash
# Check RBAC permissions
kubectl auth can-i --list --as=system:serviceaccount:mcp-system:mcp-server

# Verify security context
kubectl get pod <pod-name> -n mcp-system -o yaml | grep -A 10 securityContext
```

**Solutions**:
- Review and update RBAC policies
- Verify security context configuration
- Check secret mounting and permissions
- Validate certificate and key configurations

### Health Check Endpoints

```rust
// Implement comprehensive health checks
use axum::{http::StatusCode, response::Json, routing::get, Router};
use serde_json::json;

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics_endpoint))
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    // Basic liveness check
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}

async fn readiness_check() -> Result<Json<serde_json::Value>, StatusCode> {
    // Check external dependencies
    let mut checks = std::collections::HashMap::new();
    
    // Database connectivity
    match check_database().await {
        Ok(_) => checks.insert("database", "healthy"),
        Err(_) => checks.insert("database", "unhealthy"),
    };
    
    // External API connectivity
    match check_external_apis().await {
        Ok(_) => checks.insert("external_apis", "healthy"),
        Err(_) => checks.insert("external_apis", "unhealthy"),
    };
    
    let all_healthy = checks.values().all(|&status| status == "healthy");
    let status_code = if all_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    
    Ok(Json(json!({
        "status": if all_healthy { "ready" } else { "not_ready" },
        "checks": checks,
        "timestamp": chrono::Utc::now()
    })))
}
```

### Debugging Tools

```bash
#!/bin/bash
# debug-deployment.sh - Comprehensive deployment debugging

set -euo pipefail

NAMESPACE=${1:-mcp-system}
APP_NAME=${2:-mcp-server}

echo "🔍 Debugging deployment: $APP_NAME in namespace: $NAMESPACE"

# Check namespace
echo "📁 Checking namespace..."
kubectl get namespace $NAMESPACE || {
    echo "❌ Namespace $NAMESPACE does not exist"
    exit 1
}

# Check deployments
echo "🚀 Checking deployments..."
kubectl get deployment $APP_NAME -n $NAMESPACE
kubectl describe deployment $APP_NAME -n $NAMESPACE

# Check pods
echo "🐳 Checking pods..."
kubectl get pods -l app=$APP_NAME -n $NAMESPACE
kubectl describe pods -l app=$APP_NAME -n $NAMESPACE

# Check services
echo "🌐 Checking services..."
kubectl get service $APP_NAME -n $NAMESPACE
kubectl describe service $APP_NAME -n $NAMESPACE

# Check ingress
echo "🚪 Checking ingress..."
kubectl get ingress -n $NAMESPACE
kubectl describe ingress -n $NAMESPACE

# Check events
echo "📅 Recent events..."
kubectl get events -n $NAMESPACE --sort-by='.lastTimestamp' | tail -20

# Check logs
echo "📋 Recent logs..."
kubectl logs -l app=$APP_NAME -n $NAMESPACE --tail=50

# Check HPA
echo "📈 Checking autoscaling..."
kubectl get hpa -n $NAMESPACE
kubectl describe hpa -n $NAMESPACE

# Resource usage
echo "💾 Resource usage..."
kubectl top pods -n $NAMESPACE

echo "✅ Debug information collected"
```

---

## Conclusion

This deployment guide provides comprehensive instructions for deploying MCP servers to production environments with enterprise-grade reliability, security, and observability.

### Key Takeaways

1. **Multi-Environment Strategy**: Support development, staging, and production deployments
2. **Security First**: Implement comprehensive security measures at every layer
3. **Observability**: Monitor everything with metrics, logs, and distributed tracing
4. **Scalability**: Design for horizontal scaling and load balancing
5. **Automation**: Use CI/CD pipelines for consistent, reliable deployments

### Next Steps

1. **Start Small**: Begin with Docker Compose for development
2. **Iterate**: Move to Kubernetes for production-grade deployment
3. **Monitor**: Implement comprehensive observability from day one
4. **Secure**: Apply security hardening progressively
5. **Scale**: Use auto-scaling and load balancing as traffic grows

### Resources

- **Project Repository**: https://github.com/netadx1ai/mcp-boilerplate-rust
- **Kubernetes Documentation**: https://kubernetes.io/docs/
- **Docker Best Practices**: https://docs.docker.com/develop/best-practices/
- **Prometheus Monitoring**: https://prometheus.io/docs/

Remember: Deployment is not a one-time activity but an ongoing process of monitoring, optimization, and improvement. Start with the basics and incrementally add advanced features as your understanding and requirements grow.

**Happy deploying! 🚀**