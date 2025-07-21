# =====================================================================================
# Multi-stage Dockerfile for StableRWA Gateway Service
# =====================================================================================

# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY service-gateway/Cargo.toml ./service-gateway/
COPY core-utils/Cargo.toml ./core-utils/
COPY core-security/Cargo.toml ./core-security/
COPY core-blockchain/Cargo.toml ./core-blockchain/

# Create dummy source files to cache dependencies
RUN mkdir -p service-gateway/src core-utils/src core-security/src core-blockchain/src && \
    echo "fn main() {}" > service-gateway/src/main.rs && \
    echo "// dummy" > core-utils/src/lib.rs && \
    echo "// dummy" > core-security/src/lib.rs && \
    echo "// dummy" > core-blockchain/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --bin service-gateway
RUN rm -rf service-gateway/src core-utils/src core-security/src core-blockchain/src

# Copy source code
COPY service-gateway/src ./service-gateway/src
COPY core-utils/src ./core-utils/src
COPY core-security/src ./core-security/src
COPY core-blockchain/src ./core-blockchain/src

# Build application
RUN touch service-gateway/src/main.rs && \
    cargo build --release --bin service-gateway

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

# Create necessary directories
RUN mkdir -p /app/config /app/logs /app/data && \
    chown -R appuser:appuser /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/service-gateway /app/gateway
COPY --chown=appuser:appuser service-gateway/config/ /app/config/

# Set permissions
RUN chmod +x /app/gateway

# Switch to app user
USER appuser

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV CONFIG_PATH=/app/config

# Run the application
CMD ["./gateway"]
