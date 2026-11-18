# syntax=docker/dockerfile:1.7
# Production image (§29, M17.1): build with dx, run on debian-slim with pg_dump/pg_restore 17.
#   docker build -t starter .
#   docker run --env-file .env -p 8080:8080 starter
ARG RUST_VERSION=1.100
ARG DX_VERSION=0.7.10

FROM rust:${RUST_VERSION}-bookworm AS build
ARG DX_VERSION
RUN rustup target add wasm32-unknown-unknown \
 && cargo install dioxus-cli --version ${DX_VERSION} --locked
WORKDIR /src
COPY . .
# Queries were checked at commit time (`.sqlx/`); no database needed to build.
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    dx bundle --release --platform web --out-dir /out \
 && cp -r /out /bundle

FROM debian:bookworm-slim AS runtime
# PostgreSQL 17 client tools (backups/restore must match the server's major version, R9).
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl gnupg \
 && install -d /usr/share/postgresql-common/pgdg \
 && curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc \
 && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
 && apt-get update && apt-get install -y --no-install-recommends postgresql-client-17 \
 && apt-get purge -y curl gnupg && apt-get autoremove -y && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home /app app \
 && mkdir -p /app/data/backups /app/data/files && chown -R app /app
WORKDIR /app
COPY --from=build --chown=app /bundle/ /app/
USER app
ENV IP=0.0.0.0 PORT=8080 APP_ENV=production COOKIE_SECURE=true \
    BACKUP_DIR=/app/data/backups FILES_DIR=/app/data/files
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=3 CMD ["/app/server", "health"]
ENTRYPOINT ["/app/server"]
CMD ["serve"]
