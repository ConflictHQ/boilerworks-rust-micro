# Boilerworks Rust Micro

> High-performance Rust microservice with Axum, SQLx, and API-key auth.
> No frontend, no sessions -- pure API service.

## Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust (Axum 0.8 + tokio) |
| Queries | SQLx (runtime, async Postgres) |
| Migrations | SQLx embedded |
| Database | PostgreSQL 16 |
| Auth | API-key (SHA256, per-key scopes) |
| Linting | Clippy + rustfmt |

## Getting Started

```bash
# Start services
docker compose up -d --build

# Get your seed API key (shown once on first boot)
docker compose logs api | grep "Plaintext key"

# Test it
curl http://localhost:8000/health
curl -H "X-API-Key: bw_seed_key_change_me_in_production" http://localhost:8000/events
```

## Endpoints

| Method | Path | Auth | Scope | Description |
|--------|------|------|-------|-------------|
| GET | /health | None | - | Health check |
| POST | /events | API Key | events.write | Create event |
| GET | /events | API Key | events.read | List events |
| GET | /events/{id} | API Key | events.read | Event detail |
| DELETE | /events/{id} | API Key | events.write | Soft delete |
| POST | /api-keys | API Key | keys.manage | Create key |
| GET | /api-keys | API Key | keys.manage | List keys |
| DELETE | /api-keys/{id} | API Key | keys.manage | Revoke key |

## Commands

```bash
make up        # Start Docker services
make down      # Stop services
make build     # Build release binary
make test      # Run tests (needs Postgres on :5432)
make lint      # Clippy + fmt check
make logs      # Tail container logs
```

## Documentation

- [bootstrap.md](bootstrap.md) -- Conventions and patterns
- [CLAUDE.md](CLAUDE.md) -- Agent shim

---

Boilerworks is a [CONFLICT](https://weareconflict.com) brand. CONFLICT is a registered trademark of CONFLICT LLC.
