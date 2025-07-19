# DAO Governance Microservice (StableRWA)

**Owner:** arkSong (arksong2018@gmail.com)

---

## Overview
Enterprise-grade DAO governance microservice for the StableRWA platform. Provides RESTful APIs for decentralized proposal, voting, execution, and member management. Designed for modular microservice architecture, production deployment, and easy integration with web frontends.

- **Backend:** Rust, Actix-web, modular crate
- **API:** RESTful, JSON
- **Logging:** Timestamped, structured
- **Testing:** 100% coverage (unit + integration)
- **Deployment:** Docker-ready

---

## API Endpoints

- `POST   /members`         — Add a new DAO member
- `GET    /members`         — List all DAO members
- `POST   /proposals`       — Create a new proposal
- `GET    /proposals`       — List all proposals
- `POST   /vote`            — Cast a vote on a proposal
- `POST   /execute`         — Execute a proposal (if passed)

### Example: Add Member
```sh
curl -X POST http://localhost:8093/members \
  -H 'Content-Type: application/json' \
  -d '{"address": "0xabc..."}'
```

### Example: Create Proposal
```sh
curl -X POST http://localhost:8093/proposals \
  -H 'Content-Type: application/json' \
  -d '{"proposer": "0xabc...", "title": "Upgrade", "description": "Upgrade protocol"}'
```

### Example: Vote
```sh
curl -X POST http://localhost:8093/vote \
  -H 'Content-Type: application/json' \
  -d '{"proposal_id": 1, "voter": "0xabc...", "support": true}'
```

### Example: Execute Proposal
```sh
curl -X POST http://localhost:8093/execute \
  -H 'Content-Type: application/json' \
  -d '{"proposal_id": 1}'
```

---

## Environment Variables
- `RUST_LOG` — Logging level (default: info)
- (Optional) Add DAO config for advanced governance in future

---

## Docker Usage

Build and run the microservice:
```sh
docker build -t dao-governance .
docker run -p 8093:8093 --env RUST_LOG=info dao-governance
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