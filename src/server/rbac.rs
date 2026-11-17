//! Role-based access control (§8): the code-defined permission registry, its startup sync into
//! the `permissions` table, the `require` check, and the guards every user/role service uses.
use std::collections::HashSet;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::ctx::{Ctx, CurrentUser};
use super::error::AppError;

/// Every permission the app knows (`module.action`, description). Admins assign these to roles;
/// they cannot invent new ones. Own-account actions (own password, 2FA, sessions,
/// notifications) need a session only, no key.
pub const PERMISSIONS: &[(&str, &str)] = &[
    ("dashboard.view", "View dashboard"),
    ("users.view", "List and view users"),
    ("users.create", "Create users"),
    ("users.update", "Edit users, change role"),
    ("users.delete", "Delete users"),
    ("users.reset_password", "Reset another user's password"),
    ("roles.view", "View roles"),
    ("roles.create", "Create roles"),
    ("roles.update", "Edit roles and their permissions"),
    ("roles.delete", "Delete roles"),
    ("permissions.view", "View permission catalog"),
    ("audit.view", "View audit log"),
    ("audit.export", "Export audit log"),
    ("settings.view", "View system settings"),
    ("settings.update", "Change system settings"),
    ("backup.view", "View backups"),
    ("backup.create", "Create backups"),
    ("backup.download", "Download backups"),
    ("backup.restore", "Restore a backup"),
    ("backup.delete", "Delete backups"),
    // ---- revision 2 ----
    ("users.restore", "Restore soft-deleted users"),
    ("users.purge", "Permanently delete users"),
    ("users.export", "Export users to CSV"),
    (
        "users.reset_2fa",
        "Reset another user's two-factor authentication",
    ),
    ("users.revoke_sessions", "Revoke another user's sessions"),
    ("roles.export", "Export roles to CSV"),
    ("backup.export", "Export backup list to CSV"),
    ("flags.view", "View feature flags"),
    ("flags.update", "Change feature flags"),
    ("email.view_log", "View the email delivery log"),
    ("email.send_test", "Send a test email"),
    ("api_keys.create", "Create own API tokens"),
    ("api_keys.view_all", "View every user's API tokens"),
    ("api_keys.revoke_all", "Revoke any API token"),
    ("notifications.broadcast", "Send an announcement to users"),
    (
        "organizations.view",
        "View organizations (multi-tenant mode)",
    ),
    (
        "organizations.manage",
        "Create/edit/disable organizations (multi-tenant mode)",
    ),
];

/// Every key in the registry (what Super Admin holds, computed rather than stored).
pub fn all_keys() -> HashSet<String> {
    PERMISSIONS.iter().map(|(k, _)| (*k).to_string()).collect()
}

pub fn is_known(key: &str) -> bool {
    PERMISSIONS.iter().any(|(k, _)| *k == key)
}

/// Startup sync: insert missing keys, refresh descriptions, delete keys no longer in code
/// (their grants go with them via `ON DELETE CASCADE`).
pub async fn sync_permissions(pool: &PgPool) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    let keys: Vec<String> = PERMISSIONS.iter().map(|(k, _)| (*k).to_string()).collect();
    for (key, desc) in PERMISSIONS {
        let (module, action) = key.split_once('.').unwrap_or((key, ""));
        sqlx::query!(
            r#"INSERT INTO permissions (key, module, action, description) VALUES ($1, $2, $3, $4)
               ON CONFLICT (key) DO UPDATE
               SET module = EXCLUDED.module, action = EXCLUDED.action,
                   description = EXCLUDED.description"#,
            key,
            module,
            action,
            desc
        )
        .execute(&mut *tx)
        .await?;
    }
    let removed = sqlx::query!("DELETE FROM permissions WHERE NOT (key = ANY($1))", &keys)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if removed > 0 {
        tracing::warn!(
            removed,
            "removed permissions no longer in the code registry"
        );
    }
    tx.commit().await
}

