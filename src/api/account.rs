//! Email-based account flows (§20). Public: forgot / reset / accept invite / verify.
use dioxus::prelude::*;
use uuid::Uuid;

use crate::domain::account::EmailLogPage;
use crate::domain::users::UserInput;

#[post("/api/auth/forgot-password")]
pub async fn forgot_password(email: String) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    Ok(crate::server::services::account::forgot_password(
        &crate::server::state::get()?.pool,
        &ctx,
        &email,
    )
    .await?)
}

/// `purpose`: `password_reset` or `invite`.
#[post("/api/auth/set-password")]
pub async fn set_password_with_token(
    token: String,
    purpose: String,
    password: String,
) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    Ok(crate::server::services::account::set_password_with_token(
        &crate::server::state::get()?.pool,
        &ctx,
        &token,
        &purpose,
        password,
    )
    .await?)
}

#[post("/api/auth/verify-email")]
pub async fn verify_email(token: String) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    Ok(crate::server::services::account::verify_email(
        &crate::server::state::get()?.pool,
        &ctx,
        &token,
    )
    .await?)
}

#[post("/api/profile/verify-email")]
pub async fn send_verification() -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::account::send_verification(
        &crate::server::state::get()?.pool,
        &ctx,
    )
    .await?)
}

#[post("/api/users/invite")]
pub async fn invite_user(input: UserInput) -> Result<Uuid, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("users.create")?;
    Ok(
        crate::server::services::account::invite(&crate::server::state::get()?.pool, &ctx, input)
            .await?,
    )
}

#[post("/api/email/test")]
pub async fn send_test_email(to: String) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("email.send_test")?;
    Ok(
        crate::server::services::account::send_test(&crate::server::state::get()?.pool, &ctx, to)
            .await?,
    )
}

#[get("/api/email/log?offset")]
pub async fn email_log(offset: i64) -> Result<EmailLogPage, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("email.view_log")?;
    Ok(
        crate::server::services::account::log(&crate::server::state::get()?.pool, &ctx, offset)
            .await?,
    )
}
