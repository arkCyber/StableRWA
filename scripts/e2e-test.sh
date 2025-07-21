#!/bin/bash

# =====================================================================================
# File: scripts/e2e-test.sh
# Description: End-to-end testing script for RWA platform
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
GATEWAY_URL="http://localhost:8080"
ASSET_SERVICE_URL="http://localhost:8081"
AI_SERVICE_URL="http://localhost:8082"
USER_SERVICE_URL="http://localhost:8083"
TIMEOUT=300 # 5 minutes timeout

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((TESTS_FAILED++))
    FAILED_TESTS+=("$1")
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Wait for service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=60
    local attempt=1

    log_info "Waiting for $service_name to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "$url/health" > /dev/null 2>&1; then
            log_success "$service_name is ready"
            return 0
        fi
        
        echo -n "."
        sleep 5
        ((attempt++))
    done
    
    log_error "$service_name failed to start within timeout"
    return 1
}

# Test service health
test_health_checks() {
    log_info "Testing service health checks..."
    
    # Test Gateway
    if curl -s -f "$GATEWAY_URL/health" > /dev/null; then
        log_success "Gateway health check passed"
    else
        log_error "Gateway health check failed"
    fi
    
    # Test Asset Service
    if curl -s -f "$ASSET_SERVICE_URL/health" > /dev/null; then
        log_success "Asset service health check passed"
    else
        log_error "Asset service health check failed"
    fi
    
    # Test AI Service
    if curl -s -f "$AI_SERVICE_URL/health" > /dev/null; then
        log_success "AI service health check passed"
    else
        log_error "AI service health check failed"
    fi
}

# Test user authentication
test_authentication() {
    log_info "Testing user authentication..."
    
    # Register test user
    local test_user="e2e_test_user_$(date +%s)"
    local test_email="${test_user}@test.com"
    local test_password="test_password_123"
    
    local register_response=$(curl -s -w "%{http_code}" -o /tmp/register_response.json \
        -X POST "$GATEWAY_URL/api/v1/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$test_user\",\"email\":\"$test_email\",\"password\":\"$test_password\"}")
    
    if [[ "$register_response" == "200" || "$register_response" == "201" ]]; then
        log_success "User registration passed"
    else
        log_error "User registration failed with status: $register_response"
        return 1
    fi
    
    # Login test user
    local login_response=$(curl -s -w "%{http_code}" -o /tmp/login_response.json \
        -X POST "$GATEWAY_URL/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$test_user\",\"password\":\"$test_password\"}")
    
    if [[ "$login_response" == "200" ]]; then
        log_success "User login passed"
        # Extract token for subsequent tests
        export AUTH_TOKEN=$(jq -r '.access_token' /tmp/login_response.json)
    else
        log_error "User login failed with status: $login_response"
        return 1
    fi
}

# Test asset management
test_asset_management() {
    log_info "Testing asset management..."
    
    if [[ -z "$AUTH_TOKEN" ]]; then
        log_error "No auth token available for asset management test"
        return 1
    fi
    
    # Create asset
    local asset_response=$(curl -s -w "%{http_code}" -o /tmp/asset_response.json \
        -X POST "$GATEWAY_URL/api/v1/assets" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $AUTH_TOKEN" \
        -d '{
            "name": "E2E Test Property",
            "description": "Test property for end-to-end testing",
            "asset_type": "RealEstate",
            "location": "Test City, Test Country",
            "valuation": {
                "amount": 500000,
                "currency": "USD",
                "valuation_date": "2024-01-01T00:00:00Z"
            }
        }')
    
    if [[ "$asset_response" == "200" || "$asset_response" == "201" ]]; then
        log_success "Asset creation passed"
        export ASSET_ID=$(jq -r '.id' /tmp/asset_response.json)
    else
        log_error "Asset creation failed with status: $asset_response"
        return 1
    fi
    
    # Get asset
    local get_response=$(curl -s -w "%{http_code}" -o /tmp/get_asset_response.json \
        -X GET "$GATEWAY_URL/api/v1/assets/$ASSET_ID" \
        -H "Authorization: Bearer $AUTH_TOKEN")
    
    if [[ "$get_response" == "200" ]]; then
        log_success "Asset retrieval passed"
    else
        log_error "Asset retrieval failed with status: $get_response"
    fi
}

