#!/bin/bash

# =====================================================================================
# RWA Platform Deployment Script
# Supports multiple environments and deployment strategies
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-ghcr.io/rwa-platform}"
VERSION="${VERSION:-latest}"
ENVIRONMENT="${ENVIRONMENT:-development}"
NAMESPACE="rwa-platform"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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

# Help function
show_help() {
    cat << EOF
RWA Platform Deployment Script

Usage: $0 [OPTIONS] COMMAND

Commands:
    build           Build Docker images
    push            Push images to registry
    deploy          Deploy to Kubernetes
    rollback        Rollback to previous version
    status          Check deployment status
    logs            Show service logs
    clean           Clean up resources

Options:
    -e, --environment ENV    Target environment (development|staging|production)
    -v, --version VERSION    Image version tag (default: latest)
    -r, --registry REGISTRY  Docker registry URL
    -n, --namespace NS       Kubernetes namespace
    -h, --help              Show this help message

Examples:
    $0 build
    $0 -e production -v v1.0.0 deploy
    $0 -e staging rollback
    $0 logs gateway

Environment Variables:
    DOCKER_REGISTRY         Docker registry URL
    VERSION                 Image version
    ENVIRONMENT            Target environment
    KUBECONFIG             Kubernetes config file path

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            -r|--registry)
                DOCKER_REGISTRY="$2"
                shift 2
                ;;
            -n|--namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            build|push|deploy|rollback|status|logs|clean)
                COMMAND="$1"
                shift
                ;;
            *)
                if [[ -z "${COMMAND:-}" ]]; then
                    log_error "Unknown option: $1"
                    show_help
                    exit 1
                else
                    ARGS+=("$1")
                    shift
                fi
                ;;
        esac
    done

    # Set namespace based on environment
    case $ENVIRONMENT in
        development)
            NAMESPACE="rwa-platform-dev"
            ;;
        staging)
            NAMESPACE="rwa-platform-staging"
            ;;
        production)
            NAMESPACE="rwa-platform"
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT"
            exit 1
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi

    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed"
        exit 1
    fi

    # Check Kubernetes connection
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

# Build Docker images
build_images() {
    log_info "Building Docker images for version $VERSION..."

    cd "$PROJECT_ROOT"

    # Build multi-stage images
    docker build \
        --target gateway \
        --tag "$DOCKER_REGISTRY/gateway:$VERSION" \
        --tag "$DOCKER_REGISTRY/gateway:latest" \
        .

    docker build \
        --target user-service \
        --tag "$DOCKER_REGISTRY/user-service:$VERSION" \
        --tag "$DOCKER_REGISTRY/user-service:latest" \
        .

    docker build \
        --target asset-service \
        --tag "$DOCKER_REGISTRY/asset-service:$VERSION" \
        --tag "$DOCKER_REGISTRY/asset-service:latest" \
        .

    docker build \
        --target payment-service \
        --tag "$DOCKER_REGISTRY/payment-service:$VERSION" \
        --tag "$DOCKER_REGISTRY/payment-service:latest" \
        .

    log_success "Docker images built successfully"
}

# Push images to registry
push_images() {
    log_info "Pushing images to registry $DOCKER_REGISTRY..."

    docker push "$DOCKER_REGISTRY/gateway:$VERSION"
    docker push "$DOCKER_REGISTRY/gateway:latest"
    
    docker push "$DOCKER_REGISTRY/user-service:$VERSION"
    docker push "$DOCKER_REGISTRY/user-service:latest"
    
    docker push "$DOCKER_REGISTRY/asset-service:$VERSION"
    docker push "$DOCKER_REGISTRY/asset-service:latest"
    
    docker push "$DOCKER_REGISTRY/payment-service:$VERSION"
    docker push "$DOCKER_REGISTRY/payment-service:latest"

    log_success "Images pushed successfully"
}

