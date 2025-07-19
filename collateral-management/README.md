# Collateral Management Microservice (StableRWA)

**Owner:** arkSong (arksong2018@gmail.com)

---

## Overview
Enterprise-grade collateral management microservice for the StableRWA platform. Provides RESTful APIs for registering, updating, releasing, and querying collateral assets. Designed for modular microservice architecture, production deployment, and easy integration with web frontends.

- **Backend:** Rust, Actix-web, modular crate
- **API:** RESTful, JSON
- **Logging:** Timestamped, structured
- **Testing:** 100% coverage (unit + integration)
- **Deployment:** Docker-ready

---

## API Endpoints

- `POST   /collateral/register`         — Register a new collateral asset
- `POST   /collateral/update`           — Update collateral value
- `POST   /collateral/release`          — Release a collateral asset
- `GET    /collateral/{id}`             — Get collateral by ID
- `GET    /collaterals/owner/{owner}`   — List all collaterals for an owner
- `GET    /collaterals`                 — List all collaterals

### Example: Register Collateral
```sh
curl -X POST http://localhost:8094/collateral/register \
  -H 'Content-Type: application/json' \
  -d '{"owner": "0xabc...", "asset_type": "RealEstate", "value": 1000.0}'
```

### Example: Update Value
```sh
curl -X POST http://localhost:8094/collateral/update \
  -H 'Content-Type: application/json' \
  -d '{"id": 1, "value": 1200.0}'
```

### Example: Release Collateral
```sh
curl -X POST http://localhost:8094/collateral/release \
  -H 'Content-Type: application/json' \
  -d '{"id": 1}'
```

### Example: Query by ID
```sh
curl http://localhost:8094/collateral/1
```

### Example: List by Owner
```sh
curl http://localhost:8094/collaterals/owner/0xabc...
```

### Example: List All
```sh
curl http://localhost:8094/collaterals
```

---

## Environment Variables
- `RUST_LOG` — Logging level (default: info)
- (Optional) Add valuation/risk config for advanced management in future

---

## Docker Usage

Build and run the microservice:
```sh
docker build -t collateral-management .
docker run -p 8094:8094 --env RUST_LOG=info collateral-management
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