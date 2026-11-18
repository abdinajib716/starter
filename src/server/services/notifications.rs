//! Notifications (§23, M8.8): bell, list, read state, preferences, announcements, retention.
use std::collections::HashMap;

use serde_json::{Value, json};
use sqlx::PgPool;

use super::auth::user_entry;
use crate::domain::notifications::{NotificationPage, NotificationPref, NotificationRow};
use crate::server::audit::{self, Entry};
use crate::server::ctx::Ctx;
use crate::server::error::AppError;
use crate::server::modules::Modules;
use crate::server::notify::{self, KINDS, Target};

fn enabled() -> Result<(), AppError> {
    if Modules::get().notifications() {
        Ok(())
    } else {
        Err(AppError::Disabled("notifications.disabled"))
    }
}

fn params(v: Value) -> HashMap<String, String> {
    v.as_object()
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| v.to_string()),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

pub async fn list(
    pool: &PgPool,
    ctx: &Ctx,
    unread_only: bool,
    limit: i64,
    offset: i64,
) -> Result<NotificationPage, AppError> {
    enabled()?;
    let user = ctx.require_user().map_err(|_| AppError::Unauthenticated)?;
    let rows = sqlx::query!(
        r#"SELECT id, kind, params, link, read_at IS NOT NULL AS "read!", created_at
           FROM notifications WHERE user_id = $1 AND (NOT $2 OR read_at IS NULL)
           ORDER BY created_at DESC, id DESC LIMIT $3 OFFSET $4"#,
        user.id,
        unread_only,
        limit.clamp(1, 100),
        offset.max(0)
    )
    .fetch_all(pool)
    .await?;
    let c = sqlx::query!(
        r#"SELECT count(*) FILTER (WHERE read_at IS NULL) AS "unread!", count(*) AS "total!"
           FROM notifications WHERE user_id = $1"#,
        user.id
    )
    .fetch_one(pool)
    .await?;
    Ok(NotificationPage {
        rows: rows
            .into_iter()
            .map(|r| NotificationRow {
                id: r.id,
                kind: r.kind,
                params: params(r.params),
                link: r.link,
                read: r.read,
                created_at: r.created_at,
            })
            .collect(),
        unread: c.unread,
        total: c.total,
    })
}

/// Mark some (or, with `None`, all) of my notifications read.
pub async fn mark_read(pool: &PgPool, ctx: &Ctx, ids: Option<Vec<i64>>) -> Result<(), AppError> {
    let user = ctx.require_user().map_err(|_| AppError::Unauthenticated)?;
    sqlx::query!(
        "UPDATE notifications SET read_at = now()
         WHERE user_id = $1 AND read_at IS NULL AND ($2::bigint[] IS NULL OR id = ANY($2))",
        user.id,
        ids.as_deref()
    )
    .execute(pool)
    .await?;
    sqlx::query!("SELECT pg_notify('notify', $1)", user.id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn prefs(pool: &PgPool, ctx: &Ctx) -> Result<Vec<NotificationPref>, AppError> {
    let user = ctx.require_user().map_err(|_| AppError::Unauthenticated)?;
    let stored = sqlx::query!(
        "SELECT kind, in_app, email FROM notification_prefs WHERE user_id = $1",
        user.id
    )
    .fetch_all(pool)
    .await?;
    Ok(KINDS
        .iter()
        .map(|k| {
            let s = stored.iter().find(|s| s.kind == *k);
            NotificationPref {
                kind: (*k).to_string(),
                in_app: s.is_none_or(|s| s.in_app),
                email: s.is_none_or(|s| s.email),
            }
        })
        .collect())
}

pub async fn set_pref(pool: &PgPool, ctx: &Ctx, p: NotificationPref) -> Result<(), AppError> {
    let user = ctx.require_user().map_err(|_| AppError::Unauthenticated)?;
    if !KINDS.contains(&p.kind.as_str()) {
        return Err(AppError::Validation("notifications.unknown_kind"));
    }
    sqlx::query!(
        "INSERT INTO notification_prefs (user_id, kind, in_app, email) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, kind) DO UPDATE SET in_app = $3, email = $4",
        user.id,
        p.kind,
        p.in_app,
        p.email
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Announcement to everyone (`notifications.broadcast`).
pub async fn announce(
    pool: &PgPool,
    ctx: &Ctx,
    title: String,
    body: String,
) -> Result<u64, AppError> {
    enabled()?;
    let actor = ctx.require("notifications.broadcast")?;
    let (title, body) = (title.trim().to_string(), body.trim().to_string());
    if title.is_empty() || title.chars().count() > 120 || body.chars().count() > 1000 {
        return Err(AppError::Validation("notifications.invalid_announcement"));
    }
    let mut tx = pool.begin().await?;
    let n = notify::send(
        &mut tx,
        ctx.org_id,
        Target::All,
        "announcement",
        json!({ "title": title, "body": body, "from": actor.full_name }),
        None,
    )
    .await?;
    audit::record(
        &mut tx,
        Entry {
            entity_type: Some("notification"),
            entity_id: None,
            after: Some(json!({ "title": title, "recipients": n })),
            ..user_entry(ctx, actor, "notification.broadcast")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(n)
}

/// Drop notifications older than `notifications.retention_days` (M8.10).
pub async fn retention(pool: &PgPool) -> sqlx::Result<u64> {
    let days = crate::server::settings::cache()
        .map(|c| c.get("notifications.retention_days"))
        .and_then(|v| v.as_i64())
        .unwrap_or(90);
    Ok(sqlx::query!(
        "DELETE FROM notifications WHERE created_at < now() - make_interval(days => $1)",
        i32::try_from(days).unwrap_or(90)
    )
    .execute(pool)
    .await?
    .rows_affected())
}
