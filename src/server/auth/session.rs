//! DB-backed sessions (§7): create, load (with sliding expiry), delete.
use std::collections::HashSet;
use std::net::IpAddr;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::token;
use crate::server::ctx::CurrentUser;

/// Insert a session in `tx` and return the raw token for the cookie (only its hash is stored).
pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    ip: Option<IpAddr>,
    user_agent: Option<&str>,
    hours: i64,
) -> sqlx::Result<String> {
    let raw = token::generate();
    sqlx::query!(
        r#"INSERT INTO sessions (token_hash, user_id, ip, user_agent, expires_at)
           VALUES ($1, $2, $3, $4, now() + make_interval(hours => $5))"#,
        token::hash(&raw),
        user_id,
        ip.map(ipnet::IpNet::from),
        user_agent,
        i32::try_from(hours).unwrap_or(12),
    )
    .execute(&mut **tx)
    .await?;
    Ok(raw)
}

/// Resolve a cookie token to the signed-in user. Expired sessions, inactive or deleted
/// users and non-`web` sessions (e.g. `pending_2fa`, M9) yield `None`.
pub async fn load(pool: &PgPool, raw: &str, hours: i64) -> sqlx::Result<Option<CurrentUser>> {
    let row = sqlx::query!(
        r#"SELECT s.id AS session_id, s.last_seen_at < now() - interval '1 minute' AS "stale!",
                  u.id AS user_id, u.org_id, u.email::text AS "email!", u.full_name,
                  u.must_change_pw, u.locale, u.timezone,
                  r.id AS role_id, r.name AS role_name, r.is_system,
                  COALESCE(array_agg(rp.permission_key) FILTER (WHERE rp.permission_key IS NOT NULL),
                           '{}') AS "permissions!"
           FROM sessions s
           JOIN users u ON u.id = s.user_id
           JOIN roles r ON r.id = u.role_id
           LEFT JOIN role_permissions rp ON rp.role_id = r.id
           WHERE s.token_hash = $1 AND s.kind = 'web' AND s.expires_at > now()
             AND u.is_active AND u.deleted_at IS NULL
           GROUP BY s.id, u.id, r.id"#,
        token::hash(raw),
    )
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else { return Ok(None) };
    // Super Admin holds every permission, computed from the registry rather than stored (§8).
    let is_super_admin = r.is_system && r.role_name == crate::server::seed::SUPER_ADMIN;
    let permissions = if is_super_admin {
        crate::server::rbac::all_keys()
    } else {
        r.permissions.into_iter().collect::<HashSet<_>>()
    };
    if r.stale {
        // Sliding expiry, written at most once a minute per session.
        sqlx::query!(
            "UPDATE sessions SET last_seen_at = now(),
                    expires_at = now() + make_interval(hours => $2) WHERE id = $1",
            r.session_id,
            i32::try_from(hours).unwrap_or(12),
        )
        .execute(pool)
        .await?;
    }
    Ok(Some(CurrentUser {
        id: r.user_id,
        session_id: r.session_id,
        org_id: r.org_id,
        email: r.email,
        full_name: r.full_name,
        role_id: r.role_id,
        role_name: r.role_name,
        is_super_admin,
        permissions,
        must_change_pw: r.must_change_pw,
        locale: r.locale,
        timezone: r.timezone,
    }))
}
