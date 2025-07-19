# =====================================================================================
# Dockerfile for user-service microservice (Rust, Actix-web)
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

# Build stage
FROM rust:1.77 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin user-service

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/user-service /usr/local/bin/user-service
EXPOSE 8081
CMD ["/usr/local/bin/user-service"] 