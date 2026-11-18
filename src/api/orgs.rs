//! Organizations server functions (multi-tenant mode, §18).
use dioxus::prelude::*;
use uuid::Uuid;

use crate::domain::orgs::{NewOrg, OrgRow};

#[get("/api/organizations")]
pub async fn list_orgs() -> Result<Vec<OrgRow>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("organizations.view")?;
    Ok(crate::server::services::orgs::list(&crate::server::state::get()?.pool, &ctx).await?)
}

#[post("/api/organizations/create")]
pub async fn create_org(org: NewOrg) -> Result<Uuid, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("organizations.manage")?;
    Ok(
        crate::server::services::orgs::create(&crate::server::state::get()?.pool, &ctx, org)
            .await?,
    )
}

#[post("/api/organizations/update")]
pub async fn update_org(
    id: Uuid,
    name: String,
    active: bool,
    max_users: Option<i32>,
) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("organizations.manage")?;
    Ok(crate::server::services::orgs::update(
        &crate::server::state::get()?.pool,
        &ctx,
        id,
        name,
        active,
        max_users,
    )
    .await?)
}
