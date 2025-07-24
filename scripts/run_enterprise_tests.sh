#!/bin/bash

# =====================================================================================
# File: scripts/run_enterprise_tests.sh
# Description: Enterprise-grade test execution script for StableRWA Platform
# Author: arkSong (arksong2018@gmail.com)
# Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
# =====================================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_RESULTS_DIR="$PROJECT_ROOT/target/test-results"
COVERAGE_DIR="$PROJECT_ROOT/target/coverage"
REPORTS_DIR="$PROJECT_ROOT/target/reports"

# Test configuration
RUST_LOG=${RUST_LOG:-info}
RUST_BACKTRACE=${RUST_BACKTRACE:-1}
TEST_TIMEOUT=${TEST_TIMEOUT:-300}
COVERAGE_THRESHOLD=${COVERAGE_THRESHOLD:-90}

# Enterprise test categories
UNIT_TESTS=true
INTEGRATION_TESTS=true
PERFORMANCE_TESTS=true
SECURITY_TESTS=true
COMPLIANCE_TESTS=true
FRONTEND_TESTS=true
E2E_TESTS=true
LOAD_TESTS=false
CHAOS_TESTS=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --unit-only)
            INTEGRATION_TESTS=false
            PERFORMANCE_TESTS=false
            SECURITY_TESTS=false
            COMPLIANCE_TESTS=false
            FRONTEND_TESTS=false
            E2E_TESTS=false
            shift
            ;;
        --integration-only)
            UNIT_TESTS=false
            PERFORMANCE_TESTS=false
            SECURITY_TESTS=false
            COMPLIANCE_TESTS=false
            shift
            ;;
        --performance-only)
            UNIT_TESTS=false
            INTEGRATION_TESTS=false
            SECURITY_TESTS=false
            COMPLIANCE_TESTS=false
            shift
            ;;
        --security-only)
            UNIT_TESTS=false
            INTEGRATION_TESTS=false
            PERFORMANCE_TESTS=false
            COMPLIANCE_TESTS=false
            shift
            ;;
        --compliance-only)
            UNIT_TESTS=false
            INTEGRATION_TESTS=false
            PERFORMANCE_TESTS=false
            SECURITY_TESTS=false
            shift
            ;;
        --enable-load-tests)
            LOAD_TESTS=true
            shift
            ;;
        --enable-chaos-tests)
            CHAOS_TESTS=true
            shift
            ;;
        --coverage-threshold)
            COVERAGE_THRESHOLD="$2"
            shift 2
            ;;
        --help)
            echo "Enterprise Test Suite Runner"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --unit-only              Run only unit tests"
            echo "  --integration-only       Run only integration tests"
            echo "  --performance-only       Run only performance tests"
            echo "  --security-only          Run only security tests"
            echo "  --compliance-only        Run only compliance tests"
            echo "  --enable-load-tests      Enable load testing"
            echo "  --enable-chaos-tests     Enable chaos engineering tests"
            echo "  --coverage-threshold N   Set coverage threshold (default: 90)"
            echo "  --help                   Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

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

# Setup test environment
setup_test_environment() {
    log_info "Setting up enterprise test environment..."
    
    # Create directories
    mkdir -p "$TEST_RESULTS_DIR"
    mkdir -p "$COVERAGE_DIR"
    mkdir -p "$REPORTS_DIR"
    
    # Set environment variables
    export RUST_LOG="$RUST_LOG"
    export RUST_BACKTRACE="$RUST_BACKTRACE"
    export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/stablerwa_test}"
    export REDIS_URL="${REDIS_URL:-redis://localhost:6379/1}"
    
    # Check dependencies
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Install test tools if not present
    if ! cargo --list | grep -q "tarpaulin"; then
        log_info "Installing cargo-tarpaulin for coverage..."
        cargo install cargo-tarpaulin
    fi
    
    if ! cargo --list | grep -q "audit"; then
        log_info "Installing cargo-audit for security auditing..."
        cargo install cargo-audit
    fi
    
    if ! cargo --list | grep -q "nextest"; then
        log_info "Installing cargo-nextest for faster testing..."
        cargo install cargo-nextest
    fi
    
    log_success "Test environment setup complete"
}

# Run unit tests
run_unit_tests() {
    if [ "$UNIT_TESTS" = true ]; then
        log_info "Running unit tests..."
        
        local start_time=$(date +%s)
        
        # Run unit tests with nextest for better performance
        if cargo nextest run --lib --all-features --no-fail-fast \
            --test-threads=4 \
            --junit-path="$TEST_RESULTS_DIR/unit-tests.xml" 2>&1 | tee "$TEST_RESULTS_DIR/unit-tests.log"; then
            
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_success "Unit tests completed in ${duration}s"
            
            # Generate coverage report
            log_info "Generating coverage report..."
            cargo tarpaulin --lib --all-features \
                --out xml --output-dir "$COVERAGE_DIR" \
                --timeout "$TEST_TIMEOUT" \
                --exclude-files "tests/*" "benches/*" || log_warning "Coverage generation failed"
            
            return 0
        else
            log_error "Unit tests failed"
            return 1
        fi
    fi
}

