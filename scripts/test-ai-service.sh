#!/bin/bash

# =====================================================================================
# File: scripts/test-ai-service.sh
# Description: Comprehensive testing script for StableRWA AI Service
# Author: arkSong (arksong2018@gmail.com)
# Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
# =====================================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
AI_SERVICE_URL="http://localhost:8090"
TIMEOUT=30

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

# Function to check if service is running
check_service() {
    print_status "Checking if AI service is running..."
    
    if curl -s --max-time $TIMEOUT "$AI_SERVICE_URL/health" > /dev/null; then
        print_success "AI service is running"
        return 0
    else
        print_error "AI service is not running or not responding"
        return 1
    fi
}

# Function to test health endpoint
test_health() {
    print_status "Testing health endpoint..."
    
    response=$(curl -s --max-time $TIMEOUT "$AI_SERVICE_URL/health")
    
    if echo "$response" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
        print_success "Health check passed"
        echo "Response: $response" | jq '.'
    else
        print_error "Health check failed"
        echo "Response: $response"
        return 1
    fi
}

# Function to test AI model endpoint
test_ai_model() {
    print_status "Testing AI model endpoint..."
    
    response=$(curl -s --max-time $TIMEOUT "$AI_SERVICE_URL/ai/model")
    
    if echo "$response" | jq -e '.service' > /dev/null 2>&1; then
        print_success "AI model endpoint test passed"
        echo "Available models:"
        echo "$response" | jq '.models[]'
    else
        print_error "AI model endpoint test failed"
        echo "Response: $response"
        return 1
    fi
}

# Function to test AI completion endpoint
test_ai_completion() {
    print_status "Testing AI completion endpoint..."
    
    local test_prompt="Analyze the potential of real estate tokenization in the current market."
    
    response=$(curl -s --max-time $TIMEOUT \
        -X POST "$AI_SERVICE_URL/ai/complete" \
        -H "Content-Type: application/json" \
        -d "{
            \"prompt\": \"$test_prompt\",
            \"max_tokens\": 150,
            \"temperature\": 0.7
        }")
    
    if echo "$response" | jq -e '.content' > /dev/null 2>&1; then
        print_success "AI completion test passed"
        echo "Prompt: $test_prompt"
        echo "Response:"
        echo "$response" | jq '.content' -r
        echo ""
        echo "Full response details:"
        echo "$response" | jq '.'
    else
        print_error "AI completion test failed"
        echo "Response: $response"
        return 1
    fi
}

# Function to test asset valuation
test_asset_valuation() {
    print_status "Testing asset valuation with AI..."
    
    local valuation_prompt="Estimate the value of a 2,000 square foot commercial property in Manhattan, built in 2018, with modern amenities including parking and security systems."
    
    response=$(curl -s --max-time $TIMEOUT \
        -X POST "$AI_SERVICE_URL/ai/complete" \
        -H "Content-Type: application/json" \
        -d "{
            \"prompt\": \"$valuation_prompt\",
            \"max_tokens\": 200,
            \"temperature\": 0.3
        }")
    
    if echo "$response" | jq -e '.content' > /dev/null 2>&1; then
        print_success "Asset valuation test passed"
        echo "Valuation analysis:"
        echo "$response" | jq '.content' -r
    else
        print_error "Asset valuation test failed"
        echo "Response: $response"
        return 1
    fi
}

# Function to test risk assessment
test_risk_assessment() {
    print_status "Testing risk assessment with AI..."
    
    local risk_prompt="Assess the investment risk for tokenizing a $5 million real estate portfolio in the current economic climate, considering market volatility and regulatory factors."
    
    response=$(curl -s --max-time $TIMEOUT \
        -X POST "$AI_SERVICE_URL/ai/complete" \
        -H "Content-Type: application/json" \
        -d "{
            \"prompt\": \"$risk_prompt\",
            \"max_tokens\": 180,
            \"temperature\": 0.4
        }")
    
    if echo "$response" | jq -e '.content' > /dev/null 2>&1; then
        print_success "Risk assessment test passed"
        echo "Risk analysis:"
        echo "$response" | jq '.content' -r
    else
        print_error "Risk assessment test failed"
        echo "Response: $response"
        return 1
    fi
}

# Function to test market analysis
test_market_analysis() {
    print_status "Testing market analysis with AI..."
    
    local market_prompt="Analyze the current trends in RWA (Real World Asset) tokenization market, including growth prospects, challenges, and opportunities for the next 12 months."
    
    response=$(curl -s --max-time $TIMEOUT \
        -X POST "$AI_SERVICE_URL/ai/complete" \
        -H "Content-Type: application/json" \
        -d "{
            \"prompt\": \"$market_prompt\",
            \"max_tokens\": 250,
            \"temperature\": 0.6
        }")
    
    if echo "$response" | jq -e '.content' > /dev/null 2>&1; then
        print_success "Market analysis test passed"
        echo "Market analysis:"
        echo "$response" | jq '.content' -r
    else
        print_error "Market analysis test failed"
        echo "Response: $response"
        return 1
    fi
}

