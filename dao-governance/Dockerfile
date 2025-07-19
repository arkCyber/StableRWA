# =====================================================================================
# File: dao-governance/Dockerfile
# Description: Dockerfile for building and running the DAO governance microservice.
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

FROM rust:1.74 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/dao-governance /usr/local/bin/dao-governance
ENV RUST_LOG=info
EXPOSE 8093
CMD ["/usr/local/bin/dao-governance"] 