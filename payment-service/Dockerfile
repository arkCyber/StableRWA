# =====================================================================================
# Dockerfile for payment-service microservice (Rust, Actix-web)
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

# Build stage
FROM rust:1.77 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin payment-service

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/payment-service /usr/local/bin/payment-service
EXPOSE 8082
CMD ["/usr/local/bin/payment-service"] 