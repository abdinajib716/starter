use dioxus::prelude::*;

use crate::domain::ServerInfo;

/// Public status. On the allow-list of unauthenticated endpoints (M2.12).
#[get("/api/system/info")]
pub async fn server_info() -> Result<ServerInfo, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    let state = crate::server::state::get()?;
    Ok(state.server_info(&ctx).await)
}
