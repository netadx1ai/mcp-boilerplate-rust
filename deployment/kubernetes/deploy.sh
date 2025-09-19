#!/bin/bash
# Kubernetes Deployment Script for MCP Servers
# Comprehensive deployment automation for all production MCP servers

set -euo pipefail

# Color output for better UX
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
NAMESPACE="mcp-servers"
VERSION="${VERSION:-0.3.0}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-mcp-servers}"

# MCP Server configurations
declare -A SERVERS=(
    ["news-data-server"]="8081"
    ["template-server"]="8082"
    ["analytics-server"]="8083"
    ["database-server"]="8084"
    ["api-gateway-server"]="8085"
    ["workflow-server"]="8086"
)

# Utility functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed or not in PATH"
        exit 1
    fi
    
    # Check cluster connectivity
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    # Check if namespace exists
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_warning "Namespace $NAMESPACE does not exist, will create it"
    fi
    
    log_success "Prerequisites check passed"
}

# Display usage information
usage() {
    cat << EOF
Kubernetes Deployment Script for MCP Servers

Usage: $0 [COMMAND] [OPTIONS]

Commands:
    deploy-all          Deploy all MCP servers to Kubernetes
    deploy <server>     Deploy specific server
    undeploy-all        Remove all MCP server deployments
    undeploy <server>   Remove specific server deployment
    update <server>     Update specific server deployment
    status              Show status of all deployments
    logs <server>       Show logs for specific server
    scale <server> <replicas>  Scale server to specified replicas
    restart <server>    Restart server deployment
    port-forward <server>      Port forward to server
    setup-namespace     Create namespace and common resources
    cleanup-namespace   Remove namespace and all resources
    generate-manifests  Generate all Kubernetes manifests
    validate            Validate all manifests
    health-check        Check health of all servers

Options:
    --namespace <ns>    Set Kubernetes namespace (default: mcp-servers)
    --version <ver>     Set image version (default: 0.3.0)
    --registry <reg>    Set Docker registry (default: mcp-servers)
    --dry-run           Show what would be deployed without executing
    --verbose           Enable verbose output
    --wait              Wait for deployments to be ready

Examples:
    $0 deploy-all --wait
    $0 deploy news-data-server
    $0 scale template-server 5
    $0 port-forward analytics-server
    $0 status

Available Servers: ${!SERVERS[@]}
EOF
}

# Generate deployment manifest for a server
generate_deployment_manifest() {
    local server="$1"
    local port="${SERVERS[$server]}"
    
    cat > "/tmp/${server}-deployment.yaml" << EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ${server}
  namespace: ${NAMESPACE}
  labels:
    app: ${server}
    app.kubernetes.io/name: ${server}
    app.kubernetes.io/component: server
    app.kubernetes.io/part-of: mcp-ecosystem
    app.kubernetes.io/version: "${VERSION}"
    app.kubernetes.io/managed-by: kubectl
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: ${server}
  template:
    metadata:
      labels:
        app: ${server}
        app.kubernetes.io/name: ${server}
        app.kubernetes.io/component: server
        app.kubernetes.io/part-of: mcp-ecosystem
        app.kubernetes.io/version: "${VERSION}"
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: mcp-server-sa
      automountServiceAccountToken: false
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
        seccompProfile:
          type: RuntimeDefault
      containers:
      - name: ${server}
        image: ${DOCKER_REGISTRY}/${server}:${VERSION}
        imagePullPolicy: IfNotPresent
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          runAsNonRoot: true
          runAsUser: 1000
          capabilities:
            drop:
            - ALL
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        - name: metrics
          containerPort: 9090
          protocol: TCP
        envFrom:
        - configMapRef:
            name: mcp-common-config
        - configMapRef:
            name: ${server}-config
            optional: true
        - secretRef:
            name: mcp-secrets
            optional: true
        resources:
          requests:
            cpu: "100m"
            memory: "128Mi"
            ephemeral-storage: "100Mi"
          limits:
            cpu: "500m"
            memory: "512Mi"
            ephemeral-storage: "1Gi"
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: data-volume
          mountPath: /app/data
        - name: logs-volume
          mountPath: /app/logs
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 30
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 3
          failureThreshold: 3
        startupProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 10
      volumes:
      - name: tmp
        emptyDir:
          sizeLimit: "100Mi"
      - name: data-volume
        emptyDir:
          sizeLimit: "500Mi"
      - name: logs-volume
        emptyDir:
          sizeLimit: "200Mi"
      terminationGracePeriodSeconds: 30
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: ${server}
              topologyKey: kubernetes.io/hostname

---
apiVersion: v1
kind: Service
metadata:
  name: ${server}
  namespace: ${NAMESPACE}
  labels:
    app: ${server}
    app.kubernetes.io/name: ${server}
    app.kubernetes.io/component: service
    app.kubernetes.io/part-of: mcp-ecosystem
    app.kubernetes.io/version: "${VERSION}"
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
spec:
  type: ClusterIP
  selector:
    app: ${server}
  ports:
  - name: http
    port: 80
    targetPort: http
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: metrics
    protocol: TCP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ${server}-hpa
  namespace: ${NAMESPACE}
  labels:
    app: ${server}
    app.kubernetes.io/name: ${server}-hpa
    app.kubernetes.io/component: autoscaler
    app.kubernetes.io/part-of: mcp-ecosystem
    app.kubernetes.io/version: "${VERSION}"
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ${server}
  minReplicas: 1
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
      selectPolicy: Max

EOF
}

