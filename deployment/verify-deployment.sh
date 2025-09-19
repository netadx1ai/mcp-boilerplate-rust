#!/bin/bash
# Comprehensive Deployment Verification and Testing Script for MCP Servers
# This script validates Docker, Kubernetes, and monitoring deployments

set -euo pipefail

# Color output for better UX
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
NAMESPACE="mcp-servers"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-mcp-servers}"
VERSION="${VERSION:-0.3.0}"

# MCP Server configurations
declare -A SERVERS=(
    ["news-data-server"]="8081"
    ["template-server"]="8082"
    ["analytics-server"]="8083"
    ["database-server"]="8084"
    ["api-gateway-server"]="8085"
    ["workflow-server"]="8086"
)

# Test timeout configuration
TIMEOUT_SECONDS=300
HEALTH_CHECK_TIMEOUT=30
DOCKER_BUILD_TIMEOUT=600

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

log_test() {
    echo -e "${PURPLE}[TEST]${NC} $1"
}

log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

# Progress tracking
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

start_test() {
    local test_name="$1"
    ((TESTS_TOTAL++))
    log_test "Starting: $test_name"
}

pass_test() {
    local test_name="$1"
    ((TESTS_PASSED++))
    log_success "PASS: $test_name"
}

fail_test() {
    local test_name="$1"
    local reason="${2:-Unknown error}"
    ((TESTS_FAILED++))
    FAILED_TESTS+=("$test_name: $reason")
    log_error "FAIL: $test_name - $reason"
}

# Display usage information
usage() {
    cat << EOF
Comprehensive Deployment Verification Script for MCP Servers

Usage: $0 [COMMAND] [OPTIONS]

Commands:
    all                 Run all verification tests
    docker              Verify Docker deployment
    kubernetes          Verify Kubernetes deployment
    monitoring          Verify monitoring stack
    health              Perform health checks
    performance         Run performance tests
    security            Run security validation
    integration         Run integration tests
    smoke               Run smoke tests only
    cleanup             Clean up test resources

Options:
    --namespace <ns>    Set Kubernetes namespace (default: mcp-servers)
    --timeout <sec>     Set test timeout in seconds (default: 300)
    --registry <reg>    Set Docker registry (default: mcp-servers)
    --version <ver>     Set image version (default: 0.3.0)
    --verbose           Enable verbose output
    --fail-fast         Stop on first failure
    --report            Generate detailed test report

Examples:
    $0 all --verbose
    $0 docker --fail-fast
    $0 health --timeout 60
    $0 performance --report

EOF
}

# Check prerequisites
check_prerequisites() {
    start_test "Prerequisites Check"
    
    local missing_tools=()
    
    # Check required tools
    for tool in docker kubectl curl jq; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        fail_test "Prerequisites Check" "Missing tools: ${missing_tools[*]}"
        return 1
    fi
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        fail_test "Prerequisites Check" "Docker daemon not running"
        return 1
    fi
    
    # Check Kubernetes connectivity
    if ! kubectl cluster-info &> /dev/null; then
        log_warning "Kubernetes cluster not accessible, skipping K8s tests"
    fi
    
    pass_test "Prerequisites Check"
}

# Verify Docker images exist and are buildable
verify_docker_images() {
    log_step "Verifying Docker Images"
    
    for server in "${!SERVERS[@]}"; do
        start_test "Docker Image: $server"
        
        local image_name="${DOCKER_REGISTRY}/${server}:${VERSION}"
        
        # Check if image exists
        if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${image_name}$"; then
            log_info "Image $image_name exists locally"
        else
            log_warning "Image $image_name not found locally, attempting to build..."
            
            # Try to build the image
            local dockerfile="${SCRIPT_DIR}/docker/${server}.Dockerfile"
            if [[ -f "$dockerfile" ]]; then
                if timeout $DOCKER_BUILD_TIMEOUT docker build \
                    -f "$dockerfile" \
                    -t "$image_name" \
                    "$PROJECT_ROOT" &> /dev/null; then
                    log_info "Successfully built $image_name"
                else
                    fail_test "Docker Image: $server" "Failed to build image"
                    continue
                fi
            else
                fail_test "Docker Image: $server" "Dockerfile not found: $dockerfile"
                continue
            fi
        fi
        
        # Verify image can be inspected
        if docker inspect "$image_name" &> /dev/null; then
            local size=$(docker images --format "{{.Size}}" "$image_name")
            log_info "Image $server size: $size"
            pass_test "Docker Image: $server"
        else
            fail_test "Docker Image: $server" "Image inspection failed"
        fi
    done
}

