#!/bin/bash
# Docker Build and Management Script for MCP Servers
# Provides comprehensive Docker operations for all production servers

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
DOCKER_REGISTRY="${DOCKER_REGISTRY:-mcp-servers}"
VERSION="${VERSION:-0.3.0}"

# MCP Server list
SERVERS=(
    "news-data-server"
    "template-server"
    "analytics-server"
    "database-server"
    "api-gateway-server"
    "workflow-server"
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

# Display usage information
usage() {
    cat << EOF
Docker Build and Management Script for MCP Servers

Usage: $0 [COMMAND] [OPTIONS]

Commands:
    build-all           Build all MCP server Docker images
    build <server>      Build specific server image
    push-all           Push all images to registry
    push <server>      Push specific server image
    pull-all           Pull all images from registry
    pull <server>      Pull specific server image
    run <server>       Run specific server container
    stop <server>      Stop specific server container
    restart <server>   Restart specific server container
    logs <server>      Show logs for specific server
    health <server>    Check health of specific server
    clean              Remove all MCP server images and containers
    test-build         Test build all servers (no cache)
    dev-up             Start development environment (docker-compose)
    dev-down           Stop development environment
    dev-logs           Show development environment logs
    list               List available servers
    status             Show status of all containers

Options:
    --no-cache         Build without cache
    --parallel         Build images in parallel
    --verbose          Enable verbose output
    --registry <reg>   Set Docker registry (default: mcp-servers)
    --version <ver>    Set image version (default: 0.3.0)

Examples:
    $0 build-all --no-cache
    $0 build news-data-server
    $0 run template-server
    $0 dev-up
    $0 status

Available Servers: ${SERVERS[*]}
EOF
}

# Check if server name is valid
validate_server() {
    local server="$1"
    for valid_server in "${SERVERS[@]}"; do
        if [[ "$valid_server" == "$server" ]]; then
            return 0
        fi
    done
    log_error "Invalid server name: $server"
    log_info "Available servers: ${SERVERS[*]}"
    exit 1
}

# Build single server image
build_server() {
    local server="$1"
    local no_cache="${2:-false}"
    local verbose="${3:-false}"
    
    validate_server "$server"
    
    local dockerfile="$SCRIPT_DIR/${server}.Dockerfile"
    local image_name="${DOCKER_REGISTRY}/${server}:${VERSION}"
    local latest_name="${DOCKER_REGISTRY}/${server}:latest"
    
    if [[ ! -f "$dockerfile" ]]; then
        log_error "Dockerfile not found: $dockerfile"
        exit 1
    fi
    
    log_info "Building $server Docker image..."
    log_info "Image: $image_name"
    log_info "Context: $PROJECT_ROOT"
    log_info "Dockerfile: $dockerfile"
    
    local build_args=""
    if [[ "$no_cache" == "true" ]]; then
        build_args="--no-cache"
    fi
    
    if [[ "$verbose" == "true" ]]; then
        build_args="$build_args --progress=plain"
    fi
    
    # Build with version tag
    if docker build $build_args \
        -f "$dockerfile" \
        -t "$image_name" \
        -t "$latest_name" \
        "$PROJECT_ROOT"; then
        log_success "Successfully built $server image"
        
        # Show image info
        log_info "Image details:"
        docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" \
            | grep -E "(REPOSITORY|${server})"
    else
        log_error "Failed to build $server image"
        exit 1
    fi
}

# Build all server images
build_all() {
    local no_cache="${1:-false}"
    local parallel="${2:-false}"
    local verbose="${3:-false}"
    
    log_info "Building all MCP server images..."
    
    if [[ "$parallel" == "true" ]]; then
        log_info "Building in parallel mode..."
        local pids=()
        
        for server in "${SERVERS[@]}"; do
            log_info "Starting build for $server..."
            build_server "$server" "$no_cache" "$verbose" &
            pids+=($!)
        done
        
        # Wait for all builds to complete
        local failed=0
        for pid in "${pids[@]}"; do
            if ! wait "$pid"; then
                failed=1
            fi
        done
        
        if [[ $failed -eq 0 ]]; then
            log_success "All server images built successfully"
        else
            log_error "Some builds failed"
            exit 1
        fi
    else
        # Sequential build
        for server in "${SERVERS[@]}"; do
            build_server "$server" "$no_cache" "$verbose"
        done
        log_success "All server images built successfully"
    fi
    
    # Show summary
    log_info "Build summary:"
    docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" \
        | grep -E "(REPOSITORY|${DOCKER_REGISTRY})"
}

# Push server image to registry
push_server() {
    local server="$1"
    validate_server "$server"
    
    local image_name="${DOCKER_REGISTRY}/${server}:${VERSION}"
    local latest_name="${DOCKER_REGISTRY}/${server}:latest"
    
    log_info "Pushing $server image to registry..."
    
    if docker push "$image_name" && docker push "$latest_name"; then
        log_success "Successfully pushed $server image"
    else
        log_error "Failed to push $server image"
        exit 1
    fi
}

# Run server container
run_server() {
    local server="$1"
    validate_server "$server"
    
    local image_name="${DOCKER_REGISTRY}/${server}:${VERSION}"
    local container_name="mcp-${server}"
    
    # Stop existing container if running
    if docker ps -q -f name="$container_name" | grep -q .; then
        log_warning "Stopping existing container: $container_name"
        docker stop "$container_name"
        docker rm "$container_name"
    fi
    
    # Determine port mapping
    local port_map=""
    case "$server" in
        "news-data-server") port_map="-p 8081:8080" ;;
        "template-server") port_map="-p 8082:8080" ;;
        "analytics-server") port_map="-p 8083:8080" ;;
        "database-server") port_map="-p 8084:8080" ;;
        "api-gateway-server") port_map="-p 8085:8080" ;;
        "workflow-server") port_map="-p 8086:8080" ;;
    esac
    
    log_info "Starting $server container..."
    if docker run -d \
        --name "$container_name" \
        $port_map \
        --restart unless-stopped \
        --network mcp-network 2>/dev/null || docker run -d \
        --name "$container_name" \
        $port_map \
        --restart unless-stopped \
        "$image_name"; then
        log_success "Successfully started $server container"
        log_info "Container name: $container_name"
        log_info "Port mapping: $port_map"
    else
        log_error "Failed to start $server container"
        exit 1
    fi
}

