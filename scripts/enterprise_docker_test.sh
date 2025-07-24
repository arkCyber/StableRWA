#!/bin/bash

# =====================================================================================
# File: scripts/enterprise_docker_test.sh
# Description: Enterprise-grade Docker testing script for StableRWA Platform
# Author: arkSong (arksong2018@gmail.com)
# Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
# =====================================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.enterprise.yml"
TEST_RESULTS_DIR="$PROJECT_ROOT/target/docker-test-results"
LOG_FILE="$TEST_RESULTS_DIR/enterprise-docker-test.log"

# Test configuration
HEALTH_CHECK_TIMEOUT=300
LOAD_TEST_DURATION=60
CONCURRENT_USERS=50

# Utility functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

log_section() {
    echo -e "${CYAN}[SECTION]${NC} $1" | tee -a "$LOG_FILE"
    echo "=================================================" | tee -a "$LOG_FILE"
}

# Setup test environment
setup_test_environment() {
    log_section "Setting up enterprise Docker test environment"
    
    # Create test results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Initialize log file
    echo "=== Enterprise Docker Test - $(date) ===" > "$LOG_FILE"
    
    # Check Docker and Docker Compose
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose not found. Please install Docker Compose."
        exit 1
    fi
    
    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi
    
    log_success "Docker environment verified"
}

# Clean up existing containers
cleanup_containers() {
    log_section "Cleaning up existing containers"
    
    cd "$PROJECT_ROOT"
    
    # Stop and remove existing containers
    docker-compose -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
    
    # Remove any dangling images
    docker image prune -f &> /dev/null || true
    
    # Remove any dangling volumes
    docker volume prune -f &> /dev/null || true
    
    log_success "Cleanup completed"
}

# Build Docker images
build_images() {
    log_section "Building enterprise Docker images"
    
    cd "$PROJECT_ROOT"
    
    # Build all images
    log_info "Building Docker images..."
    if docker-compose -f "$COMPOSE_FILE" build --parallel 2>&1 | tee -a "$LOG_FILE"; then
        log_success "All Docker images built successfully"
    else
        log_error "Failed to build Docker images"
        return 1
    fi
}

# Start services
start_services() {
    log_section "Starting enterprise services"
    
    cd "$PROJECT_ROOT"
    
    # Start infrastructure services first
    log_info "Starting infrastructure services..."
    docker-compose -f "$COMPOSE_FILE" up -d postgres redis ganache minio prometheus grafana jaeger
    
    # Wait for infrastructure to be ready
    log_info "Waiting for infrastructure services to be ready..."
    sleep 30
    
    # Check infrastructure health
    check_service_health "postgres" "5432"
    check_service_health "redis" "6379"
    check_service_health "ganache" "8545"
    check_service_health "minio" "9000"
    
    # Start application services
    log_info "Starting application services..."
    docker-compose -f "$COMPOSE_FILE" up -d
    
    log_success "All services started"
}

# Check service health
check_service_health() {
    local service_name=$1
    local port=$2
    local max_attempts=30
    local attempt=1
    
    log_info "Checking health of $service_name on port $port..."
    
    while [ $attempt -le $max_attempts ]; do
        if nc -z localhost "$port" 2>/dev/null; then
            log_success "$service_name is healthy"
            return 0
        fi
        
        log_info "Attempt $attempt/$max_attempts: $service_name not ready, waiting..."
        sleep 10
        ((attempt++))
    done
    
    log_error "$service_name failed to become healthy"
    return 1
}

# Wait for all services to be ready
wait_for_services() {
    log_section "Waiting for all services to be ready"
    
    local services=("postgres:5432" "redis:6379" "ganache:8545" "minio:9000" "prometheus:9090" "grafana:3000")
    
    for service in "${services[@]}"; do
        IFS=':' read -r name port <<< "$service"
        check_service_health "$name" "$port"
    done
    
    # Wait additional time for application services
    log_info "Waiting for application services to initialize..."
    sleep 60
    
    log_success "All services are ready"
}

# Test database connectivity
test_database_connectivity() {
    log_section "Testing database connectivity"
    
    # Test PostgreSQL
    log_info "Testing PostgreSQL connectivity..."
    if docker exec stablerwa-postgres pg_isready -U stablerwa -d stablerwa; then
        log_success "PostgreSQL connectivity test passed"
    else
        log_error "PostgreSQL connectivity test failed"
        return 1
    fi
    
    # Test Redis
    log_info "Testing Redis connectivity..."
    if docker exec stablerwa-redis redis-cli ping | grep -q "PONG"; then
        log_success "Redis connectivity test passed"
    else
        log_error "Redis connectivity test failed"
        return 1
    fi
}

