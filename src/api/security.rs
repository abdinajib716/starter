//! Sessions + two-factor server functions (§21, §22). Own account: session only.
use dioxus::prelude::*;
use uuid::Uuid;

use crate::domain::security::{EnrollStart, SessionRow, TwoFaStatus};

#[get("/api/profile/sessions")]
pub async fn my_sessions() -> Result<Vec<SessionRow>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(
        crate::server::services::security::my_sessions(&crate::server::state::get()?.pool, &ctx)
            .await?,
    )
}

/// Sign out one other device, or (with `None`) all other devices.
#[post("/api/profile/sessions/revoke")]
pub async fn revoke_my_session(id: Option<Uuid>) -> Result<u64, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(
        crate::server::services::security::revoke_mine(
            &crate::server::state::get()?.pool,
            &ctx,
            id,
        )
        .await?,
    )
}

#[get("/api/users/:id/sessions")]
pub async fn user_sessions(id: Uuid) -> Result<Vec<SessionRow>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("users.view")?;
    Ok(crate::server::services::security::user_sessions(
        &crate::server::state::get()?.pool,
        &ctx,
        id,
    )
    .await?)
}

#[post("/api/users/:id/sessions/revoke")]
pub async fn revoke_user_sessions(id: Uuid) -> Result<u64, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("users.revoke_sessions")?;
    Ok(crate::server::services::security::revoke_user_sessions(
        &crate::server::state::get()?.pool,
        &ctx,
        id,
    )
    .await?)
}

#[post("/api/users/:id/reset-2fa")]
pub async fn reset_user_2fa(id: Uuid, password: String) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("users.reset_2fa")?;
    Ok(crate::server::services::security::admin_reset(
        &crate::server::state::get()?.pool,
        &ctx,
        id,
        password,
    )
    .await?)
}

#[get("/api/profile/2fa")]
pub async fn twofa_status() -> Result<TwoFaStatus, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::security::status(&crate::server::state::get()?.pool, &ctx).await?)
}

#[post("/api/profile/2fa/begin")]
pub async fn twofa_begin() -> Result<EnrollStart, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(
        crate::server::services::security::begin_enroll(&crate::server::state::get()?.pool, &ctx)
            .await?,
    )
}

/// Returns the 10 recovery codes (shown once).
#[post("/api/profile/2fa/confirm")]
pub async fn twofa_confirm(code: String) -> Result<Vec<String>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::security::confirm_enroll(
        &crate::server::state::get()?.pool,
        &ctx,
        &code,
    )
    .await?)
}

#[post("/api/profile/2fa/disable")]
pub async fn twofa_disable(password: String) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::security::disable(
        &crate::server::state::get()?.pool,
        &ctx,
        password,
    )
    .await?)
}

#[post("/api/profile/2fa/recovery")]
pub async fn twofa_regenerate(code: String) -> Result<Vec<String>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::security::regenerate_codes(
        &crate::server::state::get()?.pool,
        &ctx,
        &code,
    )
    .await?)
}
