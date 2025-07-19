# Audit Microservice (StableRWA)

**Owner:** arkSong (arksong2018@gmail.com)

---

## Overview
Enterprise-grade audit microservice for the StableRWA platform. Provides RESTful APIs for logging, querying, compliance checking, and reporting audit events. Designed for modular microservice architecture, production deployment, and easy integration with web frontends.

- **Backend:** Rust, Actix-web, modular crate
- **API:** RESTful, JSON
- **Logging:** Timestamped, structured
- **Testing:** 100% coverage (unit + integration)
- **Deployment:** Docker-ready

---

## API Endpoints

- `POST   /audit/log`         — Log a new audit event
- `GET    /audit/events`      — List all audit events
- `POST   /audit/compliance`  — Run compliance check on an event
- `GET    /audit/report`      — Generate audit report

### Example: Log Event
```sh
curl -X POST http://localhost:8095/audit/log \
  -H 'Content-Type: application/json' \
  -d '{"event_type": "login", "actor": "user1", "target": "system", "description": "User login"}'
```

### Example: List Events
```sh
curl http://localhost:8095/audit/events
```

### Example: Compliance Check
```sh
curl -X POST http://localhost:8095/audit/compliance \
  -H 'Content-Type: application/json' \
  -d '{"id": 1}'
```

### Example: Generate Report
```sh
curl http://localhost:8095/audit/report
```

---

## Environment Variables
- `RUST_LOG` — Logging level (default: info)
- (Optional) Add compliance/report config for advanced audit in future

---

## Docker Usage

Build and run the microservice:
```sh
docker build -t audit-service .
docker run -p 8095:8095 --env RUST_LOG=info audit-service
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