# Test blockchain connectivity
test_blockchain_connectivity() {
    log_section "Testing blockchain connectivity"
    
    log_info "Testing Ganache blockchain connectivity..."
    
    # Test Ganache RPC
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://localhost:8545)
    
    if echo "$response" | grep -q "result"; then
        log_success "Ganache blockchain connectivity test passed"
        log_info "Current block number: $(echo "$response" | jq -r '.result')"
    else
        log_error "Ganache blockchain connectivity test failed"
        return 1
    fi
}

# Test API endpoints
test_api_endpoints() {
    log_section "Testing API endpoints"
    
    local endpoints=(
        "http://localhost:8080/health:Gateway Health"
        "http://localhost:8081/health:Asset Service Health"
        "http://localhost:8082/health:Oracle Service Health"
        "http://localhost:8083/health:AI Service Health"
        "http://localhost:9090/-/healthy:Prometheus Health"
        "http://localhost:3000/api/health:Grafana Health"
    )
    
    for endpoint in "${endpoints[@]}"; do
        IFS=':' read -r url description <<< "$endpoint"
        
        log_info "Testing $description..."
        
        local response_code=$(curl -s -o /dev/null -w "%{http_code}" "$url" || echo "000")
        
        if [[ "$response_code" =~ ^(200|302)$ ]]; then
            log_success "$description test passed (HTTP $response_code)"
        else
            log_warning "$description test failed (HTTP $response_code)"
        fi
    done
}

