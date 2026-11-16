//! Idempotent boot-time seeding. M0: default organization (§18). M1: Super Admin role + first
//! admin (§7 "First run"). M2 adds the permission registry and settings.
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::password;
use super::config::Config;

/// Name of the computed all-permissions system role (§8).
pub const SUPER_ADMIN: &str = "Super Admin";

/// Return the default org id, creating it on first run.
pub async fn ensure_default_org(pool: &PgPool) -> sqlx::Result<Uuid> {
    if let Some(id) = sqlx::query_scalar!("SELECT id FROM organizations WHERE is_default")
        .fetch_optional(pool)
        .await?
    {
        return Ok(id);
    }
    // Racing instances: the partial unique index `one_default_org` lets only one insert win.
    let inserted = sqlx::query_scalar!(
        r#"INSERT INTO organizations (name, slug, is_default)
           VALUES ('Default', 'default', true)
           ON CONFLICT DO NOTHING
           RETURNING id"#
    )
    .fetch_optional(pool)
    .await?;
    match inserted {
        Some(id) => {
            tracing::info!(%id, "created default organization");
            Ok(id)
        }
        None => {
            sqlx::query_scalar!("SELECT id FROM organizations WHERE is_default")
                .fetch_one(pool)
                .await
        }
    }
}

/// Platform role `Super Admin` (`org_id IS NULL`, `is_system`). Returns its id.
pub async fn ensure_super_admin_role(pool: &PgPool) -> sqlx::Result<Uuid> {
    sqlx::query!(
        r#"INSERT INTO roles (org_id, name, description, is_system)
           VALUES (NULL, $1, 'All permissions; cannot be edited or deleted', true)
           ON CONFLICT DO NOTHING"#,
        SUPER_ADMIN
    )
    .execute(pool)
    .await?;
    sqlx::query_scalar!(
        "SELECT id FROM roles WHERE org_id IS NULL AND name = $1",
        SUPER_ADMIN
    )
    .fetch_one(pool)
    .await
}

/// First run only: if the org has no users, create the admin from `SEED_ADMIN_*`
/// with `must_change_pw = true`. Never touches an existing database's users.
pub async fn ensure_first_admin(pool: &PgPool, cfg: &Config, org_id: Uuid) -> anyhow::Result<()> {
    let has_users: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE org_id = $1) AS "e!""#,
        org_id
    )
    .fetch_one(pool)
    .await?;
    if has_users {
        return Ok(());
    }
    let (Some(email), Some(pw)) = (&cfg.seed_admin_email, &cfg.seed_admin_password) else {
        tracing::warn!(
            "no users and SEED_ADMIN_EMAIL/SEED_ADMIN_PASSWORD unset: nobody can log in"
        );
        return Ok(());
    };
    let role_id = ensure_super_admin_role(pool).await?;
    let hash = password::hash(pw.clone()).await?;
    sqlx::query!(
        r#"INSERT INTO users (org_id, email, full_name, password_hash, role_id, must_change_pw)
           VALUES ($1, $2::text::citext, 'Administrator', $3, $4, true)
           ON CONFLICT DO NOTHING"#,
        org_id,
        email.trim().to_lowercase(),
        hash,
        role_id
    )
    .execute(pool)
    .await?;
    tracing::info!(%email, "created first admin (must change password at first login)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn default_org_is_created_once(pool: PgPool) -> sqlx::Result<()> {
        let first = ensure_default_org(&pool).await?;
        let second = ensure_default_org(&pool).await?;
        assert_eq!(first, second);
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations WHERE is_default")
            .fetch_one(&pool)
            .await?;
        assert_eq!(n, 1);
        Ok(())
    }

    /// M4.8: the audit log is append-only at the database level.
    #[sqlx::test(migrations = "./migrations")]
    async fn audit_log_rejects_update_and_delete(pool: PgPool) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO audit_logs (action) VALUES ('test.insert')")
            .execute(&pool)
            .await?;
        let upd = sqlx::query("UPDATE audit_logs SET action = 'x'")
            .execute(&pool)
            .await;
        let del = sqlx::query("DELETE FROM audit_logs").execute(&pool).await;
        assert!(upd.unwrap_err().to_string().contains("append-only"));
        assert!(del.unwrap_err().to_string().contains("append-only"));
        Ok(())
    }
}
