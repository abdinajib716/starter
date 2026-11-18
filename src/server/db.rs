//! Postgres pool + embedded migrations (run on boot in M0; moves to `starter migrate` in M14.7).
use std::time::Duration;

use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

use super::config::Config;

tokio::task_local! {
    /// The organization of the request being served (multi mode, §18). Set by `ctx_layer`.
    pub static ORG: uuid::Uuid;
}

pub async fn connect(cfg: &Config) -> anyhow::Result<PgPool> {
    let multi = cfg.tenancy_mode == crate::domain::TenancyMode::Multi;
    PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        // Multi mode: every connection handed to a request carries its organization, so the
        // RLS policies (`app_org()`) hide other tenants' rows even if a query forgets `org_id`.
        .before_acquire(move |conn, _meta| {
            Box::pin(async move {
                if multi {
                    let org = ORG.try_with(|o| o.to_string()).unwrap_or_default();
                    sqlx::query("SELECT set_config('app.org_id', $1, false)")
                        .bind(org)
                        .execute(&mut *conn)
                        .await?;
                }
                Ok(true)
            })
        })
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
