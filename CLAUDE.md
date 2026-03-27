# Claude -- Boilerworks Rust Micro

Primary conventions doc: [`bootstrap.md`](bootstrap.md)

Read it before writing any code.

## Stack

- **Backend**: Rust (Axum 0.8 + tokio)
- **Frontend**: None (API-only microservice)
- **API**: REST with JSON responses
- **Queries**: SQLx (runtime, not compile-time checked)
- **Migrations**: SQLx embedded migrations
- **Auth**: API-key (SHA256 hashed, per-key scopes)

## Quick Reference

| Endpoint | URL |
|----------|-----|
| Health | http://localhost:8082/health |
| Events | http://localhost:8082/events |
| API Keys | http://localhost:8082/api-keys |

## Commands

```bash
make up        # Start Docker services
make down      # Stop services
make build     # Build release binary
make test      # Run tests (sequential, needs Postgres)
make lint      # Clippy + fmt check
make logs      # Tail container logs
```

## Structure

```
src/
  main.rs           -- entry point, config, seed logic
  lib.rs            -- public re-exports for tests
  config.rs         -- Config struct from env
  db.rs             -- PgPool creation + embedded migrations
  models.rs         -- ApiKey, Event, request/response structs
  auth.rs           -- ApiKeyAuth extractor, require_scope()
  response.rs       -- ApiResponse<T> wrapper
  routes.rs         -- Router with all routes + middleware
  handlers/
    health.rs       -- GET /health
    events.rs       -- POST/GET/DELETE /events
    api_keys.rs     -- POST/GET/DELETE /api-keys
migrations/
  001_init.sql      -- api_keys + events tables
tests/
  integration.rs    -- reqwest-based integration tests (17 tests)
```

## Rules

- API-key auth on all endpoints except /health
- UUID primary keys, never expose internal IDs
- Soft deletes on events (deleted_at field)
- Scopes: `events.read`, `events.write`, `keys.manage`, `*` (wildcard)
- All responses wrapped in `ApiResponse{ok, data, message, errors}`
- SQLx runtime queries only (no compile-time checking, no DATABASE_URL needed at build)
