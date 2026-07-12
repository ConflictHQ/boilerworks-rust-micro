# Calliope — Boilerworks Rust Micro
<!-- Agent shim for https://github.com/calliopeai/calliope-cli -->

Primary conventions doc: [`bootstrap.md`](bootstrap.md)

Read it before writing any code.

---

## Project-specific notes

- Stack: Rust (Axum 0.8 + tokio), SQLx runtime queries (no compile-time checking, no `DATABASE_URL` at build), embedded migrations, PostgreSQL 16. API-only — no frontend, no sessions.
- API-key auth (SHA256 hashed, per-key scopes) on all endpoints except `/health`; scopes are `events.read`, `events.write`, `keys.manage`, `*`.
- UUID primary keys, never expose internal IDs; soft deletes on events (`deleted_at`).
- All responses wrapped in `ApiResponse{ok, data, message, errors}`.
- Router + middleware in `src/routes.rs`, handlers in `src/handlers/`; `make up` / `make test` (sequential, needs Postgres) / `make lint` (Clippy + fmt).
