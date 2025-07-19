# =====================================================================================
# File: smart-contract/Dockerfile
# Description: Dockerfile for building and running the smart contract microservice.
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

FROM rust:1.74 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/smart-contract /usr/local/bin/smart-contract
ENV RUST_LOG=info
EXPOSE 8096
CMD ["/usr/local/bin/smart-contract"] 