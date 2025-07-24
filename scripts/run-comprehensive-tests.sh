#!/bin/bash

# =====================================================================================
# File: scripts/run-comprehensive-tests.sh
# Description: Comprehensive test runner for all RWA platform modules
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
CARGO_FLAGS="--release"
TEST_TIMEOUT="300s"
BENCHMARK_TIMEOUT="600s"
COVERAGE_THRESHOLD="80"

# Logging
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

# Check if required tools are installed
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi
    
    if ! command -v rustc &> /dev/null; then
        missing_deps+=("rustc")
    fi
    
    if ! cargo --list | grep -q "tarpaulin"; then
        log_warning "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin || missing_deps+=("cargo-tarpaulin")
    fi
    
    if ! cargo --list | grep -q "audit"; then
        log_warning "cargo-audit not found. Installing..."
        cargo install cargo-audit || missing_deps+=("cargo-audit")
    fi
    
    if ! cargo --list | grep -q "clippy"; then
        log_warning "clippy not found. Installing..."
        rustup component add clippy || missing_deps+=("clippy")
    fi
    
    if ! cargo --list | grep -q "fmt"; then
        log_warning "rustfmt not found. Installing..."
        rustup component add rustfmt || missing_deps+=("rustfmt")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    log_success "All dependencies are available"
}

# Clean previous build artifacts
clean_build() {
    log_info "Cleaning previous build artifacts..."
    cargo clean
    log_success "Build artifacts cleaned"
}

# Format code
format_code() {
    log_info "Formatting code..."
    if cargo fmt --all -- --check; then
        log_success "Code is properly formatted"
    else
        log_warning "Code formatting issues found. Running formatter..."
        cargo fmt --all
        log_success "Code formatted"
    fi
}

# Run clippy lints
run_clippy() {
    log_info "Running clippy lints..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        log_success "No clippy warnings found"
    else
        log_error "Clippy warnings found. Please fix them before proceeding."
        return 1
    fi
}

# Run security audit
run_security_audit() {
    log_info "Running security audit..."
    if cargo audit; then
        log_success "No security vulnerabilities found"
    else
        log_warning "Security audit found issues. Please review them."
    fi
}

# Run unit tests for a specific module
run_module_tests() {
    local module=$1
    log_info "Running unit tests for $module..."
    
    if [ -d "$module" ]; then
        cd "$module"
        if timeout "$TEST_TIMEOUT" cargo test $CARGO_FLAGS --lib; then
            log_success "Unit tests passed for $module"
        else
            log_error "Unit tests failed for $module"
            cd ..
            return 1
        fi
        cd ..
    else
        log_warning "Module $module not found, skipping..."
    fi
}

# Run integration tests for a specific module
run_module_integration_tests() {
    local module=$1
    log_info "Running integration tests for $module..."
    
    if [ -d "$module" ] && [ -d "$module/tests" ]; then
        cd "$module"
        if timeout "$TEST_TIMEOUT" cargo test $CARGO_FLAGS --test '*'; then
            log_success "Integration tests passed for $module"
        else
            log_error "Integration tests failed for $module"
            cd ..
            return 1
        fi
        cd ..
    else
        log_warning "Integration tests not found for $module, skipping..."
    fi
}

# Run benchmarks for a specific module
run_module_benchmarks() {
    local module=$1
    log_info "Running benchmarks for $module..."
    
    if [ -d "$module" ] && [ -d "$module/benches" ]; then
        cd "$module"
        if timeout "$BENCHMARK_TIMEOUT" cargo bench --bench '*'; then
            log_success "Benchmarks completed for $module"
        else
            log_warning "Benchmarks failed for $module"
        fi
        cd ..
    else
        log_warning "Benchmarks not found for $module, skipping..."
    fi
}

# Generate test coverage for a specific module
generate_module_coverage() {
    local module=$1
    log_info "Generating test coverage for $module..."
    
    if [ -d "$module" ]; then
        cd "$module"
        if cargo tarpaulin --out Html --output-dir ../target/tarpaulin/$module --timeout 120; then
            log_success "Coverage report generated for $module"
        else
            log_warning "Coverage generation failed for $module"
        fi
        cd ..
    else
        log_warning "Module $module not found, skipping coverage..."
    fi
}

