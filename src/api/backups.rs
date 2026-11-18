//! Backup server functions (§9.6.1). Download and CSV are plain routes (streamed files).
use dioxus::prelude::*;
use uuid::Uuid;

use crate::domain::backups::BackupRow;

#[cfg(feature = "server")]
async fn ctx() -> Result<(crate::server::ctx::Ctx, &'static sqlx::PgPool), ServerFnError> {
    Ok((
        crate::server::ctx::Ctx::current().await?,
        &crate::server::state::get()?.pool,
    ))
}

#[get("/api/backups")]
pub async fn list_backups() -> Result<Vec<BackupRow>, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("backup.view")?;
    Ok(crate::server::services::backups::list(pool, &ctx).await?)
}

#[post("/api/backups/create")]
pub async fn create_backup() -> Result<Uuid, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("backup.create")?;
    Ok(crate::server::services::backups::create(pool, &ctx).await?)
}

#[post("/api/backups/delete")]
pub async fn delete_backup(id: Uuid) -> Result<(), ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("backup.delete")?;
    Ok(crate::server::services::backups::delete(pool, &ctx, id).await?)
}

/// Restore (M12.7): typed file name + own password; runs in the background under maintenance.
#[post("/api/backups/restore")]
pub async fn restore_backup(
    req: crate::domain::backups::RestoreRequest,
) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("backup.restore")?;
    Ok(
        crate::server::services::backups::restore(&crate::server::state::get()?.pool, &ctx, req)
            .await?,
    )
}

/// "Test off-site connection" (M12.6).
#[post("/api/backups/offsite-test")]
pub async fn test_offsite() -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("settings.update")?;
    Ok(
        crate::server::services::backups::test_offsite(&crate::server::state::get()?.pool, &ctx)
            .await?,
    )
}
