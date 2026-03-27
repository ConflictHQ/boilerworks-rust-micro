# Build stage
FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
# Create dummy src for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

COPY . .
# Touch main.rs to ensure rebuild with real source
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/boilerworks-rust-micro /app/api
COPY migrations /app/migrations

EXPOSE 8080

CMD ["/app/api"]
