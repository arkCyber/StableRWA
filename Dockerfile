# =====================================================================================
# Multi-stage Dockerfile for RWA Platform
# Optimized for production deployment with security and performance
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
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY workspace.toml ./

# Copy all crate manifests
COPY core-blockchain/Cargo.toml ./core-blockchain/
COPY core-config/Cargo.toml ./core-config/
COPY core-database/Cargo.toml ./core-database/
COPY core-events/Cargo.toml ./core-events/
COPY core-observability/Cargo.toml ./core-observability/
COPY core-security/Cargo.toml ./core-security/
COPY core-utils/Cargo.toml ./core-utils/
COPY service-asset/Cargo.toml ./service-asset/
COPY service-gateway/Cargo.toml ./service-gateway/
COPY service-payment/Cargo.toml ./service-payment/
COPY service-user/Cargo.toml ./service-user/

# Create dummy source files to cache dependencies
RUN mkdir -p core-blockchain/src core-config/src core-database/src core-events/src \
    core-observability/src core-security/src core-utils/src service-asset/src \
    service-gateway/src service-payment/src service-user/src && \
    echo "fn main() {}" > core-blockchain/src/main.rs && \
    echo "fn main() {}" > core-config/src/main.rs && \
    echo "fn main() {}" > core-database/src/main.rs && \
    echo "fn main() {}" > core-events/src/main.rs && \
    echo "fn main() {}" > core-observability/src/main.rs && \
    echo "fn main() {}" > core-security/src/main.rs && \
    echo "fn main() {}" > core-utils/src/main.rs && \
    echo "fn main() {}" > service-asset/src/main.rs && \
    echo "fn main() {}" > service-gateway/src/main.rs && \
    echo "fn main() {}" > service-payment/src/main.rs && \
    echo "fn main() {}" > service-user/src/main.rs && \
    find . -name "*.rs" -exec touch {} \;

# Build dependencies (this layer will be cached)
RUN cargo build --release --workspace
RUN rm -rf core-*/src service-*/src

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Create necessary directories
RUN mkdir -p /app/config /app/logs /app/data && \
    chown -R appuser:appuser /app

# Copy binaries from builder
COPY --from=builder /app/target/release/service-gateway /app/
COPY --from=builder /app/target/release/service-user /app/
COPY --from=builder /app/target/release/service-asset /app/
COPY --from=builder /app/target/release/service-payment /app/

# Copy configuration files
COPY --chown=appuser:appuser config/ /app/config/

# Switch to app user
USER appuser

# Set working directory
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command (can be overridden)
CMD ["./service-gateway"]

# =====================================================================================
# Gateway service specific image
# =====================================================================================
FROM runtime as gateway
EXPOSE 8080
CMD ["./service-gateway"]

# =====================================================================================
# User service specific image
# =====================================================================================
FROM runtime as user-service
EXPOSE 8081
CMD ["./service-user"]

# =====================================================================================
# Asset service specific image
# =====================================================================================
FROM runtime as asset-service
EXPOSE 8082
CMD ["./service-asset"]

# =====================================================================================
# Payment service specific image
# =====================================================================================
FROM runtime as payment-service
EXPOSE 8083
CMD ["./service-payment"]

# =====================================================================================
# Development image with additional tools
# =====================================================================================
FROM rust:1.75-slim as development

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    git \
    curl \
    vim \
    postgresql-client \
    redis-tools \
    && rm -rf /var/lib/apt/lists/*

# Install cargo tools
RUN cargo install cargo-watch cargo-edit sqlx-cli

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Switch to app user
USER appuser

# Default command for development
CMD ["cargo", "watch", "-x", "run"]

# =====================================================================================
# Testing image
# =====================================================================================
FROM development as testing

# Copy source code
COPY --chown=appuser:appuser . .

# Run tests
CMD ["cargo", "test", "--workspace"]
