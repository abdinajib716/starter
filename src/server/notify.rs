//! In-app notifications (§23, M8.7): written inside the business transaction, then pushed to
//! the user's open tabs through `pg_notify('notify', user_id)` → SSE (`server::realtime`).
use std::net::IpAddr;

use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Who receives a notification.
pub enum Target<'a> {
    User(Uuid),
    /// Every active user holding this permission (Super Admins included).
    Permission(&'a str),
    /// Every active user of the organization.
    All,
}

/// Kinds a user may switch off in Profile → Notifications.
pub const KINDS: &[&str] = &[
    "new_device",
    "account_locked",
    "password_changed",
    "twofa_changed",
    "backup_failed",
    "announcement",
];

pub async fn send(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    target: Target<'_>,
    kind: &str,
    params: Value,
    link: Option<&str>,
) -> sqlx::Result<u64> {
    let (user, perm, all) = match target {
        Target::User(u) => (Some(u), None, false),
        Target::Permission(p) => (None, Some(p.to_string()), false),
        Target::All => (None, None, true),
    };
    let ids = sqlx::query_scalar!(
        r#"INSERT INTO notifications (org_id, user_id, kind, params, link)
           SELECT $1, u.id, $5, $6, $7
           FROM users u JOIN roles r ON r.id = u.role_id
           WHERE u.org_id = $1 AND u.deleted_at IS NULL AND u.is_active
             AND ( ($2::uuid IS NOT NULL AND u.id = $2)
                OR ($3::text IS NOT NULL AND (
                      (r.is_system AND r.name = 'Super Admin')
                      OR EXISTS (SELECT 1 FROM role_permissions rp
                                 WHERE rp.role_id = r.id AND rp.permission_key = $3)))
                OR $4 )
             AND NOT EXISTS (SELECT 1 FROM notification_prefs p
                             WHERE p.user_id = u.id AND p.kind = $5 AND NOT p.in_app)
           RETURNING user_id"#,
        org_id,
        user,
        perm,
        all,
        kind,
        params,
        link,
    )
    .fetch_all(&mut **tx)
    .await?;
    for id in &ids {
        sqlx::query!("SELECT pg_notify('notify', $1)", id.to_string())
            .execute(&mut **tx)
            .await?;
    }
    Ok(ids.len() as u64)
}

pub async fn new_device(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Uuid,
    user: Uuid,
    device: &str,
    ip: Option<IpAddr>,
) -> sqlx::Result<()> {
    let params = json!({ "device": device, "ip": ip.map(|i| i.to_string()).unwrap_or_else(|| "none".into()) });
    send(
        tx,
        org_id,
        Target::User(user),
        "new_device",
        params.clone(),
        Some("/profile?tab=security"),
    )
    .await?;
    crate::server::email::queue_for_user(tx, user, "new_login_alert", params).await?;
    Ok(())
}
