# =====================================================================================
# File: nft-minting/Dockerfile
# Description: Dockerfile for building and running the NFT minting microservice.
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

FROM rust:1.74 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/nft-minting /usr/local/bin/nft-minting
ENV RUST_LOG=info
EXPOSE 8092
CMD ["/usr/local/bin/nft-minting"] 