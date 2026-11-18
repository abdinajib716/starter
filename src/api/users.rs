//! Users server functions (§9.2). Thin: context → permission check (in the service) → result.
use dioxus::prelude::*;
use uuid::Uuid;

use crate::domain::users::{
    BulkResult, BulkUserAction, RoleOption, UserDetail, UserInput, UserPage, UserQuery,
};

#[cfg(feature = "server")]
async fn ctx() -> Result<(crate::server::ctx::Ctx, &'static sqlx::PgPool), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    Ok((ctx, &crate::server::state::get()?.pool))
}

#[post("/api/users/list")]
pub async fn list_users(query: UserQuery) -> Result<UserPage, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("users.view")?;
    Ok(crate::server::services::users::list(pool, &ctx, &query).await?)
}

#[get("/api/users/roles")]
pub async fn user_role_options() -> Result<Vec<RoleOption>, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("users.view")?;
    Ok(crate::server::services::users::role_options(pool, &ctx).await?)
}

#[post("/api/users/create")]
pub async fn create_user(input: UserInput) -> Result<Uuid, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("users.create")?;
    Ok(crate::server::services::users::create(pool, &ctx, input).await?)
}

#[post("/api/users/update")]
pub async fn update_user(id: Uuid, input: UserInput) -> Result<(), ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("users.update")?;
    Ok(crate::server::services::users::update(pool, &ctx, id, input).await?)
}

/// One action on one user (row menu). Permission depends on the action (checked in the service).
#[post("/api/users/apply")]
pub async fn apply_user_action(action: BulkUserAction, id: Uuid) -> Result<(), ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require_user()?;
    Ok(crate::server::services::users::apply(pool, &ctx, action, id).await?)
}

#[post("/api/users/bulk")]
pub async fn bulk_users(
    action: BulkUserAction,
    ids: Vec<Uuid>,
) -> Result<BulkResult, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require_user()?;
    Ok(crate::server::services::users::bulk(pool, &ctx, action, ids).await?)
}

#[post("/api/users/reset-password")]
pub async fn reset_user_password(id: Uuid, password: String) -> Result<(), ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("users.reset_password")?;
    Ok(crate::server::services::users::reset_password(pool, &ctx, id, password).await?)
}

#[post("/api/users/purge")]
pub async fn purge_user(id: Uuid, confirm_email: String) -> Result<(), ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("users.purge")?;
    Ok(crate::server::services::users::purge(pool, &ctx, id, &confirm_email).await?)
}

#[post("/api/users/detail")]
pub async fn get_user(id: Uuid) -> Result<UserDetail, ServerFnError> {
    let (ctx, pool) = ctx().await?;
    ctx.require("users.view")?;
    Ok(crate::server::services::users::detail(pool, &ctx, id).await?)
}

/// Bulk action on every user the filters match (all pages, M5.6 `Selection::Filter`).
#[post("/api/users/bulk-matching")]
pub async fn bulk_users_matching(
    action: BulkUserAction,
    query: UserQuery,
) -> Result<BulkResult, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("users.view")?;
    let st = crate::server::state::get()?;
    Ok(crate::server::services::users::bulk_matching(&st.pool, &ctx, action, &query).await?)
}

/// How many users the filters match (for the "select all N matching" banner).
#[post("/api/users/matching-count")]
pub async fn count_users_matching(query: UserQuery) -> Result<u64, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("users.view")?;
    let st = crate::server::state::get()?;
    Ok(
        crate::server::services::users::matching_ids(&st.pool, &ctx, &query)
            .await?
            .len() as u64,
    )
}
