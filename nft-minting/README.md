# NFT Minting Microservice (StableRWA)

**Owner:** arkSong (arksong2018@gmail.com)

---

## Overview
Enterprise-grade NFT minting microservice for the StableRWA platform. Provides RESTful APIs for minting, querying, and managing NFTs (ERC-721 compatible). Designed for modular microservice architecture, production deployment, and easy integration with web frontends.

- **Backend:** Rust, Actix-web, modular crate
- **API:** RESTful, JSON
- **Logging:** Timestamped, structured
- **Testing:** 100% coverage (unit + integration)
- **Deployment:** Docker-ready

---

## API Endpoints

- `POST   /mint-nft`         — Mint a new NFT
- `GET    /nft/{id}`         — Get NFT details by ID
- `GET    /nft/owner/{addr}` — List all NFTs owned by an address

### Example: Mint NFT
```sh
curl -X POST http://localhost:8092/mint-nft \
  -H 'Content-Type: application/json' \
  -d '{"owner": "0xabc...", "metadata_uri": "ipfs://Qm..."}'
```

### Example: Get NFT by ID
```sh
curl http://localhost:8092/nft/1
```

### Example: Get NFTs by Owner
```sh
curl http://localhost:8092/nft/owner/0xabc...
```

---

## Environment Variables
- `RUST_LOG` — Logging level (default: info)
- (Optional) Add blockchain node/private key config for on-chain minting in future

---

## Docker Usage

Build and run the microservice:
```sh
docker build -t nft-minting .
docker run -p 8092:8092 --env RUST_LOG=info nft-minting
```

---

## Testing

Run all tests (unit + integration):
```sh
cargo test
```

---

## Contact
- **Owner:** arkSong (arksong2018@gmail.com)
- For questions, suggestions, or contributions, please contact the owner. 