//! Notification server functions (§23). Own notifications: session only.
use dioxus::prelude::*;

use crate::domain::notifications::{NotificationPage, NotificationPref};

#[get("/api/notifications?unread_only&limit&offset")]
pub async fn list_notifications(
    unread_only: bool,
    limit: i64,
    offset: i64,
) -> Result<NotificationPage, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::notifications::list(
        &crate::server::state::get()?.pool,
        &ctx,
        unread_only,
        limit,
        offset,
    )
    .await?)
}

/// `None` = all.
#[post("/api/notifications/read")]
pub async fn mark_read(ids: Option<Vec<i64>>) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::notifications::mark_read(
        &crate::server::state::get()?.pool,
        &ctx,
        ids,
    )
    .await?)
}

#[get("/api/notifications/prefs")]
pub async fn notification_prefs() -> Result<Vec<NotificationPref>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(
        crate::server::services::notifications::prefs(&crate::server::state::get()?.pool, &ctx)
            .await?,
    )
}

#[post("/api/notifications/prefs")]
pub async fn set_notification_pref(pref: NotificationPref) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require_user()?;
    Ok(crate::server::services::notifications::set_pref(
        &crate::server::state::get()?.pool,
        &ctx,
        pref,
    )
    .await?)
}

#[post("/api/notifications/announce")]
pub async fn send_announcement(title: String, body: String) -> Result<u64, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    ctx.require("notifications.broadcast")?;
    Ok(crate::server::services::notifications::announce(
        &crate::server::state::get()?.pool,
        &ctx,
        title,
        body,
    )
    .await?)
}
