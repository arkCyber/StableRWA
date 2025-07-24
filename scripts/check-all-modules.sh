#!/bin/bash

# =====================================================================================
# File: scripts/check-all-modules.sh
# Description: Comprehensive module checking script for RWA platform
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

set -euo pipefail

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

print_header() {
    echo ""
    echo "========================================"
    echo "$1"
    echo "========================================"
}

# Core modules to check
CORE_MODULES=(
    "core-utils"
    "core-security"
    "core-nft"
    "core-smart-contract"
    "core-monitoring"
    "core-ai-risk"
    "core-privacy"
    "core-defi"
    "core-layer2"
    "core-oracle"
    "core-wallet"
    "core-regtech"
    "core-blockchain"
)

# Statistics
total_modules=0
successful_modules=0
failed_modules=0
modules_with_tests=0
modules_without_tests=0

# Function to check if module exists and has proper structure
check_module_structure() {
    local module=$1
    log_info "Checking module structure: $module"
    
    if [ ! -d "$module" ]; then
        log_error "Module directory not found: $module"
        return 1
    fi
    
    if [ ! -f "$module/Cargo.toml" ]; then
        log_error "Cargo.toml not found in $module"
        return 1
    fi
    
    if [ ! -d "$module/src" ]; then
        log_error "src directory not found in $module"
        return 1
    fi
    
    if [ ! -f "$module/src/lib.rs" ]; then
        log_error "lib.rs not found in $module/src"
        return 1
    fi
    
    log_success "Module structure OK: $module"
    return 0
}

# Function to check if module compiles
check_module_compilation() {
    local module=$1
    log_info "Checking compilation: $module"
    
    cd "$module"
    
    if cargo check --lib --no-default-features --quiet 2>/dev/null; then
        log_success "Compilation OK: $module"
        cd ..
        return 0
    else
        log_error "Compilation failed: $module"
        cd ..
        return 1
    fi
}

# Function to check if module has tests and run them
check_module_tests() {
    local module=$1
    log_info "Checking tests: $module"
    
    cd "$module"
    
    # Check if tests exist
    if cargo test --lib --no-default-features --no-run --quiet 2>/dev/null; then
        log_info "Tests found in $module, running..."
        if cargo test --lib --no-default-features --quiet 2>/dev/null; then
            log_success "Tests passed: $module"
            cd ..
            return 0
        else
            log_warning "Tests failed: $module"
            cd ..
            return 1
        fi
    else
        log_warning "No runnable tests found: $module"
        cd ..
        return 2
    fi
}

# Function to check for placeholder functions
check_placeholder_functions() {
    local module=$1
    log_info "Checking for placeholder functions: $module"
    
    local placeholder_count=0
    
    # Search for common placeholder patterns
    if [ -d "$module/src" ]; then
        placeholder_count=$(find "$module/src" -name "*.rs" -exec grep -l "unimplemented!\|todo!\|panic!(\"not implemented\"\|panic!(\"TODO" {} \; 2>/dev/null | wc -l)
    fi
    
    if [ "$placeholder_count" -gt 0 ]; then
        log_warning "Found $placeholder_count files with placeholder functions in $module"
        return 1
    else
        log_success "No placeholder functions found: $module"
        return 0
    fi
}

# Function to generate module report
generate_module_report() {
    local module=$1
    local structure_ok=$2
    local compilation_ok=$3
    local tests_status=$4
    local placeholder_ok=$5
    
    echo "## $module" >> module_report.md
    echo "" >> module_report.md
    
    if [ "$structure_ok" -eq 0 ]; then
        echo "- ✅ Structure: OK" >> module_report.md
    else
        echo "- ❌ Structure: FAILED" >> module_report.md
    fi
    
    if [ "$compilation_ok" -eq 0 ]; then
        echo "- ✅ Compilation: OK" >> module_report.md
    else
        echo "- ❌ Compilation: FAILED" >> module_report.md
    fi
    
    case $tests_status in
        0)
            echo "- ✅ Tests: PASSED" >> module_report.md
            ;;
        1)
            echo "- ⚠️ Tests: FAILED" >> module_report.md
            ;;
        2)
            echo "- ⚠️ Tests: NOT FOUND" >> module_report.md
            ;;
        *)
            echo "- ❓ Tests: UNKNOWN" >> module_report.md
            ;;
    esac
    
    if [ "$placeholder_ok" -eq 0 ]; then
        echo "- ✅ Placeholders: NONE" >> module_report.md
    else
        echo "- ⚠️ Placeholders: FOUND" >> module_report.md
    fi
    
    echo "" >> module_report.md
}

# Main execution
main() {
    print_header "RWA Platform Module Status Check"
    
    # Initialize report
    echo "# RWA Platform Module Status Report" > module_report.md
    echo "" >> module_report.md
    echo "Generated on: $(date)" >> module_report.md
    echo "" >> module_report.md
    
    # Check each module
    for module in "${CORE_MODULES[@]}"; do
        print_header "Checking Module: $module"
        
        total_modules=$((total_modules + 1))
        
        # Check structure
        structure_ok=1
        if check_module_structure "$module"; then
            structure_ok=0
        fi
        
        # Check compilation
        compilation_ok=1
        if [ "$structure_ok" -eq 0 ] && check_module_compilation "$module"; then
            compilation_ok=0
            successful_modules=$((successful_modules + 1))
        else
            failed_modules=$((failed_modules + 1))
        fi
        
        # Check tests
        tests_status=3
        if [ "$compilation_ok" -eq 0 ]; then
            check_module_tests "$module"
            tests_status=$?
            if [ "$tests_status" -eq 0 ] || [ "$tests_status" -eq 1 ]; then
                modules_with_tests=$((modules_with_tests + 1))
            else
                modules_without_tests=$((modules_without_tests + 1))
            fi
        fi
        
        # Check placeholders
        placeholder_ok=1
        if [ "$structure_ok" -eq 0 ]; then
            if check_placeholder_functions "$module"; then
                placeholder_ok=0
            fi
        fi
        
        # Generate report for this module
        generate_module_report "$module" "$structure_ok" "$compilation_ok" "$tests_status" "$placeholder_ok"
        
        echo ""
    done
    
    # Final summary
    print_header "FINAL SUMMARY"
    
    echo "📊 Module Statistics:"
    echo "   • Total modules checked: $total_modules"
    echo "   • Successful compilations: $successful_modules"
    echo "   • Failed compilations: $failed_modules"
    echo "   • Modules with tests: $modules_with_tests"
    echo "   • Modules without tests: $modules_without_tests"
    
    # Add summary to report
    echo "## Summary" >> module_report.md
    echo "" >> module_report.md
    echo "- Total modules: $total_modules" >> module_report.md
    echo "- Successful compilations: $successful_modules" >> module_report.md
    echo "- Failed compilations: $failed_modules" >> module_report.md
    echo "- Modules with tests: $modules_with_tests" >> module_report.md
    echo "- Modules without tests: $modules_without_tests" >> module_report.md
    
    if [ "$failed_modules" -eq 0 ]; then
        log_success "🎉 All modules are in good shape!"
        echo ""
        echo "✅ All $total_modules core modules compiled successfully"
        echo "📋 Detailed report saved to: module_report.md"
        exit 0
    else
        log_error "❌ Some modules need attention"
        echo ""
        echo "⚠️  $failed_modules out of $total_modules modules failed compilation"
        echo "📋 Detailed report saved to: module_report.md"
        exit 1
    fi
}

# Run main function
main "$@"