# Run all tests for core modules
run_core_module_tests() {
    local modules=(
        "core-monitoring"
        "core-oracle" 
        "core-smart-contract"
        "core-ai-risk"
        "core-privacy"
        "core-regtech"
        "core-analytics"
        "core-bridge"
        "core-trading"
        "core-defi"
        "core-layer2"
        "core-nft"
        "core-utils"
        "core-security"
    )
    
    local failed_modules=()
    
    for module in "${modules[@]}"; do
        log_info "Testing module: $module"
        
        # Run unit tests
        if ! run_module_tests "$module"; then
            failed_modules+=("$module (unit tests)")
        fi
        
        # Run integration tests
        if ! run_module_integration_tests "$module"; then
            failed_modules+=("$module (integration tests)")
        fi
        
        # Generate coverage
        generate_module_coverage "$module"
        
        log_info "Completed testing for $module"
        echo "----------------------------------------"
    done
    
    if [ ${#failed_modules[@]} -ne 0 ]; then
        log_error "Failed modules: ${failed_modules[*]}"
        return 1
    fi
    
    log_success "All core module tests passed"
}

# Run benchmarks for core modules
run_core_module_benchmarks() {
    local modules=(
        "core-monitoring"
        "core-oracle"
        "core-smart-contract"
    )
    
    log_info "Running benchmarks for core modules..."
    
    for module in "${modules[@]}"; do
        run_module_benchmarks "$module"
    done
    
    log_success "All benchmarks completed"
}

# Run workspace-level tests
run_workspace_tests() {
    log_info "Running workspace-level tests..."
    
    if timeout "$TEST_TIMEOUT" cargo test $CARGO_FLAGS --workspace; then
        log_success "Workspace tests passed"
    else
        log_error "Workspace tests failed"
        return 1
    fi
}

# Generate comprehensive coverage report
generate_comprehensive_coverage() {
    log_info "Generating comprehensive coverage report..."
    
    mkdir -p target/coverage
    
    if cargo tarpaulin --all --out Html --output-dir target/coverage --timeout 300; then
        log_success "Comprehensive coverage report generated in target/coverage/"
        
        # Extract coverage percentage
        if [ -f "target/coverage/tarpaulin-report.html" ]; then
            local coverage=$(grep -o '[0-9]*\.[0-9]*%' target/coverage/tarpaulin-report.html | head -1 | sed 's/%//')
            if (( $(echo "$coverage >= $COVERAGE_THRESHOLD" | bc -l) )); then
                log_success "Coverage threshold met: $coverage% >= $COVERAGE_THRESHOLD%"
            else
                log_warning "Coverage below threshold: $coverage% < $COVERAGE_THRESHOLD%"
            fi
        fi
    else
        log_warning "Comprehensive coverage generation failed"
    fi
}

# Main execution
main() {
    log_info "Starting comprehensive test suite for RWA platform"
    echo "========================================================"
    
    # Check dependencies
    check_dependencies
    
    # Clean build
    clean_build
    
    # Format code
    format_code
    
    # Run clippy
    if ! run_clippy; then
        log_error "Clippy checks failed. Exiting."
        exit 1
    fi
    
    # Run security audit
    run_security_audit
    
    # Run core module tests
    if ! run_core_module_tests; then
        log_error "Core module tests failed. Exiting."
        exit 1
    fi
    
    # Run workspace tests
    if ! run_workspace_tests; then
        log_error "Workspace tests failed. Exiting."
        exit 1
    fi
    
    # Generate comprehensive coverage
    generate_comprehensive_coverage
    
    # Run benchmarks (optional, can be time-consuming)
    if [ "${RUN_BENCHMARKS:-false}" = "true" ]; then
        run_core_module_benchmarks
    else
        log_info "Skipping benchmarks (set RUN_BENCHMARKS=true to run them)"
    fi
    
    log_success "All tests completed successfully!"
    echo "========================================================"
    log_info "Test reports available in:"
    log_info "  - Coverage: target/coverage/tarpaulin-report.html"
    log_info "  - Individual module coverage: target/tarpaulin/*/tarpaulin-report.html"
    if [ "${RUN_BENCHMARKS:-false}" = "true" ]; then
        log_info "  - Benchmarks: target/criterion/*/report/index.html"
    fi
}

# Handle script arguments
case "${1:-}" in
    "clean")
        clean_build
        ;;
    "format")
        format_code
        ;;
    "clippy")
        run_clippy
        ;;
    "audit")
        run_security_audit
        ;;
    "coverage")
        generate_comprehensive_coverage
        ;;
    "benchmarks")
        run_core_module_benchmarks
        ;;
    *)
        main
        ;;
esac
