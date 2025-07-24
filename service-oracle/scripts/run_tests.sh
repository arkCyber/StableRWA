#!/bin/bash

# =====================================================================================
# RWA Tokenization Platform - Oracle Service Test Runner
# 
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_DATABASE_URL="${TEST_DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/oracle_test}"
TEST_REDIS_URL="${TEST_REDIS_URL:-redis://localhost:6379/1}"
COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-80}"

# Functions
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

check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check if PostgreSQL is available
    if ! pg_isready -h localhost -p 5432 &> /dev/null; then
        log_warning "PostgreSQL not available. Some integration tests may fail."
    fi
    
    # Check if Redis is available
    if ! redis-cli -h localhost -p 6379 ping &> /dev/null; then
        log_warning "Redis not available. Some integration tests may fail."
    fi
    
    log_success "Dependencies checked"
}

setup_test_environment() {
    log_info "Setting up test environment..."
    
    # Set environment variables
    export TEST_DATABASE_URL
    export TEST_REDIS_URL
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    
    # Create test database if it doesn't exist
    if pg_isready -h localhost -p 5432 &> /dev/null; then
        log_info "Creating test database..."
        createdb oracle_test 2>/dev/null || log_info "Test database already exists"
        
        # Run migrations
        if command -v sqlx &> /dev/null; then
            log_info "Running database migrations..."
            sqlx migrate run --database-url "$TEST_DATABASE_URL" || log_warning "Migration failed"
        else
            log_warning "sqlx-cli not found. Skipping migrations."
        fi
    fi
    
    log_success "Test environment setup complete"
}

cleanup_test_environment() {
    log_info "Cleaning up test environment..."
    
    # Clean up test database
    if pg_isready -h localhost -p 5432 &> /dev/null; then
        dropdb oracle_test 2>/dev/null || log_info "Test database cleanup skipped"
    fi
    
    # Clean up Redis test data
    if redis-cli -h localhost -p 6379 ping &> /dev/null; then
        redis-cli -h localhost -p 6379 -n 1 FLUSHDB &> /dev/null || log_info "Redis cleanup skipped"
    fi
    
    log_success "Test environment cleaned up"
}

run_unit_tests() {
    log_info "Running unit tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run unit tests with output capture
    if cargo test --lib --verbose 2>&1 | tee test_output.log; then
        log_success "Unit tests passed"
        return 0
    else
        log_error "Unit tests failed"
        return 1
    fi
}

run_integration_tests() {
    log_info "Running integration tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run integration tests
    if cargo test --test integration_tests --verbose 2>&1 | tee -a test_output.log; then
        log_success "Integration tests passed"
        return 0
    else
        log_error "Integration tests failed"
        return 1
    fi
}

run_doc_tests() {
    log_info "Running documentation tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run doc tests
    if cargo test --doc --verbose 2>&1 | tee -a test_output.log; then
        log_success "Documentation tests passed"
        return 0
    else
        log_error "Documentation tests failed"
        return 1
    fi
}

run_benchmark_tests() {
    log_info "Running benchmark tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run benchmark tests if available
    if cargo test --release --verbose bench 2>&1 | tee -a test_output.log; then
        log_success "Benchmark tests passed"
        return 0
    else
        log_warning "Benchmark tests failed or not available"
        return 0  # Don't fail the entire test suite for benchmarks
    fi
}

generate_coverage_report() {
    log_info "Generating coverage report..."
    
    cd "$PROJECT_ROOT"
    
    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_warning "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin || {
            log_error "Failed to install cargo-tarpaulin"
            return 1
        }
    fi
    
    # Generate coverage report
    if cargo tarpaulin --out Html --output-dir coverage --verbose 2>&1 | tee -a test_output.log; then
        # Extract coverage percentage
        COVERAGE=$(grep -o '[0-9]*\.[0-9]*%' coverage/tarpaulin-report.html | head -1 | sed 's/%//')
        
        if [ -n "$COVERAGE" ]; then
            log_info "Code coverage: ${COVERAGE}%"
            
            # Check if coverage meets threshold
            if (( $(echo "$COVERAGE >= $COVERAGE_THRESHOLD" | bc -l) )); then
                log_success "Coverage threshold met (${COVERAGE}% >= ${COVERAGE_THRESHOLD}%)"
            else
                log_error "Coverage threshold not met (${COVERAGE}% < ${COVERAGE_THRESHOLD}%)"
                return 1
            fi
        else
            log_warning "Could not extract coverage percentage"
        fi
        
        log_success "Coverage report generated: coverage/tarpaulin-report.html"
        return 0
    else
        log_error "Failed to generate coverage report"
        return 1
    fi
}

run_linting() {
    log_info "Running code linting..."
    
    cd "$PROJECT_ROOT"
    
    # Run clippy
    if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee -a test_output.log; then
        log_success "Linting passed"
        return 0
    else
        log_error "Linting failed"
        return 1
    fi
}

