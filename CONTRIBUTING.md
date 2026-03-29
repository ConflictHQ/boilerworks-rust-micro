# Contributing to Boilerworks Rust Micro

Thank you for your interest in contributing!

## Getting Started

```bash
# Start Postgres
docker compose up -d postgres

# Build
cargo build

# Run tests
DATABASE_URL=postgres://postgres:postgres@localhost:5432/boilerworks cargo test -- --test-threads=1

# Lint
cargo clippy -- -D warnings
cargo fmt -- --check
```

## Development Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Run `make lint` and `make test`
5. Submit a pull request

## Code Style

- `cargo fmt` for formatting (edition 2021)
- `cargo clippy -- -D warnings` must pass
- SQLx runtime queries only (no compile-time checked macros)
- All responses use `ApiResponse<T>` wrapper

## Questions?

Open an issue in this repository.
