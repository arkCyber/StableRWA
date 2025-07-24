#!/bin/bash

# =====================================================================================
# File: scripts/basic_docker_test.sh
# Description: Basic Docker testing script for StableRWA Platform
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
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.basic.yml"
TEST_RESULTS_DIR="$PROJECT_ROOT/target/docker-test-results"
LOG_FILE="$TEST_RESULTS_DIR/basic-docker-test.log"

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
    log_section "Setting up basic Docker test environment"
    
    # Create test results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Initialize log file
    echo "=== Basic Docker Test - $(date) ===" > "$LOG_FILE"
    
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
        sleep 5
        ((attempt++))
    done
    
    log_error "$service_name failed to become healthy"
    return 1
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
    if docker exec stablerwa-redis redis-cli -a StableRWA2024! ping | grep -q "PONG"; then
        log_success "Redis connectivity test passed"
    else
        log_error "Redis connectivity test failed"
        return 1
    fi
}

# Test database operations
test_database_operations() {
    log_section "Testing database operations"
    
    # Test PostgreSQL operations
    log_info "Testing PostgreSQL operations..."
    
    # Create test table
    docker exec stablerwa-postgres psql -U stablerwa -d stablerwa -c "
        CREATE TABLE IF NOT EXISTS test_table (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    " &> /dev/null
    
    # Insert test data
    docker exec stablerwa-postgres psql -U stablerwa -d stablerwa -c "
        INSERT INTO test_table (name) VALUES ('Test Entry 1'), ('Test Entry 2');
    " &> /dev/null
    
    # Query test data
    local result=$(docker exec stablerwa-postgres psql -U stablerwa -d stablerwa -t -c "SELECT COUNT(*) FROM test_table;")
    if [[ "$result" =~ [0-9]+ ]] && [ "${result// /}" -ge 2 ]; then
        log_success "PostgreSQL operations test passed"
    else
        log_error "PostgreSQL operations test failed"
        return 1
    fi
    
    # Test Redis operations
    log_info "Testing Redis operations..."
    
    # Set test data
    docker exec stablerwa-redis redis-cli -a StableRWA2024! SET test_key "test_value" &> /dev/null
    
    # Get test data
    local redis_result=$(docker exec stablerwa-redis redis-cli -a StableRWA2024! GET test_key 2>/dev/null)
    if [ "$redis_result" = "test_value" ]; then
        log_success "Redis operations test passed"
    else
        log_error "Redis operations test failed"
        return 1
    fi
}

# Performance testing
run_performance_tests() {
    log_section "Running basic performance tests"
    
    # PostgreSQL performance test
    log_info "Running PostgreSQL performance test..."
    
    local start_time=$(date +%s)
    
    # Insert 1000 records
    docker exec stablerwa-postgres psql -U stablerwa -d stablerwa -c "
        INSERT INTO test_table (name) 
        SELECT 'Performance Test ' || generate_series(1, 1000);
    " &> /dev/null
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_info "PostgreSQL performance test results:"
    log_info "  Inserted 1000 records in ${duration}s"
    
    if [ $duration -le 10 ]; then
        log_success "PostgreSQL performance test passed"
    else
        log_warning "PostgreSQL performance test slow but acceptable"
    fi
    
    # Redis performance test
    log_info "Running Redis performance test..."
    
    start_time=$(date +%s)
    
    # Set 1000 keys
    for i in {1..1000}; do
        docker exec stablerwa-redis redis-cli -a StableRWA2024! SET "perf_key_$i" "value_$i" &> /dev/null
    done
    
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    
    log_info "Redis performance test results:"
    log_info "  Set 1000 keys in ${duration}s"
    
    if [ $duration -le 5 ]; then
        log_success "Redis performance test passed"
    else
        log_warning "Redis performance test slow but acceptable"
    fi
}

# Generate test report
generate_test_report() {
    log_section "Generating test report"
    
    local report_file="$TEST_RESULTS_DIR/basic-docker-test-report.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>StableRWA Basic Docker Test Report</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 40px;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
            color: #e8d5b7;
            min-height: 100vh;
            line-height: 1.6;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: rgba(42, 39, 56, 0.95);
            border-radius: 15px;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
            overflow: hidden;
            backdrop-filter: blur(10px);
        }

        .header {
            background: linear-gradient(135deg, #d4a574 0%, #c8956d 50%, #b8835a 100%);
            padding: 30px;
            color: #2a2738;
            text-align: center;
            position: relative;
        }

        .header::before {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><pattern id="grain" width="100" height="100" patternUnits="userSpaceOnUse"><circle cx="25" cy="25" r="1" fill="%23ffffff" opacity="0.1"/><circle cx="75" cy="75" r="1" fill="%23ffffff" opacity="0.1"/><circle cx="50" cy="10" r="0.5" fill="%23ffffff" opacity="0.1"/></pattern></defs><rect width="100" height="100" fill="url(%23grain)"/></svg>');
            opacity: 0.3;
        }

        .header h1 {
            font-size: 2.5em;
            font-weight: 700;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
            position: relative;
            z-index: 1;
        }

        .header p {
            font-size: 1.1em;
            opacity: 0.9;
            position: relative;
            z-index: 1;
        }

        .content {
            padding: 30px;
        }

        .section {
            margin: 25px 0;
            padding: 20px;
            border-left: 4px solid #d4a574;
            background: rgba(52, 48, 64, 0.6);
            border-radius: 0 10px 10px 0;
            transition: all 0.3s ease;
        }

        .section:hover {
            background: rgba(52, 48, 64, 0.8);
            transform: translateX(5px);
        }

        .success {
            border-left-color: #90c695;
            background: rgba(144, 198, 149, 0.1);
        }

        .warning {
            border-left-color: #f4d03f;
            background: rgba(244, 208, 63, 0.1);
        }

        .error {
            border-left-color: #e74c3c;
            background: rgba(231, 76, 60, 0.1);
        }

        .section h2 {
            color: #d4a574;
            margin-bottom: 15px;
            font-size: 1.5em;
            font-weight: 600;
        }

        .metrics {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }

        .metric {
            background: linear-gradient(135deg, rgba(212, 165, 116, 0.2) 0%, rgba(184, 131, 90, 0.2) 100%);
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            border: 1px solid rgba(212, 165, 116, 0.3);
            transition: all 0.3s ease;
        }

        .metric:hover {
            transform: translateY(-5px);
            box-shadow: 0 10px 20px rgba(212, 165, 116, 0.2);
        }

        .metric h3 {
            color: #d4a574;
            margin-bottom: 10px;
            font-size: 1.2em;
        }

        .metric p {
            font-size: 1.4em;
            font-weight: bold;
            color: #f0e6d2;
        }

        pre {
            background: rgba(26, 26, 46, 0.8);
            color: #e8d5b7;
            padding: 20px;
            border-radius: 8px;
            overflow-x: auto;
            border: 1px solid rgba(212, 165, 116, 0.2);
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
            line-height: 1.4;
        }

        ul {
            list-style: none;
            padding-left: 0;
        }

        li {
            padding: 8px 0;
            border-bottom: 1px solid rgba(212, 165, 116, 0.1);
            font-size: 1.1em;
        }

        li:last-child {
            border-bottom: none;
        }

        .status-icon {
            margin-right: 10px;
            font-size: 1.2em;
        }

        /* Scrollbar styling */
        ::-webkit-scrollbar {
            width: 8px;
        }

        ::-webkit-scrollbar-track {
            background: rgba(26, 26, 46, 0.5);
        }

        ::-webkit-scrollbar-thumb {
            background: rgba(212, 165, 116, 0.5);
            border-radius: 4px;
        }

        ::-webkit-scrollbar-thumb:hover {
            background: rgba(212, 165, 116, 0.7);
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 StableRWA Basic Docker Test Report</h1>
            <p>Generated on: $(date)</p>
            <p>Test Environment: Docker Compose Basic Setup</p>
        </div>

        <div class="content">
            <div class="section success">
                <h2>📊 Test Summary</h2>
                <div class="metrics">
                    <div class="metric">
                        <h3>Services Tested</h3>
                        <p>2 Services</p>
                    </div>
                    <div class="metric">
                        <h3>Database</h3>
                        <p>PostgreSQL 15</p>
                    </div>
                    <div class="metric">
                        <h3>Cache</h3>
                        <p>Redis 7</p>
                    </div>
                    <div class="metric">
                        <h3>Test Duration</h3>
                        <p>$(date)</p>
                    </div>
                </div>
            </div>

            <div class="section">
                <h2>🔧 Service Status</h2>
                <ul>
                    <li><span class="status-icon">✅</span>PostgreSQL Database - Running & Healthy</li>
                    <li><span class="status-icon">✅</span>Redis Cache - Running & Healthy</li>
                </ul>
            </div>

            <div class="section">
                <h2>📋 Test Results</h2>
                <pre>$(cat "$LOG_FILE")</pre>
            </div>

            <div class="section">
                <h2>🐳 Docker Container Status</h2>
                <pre>$(docker-compose -f "$COMPOSE_FILE" ps)</pre>
            </div>

            <div class="section">
                <h2>📈 Resource Usage</h2>
                <pre>$(docker stats --no-stream stablerwa-postgres stablerwa-redis)</pre>
            </div>
        </div>
    </div>
</body>
</html>
EOF
    
    log_success "Test report generated: $report_file"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    log_section "Starting StableRWA Basic Docker Test Suite"
    
    # Setup
    setup_test_environment
    
    # Check services
    check_service_health "postgres" "5432"
    check_service_health "redis" "6379"
    
    # Run tests
    test_database_connectivity
    test_database_operations
    run_performance_tests
    
    # Generate report
    generate_test_report
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    log_section "Basic Docker Test Suite Completed"
    log_success "Total test duration: ${total_duration} seconds"
    log_info "Test results available at: $TEST_RESULTS_DIR"
    log_info "Test report: $TEST_RESULTS_DIR/basic-docker-test-report.html"
    log_info "Services are running. Use 'docker-compose -f $COMPOSE_FILE ps' to check status."
}

# Run main function
main "$@"
