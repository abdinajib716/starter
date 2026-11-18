//! Process-wide state, initialised once (hot-patching re-runs the router closure, not this).
use std::sync::Arc;

use dioxus::prelude::ServerFnError;
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use super::{config::Config, ctx::Ctx, db, seed};
use crate::domain::ServerInfo;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    /// Cached so the per-request `Ctx` layer needs no DB round trip in single mode.
    pub default_org_id: Uuid,
}

static STATE: OnceCell<AppState> = OnceCell::const_new();

pub async fn init() -> anyhow::Result<&'static AppState> {
    STATE
        .get_or_try_init(|| async {
            let config = Config::load()?;
            let pool = db::connect(&config).await?;
            db::migrate(&pool).await?;
            let default_org_id = seed::ensure_default_org(&pool).await?;
            super::rbac::sync_permissions(&pool).await?;
            super::settings::seed(&pool).await?;
            super::flags::sync(&pool).await?;
            super::flags::reload(&pool).await?;
            super::settings::start(&pool).await?;
            seed::ensure_super_admin_role(&pool).await?;
            seed::ensure_first_admin(&pool, &config, default_org_id).await?;
            super::services::backups::start_scheduler(pool.clone());
            super::services::backups::start_offsite_worker(pool.clone());
            super::email::start_worker(pool.clone());
            super::realtime::start(pool.clone());
            super::services::security::start_jobs(pool.clone());
            super::system_state::start(pool.clone()).await?;
            check_instances(&pool).await?;
            tracing::info!(tenancy = %config.tenancy_mode, "app state ready");
            Ok(AppState {
                pool,
                config: Arc::new(config),
                default_org_id,
            })
        })
        .await
}

/// For server fns. Always initialised before the router serves requests.
pub fn get() -> Result<&'static AppState, ServerFnError> {
    STATE
        .get()
        .ok_or_else(|| ServerFnError::new("server not initialised"))
}

impl AppState {
    pub async fn db_ok(&self) -> bool {
        sqlx::query_scalar!(r#"SELECT 1 AS "one!""#)
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    pub async fn server_info(&self, _ctx: &Ctx) -> ServerInfo {
        ServerInfo {
            version: env!("CARGO_PKG_VERSION").into(),
            tenancy: self.config.tenancy_mode,
            db: if self.db_ok().await { "ok" } else { "down" }.into(),
            password_reset: super::modules::Modules::get().password_reset(),
            app_name: super::modules::Modules::get().app_name(),
        }
    }
}

/// M14.5: several app instances can't share files on a local disk.
async fn check_instances(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let instances: u32 = std::env::var("APP_INSTANCES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let driver = sqlx::query_scalar!(
        r#"SELECT value #>> '{}' AS "v" FROM settings WHERE key = 'storage.primary_driver'"#
    )
    .fetch_optional(pool)
    .await?
    .flatten()
    .unwrap_or_else(|| "local".into());
    if instances > 1 && driver == "local" {
        anyhow::bail!(
            "APP_INSTANCES={instances} but storage.primary_driver = local: use R2/S3 storage for more than one instance"
        );
    }
    Ok(())
}