/// The signed-in user holding `key`, or `401` / `403`.
pub fn require<'a>(ctx: &'a Ctx, key: &str) -> Result<&'a CurrentUser, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    debug_assert!(is_known(key), "unknown permission key {key}");
    if user.permissions.contains(key) {
        Ok(user)
    } else {
        Err(AppError::Forbidden)
    }
}

// ---------- guards (§8 rules) ----------

/// A user can't delete or deactivate themself.
pub fn not_self(actor: &CurrentUser, target: Uuid) -> Result<(), AppError> {
    if actor.id == target {
        Err(AppError::Conflict("users.self_action"))
    } else {
        Ok(())
    }
}

/// No privilege escalation: every granted key must be one the actor holds.
pub fn can_grant<'a>(
    actor: &CurrentUser,
    keys: impl IntoIterator<Item = &'a String>,
) -> Result<(), AppError> {
    for k in keys {
        if !is_known(k) {
            return Err(AppError::Validation("permissions.unknown"));
        }
        if !actor.permissions.contains(k) {
            return Err(AppError::Forbidden);
        }
    }
    Ok(())
}

/// The actor may assign `role_id` only if they hold every permission that role has
/// (Super Admin role: only a Super Admin).
pub async fn can_assign_role(
    tx: &mut Transaction<'_, Postgres>,
    actor: &CurrentUser,
    role_id: Uuid,
) -> Result<(), AppError> {
    let role = sqlx::query!(
        r#"SELECT r.is_system, r.name,
                  COALESCE(array_agg(rp.permission_key) FILTER (WHERE rp.permission_key IS NOT NULL),
                           '{}') AS "keys!"
           FROM roles r LEFT JOIN role_permissions rp ON rp.role_id = r.id
           WHERE r.id = $1 GROUP BY r.id"#,
        role_id
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(AppError::Validation("roles.not_found"))?;
    if role.is_system && role.name == super::seed::SUPER_ADMIN {
        return if actor.is_super_admin {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        };
    }
    can_grant(actor, role.keys.iter())
}

/// Refuse a change that would leave no active Super Admin. Call inside the transaction,
/// before deactivating, deleting or changing the role of `user_id`. Locks the Super Admin rows
/// so two admins demoting each other at once cannot both succeed.
pub async fn keep_last_super_admin(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), AppError> {
    let ids = sqlx::query_scalar!(
        r#"SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id
           WHERE r.is_system AND r.name = $1 AND u.is_active AND u.deleted_at IS NULL
           FOR UPDATE OF u"#,
        super::seed::SUPER_ADMIN
    )
    .fetch_all(&mut **tx)
    .await?;
    if ids.len() == 1 && ids[0] == user_id {
        Err(AppError::Conflict("users.last_super_admin"))
    } else {
        Ok(())
    }
}

/// A role's permissions changed: sign out everyone holding it, so the change applies at once
/// (sessions carry the permission set). Returns the number of sessions removed.
pub async fn revoke_role_sessions(
    tx: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
) -> sqlx::Result<u64> {
    Ok(sqlx::query!(
        "DELETE FROM sessions WHERE user_id IN (SELECT id FROM users WHERE role_id = $1)",
        role_id
    )
    .execute(&mut **tx)
    .await?
    .rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::request::RequestMeta;

    fn user(perms: &[&str]) -> CurrentUser {
        CurrentUser {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            email: "a@example.com".into(),
            full_name: "A".into(),
            role_id: Uuid::new_v4(),
            role_name: "Editor".into(),
            is_super_admin: false,
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            must_change_pw: false,
            locale: None,
        }
    }

    fn ctx(u: Option<CurrentUser>) -> Ctx {
        Ctx {
            org_id: Uuid::nil(),
            user: u,
            request_id: None,
            meta: RequestMeta::default(),
        }
    }

    #[test]
    fn registry_keys_are_unique_and_well_formed() {
        assert_eq!(all_keys().len(), PERMISSIONS.len());
        for (k, d) in PERMISSIONS {
            let (m, a) = k.split_once('.').expect("module.action");
            assert!(!m.is_empty() && !a.is_empty() && !d.is_empty(), "{k}");
        }
    }

    #[test]
    fn require_checks_session_then_permission() {
        assert!(matches!(
            require(&ctx(None), "users.view"),
            Err(AppError::Unauthenticated)
        ));
        let c = ctx(Some(user(&["users.view"])));
        assert!(require(&c, "users.view").is_ok());
        assert!(matches!(
            require(&c, "users.delete"),
            Err(AppError::Forbidden)
        ));
    }

    #[test]
    fn grant_only_what_you_hold() {
        let actor = user(&["roles.update", "users.view"]);
        let ok = ["users.view".to_string()];
        let escalate = ["users.delete".to_string()];
        let unknown = ["users.fly".to_string()];
        assert!(can_grant(&actor, ok.iter()).is_ok());
        assert!(matches!(
            can_grant(&actor, escalate.iter()),
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            can_grant(&actor, unknown.iter()),
            Err(AppError::Validation("permissions.unknown"))
        ));
    }

    #[test]
    fn cannot_act_on_self() {
        let a = user(&[]);
        assert!(not_self(&a, a.id).is_err());
        assert!(not_self(&a, Uuid::new_v4()).is_ok());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn sync_inserts_and_removes(pool: PgPool) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO permissions VALUES ('old.thing','old','thing','gone')")
            .execute(&pool)
            .await?;
        sync_permissions(&pool).await?;
        sync_permissions(&pool).await?; // idempotent
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM permissions")
            .fetch_one(&pool)
            .await?;
        assert_eq!(n as usize, PERMISSIONS.len());
        let old: i64 = sqlx::query_scalar("SELECT count(*) FROM permissions WHERE key='old.thing'")
            .fetch_one(&pool)
            .await?;
        assert_eq!(old, 0);
        Ok(())
    }

    /// Fixture: org + Super Admin role + `n` active super admins; returns their ids.
    async fn super_admins(pool: &PgPool, n: usize) -> (Uuid, Vec<Uuid>) {
        let org = crate::server::seed::ensure_default_org(pool).await.unwrap();
        let role = crate::server::seed::ensure_super_admin_role(pool)
            .await
            .unwrap();
        let mut ids = vec![];
        for i in 0..n {
            let id: Uuid = sqlx::query_scalar(
                "INSERT INTO users (org_id, email, full_name, password_hash, role_id)
                 VALUES ($1, $2, 'S', 'x', $3) RETURNING id",
            )
            .bind(org)
            .bind(format!("s{i}@example.com"))
            .bind(role)
            .fetch_one(pool)
            .await
            .unwrap();
            ids.push(id);
        }
        (role, ids)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn last_super_admin_is_protected(pool: PgPool) -> sqlx::Result<()> {
        let (_, ids) = super_admins(&pool, 1).await;
        let mut tx = pool.begin().await?;
        let r = keep_last_super_admin(&mut tx, ids[0]).await;
        assert!(matches!(
            r,
            Err(AppError::Conflict("users.last_super_admin"))
        ));
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn one_of_two_super_admins_may_go(pool: PgPool) -> sqlx::Result<()> {
        let (_, ids) = super_admins(&pool, 2).await;
        let mut tx = pool.begin().await?;
        assert!(keep_last_super_admin(&mut tx, ids[0]).await.is_ok());
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn role_change_revokes_its_sessions(pool: PgPool) -> sqlx::Result<()> {
        let (role, ids) = super_admins(&pool, 2).await;
        for id in &ids {
            sqlx::query(
                "INSERT INTO sessions (token_hash, user_id, expires_at)
                 VALUES (gen_random_bytes(32), $1, now() + interval '1 hour')",
            )
            .bind(id)
            .execute(&pool)
            .await?;
        }
        let mut tx = pool.begin().await?;
        assert_eq!(revoke_role_sessions(&mut tx, role).await?, 2);
        tx.commit().await?;
        let left: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions")
            .fetch_one(&pool)
            .await?;
        assert_eq!(left, 0);
        Ok(())
    }
}
