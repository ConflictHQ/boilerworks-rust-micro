.PHONY: up down build test lint migrate seed logs

up:
	docker compose up -d --build

down:
	docker compose down

build:
	cargo build --release

test:
	cargo test -- --test-threads=1

lint:
	cargo clippy -- -D warnings
	cargo fmt -- --check

migrate:
	cargo run &
	sleep 3
	kill %1 2>/dev/null || true

seed:
	@echo "Set API_KEY_SEED env var and restart the API service"
	@echo "docker compose up -d api"

logs:
	docker compose logs -f
