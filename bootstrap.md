# Boilerworks Rust Micro -- Bootstrap

> Rust microservice with Axum, SQLx, tokio, and API-key authentication.
> No frontend, no sessions -- pure API service.

## Architecture

```
Caller (service, cron, webhook sender)
  |
  v (HTTP + X-API-Key header)
  |
Axum (tokio runtime)
  |-- SQLx runtime queries (Postgres 16)
  +-- JSON API responses
```

## Conventions

### Auth
- All endpoints require `X-API-Key` header except `/health`
- Keys are SHA256-hashed before storage -- plaintext never stored
- Per-key scopes: `events.read`, `events.write`, `keys.manage`, `*`
- `ApiKeyAuth` extractor validates key via `FromRequestParts`
- `require_scope()` checks scope on individual handlers

### Models
- UUID primary keys (`gen_random_uuid()`)
- Snake_case table and column names
- Audit fields: `created_at`, `updated_at`
- Soft deletes: `deleted_at` field, queries filter automatically

### API
- All responses wrapped in `ApiResponse`: `{ ok, data, message, errors }`
- JSON request bodies via serde + Axum extractors
- Validation errors return 400 with details in `errors` array

### Database
- SQLx with runtime queries (NOT compile-time checked macros)
- No DATABASE_URL required at build time
- Embedded migrations via `sqlx::migrate!("./migrations")`
- Migrations run automatically on startup

### Docker
- `docker compose up -d --build` starts API + Postgres
- Migrations run on app startup (no separate migration container)
- Seed creates admin key with `['*']` scopes (logged to stdout once)
- API exposed on host port 8082, Postgres on 5439

### Seed API Key
On first boot, check container logs for the plaintext key:
```bash
docker compose logs api | grep "Plaintext key"
```

### Testing
- Integration tests use reqwest against a real server + Postgres
- Tests must run sequentially (`--test-threads=1`) due to shared DB
- Default test DATABASE_URL: `postgres://postgres:postgres@localhost:5439/boilerworks`