# Deploy to Kubernetes
deploy_to_k8s() {
    log_info "Deploying to Kubernetes environment: $ENVIRONMENT"

    cd "$PROJECT_ROOT"

    # Create namespace if it doesn't exist
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

    # Apply configurations
    log_info "Applying configurations..."
    kubectl apply -f k8s/configmap.yaml -n "$NAMESPACE"
    kubectl apply -f k8s/secrets.yaml -n "$NAMESPACE"

    # Apply infrastructure components
    log_info "Deploying infrastructure components..."
    kubectl apply -f k8s/postgres.yaml -n "$NAMESPACE"
    kubectl apply -f k8s/redis.yaml -n "$NAMESPACE"

    # Wait for infrastructure to be ready
    log_info "Waiting for infrastructure to be ready..."
    kubectl wait --for=condition=ready pod -l app=postgres -n "$NAMESPACE" --timeout=300s
    kubectl wait --for=condition=ready pod -l app=redis -n "$NAMESPACE" --timeout=300s

    # Apply application services
    log_info "Deploying application services..."
    
    # Update image tags in deployment files
    sed -i.bak "s|image: .*gateway:.*|image: $DOCKER_REGISTRY/gateway:$VERSION|g" k8s/gateway.yaml
    sed -i.bak "s|image: .*user-service:.*|image: $DOCKER_REGISTRY/user-service:$VERSION|g" k8s/user-service.yaml
    sed -i.bak "s|image: .*asset-service:.*|image: $DOCKER_REGISTRY/asset-service:$VERSION|g" k8s/asset-service.yaml
    sed -i.bak "s|image: .*payment-service:.*|image: $DOCKER_REGISTRY/payment-service:$VERSION|g" k8s/payment-service.yaml

    kubectl apply -f k8s/gateway.yaml -n "$NAMESPACE"
    kubectl apply -f k8s/user-service.yaml -n "$NAMESPACE"
    kubectl apply -f k8s/asset-service.yaml -n "$NAMESPACE"
    kubectl apply -f k8s/payment-service.yaml -n "$NAMESPACE"

    # Apply ingress
    if [[ "$ENVIRONMENT" == "production" ]]; then
        kubectl apply -f k8s/ingress.yaml -n "$NAMESPACE"
    fi

    # Wait for deployments to be ready
    log_info "Waiting for deployments to be ready..."
    kubectl wait --for=condition=available deployment/gateway -n "$NAMESPACE" --timeout=300s
    kubectl wait --for=condition=available deployment/user-service -n "$NAMESPACE" --timeout=300s
    kubectl wait --for=condition=available deployment/asset-service -n "$NAMESPACE" --timeout=300s
    kubectl wait --for=condition=available deployment/payment-service -n "$NAMESPACE" --timeout=300s

    # Clean up backup files
    rm -f k8s/*.yaml.bak

    log_success "Deployment completed successfully"
}

# Rollback deployment
rollback_deployment() {
    log_info "Rolling back deployment in namespace: $NAMESPACE"

    kubectl rollout undo deployment/gateway -n "$NAMESPACE"
    kubectl rollout undo deployment/user-service -n "$NAMESPACE"
    kubectl rollout undo deployment/asset-service -n "$NAMESPACE"
    kubectl rollout undo deployment/payment-service -n "$NAMESPACE"

    log_success "Rollback completed"
}

# Check deployment status
check_status() {
    log_info "Checking deployment status in namespace: $NAMESPACE"

    echo
    echo "=== Deployments ==="
    kubectl get deployments -n "$NAMESPACE"

    echo
    echo "=== Pods ==="
    kubectl get pods -n "$NAMESPACE"

    echo
    echo "=== Services ==="
    kubectl get services -n "$NAMESPACE"

    if [[ "$ENVIRONMENT" == "production" ]]; then
        echo
        echo "=== Ingress ==="
        kubectl get ingress -n "$NAMESPACE"
    fi
}

# Show service logs
show_logs() {
    local service="${ARGS[0]:-gateway}"
    log_info "Showing logs for service: $service"

    kubectl logs -f deployment/"$service" -n "$NAMESPACE"
}

# Clean up resources
cleanup() {
    log_warning "Cleaning up resources in namespace: $NAMESPACE"
    
    read -p "Are you sure you want to delete all resources in $NAMESPACE? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        kubectl delete namespace "$NAMESPACE"
        log_success "Cleanup completed"
    else
        log_info "Cleanup cancelled"
    fi
}

# Main execution
main() {
    local COMMAND=""
    local ARGS=()

    parse_args "$@"

    if [[ -z "${COMMAND:-}" ]]; then
        log_error "No command specified"
        show_help
        exit 1
    fi

    log_info "RWA Platform Deployment"
    log_info "Environment: $ENVIRONMENT"
    log_info "Namespace: $NAMESPACE"
    log_info "Version: $VERSION"
    log_info "Registry: $DOCKER_REGISTRY"
    echo

    case $COMMAND in
        build)
            check_prerequisites
            build_images
            ;;
        push)
            check_prerequisites
            push_images
            ;;
        deploy)
            check_prerequisites
            build_images
            push_images
            deploy_to_k8s
            ;;
        rollback)
            check_prerequisites
            rollback_deployment
            ;;
        status)
            check_prerequisites
            check_status
            ;;
        logs)
            check_prerequisites
            show_logs
            ;;
        clean)
            check_prerequisites
            cleanup
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"
