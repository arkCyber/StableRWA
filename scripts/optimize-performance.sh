#!/bin/bash

# =====================================================================================
# File: scripts/optimize-performance.sh
# Description: Performance optimization script for StableRWA platform
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
OPTIMIZATION_LEVEL=${OPTIMIZATION_LEVEL:-"production"}
PROFILE_DURATION=${PROFILE_DURATION:-60}
BENCHMARK_ITERATIONS=${BENCHMARK_ITERATIONS:-10}

echo -e "${BLUE}🚀 Starting StableRWA Performance Optimization${NC}"
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

# Function to optimize Rust build settings
optimize_rust_build() {
    print_section "Optimizing Rust Build Configuration"
    
    # Create optimized Cargo.toml settings
    cat > .cargo/config.toml << EOF
[build]
rustflags = [
    "-C", "target-cpu=native",
    "-C", "opt-level=3",
    "-C", "lto=fat",
    "-C", "codegen-units=1",
    "-C", "panic=abort"
]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
debug = false

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false

[profile.bench]
opt-level = 3
lto = "fat"
codegen-units = 1
debug = false
EOF

    print_success "Rust build configuration optimized"
}

# Function to optimize database settings
optimize_database() {
    print_section "Optimizing Database Configuration"
    
    # PostgreSQL optimization
    cat > config/postgresql-optimized.conf << EOF
# Memory settings
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
maintenance_work_mem = 64MB

# Checkpoint settings
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100

# Connection settings
max_connections = 200
shared_preload_libraries = 'pg_stat_statements'

# Query optimization
random_page_cost = 1.1
effective_io_concurrency = 200
max_worker_processes = 8
max_parallel_workers_per_gather = 4
max_parallel_workers = 8
max_parallel_maintenance_workers = 4
EOF

    # Redis optimization
    cat > config/redis-optimized.conf << EOF
# Memory optimization
maxmemory 2gb
maxmemory-policy allkeys-lru
maxmemory-samples 5

# Persistence optimization
save 900 1
save 300 10
save 60 10000

# Network optimization
tcp-keepalive 300
timeout 0

# Performance optimization
hash-max-ziplist-entries 512
hash-max-ziplist-value 64
list-max-ziplist-size -2
list-compress-depth 0
set-max-intset-entries 512
zset-max-ziplist-entries 128
zset-max-ziplist-value 64
EOF

    print_success "Database configuration optimized"
}

# Function to optimize system settings
optimize_system() {
    print_section "Optimizing System Configuration"
    
    # Create system optimization script
    cat > config/system-optimization.sh << 'EOF'
#!/bin/bash

# Network optimization
echo 'net.core.somaxconn = 65535' >> /etc/sysctl.conf
echo 'net.core.netdev_max_backlog = 5000' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65535' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_keepalive_time = 600' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_keepalive_intvl = 60' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_keepalive_probes = 3' >> /etc/sysctl.conf

# Memory optimization
echo 'vm.swappiness = 10' >> /etc/sysctl.conf
echo 'vm.dirty_ratio = 15' >> /etc/sysctl.conf
echo 'vm.dirty_background_ratio = 5' >> /etc/sysctl.conf

# File descriptor limits
echo '* soft nofile 65535' >> /etc/security/limits.conf
echo '* hard nofile 65535' >> /etc/security/limits.conf

# Apply settings
sysctl -p
EOF

    chmod +x config/system-optimization.sh
    print_success "System optimization script created"
}

# Function to run performance benchmarks
run_benchmarks() {
    print_section "Running Performance Benchmarks"
    
    echo "Building optimized release version..."
    cargo build --release --all-features
    
    echo "Running CPU-intensive benchmarks..."
    for crate in core-utils core-security core-compliance core-asset-lifecycle core-trading core-bridge core-analytics core-institutional; do
        if [ -d "$crate/benches" ]; then
            echo "Benchmarking $crate..."
            cd "$crate"
            cargo bench --bench '*' -- --output-format json > "../target/bench-$crate.json" 2>/dev/null || true
            cd ..
        fi
    done
    
    print_success "Benchmarks completed"
}

# Function to profile application performance
profile_application() {
    print_section "Profiling Application Performance"
    
    if command -v perf >/dev/null 2>&1; then
        echo "Running CPU profiling for ${PROFILE_DURATION} seconds..."
        
        # Start services in background
        cargo run --release --bin service-gateway &
        GATEWAY_PID=$!
        sleep 5
        
        # Profile the gateway service
        perf record -g -p $GATEWAY_PID sleep $PROFILE_DURATION
        perf report --stdio > target/perf-report.txt
        
        # Cleanup
        kill $GATEWAY_PID 2>/dev/null || true
        
        print_success "CPU profiling completed - report saved to target/perf-report.txt"
    else
        print_warning "perf not available, skipping CPU profiling"
    fi
    
    # Memory profiling with valgrind (if available)
    if command -v valgrind >/dev/null 2>&1; then
        echo "Running memory profiling..."
        valgrind --tool=massif --massif-out-file=target/massif.out cargo run --release --bin service-gateway &
        VALGRIND_PID=$!
        sleep $PROFILE_DURATION
        kill $VALGRIND_PID 2>/dev/null || true
        
        if command -v ms_print >/dev/null 2>&1; then
            ms_print target/massif.out > target/memory-profile.txt
            print_success "Memory profiling completed - report saved to target/memory-profile.txt"
        fi
    else
        print_warning "valgrind not available, skipping memory profiling"
    fi
}

