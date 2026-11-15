//! Postgres pool + embedded migrations (run on boot in M0; moves to `starter migrate` in M14.7).
use std::time::Duration;

use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

use super::config::Config;

pub async fn connect(cfg: &Config) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await
        .context("cannot connect to Postgres (is `docker compose up -d db` running?)")
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("database migration failed")
}
