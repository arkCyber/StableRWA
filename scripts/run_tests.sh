#!/bin/bash

# =====================================================================================
# File: scripts/run_tests.sh
# Description: Comprehensive test runner for StableRWA platform
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
CARGO_TEST_THREADS=${CARGO_TEST_THREADS:-4}
TEST_TIMEOUT=${TEST_TIMEOUT:-300}
COVERAGE_THRESHOLD=${COVERAGE_THRESHOLD:-80}

echo -e "${BLUE}🚀 Starting StableRWA Platform Test Suite${NC}"
echo "=================================================="

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}📋 $1${NC}"
    echo "----------------------------------------"
}

# Function to print success messages
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print warning messages
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to print error messages
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to run tests for a specific crate
run_crate_tests() {
    local crate_name=$1
    local crate_path=$2
    
    echo -e "\n${YELLOW}Testing $crate_name...${NC}"
    
    if [ -d "$crate_path" ]; then
        cd "$crate_path"
        
        # Run unit tests
        if cargo test --lib --bins --tests --timeout $TEST_TIMEOUT; then
            print_success "$crate_name unit tests passed"
        else
            print_error "$crate_name unit tests failed"
            return 1
        fi
        
        # Run integration tests if they exist
        if [ -d "tests" ]; then
            if cargo test --tests --timeout $TEST_TIMEOUT; then
                print_success "$crate_name integration tests passed"
            else
                print_error "$crate_name integration tests failed"
                return 1
            fi
        fi
        
        # Run doc tests
        if cargo test --doc --timeout $TEST_TIMEOUT; then
            print_success "$crate_name doc tests passed"
        else
            print_warning "$crate_name doc tests failed (non-critical)"
        fi
        
        cd - > /dev/null
    else
        print_warning "$crate_name directory not found, skipping"
    fi
}

# Function to check code formatting
check_formatting() {
    print_section "Code Formatting Check"
    
    if command -v rustfmt >/dev/null 2>&1; then
        if cargo fmt --all -- --check; then
            print_success "Code formatting is correct"
        else
            print_error "Code formatting issues found. Run 'cargo fmt' to fix."
            return 1
        fi
    else
        print_warning "rustfmt not found, skipping formatting check"
    fi
}

# Function to run clippy lints
run_clippy() {
    print_section "Clippy Linting"
    
    if command -v cargo-clippy >/dev/null 2>&1; then
        if cargo clippy --all-targets --all-features -- -D warnings; then
            print_success "Clippy linting passed"
        else
            print_error "Clippy linting failed"
            return 1
        fi
    else
        print_warning "Clippy not found, skipping linting"
    fi
}

# Function to generate test coverage
generate_coverage() {
    print_section "Test Coverage Analysis"
    
    if command -v cargo-tarpaulin >/dev/null 2>&1; then
        echo "Generating test coverage report..."
        
        cargo tarpaulin \
            --all-features \
            --workspace \
            --timeout $TEST_TIMEOUT \
            --out Html \
            --output-dir target/coverage \
            --exclude-files "target/*" \
            --exclude-files "*/tests/*" \
            --exclude-files "*/benches/*"
        
        # Extract coverage percentage
        if [ -f "target/coverage/tarpaulin-report.html" ]; then
            print_success "Coverage report generated at target/coverage/tarpaulin-report.html"
            
            # Try to extract coverage percentage (this is a simplified approach)
            if command -v grep >/dev/null 2>&1; then
                coverage=$(grep -o '[0-9]\+\.[0-9]\+%' target/coverage/tarpaulin-report.html | head -1 | sed 's/%//')
                if [ ! -z "$coverage" ]; then
                    echo "Coverage: ${coverage}%"
                    
                    # Check if coverage meets threshold
                    if (( $(echo "$coverage >= $COVERAGE_THRESHOLD" | bc -l) )); then
                        print_success "Coverage threshold met (${coverage}% >= ${COVERAGE_THRESHOLD}%)"
                    else
                        print_warning "Coverage below threshold (${coverage}% < ${COVERAGE_THRESHOLD}%)"
                    fi
                fi
            fi
        fi
    else
        print_warning "cargo-tarpaulin not found, skipping coverage analysis"
        echo "Install with: cargo install cargo-tarpaulin"
    fi
}

# Function to run security audit
run_security_audit() {
    print_section "Security Audit"
    
    if command -v cargo-audit >/dev/null 2>&1; then
        if cargo audit; then
            print_success "Security audit passed"
        else
            print_error "Security vulnerabilities found"
            return 1
        fi
    else
        print_warning "cargo-audit not found, skipping security audit"
        echo "Install with: cargo install cargo-audit"
    fi
}