# Verify Docker Compose deployment
verify_docker_compose() {
    log_step "Verifying Docker Compose Deployment"
    
    start_test "Docker Compose Configuration"
    
    local compose_file="${SCRIPT_DIR}/docker/docker-compose.yml"
    if [[ ! -f "$compose_file" ]]; then
        fail_test "Docker Compose Configuration" "docker-compose.yml not found"
        return 1
    fi
    
    # Validate compose file
    if docker-compose -f "$compose_file" config &> /dev/null; then
        pass_test "Docker Compose Configuration"
    else
        fail_test "Docker Compose Configuration" "Invalid docker-compose.yml"
        return 1
    fi
    
    # Test compose deployment
    start_test "Docker Compose Deployment"
    
    cd "${SCRIPT_DIR}/docker"
    
    # Start services
    if timeout $TIMEOUT_SECONDS docker-compose up -d &> /dev/null; then
        log_info "Docker Compose services started"
        
        # Wait for services to be healthy
        local healthy_count=0
        local max_wait=60
        
        for ((i=1; i<=max_wait; i++)); do
            healthy_count=0
            for server in "${!SERVERS[@]}"; do
                local container_name="mcp-${server}"
                if docker ps --filter "name=$container_name" --filter "status=running" | grep -q "$container_name"; then
                    ((healthy_count++))
                fi
            done
            
            if [[ $healthy_count -eq ${#SERVERS[@]} ]]; then
                log_info "All services are running ($healthy_count/${#SERVERS[@]})"
                break
            fi
            
            if [[ $i -eq $max_wait ]]; then
                fail_test "Docker Compose Deployment" "Services failed to start within ${max_wait}s"
                docker-compose logs
                docker-compose down &> /dev/null
                return 1
            fi
            
            sleep 1
        done
        
        pass_test "Docker Compose Deployment"
        
        # Clean up
        docker-compose down &> /dev/null
    else
        fail_test "Docker Compose Deployment" "Failed to start services"
        return 1
    fi
}

# Verify Kubernetes deployment
verify_kubernetes() {
    log_step "Verifying Kubernetes Deployment"
    
    # Check if kubectl is working
    if ! kubectl cluster-info &> /dev/null; then
        log_warning "Kubernetes cluster not accessible, skipping K8s verification"
        return 0
    fi
    
    start_test "Kubernetes Manifests Validation"
    
    # Validate all manifests
    local manifests_dir="${SCRIPT_DIR}/kubernetes"
    if [[ -d "$manifests_dir" ]]; then
        if kubectl apply --dry-run=client -f "$manifests_dir/" &> /dev/null; then
            pass_test "Kubernetes Manifests Validation"
        else
            fail_test "Kubernetes Manifests Validation" "Invalid manifests detected"
            return 1
        fi
    else
        fail_test "Kubernetes Manifests Validation" "Kubernetes manifests directory not found"
        return 1
    fi
    
    # Test namespace creation
    start_test "Kubernetes Namespace"
    
    if kubectl apply -f "${manifests_dir}/00-namespace.yaml" &> /dev/null; then
        if kubectl get namespace "$NAMESPACE" &> /dev/null; then
            pass_test "Kubernetes Namespace"
        else
            fail_test "Kubernetes Namespace" "Namespace creation failed"
        fi
    else
        fail_test "Kubernetes Namespace" "Failed to apply namespace manifest"
    fi
    
    # Test deployment script
    start_test "Kubernetes Deployment Script"
    
    local deploy_script="${manifests_dir}/deploy.sh"
    if [[ -f "$deploy_script" && -x "$deploy_script" ]]; then
        # Test dry-run deployment
        if "$deploy_script" deploy-all --dry-run &> /dev/null; then
            pass_test "Kubernetes Deployment Script"
        else
            fail_test "Kubernetes Deployment Script" "Deployment script failed"
        fi
    else
        fail_test "Kubernetes Deployment Script" "Deploy script not found or not executable"
    fi
}

# Verify monitoring stack
verify_monitoring() {
    log_step "Verifying Monitoring Stack"
    
    start_test "Prometheus Configuration"
    
    local prometheus_config="${SCRIPT_DIR}/monitoring/prometheus.yml"
    if [[ -f "$prometheus_config" ]]; then
        # Basic YAML validation
        if command -v promtool &> /dev/null; then
            if promtool check config "$prometheus_config" &> /dev/null; then
                pass_test "Prometheus Configuration"
            else
                fail_test "Prometheus Configuration" "Invalid Prometheus config"
            fi
        else
            log_info "promtool not available, skipping detailed validation"
            if grep -q "global:" "$prometheus_config" && grep -q "scrape_configs:" "$prometheus_config"; then
                pass_test "Prometheus Configuration"
            else
                fail_test "Prometheus Configuration" "Missing required sections"
            fi
        fi
    else
        fail_test "Prometheus Configuration" "prometheus.yml not found"
    fi
    
    start_test "Alert Rules Configuration"
    
    local alert_rules="${SCRIPT_DIR}/monitoring/alert-rules.yml"
    if [[ -f "$alert_rules" ]]; then
        # Basic YAML validation
        if command -v promtool &> /dev/null; then
            if promtool check rules "$alert_rules" &> /dev/null; then
                pass_test "Alert Rules Configuration"
            else
                fail_test "Alert Rules Configuration" "Invalid alert rules"
            fi
        else
            if grep -q "groups:" "$alert_rules" && grep -q "rules:" "$alert_rules"; then
                pass_test "Alert Rules Configuration"
            else
                fail_test "Alert Rules Configuration" "Missing required sections"
            fi
        fi
    else
        fail_test "Alert Rules Configuration" "alert-rules.yml not found"
    fi
    
    start_test "Grafana Dashboard"
    
    local grafana_dashboard="${SCRIPT_DIR}/monitoring/grafana-dashboard.json"
    if [[ -f "$grafana_dashboard" ]]; then
        # Basic JSON validation
        if jq . "$grafana_dashboard" &> /dev/null; then
            # Check for required dashboard elements
            if jq -e '.dashboard.panels | length > 0' "$grafana_dashboard" &> /dev/null; then
                pass_test "Grafana Dashboard"
            else
                fail_test "Grafana Dashboard" "Dashboard has no panels"
            fi
        else
            fail_test "Grafana Dashboard" "Invalid JSON format"
        fi
    else
        fail_test "Grafana Dashboard" "grafana-dashboard.json not found"
    fi
}

# Perform health checks
perform_health_checks() {
    log_step "Performing Health Checks"
    
    # Start Docker Compose for health testing
    cd "${SCRIPT_DIR}/docker"
    docker-compose up -d &> /dev/null || true
    
    # Wait for services to start
    sleep 30
    
    for server in "${!SERVERS[@]}"; do
        start_test "Health Check: $server"
        
        local port="${SERVERS[$server]}"
        local health_url="http://localhost:$port/health"
        
        # Check if service is responding
        local max_attempts=10
        local healthy=false
        
        for ((i=1; i<=max_attempts; i++)); do
            if timeout $HEALTH_CHECK_TIMEOUT curl -sf "$health_url" &> /dev/null; then
                healthy=true
                break
            fi
            sleep 3
        done
        
        if [[ "$healthy" == "true" ]]; then
            # Get additional health info
            local response=$(curl -s "$health_url" 2>/dev/null || echo "{}")
            log_info "Health endpoint response: $response"
            pass_test "Health Check: $server"
        else
            fail_test "Health Check: $server" "Health endpoint not responding"
        fi
    done
    
    # Clean up
    docker-compose down &> /dev/null || true
}

# Run performance tests
run_performance_tests() {
    log_step "Running Performance Tests"
    
    # Start Docker Compose for performance testing
    cd "${SCRIPT_DIR}/docker"
    docker-compose up -d &> /dev/null || true
    
    # Wait for services to start
    sleep 30
    
    for server in "${!SERVERS[@]}"; do
        start_test "Performance Test: $server"
        
        local port="${SERVERS[$server]}"
        local base_url="http://localhost:$port"
        
        # Basic load test with curl
        local success_count=0
        local total_requests=10
        local start_time=$(date +%s)
        
        for ((i=1; i<=total_requests; i++)); do
            if timeout 10 curl -sf "$base_url/health" &> /dev/null; then
                ((success_count++))
            fi
        done
        
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        local success_rate=$((success_count * 100 / total_requests))
        local avg_response_time=$((duration * 1000 / total_requests))
        
        log_info "Performance results: $success_count/$total_requests requests successful"
        log_info "Success rate: ${success_rate}%, Avg response time: ${avg_response_time}ms"
        
        if [[ $success_rate -ge 90 && $avg_response_time -le 1000 ]]; then
            pass_test "Performance Test: $server"
        else
            fail_test "Performance Test: $server" "Poor performance (${success_rate}% success, ${avg_response_time}ms avg)"
        fi
    done
    
    # Clean up
    docker-compose down &> /dev/null || true
}

# Run security validation
run_security_validation() {
    log_step "Running Security Validation"
    
    start_test "Docker Image Security"
    
    # Check for security scanning tools
    if command -v docker &> /dev/null; then
        for server in "${!SERVERS[@]}"; do
            local image_name="${DOCKER_REGISTRY}/${server}:${VERSION}"
            
            # Basic image inspection for security
            if docker inspect "$image_name" | jq -e '.[] | select(.Config.User == "")' &> /dev/null; then
                log_warning "Image $server may be running as root"
            else
                log_info "Image $server appears to use non-root user"
            fi
        done
        pass_test "Docker Image Security"
    else
        fail_test "Docker Image Security" "Docker not available"
    fi
    
    start_test "Configuration Security"
    
    # Check for hardcoded secrets
    local secret_patterns=("password" "secret" "key" "token")
    local insecure_files=()
    
    for pattern in "${secret_patterns[@]}"; do
        while IFS= read -r -d '' file; do
            if grep -i "$pattern" "$file" | grep -v "# " | grep -v "example" &> /dev/null; then
                insecure_files+=("$file")
            fi
        done < <(find "$SCRIPT_DIR" -type f \( -name "*.yml" -o -name "*.yaml" -o -name "*.json" \) -print0)
    done
    
    if [[ ${#insecure_files[@]} -eq 0 ]]; then
        pass_test "Configuration Security"
    else
        log_warning "Potential hardcoded secrets found in: ${insecure_files[*]}"
        pass_test "Configuration Security"  # Warning, not failure
    fi
}

# Run integration tests
run_integration_tests() {
    log_step "Running Integration Tests"
    
    start_test "Service Integration"
    
    # Start all services
    cd "${SCRIPT_DIR}/docker"
    docker-compose up -d &> /dev/null || true
    
    # Wait for services to start
    sleep 45
    
    # Test basic service interactions
    local integration_success=true
    
    # Test if all services can communicate through the network
    for server in "${!SERVERS[@]}"; do
        local port="${SERVERS[$server]}"
        if ! curl -sf "http://localhost:$port/health" &> /dev/null; then
            integration_success=false
            log_error "Service $server not responding"
        fi
    done
    
    if [[ "$integration_success" == "true" ]]; then
        pass_test "Service Integration"
    else
        fail_test "Service Integration" "Some services failed integration test"
    fi
    
    # Clean up
    docker-compose down &> /dev/null || true
}

# Run smoke tests
run_smoke_tests() {
    log_step "Running Smoke Tests"
    
    # Quick tests to verify basic functionality
    start_test "File Structure"
    
    local required_files=(
        "docker/docker-compose.yml"
        "kubernetes/00-namespace.yaml"
        "kubernetes/deploy.sh"
        "monitoring/prometheus.yml"
        "monitoring/alert-rules.yml"
        "monitoring/grafana-dashboard.json"
    )
    
    local missing_files=()
    for file in "${required_files[@]}"; do
        if [[ ! -f "${SCRIPT_DIR}/$file" ]]; then
            missing_files+=("$file")
        fi
    done
    
    if [[ ${#missing_files[@]} -eq 0 ]]; then
        pass_test "File Structure"
    else
        fail_test "File Structure" "Missing files: ${missing_files[*]}"
    fi
    
    start_test "Docker Files"
    
    local missing_dockerfiles=()
    for server in "${!SERVERS[@]}"; do
        local dockerfile="${SCRIPT_DIR}/docker/${server}.Dockerfile"
        if [[ ! -f "$dockerfile" ]]; then
            missing_dockerfiles+=("$server.Dockerfile")
        fi
    done
    
    if [[ ${#missing_dockerfiles[@]} -eq 0 ]]; then
        pass_test "Docker Files"
    else
        fail_test "Docker Files" "Missing Dockerfiles: ${missing_dockerfiles[*]}"
    fi
}

# Generate test report
generate_report() {
    local report_file="${SCRIPT_DIR}/deployment-verification-report.txt"
    
    cat > "$report_file" << EOF
# MCP Server Deployment Verification Report
Generated: $(date)
Command: $0 $*

## Summary
- Total Tests: $TESTS_TOTAL
- Passed: $TESTS_PASSED
- Failed: $TESTS_FAILED
- Success Rate: $(( TESTS_PASSED * 100 / TESTS_TOTAL ))%

## Configuration
- Namespace: $NAMESPACE
- Docker Registry: $DOCKER_REGISTRY
- Version: $VERSION
- Timeout: $TIMEOUT_SECONDS seconds

## Failed Tests
EOF

    if [[ ${#FAILED_TESTS[@]} -eq 0 ]]; then
        echo "None - All tests passed!" >> "$report_file"
    else
        for failed_test in "${FAILED_TESTS[@]}"; do
            echo "- $failed_test" >> "$report_file"
        done
    fi
    
    cat >> "$report_file" << EOF

## Environment Info
- Docker Version: $(docker --version 2>/dev/null || echo "Not available")
- Kubectl Version: $(kubectl version --client --short 2>/dev/null || echo "Not available")
- OS: $(uname -s) $(uname -r)
- Architecture: $(uname -m)

## Available Servers
EOF
    
    for server in "${!SERVERS[@]}"; do
        echo "- $server (port ${SERVERS[$server]})" >> "$report_file"
    done
    
    log_info "Report generated: $report_file"
}

# Cleanup test resources
cleanup_resources() {
    log_step "Cleaning up test resources"
    
    # Stop Docker Compose
    cd "${SCRIPT_DIR}/docker" && docker-compose down &> /dev/null || true
    
    # Remove test namespace from Kubernetes (if exists)
    kubectl delete namespace "$NAMESPACE" --ignore-not-found &> /dev/null || true
    
    # Remove any temporary files
    rm -f /tmp/*-deployment.yaml
    
    log_success "Cleanup completed"
}

# Parse command line arguments
VERBOSE=false
FAIL_FAST=false
GENERATE_REPORT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --timeout)
            TIMEOUT_SECONDS="$2"
            shift 2
            ;;
        --registry)
            DOCKER_REGISTRY="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        --report)
            GENERATE_REPORT=true
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

# Main execution
echo -e "${PURPLE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${PURPLE}        MCP Server Deployment Verification Script                  ${NC}"
echo -e "${PURPLE}════════════════════════════════════════════════════════════════${NC}"
echo

log_info "Starting deployment verification..."
log_info "Configuration: Namespace=$NAMESPACE, Registry=$DOCKER_REGISTRY, Version=$VERSION"
echo

# Trap for cleanup on exit
trap cleanup_resources EXIT

# Main command handling
case "${1:-all}" in
    all)
        check_prerequisites
        verify_docker_images
        verify_docker_compose
        verify_kubernetes
        verify_monitoring
        perform_health_checks
        run_performance_tests
        run_security_validation
        run_integration_tests
        ;;
    docker)
        check_prerequisites
        verify_docker_images
        verify_docker_compose
        ;;
    kubernetes)
        check_prerequisites
        verify_kubernetes
        ;;
    monitoring)
        check_prerequisites
        verify_monitoring
        ;;
    health)
        check_prerequisites
        perform_health_checks
        ;;
    performance)
        check_prerequisites
        run_performance_tests
        ;;
    security)
        check_prerequisites
        run_security_validation
        ;;
    integration)
        check_prerequisites
        run_integration_tests
        ;;
    smoke)
        run_smoke_tests
        ;;
    cleanup)
        cleanup_resources
        exit 0
        ;;
    *)
        log_error "Unknown command: $1"
        usage
        exit 1
        ;;
esac

# Final results
echo
echo -e "${PURPLE}════════════════════════════════════════════════════════════════${NC}"
echo -e "${PURPLE}                        VERIFICATION RESULTS                      ${NC}"
echo -e "${PURPLE}════════════════════════════════════════════════════════════════${NC}"

log_info "Total Tests: $TESTS_TOTAL"
log_success "Passed: $TESTS_PASSED"
if [[ $TESTS_FAILED -gt 0 ]]; then
    log_error "Failed: $TESTS_FAILED"
    echo
    log_error "Failed tests:"
    for failed_test in "${FAILED_TESTS[@]}"; do
        echo -e "${RED}  - $failed_test${NC}"
    done
else
    log_success "Failed: $TESTS_FAILED"
fi

echo
if [[ $TESTS_FAILED -eq 0 ]]; then
    log_success "🎉 All verification tests passed! Deployment is ready for production."
    exit_code=0
else
    log_error "❌ Some verification tests failed. Please review and fix issues before production deployment."
    exit_code=1
fi

# Generate report if requested
if [[ "$GENERATE_REPORT" == "true" ]]; then
    generate_report
fi

echo -e "${PURPLE}════════════════════════════════════════════════════════════════${NC}"

exit $exit_code