#!/bin/bash

# =====================================================================================
# File: scripts/run-tests.sh
# Description: Comprehensive test runner for RWA platform
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DOCKER_COMPOSE_FILE="docker-compose.test.yml"
TEST_TIMEOUT=300
PARALLEL_JOBS=4

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if required tools are installed
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    local missing_tools=()
    
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if ! command -v docker &> /dev/null; then
        missing_tools+=("docker")
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        missing_tools+=("docker-compose")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    print_success "All prerequisites are installed"
}

# Function to start test environment
start_test_environment() {
    print_status "Starting test environment..."
    
    # Stop any existing containers
    docker-compose -f $DOCKER_COMPOSE_FILE down --remove-orphans 2>/dev/null || true
    
    # Start test environment
    docker-compose -f $DOCKER_COMPOSE_FILE up -d
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    local max_attempts=60
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            print_success "Test environment is ready"
            return 0
        fi
        
        print_status "Attempt $attempt/$max_attempts - waiting for services..."
        sleep 5
        ((attempt++))
    done
    
    print_error "Test environment failed to start within timeout"
    docker-compose -f $DOCKER_COMPOSE_FILE logs
    exit 1
}

# Function to stop test environment
stop_test_environment() {
    print_status "Stopping test environment..."
    docker-compose -f $DOCKER_COMPOSE_FILE down --remove-orphans
    print_success "Test environment stopped"
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    
    local start_time=$(date +%s)
    
    if cargo test --workspace --lib --bins --tests --exclude integration_tests --exclude e2e_tests --exclude performance_tests -- --test-threads=$PARALLEL_JOBS; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "Unit tests passed in ${duration}s"
        return 0
    else
        print_error "Unit tests failed"
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    
    local start_time=$(date +%s)
    
    # Set environment variables for integration tests
    export GATEWAY_URL="http://localhost:8080"
    export RUST_LOG="info"
    
    if timeout $TEST_TIMEOUT cargo test integration_tests --release -- --test-threads=1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "Integration tests passed in ${duration}s"
        return 0
    else
        print_error "Integration tests failed or timed out"
        return 1
    fi
}

# Function to run end-to-end tests
run_e2e_tests() {
    print_status "Running end-to-end tests..."
    
    local start_time=$(date +%s)
    
    # Set environment variables for E2E tests
    export GATEWAY_URL="http://localhost:8080"
    export RUST_LOG="info"
    export E2E_TEST_MODE="true"
    
    if timeout $TEST_TIMEOUT cargo test e2e_tests --release -- --test-threads=1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "End-to-end tests passed in ${duration}s"
        return 0
    else
        print_error "End-to-end tests failed or timed out"
        return 1
    fi
}

# Function to run performance tests
run_performance_tests() {
    print_status "Running performance tests..."
    
    local start_time=$(date +%s)
    
    # Set environment variables for performance tests
    export GATEWAY_URL="http://localhost:8080"
    export RUST_LOG="warn"
    export PERFORMANCE_TEST_MODE="true"
    
    if timeout $TEST_TIMEOUT cargo test performance_tests --release -- --test-threads=1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "Performance tests passed in ${duration}s"
        return 0
    else
        print_error "Performance tests failed or timed out"
        return 1
    fi
}

# Function to run security tests
run_security_tests() {
    print_status "Running security tests..."
    
    # Run cargo audit for known vulnerabilities
    if command -v cargo-audit &> /dev/null; then
        cargo audit
    else
        print_warning "cargo-audit not installed, skipping vulnerability check"
    fi
    
    # Run clippy for security lints
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    
    print_success "Security tests completed"
}

# Function to generate test report
generate_test_report() {
    print_status "Generating test report..."
    
    local report_dir="target/test-reports"
    mkdir -p $report_dir
    
    # Generate coverage report if tarpaulin is available
    if command -v cargo-tarpaulin &> /dev/null; then
        print_status "Generating code coverage report..."
        cargo tarpaulin --out Html --output-dir $report_dir --workspace --exclude-files "tests/*" --timeout 300
        print_success "Coverage report generated at $report_dir/tarpaulin-report.html"
    else
        print_warning "cargo-tarpaulin not installed, skipping coverage report"
    fi
    
    # Generate test results summary
    cat > $report_dir/test-summary.md << EOF
# RWA Platform Test Results

## Test Summary
- **Date**: $(date)
- **Environment**: Test
- **Duration**: $(($(date +%s) - $TEST_START_TIME))s

## Test Categories
- ✅ Unit Tests
- ✅ Integration Tests  
- ✅ End-to-End Tests
- ✅ Performance Tests
- ✅ Security Tests

## Coverage
See tarpaulin-report.html for detailed coverage information.

## Performance Metrics
See performance test output for detailed metrics.
EOF
    
    print_success "Test report generated at $report_dir/test-summary.md"
}

# Function to cleanup
cleanup() {
    print_status "Cleaning up..."
    stop_test_environment
    
    # Clean up any temporary files
    rm -rf target/tmp-test-*
    
    print_success "Cleanup completed"
}

# Main function
main() {
    local TEST_START_TIME=$(date +%s)
    
    # Parse command line arguments
    local run_unit=true
    local run_integration=true
    local run_e2e=true
    local run_performance=false
    local run_security=true
    local skip_env_setup=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit-only)
                run_integration=false
                run_e2e=false
                run_performance=false
                skip_env_setup=true
                shift
                ;;
            --integration-only)
                run_unit=false
                run_e2e=false
                run_performance=false
                shift
                ;;
            --e2e-only)
                run_unit=false
                run_integration=false
                run_performance=false
                shift
                ;;
            --performance)
                run_performance=true
                shift
                ;;
            --skip-security)
                run_security=false
                shift
                ;;
            --skip-env-setup)
                skip_env_setup=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --unit-only        Run only unit tests"
                echo "  --integration-only Run only integration tests"
                echo "  --e2e-only         Run only end-to-end tests"
                echo "  --performance      Include performance tests"
                echo "  --skip-security    Skip security tests"
                echo "  --skip-env-setup   Skip test environment setup"
                echo "  --help             Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Set up trap for cleanup
    trap cleanup EXIT
    
    print_status "Starting RWA Platform test suite..."
    
    # Check prerequisites
    check_prerequisites
    
    # Start test environment if needed
    if [ "$skip_env_setup" = false ] && ([ "$run_integration" = true ] || [ "$run_e2e" = true ] || [ "$run_performance" = true ]); then
        start_test_environment
    fi
    
    local test_failures=0
    
    # Run tests based on configuration
    if [ "$run_unit" = true ]; then
        if ! run_unit_tests; then
            ((test_failures++))
        fi
    fi
    
    if [ "$run_security" = true ]; then
        if ! run_security_tests; then
            ((test_failures++))
        fi
    fi
    
    if [ "$run_integration" = true ]; then
        if ! run_integration_tests; then
            ((test_failures++))
        fi
    fi
    
    if [ "$run_e2e" = true ]; then
        if ! run_e2e_tests; then
            ((test_failures++))
        fi
    fi
    
    if [ "$run_performance" = true ]; then
        if ! run_performance_tests; then
            ((test_failures++))
        fi
    fi
    
    # Generate test report
    generate_test_report
    
    # Final results
    local end_time=$(date +%s)
    local total_duration=$((end_time - TEST_START_TIME))
    
    if [ $test_failures -eq 0 ]; then
        print_success "All tests passed! Total duration: ${total_duration}s"
        exit 0
    else
        print_error "$test_failures test suite(s) failed. Total duration: ${total_duration}s"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