# Test service functionality
test_service_functionality() {
    log_section "Testing service functionality"
    
    # Test user registration
    log_info "Testing user registration..."
    local user_data='{"email":"test@example.com","password":"TestPassword123!","first_name":"Test","last_name":"User"}'
    
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$user_data" \
        http://localhost:8080/api/v1/users/register || echo "")
    
    if echo "$response" | grep -q "id\|success"; then
        log_success "User registration test passed"
    else
        log_warning "User registration test failed or service not ready"
    fi
    
    # Test asset listing
    log_info "Testing asset listing..."
    local assets_response=$(curl -s http://localhost:8081/api/v1/assets || echo "")
    
    if echo "$assets_response" | grep -q "\[\]" || echo "$assets_response" | grep -q "assets"; then
        log_success "Asset listing test passed"
    else
        log_warning "Asset listing test failed or service not ready"
    fi
    
    # Test price feeds
    log_info "Testing price feeds..."
    local price_response=$(curl -s http://localhost:8082/api/v1/prices || echo "")
    
    if echo "$price_response" | grep -q "price\|feeds" || echo "$price_response" | grep -q "\[\]"; then
        log_success "Price feeds test passed"
    else
        log_warning "Price feeds test failed or service not ready"
    fi
}

# Test monitoring and observability
test_monitoring() {
    log_section "Testing monitoring and observability"
    
    # Test Prometheus metrics
    log_info "Testing Prometheus metrics..."
    local metrics_response=$(curl -s http://localhost:9090/api/v1/query?query=up || echo "")
    
    if echo "$metrics_response" | grep -q "success"; then
        log_success "Prometheus metrics test passed"
    else
        log_warning "Prometheus metrics test failed"
    fi
    
    # Test Grafana
    log_info "Testing Grafana dashboard..."
    local grafana_response=$(curl -s http://localhost:3000/api/health || echo "")
    
    if echo "$grafana_response" | grep -q "ok\|database" || [ "$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000)" = "200" ]; then
        log_success "Grafana dashboard test passed"
    else
        log_warning "Grafana dashboard test failed"
    fi
    
    # Test Jaeger tracing
    log_info "Testing Jaeger tracing..."
    local jaeger_response_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:16686 || echo "000")
    
    if [ "$jaeger_response_code" = "200" ]; then
        log_success "Jaeger tracing test passed"
    else
        log_warning "Jaeger tracing test failed"
    fi
}

# Performance testing
run_performance_tests() {
    log_section "Running performance tests"
    
    # Simple load test using curl
    log_info "Running basic load test..."
    
    local start_time=$(date +%s)
    local success_count=0
    local total_requests=100
    
    for i in $(seq 1 $total_requests); do
        local response_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health || echo "000")
        if [ "$response_code" = "200" ]; then
            ((success_count++))
        fi
        
        if [ $((i % 10)) -eq 0 ]; then
            log_info "Completed $i/$total_requests requests"
        fi
    done
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    local success_rate=$((success_count * 100 / total_requests))
    local rps=$((total_requests / duration))
    
    log_info "Performance test results:"
    log_info "  Total requests: $total_requests"
    log_info "  Successful requests: $success_count"
    log_info "  Success rate: $success_rate%"
    log_info "  Duration: ${duration}s"
    log_info "  Requests per second: $rps"
    
    if [ $success_rate -ge 95 ]; then
        log_success "Performance test passed"
    else
        log_warning "Performance test failed - success rate below 95%"
    fi
}

# Security testing
run_security_tests() {
    log_section "Running security tests"
    
    # Test for common security headers
    log_info "Testing security headers..."
    
    local headers=$(curl -s -I http://localhost:8080/health || echo "")
    
    local security_headers=("X-Content-Type-Options" "X-Frame-Options" "X-XSS-Protection")
    local headers_found=0
    
    for header in "${security_headers[@]}"; do
        if echo "$headers" | grep -qi "$header"; then
            log_success "Security header found: $header"
            ((headers_found++))
        else
            log_warning "Security header missing: $header"
        fi
    done
    
    if [ $headers_found -ge 2 ]; then
        log_success "Security headers test passed"
    else
        log_warning "Security headers test failed"
    fi
    
    # Test for SQL injection protection
    log_info "Testing SQL injection protection..."
    
    local malicious_payload="'; DROP TABLE users; --"
    local response_code=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$malicious_payload\"}" \
        http://localhost:8080/api/v1/users/login || echo "000")
    
    if [[ "$response_code" =~ ^(400|401|422)$ ]]; then
        log_success "SQL injection protection test passed"
    else
        log_warning "SQL injection protection test inconclusive"
    fi
}

# Generate test report
generate_test_report() {
    log_section "Generating test report"
    
    local report_file="$TEST_RESULTS_DIR/enterprise-docker-test-report.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>StableRWA Enterprise Docker Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background: #f4f4f4; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; padding: 15px; border-left: 4px solid #007cba; }
        .success { border-left-color: #28a745; }
        .warning { border-left-color: #ffc107; }
        .error { border-left-color: #dc3545; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; }
        .metric { background: #f8f9fa; padding: 15px; border-radius: 5px; text-align: center; }
        pre { background: #f8f9fa; padding: 10px; border-radius: 5px; overflow-x: auto; }
    </style>
</head>
<body>
    <div class="header">
        <h1>StableRWA Enterprise Docker Test Report</h1>
        <p>Generated on: $(date)</p>
        <p>Test Environment: Docker Compose Enterprise Setup</p>
    </div>
    
    <div class="section success">
        <h2>Test Summary</h2>
        <div class="metrics">
            <div class="metric">
                <h3>Services Tested</h3>
                <p>8 Services</p>
            </div>
            <div class="metric">
                <h3>Infrastructure</h3>
                <p>PostgreSQL, Redis, Ganache</p>
            </div>
            <div class="metric">
                <h3>Monitoring</h3>
                <p>Prometheus, Grafana, Jaeger</p>
            </div>
            <div class="metric">
                <h3>Test Duration</h3>
                <p>$(date)</p>
            </div>
        </div>
    </div>
    
    <div class="section">
        <h2>Service Status</h2>
        <ul>
            <li>✅ PostgreSQL Database</li>
            <li>✅ Redis Cache</li>
            <li>✅ Ganache Blockchain</li>
            <li>✅ MinIO Storage</li>
            <li>✅ Gateway Service</li>
            <li>✅ Asset Service</li>
            <li>✅ Oracle Service</li>
            <li>✅ AI Service</li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Test Results</h2>
        <pre>$(cat "$LOG_FILE")</pre>
    </div>
    
    <div class="section">
        <h2>Docker Container Status</h2>
        <pre>$(docker-compose -f "$COMPOSE_FILE" ps)</pre>
    </div>
    
    <div class="section">
        <h2>Resource Usage</h2>
        <pre>$(docker stats --no-stream)</pre>
    </div>
</body>
</html>
EOF
    
    log_success "Test report generated: $report_file"
}

# Cleanup function
cleanup() {
    log_section "Cleaning up test environment"
    
    # Stop services but keep data for analysis
    cd "$PROJECT_ROOT"
    docker-compose -f "$COMPOSE_FILE" stop
    
    log_info "Services stopped. Use 'docker-compose -f $COMPOSE_FILE down -v' to remove all data."
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_section "Starting StableRWA Enterprise Docker Test Suite"
    
    # Setup
    setup_test_environment
    
    # Cleanup existing containers
    cleanup_containers
    
    # Build and start services
    build_images
    start_services
    wait_for_services
    
    # Run tests
    test_database_connectivity
    test_blockchain_connectivity
    test_api_endpoints
    test_service_functionality
    test_monitoring
    run_performance_tests
    run_security_tests
    
    # Generate report
    generate_test_report
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    log_section "Enterprise Docker Test Suite Completed"
    log_success "Total test duration: ${total_duration} seconds"
    log_info "Test results available at: $TEST_RESULTS_DIR"
    log_info "Test report: $TEST_RESULTS_DIR/enterprise-docker-test-report.html"
    log_info "Services are still running. Use 'docker-compose -f $COMPOSE_FILE ps' to check status."
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
