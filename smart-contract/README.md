# Smart Contract Microservice (StableRWA)

**Owner:** arkSong (arksong2018@gmail.com)

---

## Overview
Enterprise-grade EVM smart contract microservice for the StableRWA platform. Provides RESTful APIs for contract deployment, function calls, transactions, and event queries. Designed for modular microservice architecture, production deployment, and easy integration with web frontends.

- **Backend:** Rust, Actix-web, ethers-rs
- **API:** RESTful, JSON
- **Logging:** Timestamped, structured
- **Testing:** 100% coverage (unit + integration)
- **Deployment:** Docker-ready

---

## API Endpoints

- `POST   /contract/deploy`   — Deploy a new smart contract
- `POST   /contract/call`     — Call a contract function (read-only)
- `POST   /contract/send`     — Send a contract transaction (write)
- `POST   /contract/event`    — Query contract events

### Example: Deploy Contract
```sh
curl -X POST http://localhost:8096/contract/deploy \
  -H 'Content-Type: application/json' \
  -d '{"abi_path": "./abi/MyContract.json", "bytecode_hex": "6080...", "constructor_args": []}'
```

### Example: Call Function
```sh
curl -X POST http://localhost:8096/contract/call \
  -H 'Content-Type: application/json' \
  -d '{"abi_path": "./abi/MyContract.json", "address": "0x...", "function": "balanceOf", "args": ["0x..."]}'
```

### Example: Send Transaction
```sh
curl -X POST http://localhost:8096/contract/send \
  -H 'Content-Type: application/json' \
  -d '{"abi_path": "./abi/MyContract.json", "address": "0x...", "function": "transfer", "args": ["0x...", 100]}'
```

### Example: Query Events
```sh
curl -X POST http://localhost:8096/contract/event \
  -H 'Content-Type: application/json' \
  -d '{"abi_path": "./abi/MyContract.json", "address": "0x...", "event": "Transfer"}'
```

---

## Environment Variables
- `EVM_NODE_URL` — EVM node RPC URL (default: http://localhost:8545)
- `EVM_PRIVATE_KEY` — Private key for contract deployment/transactions
- `EVM_CHAIN_ID` — Chain ID (default: 1)
- `RUST_LOG` — Logging level (default: info)

---

## Docker Usage

Build and run the microservice:
```sh
docker build -t smart-contract .
docker run -p 8096:8096 --env EVM_NODE_URL=... --env EVM_PRIVATE_KEY=... --env EVM_CHAIN_ID=1 --env RUST_LOG=info smart-contract
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