# Function to run benchmarks
run_benchmarks() {
    print_section "Performance Benchmarks"
    
    echo "Running performance benchmarks..."
    
    # Run benchmarks for each crate that has them
    for crate in core-utils core-security core-compliance core-asset-lifecycle core-trading core-bridge core-analytics core-institutional; do
        if [ -d "$crate/benches" ]; then
            echo "Running benchmarks for $crate..."
            cd "$crate"
            if cargo bench --timeout $TEST_TIMEOUT; then
                print_success "$crate benchmarks completed"
            else
                print_warning "$crate benchmarks failed"
            fi
            cd - > /dev/null
        fi
    done
}

# Function to validate dependencies
validate_dependencies() {
    print_section "Dependency Validation"
    
    echo "Checking for unused dependencies..."
    if command -v cargo-udeps >/dev/null 2>&1; then
        if cargo +nightly udeps --all-targets; then
            print_success "No unused dependencies found"
        else
            print_warning "Unused dependencies detected"
        fi
    else
        print_warning "cargo-udeps not found, skipping unused dependency check"
        echo "Install with: cargo install cargo-udeps"
    fi
    
    echo "Checking for outdated dependencies..."
    if command -v cargo-outdated >/dev/null 2>&1; then
        cargo outdated --root-deps-only
        print_success "Dependency check completed"
    else
        print_warning "cargo-outdated not found, skipping outdated dependency check"
        echo "Install with: cargo install cargo-outdated"
    fi
}

# Main test execution
main() {
    local start_time=$(date +%s)
    local failed_tests=0
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --coverage)
                GENERATE_COVERAGE=true
                shift
                ;;
            --benchmarks)
                RUN_BENCHMARKS=true
                shift
                ;;
            --audit)
                RUN_AUDIT=true
                shift
                ;;
            --format-check)
                CHECK_FORMAT=true
                shift
                ;;
            --clippy)
                RUN_CLIPPY=true
                shift
                ;;
            --deps)
                VALIDATE_DEPS=true
                shift
                ;;
            --all)
                GENERATE_COVERAGE=true
                RUN_BENCHMARKS=true
                RUN_AUDIT=true
                CHECK_FORMAT=true
                RUN_CLIPPY=true
                VALIDATE_DEPS=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --coverage      Generate test coverage report"
                echo "  --benchmarks    Run performance benchmarks"
                echo "  --audit         Run security audit"
                echo "  --format-check  Check code formatting"
                echo "  --clippy        Run clippy linting"
                echo "  --deps          Validate dependencies"
                echo "  --all           Run all checks"
                echo "  --help          Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Set default behavior if no options specified
    if [ -z "$GENERATE_COVERAGE" ] && [ -z "$RUN_BENCHMARKS" ] && [ -z "$RUN_AUDIT" ] && [ -z "$CHECK_FORMAT" ] && [ -z "$RUN_CLIPPY" ] && [ -z "$VALIDATE_DEPS" ]; then
        echo "Running default test suite (unit and integration tests)"
        echo "Use --help to see all available options"
    fi
    
    # Core module tests
    print_section "Core Module Tests"
    
    # Test each core module
    local modules=(
        "core-utils:core-utils"
        "core-security:core-security"
        "core-compliance:core-compliance"
        "core-asset-lifecycle:core-asset-lifecycle"
        "core-trading:core-trading"
        "core-bridge:core-bridge"
        "core-analytics:core-analytics"
        "core-institutional:core-institutional"
    )
    
    for module in "${modules[@]}"; do
        IFS=':' read -r name path <<< "$module"
        if ! run_crate_tests "$name" "$path"; then
            ((failed_tests++))
        fi
    done
    
    # Integration tests
    print_section "Integration Tests"
    if cargo test --test comprehensive_integration_tests --timeout $TEST_TIMEOUT; then
        print_success "Comprehensive integration tests passed"
    else
        print_error "Comprehensive integration tests failed"
        ((failed_tests++))
    fi
    
    # Optional checks
    if [ "$CHECK_FORMAT" = true ]; then
        if ! check_formatting; then
            ((failed_tests++))
        fi
    fi
    
    if [ "$RUN_CLIPPY" = true ]; then
        if ! run_clippy; then
            ((failed_tests++))
        fi
    fi
    
    if [ "$GENERATE_COVERAGE" = true ]; then
        generate_coverage
    fi
    
    if [ "$RUN_AUDIT" = true ]; then
        if ! run_security_audit; then
            ((failed_tests++))
        fi
    fi
    
    if [ "$RUN_BENCHMARKS" = true ]; then
        run_benchmarks
    fi
    
    if [ "$VALIDATE_DEPS" = true ]; then
        validate_dependencies
    fi
    
    # Summary
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    print_section "Test Summary"
    echo "Total execution time: ${duration}s"
    
    if [ $failed_tests -eq 0 ]; then
        print_success "All tests passed! 🎉"
        echo -e "${GREEN}The StableRWA platform is ready for deployment.${NC}"
        exit 0
    else
        print_error "$failed_tests test suite(s) failed"
        echo -e "${RED}Please fix the failing tests before deployment.${NC}"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
