//! Roles + permission catalog server functions (§9.3, §9.4).
use dioxus::prelude::*;
use uuid::Uuid;

use crate::domain::roles::{PermissionCatalog, RoleDetail, RoleInput, RoleSummary};
use crate::domain::users::BulkResult;

#[cfg(feature = "server")]
async fn ctx() -> Result<(crate::server::ctx::Ctx, &'static sqlx::PgPool), ServerFnError> {
    Ok((
        crate::server::ctx::Ctx::current().await?,
        &crate::server::state::get()?.pool,
    ))
}

#[get("/api/roles")]
pub async fn list_roles() -> Result<Vec<RoleSummary>, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("roles.view")?;
    Ok(crate::server::services::roles::list(pool, &ctx).await?)
}

#[post("/api/roles/get")]
pub async fn get_role(id: Uuid) -> Result<RoleDetail, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("roles.view")?;
    Ok(crate::server::services::roles::get(pool, &ctx, id).await?)
}

/// Catalog for the Permission page (`permissions.view`) and the role editor (`roles.view`);
/// the service accepts either.
#[get("/api/permissions")]
pub async fn list_permissions() -> Result<PermissionCatalog, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require_user()?;
    Ok(crate::server::services::roles::catalog(pool, &ctx).await?)
}

#[post("/api/roles/create")]
pub async fn create_role(input: RoleInput) -> Result<Uuid, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("roles.create")?;
    Ok(crate::server::services::roles::create(pool, &ctx, input).await?)
}

/// Returns how many sessions were ended (users holding the role sign in again).
#[post("/api/roles/update")]
pub async fn update_role(id: Uuid, input: RoleInput) -> Result<u64, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("roles.update")?;
    Ok(crate::server::services::roles::update(pool, &ctx, id, input).await?)
}

#[post("/api/roles/delete")]
pub async fn delete_roles(ids: Vec<Uuid>) -> Result<BulkResult, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("roles.delete")?;
    Ok(crate::server::services::roles::delete_many(pool, &ctx, ids).await?)
}
