//! Organizations (multi-tenant mode, §18, M16.3). Platform administrators — Super Admins of the
//! default organization — create and switch off tenants; each new tenant gets an
//! "Administrator" role (every permission except managing organizations) and a first admin.
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::auth::user_entry;
use crate::domain::orgs::{NewOrg, OrgRow, valid_slug};
use crate::server::audit::{self, Entry};
use crate::server::auth::password;
use crate::server::ctx::{Ctx, CurrentUser};
use crate::server::error::AppError;

/// Only Super Admins signed in to the default organization manage tenants.
async fn platform<'a>(
    pool: &PgPool,
    ctx: &'a Ctx,
    perm: &str,
) -> Result<&'a CurrentUser, AppError> {
    let user = ctx.require(perm)?;
    let default = sqlx::query_scalar!(
        r#"SELECT is_default AS "d!" FROM organizations WHERE id = $1"#,
        ctx.org_id
    )
    .fetch_one(pool)
    .await?;
    if !(user.is_super_admin && default) {
        return Err(AppError::Forbidden);
    }
    Ok(user)
}

pub async fn list(pool: &PgPool, ctx: &Ctx) -> Result<Vec<OrgRow>, AppError> {
    platform(pool, ctx, "organizations.view").await?;
    Ok(sqlx::query_as!(
        OrgRow,
        r#"SELECT o.id, o.name, o.slug::text AS "slug!", o.is_default, o.is_active, o.max_users, o.created_at,
                  (SELECT count(*) FROM users u WHERE u.org_id = o.id AND u.deleted_at IS NULL) AS "users!"
           FROM organizations o ORDER BY o.is_default DESC, o.name"#
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create(pool: &PgPool, ctx: &Ctx, n: NewOrg) -> Result<Uuid, AppError> {
    let actor = platform(pool, ctx, "organizations.manage").await?;
    let slug = n.slug.trim().to_lowercase();
    let name = n.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(AppError::Validation("orgs.name"));
    }
    if !valid_slug(&slug) {
        return Err(AppError::Validation("orgs.slug"));
    }
    if !crate::domain::valid_email(&n.admin_email) || n.admin_name.trim().is_empty() {
        return Err(AppError::Validation("orgs.admin"));
    }
    password::check_policy(&n.admin_password, 8).map_err(|p| AppError::Validation(p.code()))?;
    let hash = password::hash(n.admin_password.clone()).await?;
    let mut tx = pool.begin().await?;
    let org = sqlx::query_scalar!(
        "INSERT INTO organizations (name, slug, max_users) VALUES ($1, $2::text::citext, $3) RETURNING id",
        name,
        slug,
        n.max_users
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(d) if d.is_unique_violation() => AppError::Conflict("orgs.slug_taken"),
        _ => e.into(),
    })?;
    let role = sqlx::query_scalar!(
        "INSERT INTO roles (org_id, name, description, is_system) VALUES ($1, 'Administrator', 'Everything in this organization', false) RETURNING id",
        org
    )
    .fetch_one(&mut *tx)
    .await?;
    let perms: Vec<String> = crate::server::rbac::PERMISSIONS
        .iter()
        .map(|(k, _)| (*k).to_string())
        .filter(|k| {
            !k.starts_with("organizations.")
                && !k.starts_with("flags.")
                && !k.starts_with("backup.")
        })
        .collect();
    sqlx::query!(
        "INSERT INTO role_permissions (role_id, permission_key) SELECT $1, unnest($2::text[])",
        role,
        &perms
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"INSERT INTO users (org_id, email, full_name, password_hash, role_id, must_change_pw)
           VALUES ($1, $2::text::citext, $3, $4, $5, true)"#,
        org,
        n.admin_email.trim().to_lowercase(),
        n.admin_name.trim(),
        hash,
        role
    )
    .execute(&mut *tx)
    .await?;
    audit::record(
        &mut tx,
        Entry {
            entity_type: Some("organization"),
            entity_id: Some(org.to_string()),
            after: Some(json!({ "name": name, "slug": slug, "admin": n.admin_email, "max_users": n.max_users })),
            ..user_entry(ctx, actor, "organization.create")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(org)
}

/// Rename, switch on/off, change the user limit. The default organization can't be switched off.
pub async fn update(
    pool: &PgPool,
    ctx: &Ctx,
    id: Uuid,
    name: String,
    active: bool,
    max_users: Option<i32>,
) -> Result<(), AppError> {
    let actor = platform(pool, ctx, "organizations.manage").await?;
    let mut tx = pool.begin().await?;
    let before = sqlx::query!(
        "SELECT name, is_active, is_default, max_users FROM organizations WHERE id = $1 FOR UPDATE",
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;
    if before.is_default && !active {
        return Err(AppError::Conflict("orgs.default_active"));
    }
    sqlx::query!(
        "UPDATE organizations SET name = $2, is_active = $3, max_users = $4 WHERE id = $1",
        id,
        name.trim(),
        active,
        max_users
    )
    .execute(&mut *tx)
    .await?;
    if !active {
        // Nobody stays signed in to a switched-off tenant.
        sqlx::query!(
            "DELETE FROM sessions WHERE user_id IN (SELECT id FROM users WHERE org_id = $1)",
            id
        )
        .execute(&mut *tx)
        .await?;
    }
    audit::record(
        &mut tx,
        Entry {
            entity_type: Some("organization"),
            entity_id: Some(id.to_string()),
            before: Some(json!({ "name": before.name, "active": before.is_active, "max_users": before.max_users })),
            after: Some(json!({ "name": name.trim(), "active": active, "max_users": max_users })),
            ..user_entry(ctx, actor, "organization.update")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// M16.6: refuse a new user beyond the organization's `max_users`.
pub async fn check_user_limit(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    org: Uuid,
) -> Result<(), AppError> {
    let r = sqlx::query!(
        r#"SELECT o.max_users, (SELECT count(*) FROM users u WHERE u.org_id = o.id AND u.deleted_at IS NULL) AS "n!"
           FROM organizations o WHERE o.id = $1"#,
        org
    )
    .fetch_one(&mut **tx)
    .await?;
    match r.max_users {
        Some(max) if r.n >= i64::from(max) => Err(AppError::Conflict("orgs.user_limit")),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::request::RequestMeta;

    async fn org_with_user(pool: &PgPool, slug: &str) -> (Uuid, Uuid) {
        let org = sqlx::query_scalar!(
            "INSERT INTO organizations (name, slug) VALUES ($1, $1::text::citext) RETURNING id",
            slug
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let role = sqlx::query_scalar!(
            "INSERT INTO roles (org_id, name) VALUES ($1, 'R') RETURNING id",
            org
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let user = sqlx::query_scalar!(
            "INSERT INTO users (org_id, email, full_name, password_hash, role_id) VALUES ($1, $2::text::citext, 'U', 'x', $3) RETURNING id",
            org, format!("u@{slug}.test"), role
        ).fetch_one(pool).await.unwrap();
        (org, user)
    }

    /// M16.5: services scope every read by the request's organization.
    #[sqlx::test(migrations = "./migrations")]
    async fn services_never_leak_across_orgs(pool: PgPool) {
        let (a, _) = org_with_user(&pool, "alpha").await;
        let (b, ub) = org_with_user(&pool, "beta").await;
        let mut tx = pool.begin().await.unwrap();
        crate::server::notify::send(
            &mut tx,
            b,
            crate::server::notify::Target::All,
            "announcement",
            json!({}),
            None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        let mut user = crate::server::rbac::tests_user(&["users.view", "audit.view"]);
        user.org_id = a;
        let ctx = Ctx {
            org_id: a,
            user: Some(user),
            request_id: None,
            meta: RequestMeta::default(),
        };
        let page = crate::server::services::users::list(
            &pool,
            &ctx,
            &crate::domain::users::UserQuery {
                page_size: 50,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(page.rows.iter().all(|r| r.email.ends_with("alpha.test")));
        assert!(
            crate::server::services::users::detail(&pool, &ctx, ub)
                .await
                .is_err()
        );
        let ids = crate::server::services::users::matching_ids(&pool, &ctx, &Default::default())
            .await
            .unwrap();
        assert!(!ids.contains(&ub));
        let n = crate::server::services::notifications::list(&pool, &ctx, false, 50, 0)
            .await
            .unwrap();
        assert_eq!(n.total, 0);
    }

    /// M16.2: with `app.org_id` set, a non-owner role sees only that organization's rows even
    /// with a query that forgets `org_id`.
    #[sqlx::test(migrations = "./migrations")]
    async fn rls_hides_other_tenants(pool: PgPool) {
        let (a, _) = org_with_user(&pool, "gamma").await;
        org_with_user(&pool, "delta").await;
        let mut c = pool.acquire().await.unwrap();
        // Needs a role that may create roles (the dev/CI superuser); skip otherwise.
        if sqlx::query("DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'rls_probe') THEN CREATE ROLE rls_probe; END IF; END $$")
            .execute(&mut *c).await.is_err()
        {
            return;
        }
        sqlx::query("GRANT SELECT ON users TO rls_probe")
            .execute(&mut *c)
            .await
            .unwrap();
        sqlx::query("SET ROLE rls_probe")
            .execute(&mut *c)
            .await
            .unwrap();
        sqlx::query("SELECT set_config('app.org_id', $1, false)")
            .bind(a.to_string())
            .execute(&mut *c)
            .await
            .unwrap();
        let emails: Vec<String> = sqlx::query_scalar("SELECT email::text FROM users")
            .fetch_all(&mut *c)
            .await
            .unwrap();
        assert_eq!(emails, vec!["u@gamma.test".to_string()]);
        sqlx::query("SELECT set_config('app.org_id', '', false)")
            .execute(&mut *c)
            .await
            .unwrap();
        let all: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
            .fetch_one(&mut *c)
            .await
            .unwrap();
        assert_eq!(all, 2);
        sqlx::query("RESET ROLE").execute(&mut *c).await.unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn user_limit_is_enforced(pool: PgPool) {
        let (a, _) = org_with_user(&pool, "eps").await;
        sqlx::query!("UPDATE organizations SET max_users = 1 WHERE id = $1", a)
            .execute(&pool)
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();
        assert!(matches!(
            check_user_limit(&mut tx, a).await,
            Err(AppError::Conflict("orgs.user_limit"))
        ));
    }
}
