# Starter task runner — `just --list`
set dotenv-load := true

# Run the app with hot reload (http://127.0.0.1:8080)
dev:
    dx serve --platform web --addr 127.0.0.1 --port 8080

# Start the dev database (docker-compose.yml) and wait until healthy
db-up:
    docker compose up -d --wait db

# Stop the dev database (data is kept in the volume)
db-down:
    docker compose stop db

# Apply migrations to $DATABASE_URL (the app also migrates on boot)
migrate:
    sqlx migrate run

# New timestamped migration: just migration add_widgets
migration name:
    sqlx migrate add {{name}}

# Refresh .sqlx/ offline query data (commit the result)
prepare:
    cargo sqlx prepare -- --no-default-features --features server

# fmt check + clippy for both the server and the wasm build
lint:
    cargo fmt --all --check
    cargo clippy --no-default-features --features server -- -D warnings
    cargo clippy --target wasm32-unknown-unknown -- -D warnings

# Server tests; #[sqlx::test] creates a fresh database per test next to starter_test
test:
    #!/usr/bin/env bash
    set -euo pipefail
    test_url="${TEST_DATABASE_URL:-${DATABASE_URL%/*}/starter_test}"
    psql "$DATABASE_URL" -Atc "SELECT 1 FROM pg_database WHERE datname = 'starter_test'" | grep -q 1 \
      || psql "$DATABASE_URL" -qc "CREATE DATABASE starter_test"
    SQLX_OFFLINE=true DATABASE_URL="$test_url" cargo test --no-default-features --features server

# Install the git pre-commit hook
hooks:
    git config core.hooksPath .githooks