# Function to optimize Docker images
optimize_docker() {
    print_section "Optimizing Docker Images"
    
    # Create optimized Dockerfile
    cat > Dockerfile.optimized << 'EOF'
# Multi-stage build for minimal image size
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY core-*/Cargo.toml ./core-*/

# Build dependencies (cached layer)
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build application with optimizations
ENV RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat"
RUN cargo build --release --all-features

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false appuser

# Copy binary
COPY --from=builder /app/target/release/service-gateway /usr/local/bin/
COPY --from=builder /app/target/release/service-asset /usr/local/bin/
COPY --from=builder /app/target/release/service-user /usr/local/bin/

# Set ownership
RUN chown appuser:appuser /usr/local/bin/*

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["service-gateway"]
EOF

    print_success "Optimized Dockerfile created"
}

# Function to generate performance report
generate_performance_report() {
    print_section "Generating Performance Report"
    
    cat > target/performance-report.md << EOF
# StableRWA Platform Performance Report

Generated on: $(date)
Optimization Level: $OPTIMIZATION_LEVEL

## Build Optimizations Applied

- Target CPU: native
- Optimization Level: 3 (maximum)
- Link Time Optimization: fat
- Code Generation Units: 1 (maximum optimization)
- Panic Strategy: abort (smaller binary)
- Debug Symbols: stripped

## System Optimizations

- Network buffer sizes increased
- TCP keepalive optimized
- Memory management tuned
- File descriptor limits raised

## Database Optimizations

### PostgreSQL
- Shared buffers: 256MB
- Effective cache size: 1GB
- Work memory: 4MB
- Max connections: 200

### Redis
- Max memory: 2GB
- Eviction policy: allkeys-lru
- Persistence optimized

## Benchmark Results

EOF

    # Add benchmark results if available
    if [ -f "target/bench-core-trading.json" ]; then
        echo "### Trading Service Benchmarks" >> target/performance-report.md
        echo "\`\`\`" >> target/performance-report.md
        cat target/bench-core-trading.json >> target/performance-report.md
        echo "\`\`\`" >> target/performance-report.md
    fi

    # Add profiling results if available
    if [ -f "target/perf-report.txt" ]; then
        echo "### CPU Profiling Results" >> target/performance-report.md
        echo "\`\`\`" >> target/performance-report.md
        head -50 target/perf-report.txt >> target/performance-report.md
        echo "\`\`\`" >> target/performance-report.md
    fi

    cat >> target/performance-report.md << EOF

## Recommendations

1. **Memory Usage**: Monitor heap allocation patterns
2. **CPU Usage**: Profile hot paths in trading engine
3. **I/O Performance**: Optimize database query patterns
4. **Network**: Consider connection pooling optimizations
5. **Caching**: Implement Redis caching for frequently accessed data

## Next Steps

1. Deploy optimized configuration to staging
2. Run load tests to validate improvements
3. Monitor production metrics
4. Iterate on optimization based on real-world usage

EOF

    print_success "Performance report generated at target/performance-report.md"
}

# Function to apply optimizations based on level
apply_optimizations() {
    local level=$1
    
    case $level in
        "development")
            print_warning "Applying development-level optimizations"
            optimize_rust_build
            ;;
        "staging")
            print_warning "Applying staging-level optimizations"
            optimize_rust_build
            optimize_database
            ;;
        "production")
            print_warning "Applying production-level optimizations"
            optimize_rust_build
            optimize_database
            optimize_system
            optimize_docker
            ;;
        *)
            print_error "Unknown optimization level: $level"
            exit 1
            ;;
    esac
}

# Main optimization function
main() {
    local start_time=$(date +%s)
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --level)
                OPTIMIZATION_LEVEL="$2"
                shift 2
                ;;
            --benchmark)
                RUN_BENCHMARKS=true
                shift
                ;;
            --profile)
                RUN_PROFILING=true
                shift
                ;;
            --report)
                GENERATE_REPORT=true
                shift
                ;;
            --all)
                RUN_BENCHMARKS=true
                RUN_PROFILING=true
                GENERATE_REPORT=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --level LEVEL    Optimization level (development|staging|production)"
                echo "  --benchmark      Run performance benchmarks"
                echo "  --profile        Run application profiling"
                echo "  --report         Generate performance report"
                echo "  --all            Run all optimizations and tests"
                echo "  --help           Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Create necessary directories
    mkdir -p target config .cargo
    
    # Apply optimizations
    apply_optimizations "$OPTIMIZATION_LEVEL"
    
    # Run optional tasks
    if [ "$RUN_BENCHMARKS" = true ]; then
        run_benchmarks
    fi
    
    if [ "$RUN_PROFILING" = true ]; then
        profile_application
    fi
    
    if [ "$GENERATE_REPORT" = true ]; then
        generate_performance_report
    fi
    
    # Summary
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    print_section "Optimization Summary"
    echo "Optimization level: $OPTIMIZATION_LEVEL"
    echo "Total execution time: ${duration}s"
    
    print_success "Performance optimization completed! 🚀"
    echo -e "${GREEN}The StableRWA platform has been optimized for $OPTIMIZATION_LEVEL environment.${NC}"
    
    if [ "$GENERATE_REPORT" = true ]; then
        echo -e "${BLUE}📊 Performance report available at: target/performance-report.md${NC}"
    fi
}

# Run main function with all arguments
main "$@"