# Setup namespace and common resources
setup_namespace() {
    log_info "Setting up namespace and common resources..."
    
    if kubectl apply -f "${SCRIPT_DIR}/00-namespace.yaml"; then
        log_success "Namespace and common resources created"
    else
        log_error "Failed to create namespace and common resources"
        exit 1
    fi
}

# Deploy single server
deploy_server() {
    local server="$1"
    local dry_run="${2:-false}"
    local wait="${3:-false}"
    
    if [[ ! "${SERVERS[$server]+_}" ]]; then
        log_error "Invalid server name: $server"
        log_info "Available servers: ${!SERVERS[@]}"
        exit 1
    fi
    
    log_info "Deploying $server..."
    
    # Generate manifest
    generate_deployment_manifest "$server"
    
    # Apply manifest
    local kubectl_args="apply -f /tmp/${server}-deployment.yaml"
    if [[ "$dry_run" == "true" ]]; then
        kubectl_args="$kubectl_args --dry-run=client"
    fi
    
    if kubectl $kubectl_args; then
        log_success "Deployment manifest applied for $server"
        
        if [[ "$wait" == "true" && "$dry_run" == "false" ]]; then
            log_info "Waiting for $server deployment to be ready..."
            if kubectl wait --for=condition=available --timeout=300s deployment/$server -n $NAMESPACE; then
                log_success "$server deployment is ready"
            else
                log_error "$server deployment failed to become ready"
                exit 1
            fi
        fi
    else
        log_error "Failed to deploy $server"
        exit 1
    fi
    
    # Clean up temporary file
    rm -f "/tmp/${server}-deployment.yaml"
}

# Deploy all servers
deploy_all() {
    local dry_run="${1:-false}"
    local wait="${2:-false}"
    
    log_info "Deploying all MCP servers..."
    
    # Ensure namespace exists
    setup_namespace
    
    # Deploy each server
    for server in "${!SERVERS[@]}"; do
        deploy_server "$server" "$dry_run" "$wait"
    done
    
    if [[ "$dry_run" == "false" ]]; then
        log_success "All MCP servers deployed successfully"
        show_status
    fi
}

# Undeploy server
undeploy_server() {
    local server="$1"
    
    if [[ ! "${SERVERS[$server]+_}" ]]; then
        log_error "Invalid server name: $server"
        exit 1
    fi
    
    log_info "Undeploying $server..."
    
    # Delete resources
    if kubectl delete deployment,service,hpa "$server" "$server-hpa" -n "$NAMESPACE" --ignore-not-found; then
        log_success "Successfully undeployed $server"
    else
        log_error "Failed to undeploy $server"
        exit 1
    fi
}

# Show deployment status
show_status() {
    log_info "MCP Servers Deployment Status:"
    echo
    
    # Check namespace
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_error "Namespace $NAMESPACE does not exist"
        return 1
    fi
    
    # Show deployments
    log_info "Deployments:"
    kubectl get deployments -n "$NAMESPACE" -o wide
    echo
    
    # Show services
    log_info "Services:"
    kubectl get services -n "$NAMESPACE" -o wide
    echo
    
    # Show pods
    log_info "Pods:"
    kubectl get pods -n "$NAMESPACE" -o wide
    echo
    
    # Show HPA status
    log_info "Horizontal Pod Autoscalers:"
    kubectl get hpa -n "$NAMESPACE" || echo "No HPAs found"
}