# Function to test error handling
test_error_handling() {
    print_status "Testing error handling..."
    
    # Test with empty prompt
    response=$(curl -s --max-time $TIMEOUT \
        -X POST "$AI_SERVICE_URL/ai/complete" \
        -H "Content-Type: application/json" \
        -d '{"prompt": ""}')
    
    if curl -s --max-time $TIMEOUT \
        -X POST "$AI_SERVICE_URL/ai/complete" \
        -H "Content-Type: application/json" \
        -d '{"prompt": ""}' \
        -w "%{http_code}" -o /dev/null | grep -q "400"; then
        print_success "Error handling test passed (empty prompt rejected)"
    else
        print_warning "Error handling test: empty prompt should return 400 status"
    fi
}

# Function to test performance
test_performance() {
    print_status "Testing AI service performance..."
    
    local start_time=$(date +%s.%N)
    
    response=$(curl -s --max-time $TIMEOUT \
        -X POST "$AI_SERVICE_URL/ai/complete" \
        -H "Content-Type: application/json" \
        -d '{
            "prompt": "What are the key benefits of blockchain technology for asset tokenization?",
            "max_tokens": 100,
            "temperature": 0.5
        }')
    
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)
    
    if echo "$response" | jq -e '.content' > /dev/null 2>&1; then
        print_success "Performance test completed in ${duration}s"
        
        if (( $(echo "$duration < 10.0" | bc -l) )); then
            print_success "Response time is acceptable (< 10s)"
        else
            print_warning "Response time is slow (> 10s): ${duration}s"
        fi
    else
        print_error "Performance test failed"
        return 1
    fi
}

# Function to run all tests
run_all_tests() {
    print_status "Starting comprehensive AI service tests..."
    echo "========================================"
    
    local failed_tests=0
    
    # Check if service is running
    if ! check_service; then
        print_error "Cannot proceed with tests - AI service is not running"
        print_status "Please start the AI service with: cargo run --bin ai-service"
        exit 1
    fi
    
    echo ""
    
    # Run individual tests
    test_health || ((failed_tests++))
    echo ""
    
    test_ai_model || ((failed_tests++))
    echo ""
    
    test_ai_completion || ((failed_tests++))
    echo ""
    
    test_asset_valuation || ((failed_tests++))
    echo ""
    
    test_risk_assessment || ((failed_tests++))
    echo ""
    
    test_market_analysis || ((failed_tests++))
    echo ""
    
    test_error_handling || ((failed_tests++))
    echo ""
    
    test_performance || ((failed_tests++))
    echo ""
    
    # Summary
    echo "========================================"
    if [ $failed_tests -eq 0 ]; then
        print_success "All tests passed! 🎉"
        print_status "StableRWA AI Service is working correctly"
    else
        print_error "$failed_tests test(s) failed"
        print_status "Please check the AI service configuration and logs"
        exit 1
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Test the StableRWA AI Service endpoints"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -u, --url URL           Set AI service URL (default: $AI_SERVICE_URL)"
    echo "  -t, --timeout SECONDS   Set request timeout (default: $TIMEOUT)"
    echo "  --health                Test health endpoint only"
    echo "  --model                 Test model endpoint only"
    echo "  --completion            Test completion endpoint only"
    echo "  --valuation             Test asset valuation only"
    echo "  --risk                  Test risk assessment only"
    echo "  --market                Test market analysis only"
    echo "  --performance           Test performance only"
    echo "  --all                   Run all tests (default)"
    echo ""
    echo "Examples:"
    echo "  $0                      # Run all tests"
    echo "  $0 --completion         # Test completion endpoint only"
    echo "  $0 -u http://localhost:8091 --all  # Test with custom URL"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -u|--url)
            AI_SERVICE_URL="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --health)
            check_service && test_health
            exit $?
            ;;
        --model)
            check_service && test_ai_model
            exit $?
            ;;
        --completion)
            check_service && test_ai_completion
            exit $?
            ;;
        --valuation)
            check_service && test_asset_valuation
            exit $?
            ;;
        --risk)
            check_service && test_risk_assessment
            exit $?
            ;;
        --market)
            check_service && test_market_analysis
            exit $?
            ;;
        --performance)
            check_service && test_performance
            exit $?
            ;;
        --all)
            run_all_tests
            exit $?
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Default action: run all tests
run_all_tests
