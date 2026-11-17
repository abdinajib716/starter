//! System Setting server functions (§9.6, §27).
use dioxus::prelude::*;

use crate::domain::settings::{FlagItem, FlagUpdate, SettingsView};

#[cfg(feature = "server")]
async fn ctx() -> Result<(crate::server::ctx::Ctx, &'static sqlx::PgPool), ServerFnError> {
    Ok((
        crate::server::ctx::Ctx::current().await?,
        &crate::server::state::get()?.pool,
    ))
}

#[get("/api/settings")]
pub async fn list_settings() -> Result<SettingsView, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("settings.view")?;
    Ok(crate::server::services::system_settings::view(pool, &ctx).await?)
}

/// `changes`: (key, value as JSON text). Returns how many values changed.
#[post("/api/settings/update")]
pub async fn update_settings(changes: Vec<(String, String)>) -> Result<usize, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("settings.update")?;
    Ok(crate::server::services::system_settings::update(pool, &ctx, changes).await?)
}

#[get("/api/flags")]
pub async fn list_flags() -> Result<Vec<FlagItem>, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("flags.view")?;
    Ok(crate::server::services::system_settings::flags(pool, &ctx).await?)
}

#[post("/api/flags/update")]
pub async fn update_flag(flag: FlagUpdate) -> Result<(), ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("flags.update")?;
    Ok(crate::server::services::system_settings::update_flag(pool, &ctx, flag).await?)
}