# Check server health
check_health() {
    local server="$1"
    validate_server "$server"
    
    local container_name="mcp-${server}"
    
    if ! docker ps -q -f name="$container_name" | grep -q .; then
        log_error "Container $container_name is not running"
        return 1
    fi
    
    log_info "Checking health of $server..."
    
    # Get container port
    local port=$(docker port "$container_name" 8080/tcp 2>/dev/null | cut -d':' -f2)
    if [[ -z "$port" ]]; then
        log_warning "Could not determine port for $container_name"
        return 1
    fi
    
    # Check health endpoint
    if curl -sf "http://localhost:$port/health" >/dev/null 2>&1; then
        log_success "$server is healthy"
        return 0
    else
        log_error "$server health check failed"
        return 1
    fi
}

# Show container status
show_status() {
    log_info "MCP Server Container Status:"
    echo
    
    # Check if docker-compose is running
    if docker-compose -f "$SCRIPT_DIR/docker-compose.yml" ps >/dev/null 2>&1; then
        log_info "Docker Compose Status:"
        docker-compose -f "$SCRIPT_DIR/docker-compose.yml" ps
        echo
    fi
    
    # Show individual containers
    log_info "Individual Container Status:"
    for server in "${SERVERS[@]}"; do
        local container_name="mcp-${server}"
        if docker ps -q -f name="$container_name" | grep -q .; then
            echo -e "${GREEN}✓${NC} $server (running)"
        else
            echo -e "${RED}✗${NC} $server (stopped)"
        fi
    done
    
    echo
    log_info "Docker Images:"
    docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" \
        | grep -E "(REPOSITORY|${DOCKER_REGISTRY})" || echo "No MCP server images found"
}

