# =====================================================================================
# File: collateral-management/Dockerfile
# Description: Dockerfile for building and running the collateral management microservice.
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

FROM rust:1.74 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/collateral-management /usr/local/bin/collateral-management
ENV RUST_LOG=info
EXPOSE 8094
CMD ["/usr/local/bin/collateral-management"] 