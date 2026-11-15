//! Idempotent boot-time seeding. M0: default organization (§18). M1/M2 add roles, admin, settings.
use sqlx::PgPool;
use uuid::Uuid;

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
