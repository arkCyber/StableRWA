# =====================================================================================
# Dockerfile for service-asset microservice (Rust, Actix-web)
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

# Build stage
FROM rust:1.77 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin service-asset

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/service-asset /usr/local/bin/service-asset
EXPOSE 8080
CMD ["/usr/local/bin/service-asset"] 