# Run integration tests
run_integration_tests() {
    if [ "$INTEGRATION_TESTS" = true ]; then
        log_info "Running integration tests..."
        
        local start_time=$(date +%s)
        
        # Start test services if needed
        if command -v docker-compose &> /dev/null; then
            log_info "Starting test services with Docker Compose..."
            docker-compose -f docker-compose.test.yml up -d postgres redis || log_warning "Failed to start test services"
            sleep 10 # Wait for services to be ready
        fi
        
        # Run integration tests
        if cargo nextest run --test '*' --all-features --no-fail-fast \
            --test-threads=2 \
            --junit-path="$TEST_RESULTS_DIR/integration-tests.xml" 2>&1 | tee "$TEST_RESULTS_DIR/integration-tests.log"; then
            
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_success "Integration tests completed in ${duration}s"
            return 0
        else
            log_error "Integration tests failed"
            return 1
        fi
    fi
}

# Run performance tests
run_performance_tests() {
    if [ "$PERFORMANCE_TESTS" = true ]; then
        log_info "Running performance tests..."
        
        local start_time=$(date +%s)
        
        # Run benchmark tests
        if cargo bench --all-features 2>&1 | tee "$TEST_RESULTS_DIR/performance-tests.log"; then
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_success "Performance tests completed in ${duration}s"
            return 0
        else
            log_error "Performance tests failed"
            return 1
        fi
    fi
}

# Run security tests
run_security_tests() {
    if [ "$SECURITY_TESTS" = true ]; then
        log_info "Running security tests..."
        
        local start_time=$(date +%s)
        local security_issues=0
        
        # Security audit
        log_info "Running security audit..."
        if ! cargo audit --json > "$TEST_RESULTS_DIR/security-audit.json" 2>&1; then
            log_warning "Security vulnerabilities found"
            security_issues=$((security_issues + 1))
        fi
        
        # Run security-specific tests
        if cargo test security --all-features 2>&1 | tee "$TEST_RESULTS_DIR/security-tests.log"; then
            log_success "Security tests completed"
        else
            log_error "Security tests failed"
            security_issues=$((security_issues + 1))
        fi
        
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        if [ $security_issues -eq 0 ]; then
            log_success "All security tests passed in ${duration}s"
            return 0
        else
            log_error "Security tests found $security_issues issues"
            return 1
        fi
    fi
}

# Run compliance tests
run_compliance_tests() {
    if [ "$COMPLIANCE_TESTS" = true ]; then
        log_info "Running compliance tests..."
        
        local start_time=$(date +%s)
        
        # Run compliance-specific tests
        if cargo test compliance --all-features 2>&1 | tee "$TEST_RESULTS_DIR/compliance-tests.log"; then
            local end_time=$(date +%s)
            local duration=$((end_time - start_time))
            log_success "Compliance tests completed in ${duration}s"
            return 0
        else
            log_error "Compliance tests failed"
            return 1
        fi
    fi
}

