#!/bin/bash

# =====================================================================================
# Smoke Tests for RWA Platform
# Quick validation tests to ensure basic functionality after deployment
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

set -euo pipefail

# Configuration
ENVIRONMENT="${1:-development}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration based on environment
case $ENVIRONMENT in
    development)
        API_BASE_URL="http://localhost:8080"
        TIMEOUT=30
        ;;
    staging)
        API_BASE_URL="https://staging-api.rwa-platform.com"
        TIMEOUT=60
        ;;
    production)
        API_BASE_URL="https://api.rwa-platform.com"
        TIMEOUT=60
        ;;
    *)
        echo -e "${RED}[ERROR]${NC} Invalid environment: $ENVIRONMENT"
        exit 1
        ;;
esac

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_failure() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
    FAILED_TESTS+=("$1")
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# HTTP request helper
make_request() {
    local method="$1"
    local url="$2"
    local data="${3:-}"
    local headers="${4:-}"
    local expected_status="${5:-200}"

    local curl_cmd="curl -s -w '%{http_code}' --max-time $TIMEOUT"
    
    if [[ -n "$headers" ]]; then
        curl_cmd="$curl_cmd -H '$headers'"
    fi

    if [[ "$method" == "POST" || "$method" == "PUT" ]]; then
        curl_cmd="$curl_cmd -H 'Content-Type: application/json'"
        if [[ -n "$data" ]]; then
            curl_cmd="$curl_cmd -d '$data'"
        fi
    fi

    curl_cmd="$curl_cmd -X $method '$url'"

    local response
    response=$(eval "$curl_cmd" 2>/dev/null)
    
    local status_code="${response: -3}"
    local body="${response%???}"

    if [[ "$status_code" == "$expected_status" ]]; then
        echo "$body"
        return 0
    else
        echo "Expected status $expected_status, got $status_code: $body" >&2
        return 1
    fi
}

# Test health endpoint
test_health_check() {
    log_info "Testing health check endpoint..."
    
    if response=$(make_request "GET" "$API_BASE_URL/health" "" "" "200"); then
        if echo "$response" | grep -q '"status":"healthy"'; then
            log_success "Health check endpoint is working"
        else
            log_failure "Health check endpoint returned unexpected response: $response"
        fi
    else
        log_failure "Health check endpoint is not accessible"
    fi
}

# Test metrics endpoint
test_metrics_endpoint() {
    log_info "Testing metrics endpoint..."
    
    if response=$(make_request "GET" "$API_BASE_URL/metrics" "" "" "200"); then
        if echo "$response" | grep -q "# HELP"; then
            log_success "Metrics endpoint is working"
        else
            log_failure "Metrics endpoint returned unexpected response"
        fi
    else
        log_failure "Metrics endpoint is not accessible"
    fi
}

# Test user registration
test_user_registration() {
    log_info "Testing user registration..."
    
    local timestamp=$(date +%s)
    local test_email="smoketest${timestamp}@example.com"
    local user_data='{
        "email": "'$test_email'",
        "password": "SmokeTest123!",
        "first_name": "Smoke",
        "last_name": "Test",
        "phone": "+1555000'$timestamp'"
    }'

    if response=$(make_request "POST" "$API_BASE_URL/api/v1/auth/register" "$user_data" "" "201"); then
        if echo "$response" | grep -q '"user_id"'; then
            log_success "User registration is working"
            echo "$test_email" > /tmp/smoke_test_user_email
        else
            log_failure "User registration returned unexpected response: $response"
        fi
    else
        log_failure "User registration failed"
    fi
}

# Test user login
test_user_login() {
    log_info "Testing user login..."
    
    if [[ ! -f /tmp/smoke_test_user_email ]]; then
        log_failure "Cannot test login - no test user created"
        return
    fi

    local test_email
    test_email=$(cat /tmp/smoke_test_user_email)
    local login_data='{
        "email": "'$test_email'",
        "password": "SmokeTest123!"
    }'

    if response=$(make_request "POST" "$API_BASE_URL/api/v1/auth/login" "$login_data" "" "200"); then
        if echo "$response" | grep -q '"access_token"'; then
            log_success "User login is working"
            # Extract access token for authenticated tests
            local access_token
            access_token=$(echo "$response" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
            echo "$access_token" > /tmp/smoke_test_access_token
        else
            log_failure "User login returned unexpected response: $response"
        fi
    else
        log_failure "User login failed"
    fi
}

# Test authenticated endpoint (user profile)
test_authenticated_endpoint() {
    log_info "Testing authenticated endpoint..."
    
    if [[ ! -f /tmp/smoke_test_access_token ]]; then
        log_failure "Cannot test authenticated endpoint - no access token available"
        return
    fi

    local access_token
    access_token=$(cat /tmp/smoke_test_access_token)
    local auth_header="Authorization: Bearer $access_token"

    if response=$(make_request "GET" "$API_BASE_URL/api/v1/users/profile" "" "$auth_header" "200"); then
        if echo "$response" | grep -q '"user_id"'; then
            log_success "Authenticated endpoint is working"
        else
            log_failure "Authenticated endpoint returned unexpected response: $response"
        fi
    else
        log_failure "Authenticated endpoint failed"
    fi
}