# Port forward to server
port_forward() {
    local server="$1"
    local local_port="${SERVERS[$server]}"
    
    if [[ ! "${SERVERS[$server]+_}" ]]; then
        log_error "Invalid server name: $server"
        exit 1
    fi
    
    log_info "Port forwarding to $server on port $local_port..."
    log_info "Access server at: http://localhost:$local_port"
    log_info "Press Ctrl+C to stop port forwarding"
    
    kubectl port-forward "service/$server" "$local_port:80" -n "$NAMESPACE"
}

# Scale server
scale_server() {
    local server="$1"
    local replicas="$2"
    
    if [[ ! "${SERVERS[$server]+_}" ]]; then
        log_error "Invalid server name: $server"
        exit 1
    fi
    
    if ! [[ "$replicas" =~ ^[0-9]+$ ]]; then
        log_error "Invalid replica count: $replicas"
        exit 1
    fi
    
    log_info "Scaling $server to $replicas replicas..."
    
    if kubectl scale deployment "$server" --replicas="$replicas" -n "$NAMESPACE"; then
        log_success "Successfully scaled $server to $replicas replicas"
    else
        log_error "Failed to scale $server"
        exit 1
    fi
}

# Health check all servers
health_check() {
    log_info "Performing health check on all MCP servers..."
    
    local failed=0
    for server in "${!SERVERS[@]}"; do
        local port="${SERVERS[$server]}"
        
        # Check if deployment exists and is ready
        if kubectl get deployment "$server" -n "$NAMESPACE" &> /dev/null; then
            local ready=$(kubectl get deployment "$server" -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}')
            local desired=$(kubectl get deployment "$server" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}')
            
            if [[ "$ready" == "$desired" && "$ready" -gt 0 ]]; then
                echo -e "${GREEN}✓${NC} $server ($ready/$desired replicas ready)"
            else
                echo -e "${RED}✗${NC} $server ($ready/$desired replicas ready)"
                failed=1
            fi
        else
            echo -e "${RED}✗${NC} $server (not deployed)"
            failed=1
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        log_success "All servers are healthy"
    else
        log_error "Some servers are not healthy"
        exit 1
    fi
}

# Parse command line arguments
DRY_RUN=false
WAIT=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --registry)
            DOCKER_REGISTRY="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --wait)
            WAIT=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            break
            ;;
    esac
done

# Enable verbose output if requested
if [[ "$VERBOSE" == "true" ]]; then
    set -x
fi

# Check prerequisites
check_prerequisites

# Main command handling
case "${1:-}" in
    deploy-all)
        deploy_all "$DRY_RUN" "$WAIT"
        ;;
    deploy)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for deploy command"
            usage
            exit 1
        fi
        deploy_server "$2" "$DRY_RUN" "$WAIT"
        ;;
    undeploy-all)
        log_warning "This will remove all MCP server deployments"
        read -p "Are you sure? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            for server in "${!SERVERS[@]}"; do
                undeploy_server "$server"
            done
        fi
        ;;
    undeploy)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for undeploy command"
            exit 1
        fi
        undeploy_server "$2"
        ;;
    update)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for update command"
            exit 1
        fi
        deploy_server "$2" false true
        ;;
    status)
        show_status
        ;;
    logs)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for logs command"
            exit 1
        fi
        kubectl logs -f "deployment/${2}" -n "$NAMESPACE"
        ;;
    scale)
        if [[ -z "${2:-}" || -z "${3:-}" ]]; then
            log_error "Server name and replica count required for scale command"
            exit 1
        fi
        scale_server "$2" "$3"
        ;;
    restart)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for restart command"
            exit 1
        fi
        kubectl rollout restart "deployment/${2}" -n "$NAMESPACE"
        ;;
    port-forward)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for port-forward command"
            exit 1
        fi
        port_forward "$2"
        ;;
    setup-namespace)
        setup_namespace
        ;;
    cleanup-namespace)
        log_warning "This will delete the entire namespace and all resources"
        read -p "Are you sure? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            kubectl delete namespace "$NAMESPACE"
        fi
        ;;
    generate-manifests)
        log_info "Generating all Kubernetes manifests..."
        for server in "${!SERVERS[@]}"; do
            generate_deployment_manifest "$server"
            mv "/tmp/${server}-deployment.yaml" "${SCRIPT_DIR}/${server}.yaml"
            log_success "Generated manifest for $server"
        done
        ;;
    validate)
        log_info "Validating all manifests..."
        kubectl apply --dry-run=client -f "${SCRIPT_DIR}/"
        log_success "All manifests are valid"
        ;;
    health-check)
        health_check
        ;;
    "")
        log_error "Command required"
        usage
        exit 1
        ;;
    *)
        log_error "Unknown command: $1"
        usage
        exit 1
        ;;
esac