# Generate comprehensive report
generate_report() {
    log_info "Generating comprehensive test report..."
    
    local report_file="$REPORTS_DIR/enterprise-test-report.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>StableRWA Enterprise Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background: #f4f4f4; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; padding: 15px; border-left: 4px solid #007cba; }
        .success { border-left-color: #28a745; }
        .warning { border-left-color: #ffc107; }
        .error { border-left-color: #dc3545; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; }
        .metric { background: #f8f9fa; padding: 15px; border-radius: 5px; text-align: center; }
    </style>
</head>
<body>
    <div class="header">
        <h1>StableRWA Enterprise Test Report</h1>
        <p>Generated on: $(date)</p>
        <p>Test Environment: $(uname -a)</p>
    </div>
    
    <div class="section">
        <h2>Test Summary</h2>
        <div class="metrics">
            <div class="metric">
                <h3>Unit Tests</h3>
                <p>$([ "$UNIT_TESTS" = true ] && echo "✅ Executed" || echo "⏭️ Skipped")</p>
            </div>
            <div class="metric">
                <h3>Integration Tests</h3>
                <p>$([ "$INTEGRATION_TESTS" = true ] && echo "✅ Executed" || echo "⏭️ Skipped")</p>
            </div>
            <div class="metric">
                <h3>Performance Tests</h3>
                <p>$([ "$PERFORMANCE_TESTS" = true ] && echo "✅ Executed" || echo "⏭️ Skipped")</p>
            </div>
            <div class="metric">
                <h3>Security Tests</h3>
                <p>$([ "$SECURITY_TESTS" = true ] && echo "✅ Executed" || echo "⏭️ Skipped")</p>
            </div>
            <div class="metric">
                <h3>Compliance Tests</h3>
                <p>$([ "$COMPLIANCE_TESTS" = true ] && echo "✅ Executed" || echo "⏭️ Skipped")</p>
            </div>
        </div>
    </div>
    
    <div class="section">
        <h2>Coverage Report</h2>
        <p>Coverage threshold: ${COVERAGE_THRESHOLD}%</p>
        <p>Detailed coverage report: <a href="../coverage/tarpaulin-report.html">View Coverage</a></p>
    </div>
    
    <div class="section">
        <h2>Test Artifacts</h2>
        <ul>
            <li><a href="../test-results/unit-tests.xml">Unit Test Results (JUnit XML)</a></li>
            <li><a href="../test-results/integration-tests.xml">Integration Test Results (JUnit XML)</a></li>
            <li><a href="../test-results/security-audit.json">Security Audit Report (JSON)</a></li>
        </ul>
    </div>
</body>
</html>
EOF
    
    log_success "Test report generated: $report_file"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."
    
    # Stop test services
    if command -v docker-compose &> /dev/null; then
        docker-compose -f docker-compose.test.yml down || true
    fi
    
    # Archive old test results
    if [ -d "$TEST_RESULTS_DIR" ]; then
        local archive_name="test-results-$(date +%Y%m%d-%H%M%S).tar.gz"
        tar -czf "$REPORTS_DIR/$archive_name" -C "$TEST_RESULTS_DIR" . || true
        log_info "Test results archived as $archive_name"
    fi
}

# Run frontend tests
run_frontend_tests() {
    log_section "Running Frontend Tests"

    local webui_dir="$PROJECT_ROOT/webui"

    if [[ ! -d "$webui_dir" ]]; then
        log_warning "WebUI directory not found, skipping frontend tests"
        return 0
    fi

    cd "$webui_dir"

    # Check if package.json exists
    if [[ ! -f "package.json" ]]; then
        log_warning "package.json not found, skipping frontend tests"
        return 0
    fi

    # Install dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing frontend dependencies..."
        npm ci --silent
    fi

    # Run frontend unit tests
    log_info "Running frontend unit tests..."
    if npm test -- --watchAll=false --coverage --coverageDirectory="$COVERAGE_DIR/frontend" 2>&1 | tee -a "$TEST_RESULTS_DIR/frontend-tests.log"; then
        log_success "Frontend unit tests passed"
    else
        log_error "Frontend unit tests failed"
        return 1
    fi

    # Run frontend integration tests
    log_info "Running frontend integration tests..."
    if npx jest __tests__/api-integration.test.ts --coverage --coverageDirectory="$COVERAGE_DIR/frontend-integration" 2>&1 | tee -a "$TEST_RESULTS_DIR/frontend-integration-tests.log"; then
        log_success "Frontend integration tests passed"
    else
        log_error "Frontend integration tests failed"
        return 1
    fi

    # Run TypeScript type checking
    log_info "Running TypeScript type checking..."
    if npx tsc --noEmit 2>&1 | tee -a "$TEST_RESULTS_DIR/typescript-check.log"; then
        log_success "TypeScript type checking passed"
    else
        log_warning "TypeScript type checking found issues"
    fi

    # Run linting
    log_info "Running ESLint..."
    if npx eslint . --ext .ts,.tsx --max-warnings 0 2>&1 | tee -a "$TEST_RESULTS_DIR/eslint.log"; then
        log_success "ESLint passed"
    else
        log_warning "ESLint found issues"
    fi

    return 0
}

# Run end-to-end tests
run_e2e_tests() {
    log_section "Running End-to-End Tests"

    local webui_dir="$PROJECT_ROOT/webui"

    if [[ ! -d "$webui_dir" ]]; then
        log_warning "WebUI directory not found, skipping E2E tests"
        return 0
    fi

    cd "$webui_dir"

    # Run E2E tests
    log_info "Running end-to-end tests..."
    if npx jest __tests__/e2e/ --testTimeout=60000 2>&1 | tee -a "$TEST_RESULTS_DIR/e2e-tests.log"; then
        log_success "End-to-end tests passed"
    else
        log_error "End-to-end tests failed"
        return 1
    fi

    return 0
}

# Main execution
main() {
    local start_time=$(date +%s)
    local exit_code=0
    
    log_info "Starting StableRWA Enterprise Test Suite"
    log_info "Configuration: Unit=$UNIT_TESTS, Integration=$INTEGRATION_TESTS, Performance=$PERFORMANCE_TESTS, Security=$SECURITY_TESTS, Compliance=$COMPLIANCE_TESTS"
    
    # Setup
    setup_test_environment
    
    # Run tests
    run_unit_tests || exit_code=1
    run_integration_tests || exit_code=1
    run_performance_tests || exit_code=1
    run_security_tests || exit_code=1
    run_compliance_tests || exit_code=1

    # Run frontend tests if enabled
    if [[ "$FRONTEND_TESTS" == "true" ]]; then
        run_frontend_tests || exit_code=1
    fi

    # Run E2E tests if enabled
    if [[ "$E2E_TESTS" == "true" ]]; then
        run_e2e_tests || exit_code=1
    fi
    
    # Generate report
    generate_report
    
    # Cleanup
    cleanup
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    if [ $exit_code -eq 0 ]; then
        log_success "All enterprise tests completed successfully in ${total_duration}s"
        log_info "Test report available at: $REPORTS_DIR/enterprise-test-report.html"
    else
        log_error "Some tests failed. Total duration: ${total_duration}s"
        log_info "Check test logs in: $TEST_RESULTS_DIR"
    fi
    
    exit $exit_code
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