# Test asset creation
test_asset_creation() {
    log_info "Testing asset creation..."
    
    if [[ ! -f /tmp/smoke_test_access_token ]]; then
        log_failure "Cannot test asset creation - no access token available"
        return
    fi

    local access_token
    access_token=$(cat /tmp/smoke_test_access_token)
    local auth_header="Authorization: Bearer $access_token"
    local timestamp=$(date +%s)
    
    local asset_data='{
        "name": "Smoke Test Asset '$timestamp'",
        "description": "Asset created during smoke testing",
        "asset_type": "real_estate",
        "total_value": 500000,
        "currency": "USD",
        "location": "Test City, Test State"
    }'

    if response=$(make_request "POST" "$API_BASE_URL/api/v1/assets" "$asset_data" "$auth_header" "201"); then
        if echo "$response" | grep -q '"id"'; then
            log_success "Asset creation is working"
            # Extract asset ID for further tests
            local asset_id
            asset_id=$(echo "$response" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
            echo "$asset_id" > /tmp/smoke_test_asset_id
        else
            log_failure "Asset creation returned unexpected response: $response"
        fi
    else
        log_failure "Asset creation failed"
    fi
}

# Test asset retrieval
test_asset_retrieval() {
    log_info "Testing asset retrieval..."
    
    if [[ ! -f /tmp/smoke_test_access_token ]] || [[ ! -f /tmp/smoke_test_asset_id ]]; then
        log_failure "Cannot test asset retrieval - missing prerequisites"
        return
    fi

    local access_token
    access_token=$(cat /tmp/smoke_test_access_token)
    local asset_id
    asset_id=$(cat /tmp/smoke_test_asset_id)
    local auth_header="Authorization: Bearer $access_token"

    if response=$(make_request "GET" "$API_BASE_URL/api/v1/assets/$asset_id" "" "$auth_header" "200"); then
        if echo "$response" | grep -q '"id"'; then
            log_success "Asset retrieval is working"
        else
            log_failure "Asset retrieval returned unexpected response: $response"
        fi
    else
        log_failure "Asset retrieval failed"
    fi
}

# Test payment processing
test_payment_processing() {
    log_info "Testing payment processing..."
    
    if [[ ! -f /tmp/smoke_test_access_token ]]; then
        log_failure "Cannot test payment processing - no access token available"
        return
    fi

    local access_token
    access_token=$(cat /tmp/smoke_test_access_token)
    local auth_header="Authorization: Bearer $access_token"
    
    local payment_data='{
        "amount": 50,
        "currency": "USD",
        "payment_method_type": "credit_card",
        "provider": "stripe",
        "description": "Smoke test payment"
    }'

    if response=$(make_request "POST" "$API_BASE_URL/api/v1/payments" "$payment_data" "$auth_header" "201"); then
        if echo "$response" | grep -q '"id"'; then
            log_success "Payment processing is working"
        else
            log_failure "Payment processing returned unexpected response: $response"
        fi
    else
        log_failure "Payment processing failed"
    fi
}

# Test database connectivity (indirect)
test_database_connectivity() {
    log_info "Testing database connectivity (indirect)..."
    
    # Test by trying to list assets (requires database)
    if [[ ! -f /tmp/smoke_test_access_token ]]; then
        log_failure "Cannot test database connectivity - no access token available"
        return
    fi

    local access_token
    access_token=$(cat /tmp/smoke_test_access_token)
    local auth_header="Authorization: Bearer $access_token"

    if response=$(make_request "GET" "$API_BASE_URL/api/v1/assets?page=1&per_page=1" "" "$auth_header" "200"); then
        if echo "$response" | grep -q '"data"'; then
            log_success "Database connectivity is working"
        else
            log_failure "Database connectivity test returned unexpected response: $response"
        fi
    else
        log_failure "Database connectivity test failed"
    fi
}

# Test API rate limiting
test_rate_limiting() {
    log_info "Testing API rate limiting..."
    
    local rate_limit_hit=false
    
    # Make multiple rapid requests to trigger rate limiting
    for i in {1..20}; do
        if ! make_request "GET" "$API_BASE_URL/health" "" "" "200" >/dev/null 2>&1; then
            rate_limit_hit=true
            break
        fi
        sleep 0.1
    done

    if [[ "$rate_limit_hit" == true ]]; then
        log_success "Rate limiting is working (requests were throttled)"
    else
        log_warning "Rate limiting may not be configured (all requests succeeded)"
    fi
}

# Cleanup test data
cleanup_test_data() {
    log_info "Cleaning up test data..."
    
    # Remove temporary files
    rm -f /tmp/smoke_test_user_email
    rm -f /tmp/smoke_test_access_token
    rm -f /tmp/smoke_test_asset_id
    
    log_info "Cleanup completed"
}

# Main execution
main() {
    echo "========================================"
    echo "RWA Platform Smoke Tests"
    echo "Environment: $ENVIRONMENT"
    echo "API Base URL: $API_BASE_URL"
    echo "Timeout: ${TIMEOUT}s"
    echo "========================================"
    echo

    # Run tests
    test_health_check
    test_metrics_endpoint
    test_user_registration
    test_user_login
    test_authenticated_endpoint
    test_asset_creation
    test_asset_retrieval
    test_payment_processing
    test_database_connectivity
    test_rate_limiting

    # Cleanup
    cleanup_test_data

    # Summary
    echo
    echo "========================================"
    echo "Smoke Test Results"
    echo "========================================"
    echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        echo
        echo "Failed Tests:"
        for test in "${FAILED_TESTS[@]}"; do
            echo -e "  ${RED}✗${NC} $test"
        done
        echo
        echo -e "${RED}Smoke tests FAILED${NC}"
        exit 1
    else
        echo
        echo -e "${GREEN}All smoke tests PASSED${NC}"
        exit 0
    fi
}

# Run main function
main "$@"