# Development environment management
dev_up() {
    log_info "Starting development environment..."
    cd "$SCRIPT_DIR"
    
    if docker-compose up -d; then
        log_success "Development environment started"
        echo
        log_info "Services available at:"
        echo "  News Data Server:   http://localhost:8081"
        echo "  Template Server:    http://localhost:8082"
        echo "  Analytics Server:   http://localhost:8083"
        echo "  Database Server:    http://localhost:8084"
        echo "  API Gateway Server: http://localhost:8085"
        echo "  Workflow Server:    http://localhost:8086"
        echo "  Prometheus:         http://localhost:9090"
        echo "  Grafana:            http://localhost:3000 (admin/admin)"
        echo "  Jaeger:             http://localhost:16686"
    else
        log_error "Failed to start development environment"
        exit 1
    fi
}

dev_down() {
    log_info "Stopping development environment..."
    cd "$SCRIPT_DIR"
    
    if docker-compose down; then
        log_success "Development environment stopped"
    else
        log_error "Failed to stop development environment"
        exit 1
    fi
}

# Clean up Docker resources
cleanup() {
    log_warning "This will remove all MCP server images and containers"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cleanup cancelled"
        exit 0
    fi
    
    log_info "Cleaning up MCP server resources..."
    
    # Stop and remove containers
    for server in "${SERVERS[@]}"; do
        local container_name="mcp-${server}"
        if docker ps -q -f name="$container_name" | grep -q .; then
            log_info "Stopping container: $container_name"
            docker stop "$container_name"
            docker rm "$container_name"
        fi
    done
    
    # Remove images
    local images=$(docker images --format "{{.Repository}}:{{.Tag}}" | grep "^${DOCKER_REGISTRY}/")
    if [[ -n "$images" ]]; then
        log_info "Removing images..."
        echo "$images" | xargs docker rmi
    fi
    
    log_success "Cleanup completed"
}

# Parse command line arguments
NO_CACHE=false
PARALLEL=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-cache)
            NO_CACHE=true
            shift
            ;;
        --parallel)
            PARALLEL=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --registry)
            DOCKER_REGISTRY="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
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

# Main command handling
case "${1:-}" in
    build-all)
        build_all "$NO_CACHE" "$PARALLEL" "$VERBOSE"
        ;;
    build)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for build command"
            usage
            exit 1
        fi
        build_server "$2" "$NO_CACHE" "$VERBOSE"
        ;;
    push-all)
        for server in "${SERVERS[@]}"; do
            push_server "$server"
        done
        ;;
    push)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for push command"
            exit 1
        fi
        push_server "$2"
        ;;
    run)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for run command"
            exit 1
        fi
        run_server "$2"
        ;;
    stop)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for stop command"
            exit 1
        fi
        validate_server "$2"
        docker stop "mcp-$2"
        ;;
    restart)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for restart command"
            exit 1
        fi
        validate_server "$2"
        docker restart "mcp-$2"
        ;;
    logs)
        if [[ -z "${2:-}" ]]; then
            log_error "Server name required for logs command"
            exit 1
        fi
        validate_server "$2"
        docker logs -f "mcp-$2"
        ;;
    health)
        if [[ -z "${2:-}" ]]; then
            for server in "${SERVERS[@]}"; do
                check_health "$server"
            done
        else
            check_health "$2"
        fi
        ;;
    test-build)
        build_all true false "$VERBOSE"
        ;;
    dev-up)
        dev_up
        ;;
    dev-down)
        dev_down
        ;;
    dev-logs)
        cd "$SCRIPT_DIR"
        docker-compose logs -f
        ;;
    clean)
        cleanup
        ;;
    list)
        log_info "Available MCP servers:"
        for server in "${SERVERS[@]}"; do
            echo "  - $server"
        done
        ;;
    status)
        show_status
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