# Test AI valuation
test_ai_valuation() {
    log_info "Testing AI valuation service..."
    
    if [[ -z "$AUTH_TOKEN" ]]; then
        log_error "No auth token available for AI valuation test"
        return 1
    fi
    
    local valuation_response=$(curl -s -w "%{http_code}" -o /tmp/valuation_response.json \
        -X POST "$GATEWAY_URL/api/v1/ai/valuate" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $AUTH_TOKEN" \
        -d '{
            "asset_type": "RealEstate",
            "location": "New York, NY",
            "property_details": {
                "square_feet": 1000,
                "bedrooms": 2,
                "bathrooms": 1
            }
        }')
    
    if [[ "$valuation_response" == "200" ]]; then
        log_success "AI valuation passed"
    else
        log_error "AI valuation failed with status: $valuation_response"
    fi
}

# Test IPFS integration
test_ipfs_integration() {
    log_info "Testing IPFS integration..."
    
    if [[ -z "$AUTH_TOKEN" ]]; then
        log_error "No auth token available for IPFS test"
        return 1
    fi
    
    # Upload to IPFS
    local upload_response=$(curl -s -w "%{http_code}" -o /tmp/ipfs_upload_response.json \
        -X POST "$GATEWAY_URL/api/v1/ipfs/upload" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $AUTH_TOKEN" \
        -d '{
            "content": "E2E test document content",
            "filename": "e2e_test.txt",
            "content_type": "text/plain"
        }')
    
    if [[ "$upload_response" == "200" || "$upload_response" == "201" ]]; then
        log_success "IPFS upload passed"
        local ipfs_hash=$(jq -r '.hash' /tmp/ipfs_upload_response.json)
        
        # Retrieve from IPFS
        local retrieve_response=$(curl -s -w "%{http_code}" -o /tmp/ipfs_content.txt \
            -X GET "$GATEWAY_URL/api/v1/ipfs/$ipfs_hash" \
            -H "Authorization: Bearer $AUTH_TOKEN")
        
        if [[ "$retrieve_response" == "200" ]]; then
            log_success "IPFS retrieval passed"
        else
            log_error "IPFS retrieval failed with status: $retrieve_response"
        fi
    else
        log_error "IPFS upload failed with status: $upload_response"
    fi
}

# Test rate limiting
test_rate_limiting() {
    log_info "Testing rate limiting..."
    
    local rate_limited=false
    for i in {1..25}; do
        local response=$(curl -s -w "%{http_code}" -o /dev/null "$GATEWAY_URL/api/v1/health")
        if [[ "$response" == "429" ]]; then
            rate_limited=true
            break
        fi
        sleep 0.1
    done
    
    if [[ "$rate_limited" == true ]]; then
        log_success "Rate limiting is working"
    else
        log_warning "Rate limiting may not be configured or threshold not reached"
    fi
}

# Test metrics endpoint
test_metrics() {
    log_info "Testing metrics endpoint..."
    
    local metrics_response=$(curl -s -w "%{http_code}" -o /tmp/metrics.txt "$GATEWAY_URL/metrics")
    
    if [[ "$metrics_response" == "200" ]]; then
        if grep -q "http_requests_total" /tmp/metrics.txt; then
            log_success "Metrics endpoint is working"
        else
            log_error "Metrics endpoint missing expected metrics"
        fi
    else
        log_error "Metrics endpoint failed with status: $metrics_response"
    fi
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test artifacts..."
    rm -f /tmp/*_response.json /tmp/metrics.txt /tmp/ipfs_content.txt
}

# Main execution
main() {
    log_info "Starting end-to-end tests for RWA Platform"
    log_info "Gateway URL: $GATEWAY_URL"
    
    # Wait for services to be ready
    wait_for_service "$GATEWAY_URL" "Gateway" || exit 1
    wait_for_service "$ASSET_SERVICE_URL" "Asset Service" || exit 1
    wait_for_service "$AI_SERVICE_URL" "AI Service" || exit 1
    
    # Run tests
    test_health_checks
    test_authentication
    test_asset_management
    test_ai_valuation
    test_ipfs_integration
    test_rate_limiting
    test_metrics
    
    # Cleanup
    cleanup
    
    # Print results
    echo
    log_info "========================================="
    log_info "End-to-End Test Results"
    log_info "========================================="
    log_success "Tests Passed: $TESTS_PASSED"
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        log_error "Tests Failed: $TESTS_FAILED"
        echo
        log_error "Failed Tests:"
        for test in "${FAILED_TESTS[@]}"; do
            echo "  - $test"
        done
        echo
        exit 1
    else
        log_success "All tests passed!"
        exit 0
    fi
}

# Check dependencies
check_dependencies() {
    local deps=("curl" "jq")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            log_error "$dep is required but not installed"
            exit 1
        fi
    done
}

# Script entry point
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    check_dependencies
    main "$@"
fi