run_formatting_check() {
    log_info "Checking code formatting..."
    
    cd "$PROJECT_ROOT"
    
    # Check formatting
    if cargo fmt -- --check 2>&1 | tee -a test_output.log; then
        log_success "Code formatting is correct"
        return 0
    else
        log_error "Code formatting issues found. Run 'cargo fmt' to fix."
        return 1
    fi
}

run_security_audit() {
    log_info "Running security audit..."
    
    cd "$PROJECT_ROOT"
    
    # Check if cargo-audit is installed
    if ! command -v cargo-audit &> /dev/null; then
        log_warning "cargo-audit not found. Installing..."
        cargo install cargo-audit || {
            log_error "Failed to install cargo-audit"
            return 1
        }
    fi
    
    # Run security audit
    if cargo audit 2>&1 | tee -a test_output.log; then
        log_success "Security audit passed"
        return 0
    else
        log_error "Security audit failed"
        return 1
    fi
}

generate_test_report() {
    log_info "Generating test report..."
    
    cd "$PROJECT_ROOT"
    
    # Create test report
    cat > test_report.md << EOF
# Oracle Service Test Report

**Generated**: $(date)
**Environment**: $(uname -a)
**Rust Version**: $(rustc --version)

## Test Results

EOF
    
    # Add test results to report
    if [ -f test_output.log ]; then
        echo "## Test Output" >> test_report.md
        echo '```' >> test_report.md
        tail -50 test_output.log >> test_report.md
        echo '```' >> test_report.md
    fi
    
    # Add coverage information if available
    if [ -f coverage/tarpaulin-report.html ]; then
        echo "## Coverage Report" >> test_report.md
        echo "Coverage report available at: coverage/tarpaulin-report.html" >> test_report.md
    fi
    
    log_success "Test report generated: test_report.md"
}

print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -u, --unit              Run only unit tests"
    echo "  -i, --integration       Run only integration tests"
    echo "  -c, --coverage          Generate coverage report"
    echo "  -l, --lint              Run linting only"
    echo "  -f, --format            Check formatting only"
    echo "  -s, --security          Run security audit only"
    echo "  -a, --all               Run all tests and checks (default)"
    echo "  --no-cleanup            Skip cleanup after tests"
    echo "  --coverage-threshold N  Set coverage threshold (default: 80)"
    echo ""
    echo "Environment Variables:"
    echo "  TEST_DATABASE_URL       Test database URL"
    echo "  TEST_REDIS_URL          Test Redis URL"
    echo "  COVERAGE_THRESHOLD      Minimum coverage percentage"
}

# Main execution
main() {
    local run_unit=false
    local run_integration=false
    local run_coverage=false
    local run_lint=false
    local run_format=false
    local run_security=false
    local run_all=true
    local skip_cleanup=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_usage
                exit 0
                ;;
            -u|--unit)
                run_unit=true
                run_all=false
                shift
                ;;
            -i|--integration)
                run_integration=true
                run_all=false
                shift
                ;;
            -c|--coverage)
                run_coverage=true
                run_all=false
                shift
                ;;
            -l|--lint)
                run_lint=true
                run_all=false
                shift
                ;;
            -f|--format)
                run_format=true
                run_all=false
                shift
                ;;
            -s|--security)
                run_security=true
                run_all=false
                shift
                ;;
            -a|--all)
                run_all=true
                shift
                ;;
            --no-cleanup)
                skip_cleanup=true
                shift
                ;;
            --coverage-threshold)
                COVERAGE_THRESHOLD="$2"
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done
    
    # Start test execution
    log_info "Starting Oracle Service test suite..."
    log_info "Project root: $PROJECT_ROOT"
    
    # Check dependencies
    check_dependencies
    
    # Setup test environment
    setup_test_environment
    
    # Track test results
    local exit_code=0
    
    # Run tests based on options
    if [ "$run_all" = true ]; then
        run_formatting_check || exit_code=1
        run_linting || exit_code=1
        run_security_audit || exit_code=1
        run_unit_tests || exit_code=1
        run_integration_tests || exit_code=1
        run_doc_tests || exit_code=1
        run_benchmark_tests || exit_code=1
        generate_coverage_report || exit_code=1
    else
        [ "$run_format" = true ] && (run_formatting_check || exit_code=1)
        [ "$run_lint" = true ] && (run_linting || exit_code=1)
        [ "$run_security" = true ] && (run_security_audit || exit_code=1)
        [ "$run_unit" = true ] && (run_unit_tests || exit_code=1)
        [ "$run_integration" = true ] && (run_integration_tests || exit_code=1)
        [ "$run_coverage" = true ] && (generate_coverage_report || exit_code=1)
    fi
    
    # Generate test report
    generate_test_report
    
    # Cleanup
    if [ "$skip_cleanup" = false ]; then
        cleanup_test_environment
    fi
    
    # Final result
    if [ $exit_code -eq 0 ]; then
        log_success "All tests completed successfully!"
    else
        log_error "Some tests failed. Check the output above for details."
    fi
    
    exit $exit_code
}

# Run main function with all arguments
main "$@"
