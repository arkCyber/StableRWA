# =====================================================================================
# File: bsc-integration/Dockerfile
# Description: Dockerfile for building and running the BSC integration microservice.
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

FROM rust:1.74 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/bsc-integration /usr/local/bin/bsc-integration
ENV RUST_LOG=info
EXPOSE 8091
CMD ["/usr/local/bin/bsc-integration"] 