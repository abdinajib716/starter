//! Personal API tokens (§19, M15.4/8). Own tokens: session + `api_keys.create`; all tokens:
//! `api_keys.view_all` / `api_keys.revoke_all`.
use dioxus::prelude::*;
use uuid::Uuid;

use crate::domain::api::{NewToken, TokenRow};

#[post("/api/tokens/create")]
pub async fn create_api_token(
    name: String,
    scopes: Vec<String>,
    days: Option<i64>,
) -> Result<NewToken, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("api_keys.create")?;
    Ok(crate::server::api_v1::create_token(
        &crate::server::state::get()?.pool,
        &ctx,
        name,
        scopes,
        days,
    )
    .await?)
}

#[get("/api/tokens?all")]
pub async fn list_api_tokens(all: bool) -> Result<Vec<TokenRow>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::api_v1::list_tokens(&crate::server::state::get()?.pool, &ctx, all).await?)
}

#[post("/api/tokens/revoke")]
pub async fn revoke_api_token(id: Uuid) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::api_v1::revoke_token(&crate::server::state::get()?.pool, &ctx, id).await?)
}
