use dioxus::prelude::*;

use crate::domain::audit::{AuditPage, AuditQuery};

/// `list_audit(filters; cursor)` (§9.5), `audit.view`.
#[post("/api/audit/list")]
pub async fn list_audit(query: AuditQuery) -> Result<AuditPage, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("audit.view")?;
    let pool = &crate::server::state::get()?.pool;
    Ok(crate::server::services::audit_log::list(pool, &ctx, &query).await?)
}
