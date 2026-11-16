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
            super::settings::start(&pool).await?;
            seed::ensure_super_admin_role(&pool).await?;
            seed::ensure_first_admin(&pool, &config, default_org_id).await?;
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
        }
    }
}
