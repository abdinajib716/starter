# Starter

Rust fullstack starter: **Dioxus 0.7 fullstack** (SSR + hydration, Axum) · **SQLx** · **PostgreSQL 17** · **Rust/UI** + Tailwind v4.

- Design: [RUST FULLSTACK STARTER BLUEPRINT.md](RUST%20FULLSTACK%20STARTER%20BLUEPRINT.md)
- Progress: [TRACKER.md](TRACKER.md)

## Prerequisites

- Rust stable + `rustup target add wasm32-unknown-unknown`
- `dx` (`cargo install dioxus-cli`), `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres,rustls`), `just`
- Docker (for Postgres), `psql`

## First run

```sh
cp .env.example .env        # adjust if needed
just db-up                  # Postgres 17 on 127.0.0.1:5440
just hooks                  # pre-commit: fmt, clippy (server + wasm), .sqlx freshness
just dev                    # http://127.0.0.1:8080  (migrations run on boot)
```

Check: `curl http://127.0.0.1:8080/healthz` → `{"status":"ok","db":"ok","tenancy":"single",...}`.

## Everyday commands

| Command | What it does |
|---|---|
| `just dev` | `dx serve` with hot reload |
| `just migration <name>` | new timestamped migration in `migrations/` |
| `just migrate` | apply migrations to `$DATABASE_URL` |
| `just prepare` | refresh `.sqlx/` offline query data — **commit it** whenever a `query!` changes |
| `just lint` | `cargo fmt --check` + clippy for server and wasm (`-D warnings`) |
| `just test` | server tests (see below) |

## Tests and the test database

Tests that touch the database use `#[sqlx::test(migrations = "./migrations")]`. Each test gets a
**fresh, migrated, throwaway database**, so tests never see each other's data or the dev data.

`just test` points `DATABASE_URL` at a separate database, `starter_test` (created on first run), on
the same Postgres server; sqlx creates the per-test databases next to it. Override it with
`TEST_DATABASE_URL` in `.env`. Tests compile with `SQLX_OFFLINE=true`, so `.sqlx/` must be current
(`just prepare`).

## Layout (§31.2)

```
src/
  main.rs        entry: web → dioxus::launch, server → dioxus::serve(custom Axum router)
  app.rs         App root, Route enum, document head
  pages/         route components
  components/    app components (layout/ = shell, built in M3)
  api/           #[get]/#[post] server functions — thin
  domain/        shared DTOs (server + wasm)
  server/        feature "server" only: config, state, db, ctx, routes, seed
  ui/ hooks/     Rust/UI, vendored verbatim (see src/ui/README.md) — don't edit or rustfmt
migrations/      SQLx, timestamp-named
.sqlx/           SQLx offline data (committed)
```

Server-only crates (`sqlx`, `figment`, `tower-http`, …) are `optional` and enabled only by the
`server` feature, so they never reach the wasm bundle.

## Licenses

Includes Rust/UI (MIT) and ApexCharts (MIT) — see [NOTICE](NOTICE).
