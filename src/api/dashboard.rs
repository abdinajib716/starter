use dioxus::prelude::*;

use crate::domain::dashboard::DashboardData;

/// `get_dashboard()` (§10): one round trip, `dashboard.view`.
#[get("/api/dashboard")]
pub async fn get_dashboard() -> Result<DashboardData, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("dashboard.view")?;
    let pool = &crate::server::state::get()?.pool;
    Ok(crate::server::services::dashboard::get(pool, &ctx).await?)
}
