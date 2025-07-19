# =====================================================================================
# File: audit-service/Dockerfile
# Description: Dockerfile for building and running the audit microservice.
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

FROM rust:1.74 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/audit-service /usr/local/bin/audit-service
ENV RUST_LOG=info
EXPOSE 8095
CMD ["/usr/local/bin/audit-service"] 