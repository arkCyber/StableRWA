# =====================================================================================
# Dockerfile for auth-service microservice (Rust, Actix-web)
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

# Build stage
FROM rust:1.77 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin auth-service

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/auth-service /usr/local/bin/auth-service
EXPOSE 8083
CMD ["/usr/local/bin/auth-service"] 