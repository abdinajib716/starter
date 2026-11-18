//! Roles (§9.3) and the permission catalog (§9.4). Changing a role's permissions signs out
//! everyone holding it (§8), so the new set applies at once. System roles are read-only.
use std::collections::{BTreeSet, HashMap};

use dioxus::server::axum::{Extension, extract::Query, response::Response};
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::roles::{
    PermissionCatalog, PermissionInfo, RoleDetail, RoleInput, RoleSummary, validate_role,
};
use crate::domain::users::BulkResult;
use crate::server::audit::{self, Entry};
use crate::server::ctx::{Ctx, CurrentUser};
use crate::server::error::AppError;
use crate::server::export::{csv_response, error_response, line};
use crate::server::rbac::{self, PERMISSIONS};
use crate::server::seed::SUPER_ADMIN;

fn entry<'a>(ctx: &'a Ctx, actor: &'a CurrentUser, action: &'a str, id: Uuid) -> Entry<'a> {
    Entry {
        org_id: Some(ctx.org_id),
        actor_id: Some(actor.id),
        actor_email: Some(actor.email.as_str()),
        action,
        entity_type: Some("role"),
        entity_id: Some(id.to_string()),
        ip: ctx.meta.ip,
        user_agent: ctx.meta.user_agent.as_deref(),
        success: true,
        ..Default::default()
    }
}

fn is_super_admin_role(is_system: bool, name: &str) -> bool {
    is_system && name == SUPER_ADMIN
}

pub async fn list(pool: &PgPool, ctx: &Ctx) -> Result<Vec<RoleSummary>, AppError> {
    ctx.require("roles.view")?;
    let rows = sqlx::query!(
        r#"SELECT r.id, r.name, r.description, r.is_system, r.require_2fa,
                  (SELECT count(*) FROM users u WHERE u.role_id = r.id AND u.deleted_at IS NULL) AS "users!",
                  (SELECT count(*) FROM role_permissions rp WHERE rp.role_id = r.id) AS "perms!"
           FROM roles r WHERE r.org_id = $1 OR r.org_id IS NULL
           ORDER BY r.is_system DESC, lower(r.name)"#,
        ctx.org_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RoleSummary {
            permission_count: if is_super_admin_role(r.is_system, &r.name) {
                PERMISSIONS.len() as i64
            } else {
                r.perms
            },
            id: r.id,
            name: r.name,
            description: r.description,
            is_system: r.is_system,
            require_2fa: r.require_2fa,
            user_count: r.users,
        })
        .collect())
}

pub async fn get(pool: &PgPool, ctx: &Ctx, id: Uuid) -> Result<RoleDetail, AppError> {
    ctx.require("roles.view")?;
    let role = list(pool, ctx)
        .await?
        .into_iter()
        .find(|r| r.id == id)
        .ok_or(AppError::NotFound)?;
    let keys = if is_super_admin_role(role.is_system, &role.name) {
        PERMISSIONS.iter().map(|(k, _)| (*k).to_string()).collect()
    } else {
        sqlx::query_scalar!(
            "SELECT permission_key FROM role_permissions WHERE role_id = $1 ORDER BY 1",
            id
        )
        .fetch_all(pool)
        .await?
    };
    Ok(RoleDetail { role, keys })
}

/// The registry in code order, with the roles that hold each key.
pub async fn catalog(pool: &PgPool, ctx: &Ctx) -> Result<PermissionCatalog, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    // The catalog page needs permissions.view; the role editor (roles.*) needs the list too.
    if !["permissions.view", "roles.view"]
        .iter()
        .any(|k| user.permissions.contains(*k))
    {
        return Err(AppError::Forbidden);
    }
    let grants = sqlx::query!(
        r#"SELECT rp.permission_key, r.name FROM role_permissions rp JOIN roles r ON r.id = rp.role_id
           WHERE r.org_id = $1 OR r.org_id IS NULL ORDER BY lower(r.name)"#,
        ctx.org_id
    )
    .fetch_all(pool)
    .await?;
    let mut by_key: HashMap<String, Vec<String>> = HashMap::new();
    for g in grants {
        by_key.entry(g.permission_key).or_default().push(g.name);
    }
    Ok(PermissionCatalog {
        permissions: PERMISSIONS
            .iter()
            .map(|(key, desc)| {
                let (module, action) = key.split_once('.').unwrap_or((key, ""));
                let mut roles = vec![SUPER_ADMIN.to_string()];
                roles.extend(by_key.remove(*key).unwrap_or_default());
                PermissionInfo {
                    key: (*key).into(),
                    module: module.into(),
                    action: action.into(),
                    description: (*desc).into(),
                    roles,
                }
            })
            .collect(),
    })
}

fn clean_keys(keys: &[String]) -> BTreeSet<String> {
    keys.iter().map(|k| k.trim().to_string()).collect()
}

async fn set_keys(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    keys: &BTreeSet<String>,
) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM role_permissions WHERE role_id = $1", id)
        .execute(&mut **tx)
        .await?;
    let keys: Vec<String> = keys.iter().cloned().collect();
    sqlx::query!(
        "INSERT INTO role_permissions (role_id, permission_key) SELECT $1, unnest($2::text[])",
        id,
        &keys
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn name_taken(e: sqlx::Error) -> AppError {
    match &e {
        sqlx::Error::Database(d) if d.is_unique_violation() => {
            AppError::Conflict("roles.name_taken")
        }
        _ => e.into(),
    }
}

pub async fn create(pool: &PgPool, ctx: &Ctx, input: RoleInput) -> Result<Uuid, AppError> {
    let actor = ctx.require("roles.create")?;
    if let Some(code) = validate_role(&input) {
        return Err(AppError::Validation(code));
    }
    let keys = clean_keys(&input.keys);
    rbac::can_grant(actor, keys.iter())?;
    let mut tx = pool.begin().await?;
    let id = sqlx::query_scalar!(
        "INSERT INTO roles (org_id, name, description, require_2fa) VALUES ($1, $2, $3, $4) RETURNING id",
        ctx.org_id,
        input.name.trim(),
        input.description.trim(),
        input.require_2fa
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(name_taken)?;
    set_keys(&mut tx, id, &keys).await?;
    audit::record(
        &mut tx,
        Entry {
            after: Some(
                json!({ "name": input.name.trim(), "description": input.description.trim(),
                                "require_2fa": input.require_2fa, "permissions": keys }),
            ),
            ..entry(ctx, actor, "role.create", id)
        },
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

/// Name, description, 2FA flag and permissions. Audit stores the key diff; if the
/// permissions (or the 2FA rule) changed, the role's users are signed out.
pub async fn update(pool: &PgPool, ctx: &Ctx, id: Uuid, input: RoleInput) -> Result<u64, AppError> {
    let actor = ctx.require("roles.update")?;
    if let Some(code) = validate_role(&input) {
        return Err(AppError::Validation(code));
    }
    let keys = clean_keys(&input.keys);
    let mut tx = pool.begin().await?;
    let before = sqlx::query!(
        "SELECT name, description, is_system, require_2fa FROM roles
         WHERE id = $1 AND (org_id = $2 OR org_id IS NULL) FOR UPDATE",
        id,
        ctx.org_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;
    if before.is_system {
        return Err(AppError::Conflict("roles.system"));
    }
    let old: BTreeSet<String> = sqlx::query_scalar!(
        "SELECT permission_key FROM role_permissions WHERE role_id = $1",
        id
    )
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .collect();
    // No escalation: the actor must hold every key they add or remove.
    let changed: Vec<&String> = old.symmetric_difference(&keys).collect();
    rbac::can_grant(actor, changed.iter().copied())?;
    sqlx::query!(
        "UPDATE roles SET name = $2, description = $3, require_2fa = $4, updated_at = now() WHERE id = $1",
        id,
        input.name.trim(),
        input.description.trim(),
        input.require_2fa
    )
    .execute(&mut *tx)
    .await
    .map_err(name_taken)?;
    set_keys(&mut tx, id, &keys).await?;
    let added: Vec<&String> = keys.difference(&old).collect();
    let removed: Vec<&String> = old.difference(&keys).collect();
    let signed_out =
        if !added.is_empty() || !removed.is_empty() || before.require_2fa != input.require_2fa {
            rbac::revoke_role_sessions(&mut tx, id).await?
        } else {
            0
        };
    audit::record(
        &mut tx,
        Entry {
            before: Some(json!({ "name": before.name, "description": before.description, "require_2fa": before.require_2fa })),
            after: Some(json!({ "name": input.name.trim(), "description": input.description.trim(),
                                "require_2fa": input.require_2fa, "added": added, "removed": removed,
                                "sessions_ended": signed_out })),
            ..entry(ctx, actor, "role.update", id)
        },
    )
    .await?;
    tx.commit().await?;
    Ok(signed_out)
}

pub async fn delete(pool: &PgPool, ctx: &Ctx, id: Uuid) -> Result<(), AppError> {
    let actor = ctx.require("roles.delete")?;
    let mut tx = pool.begin().await?;
    let r = sqlx::query!(
        "SELECT name, is_system FROM roles WHERE id = $1 AND (org_id = $2 OR org_id IS NULL) FOR UPDATE",
        id,
        ctx.org_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;
    if r.is_system {
        return Err(AppError::Conflict("roles.system"));
    }
    // Deleted users still reference their role (restore brings them back with it).
    let used: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE role_id = $1) AS "e!""#,
        id
    )
    .fetch_one(&mut *tx)
    .await?;
    if used {
        return Err(AppError::Conflict("roles.in_use"));
    }
    sqlx::query!("DELETE FROM roles WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    audit::record(
        &mut tx,
        Entry {
            before: Some(json!({ "name": r.name })),
            ..entry(ctx, actor, "role.delete", id)
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Bulk delete (§25.1 pattern): roles in use or system roles are skipped with a reason.
pub async fn delete_many(pool: &PgPool, ctx: &Ctx, ids: Vec<Uuid>) -> Result<BulkResult, AppError> {
    ctx.require("roles.delete")?;
    let mut res = BulkResult::default();
    for id in ids {
        match delete(pool, ctx, id).await {
            Ok(()) => res.done += 1,
            Err(AppError::Internal(e)) => return Err(AppError::Internal(e)),
            Err(e) => res.skipped.push((id, e.code().to_string())),
        }
    }
    Ok(res)
}

/// `GET /api/export/roles.csv` (`roles.export`, audited as `export.roles`).
pub async fn export_route(
    Extension(ctx): Extension<Ctx>,
    Query(_p): Query<HashMap<String, String>>,
) -> Response {
    match export(&ctx).await {
        Ok(r) => r,
        Err(e) => error_response(e),
    }
}

async fn export(ctx: &Ctx) -> Result<Response, AppError> {
    let actor = ctx.require("roles.export")?.clone();
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    // Roles are few: list them with the read path, then stream the lines.
    let mut viewer = ctx.clone();
    if let Some(u) = viewer.user.as_mut() {
        u.permissions.insert("roles.view".into());
    }
    let roles = list(pool, &viewer).await?;
    let mut tx = pool.begin().await?;
    audit::record(
        &mut tx,
        Entry {
            entity_id: None,
            after: Some(json!({ "rows": roles.len() })),
            ..entry(ctx, &actor, "export.roles", Uuid::nil())
        },
    )
    .await?;
    tx.commit().await?;
    let (send, recv) = tokio::sync::mpsc::channel::<String>(16);
    tokio::spawn(async move {
        let _ = send
            .send(format!(
                "\u{feff}{}",
                line(&[
                    "name",
                    "description",
                    "system",
                    "require_2fa",
                    "users",
                    "permissions"
                ])
            ))
            .await;
        for r in roles {
            let l = line(&[
                &r.name,
                &r.description,
                if r.is_system { "true" } else { "false" },
                if r.require_2fa { "true" } else { "false" },
                &r.user_count.to_string(),
                &r.permission_count.to_string(),
            ]);
            if send.send(l).await.is_err() {
                return;
            }
        }
    });
    Ok(csv_response("roles.csv", false, recv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{request::RequestMeta, seed, settings};

    async fn setup(pool: &PgPool) -> (Ctx, Uuid) {
        rbac::sync_permissions(pool).await.unwrap();
        settings::seed(pool).await.unwrap();
        let org = seed::ensure_default_org(pool).await.unwrap();
        let admin_role = seed::ensure_super_admin_role(pool).await.unwrap();
        let me: Uuid = sqlx::query_scalar("INSERT INTO users (org_id, email, full_name, password_hash, role_id) VALUES ($1,'root@test.local','R','x',$2) RETURNING id")
            .bind(org)
            .bind(admin_role)
            .fetch_one(pool)
            .await
            .unwrap();
        let ctx = Ctx {
            org_id: org,
            user: Some(CurrentUser {
                id: me,
                session_id: Uuid::new_v4(),
                org_id: org,
                email: "root@test.local".into(),
                full_name: "R".into(),
                role_id: admin_role,
                role_name: SUPER_ADMIN.into(),
                is_super_admin: true,
                permissions: rbac::all_keys(),
                must_change_pw: false,
                locale: None,
                timezone: None,
                avatar: None,
                twofa_enrolled: false,
                role_require_2fa: false,
                twofa_since: time::OffsetDateTime::UNIX_EPOCH,
            }),
            request_id: None,
            meta: RequestMeta::default(),
        };
        (ctx, admin_role)
    }

    fn input(name: &str, keys: &[&str]) -> RoleInput {
        RoleInput {
            name: name.into(),
            description: String::new(),
            require_2fa: false,
            keys: keys.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn permission_change_signs_out_the_role(pool: PgPool) {
        let (ctx, _) = setup(&pool).await;
        let role = create(&pool, &ctx, input("Editor", &["users.view"]))
            .await
            .unwrap();
        let u: Uuid = sqlx::query_scalar("INSERT INTO users (org_id, email, full_name, password_hash, role_id) VALUES ($1,'e@test.local','E','x',$2) RETURNING id")
            .bind(ctx.org_id)
            .bind(role)
            .fetch_one(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (gen_random_bytes(32), $1, now() + interval '1 hour')")
            .bind(u)
            .execute(&pool)
            .await
            .unwrap();
        // renaming only: sessions kept
        assert_eq!(
            update(&pool, &ctx, role, input("Editors", &["users.view"]))
                .await
                .unwrap(),
            0
        );
        // permissions changed: signed out
        assert_eq!(
            update(
                &pool,
                &ctx,
                role,
                input("Editors", &["users.view", "users.create"])
            )
            .await
            .unwrap(),
            1
        );
        let after: serde_json::Value = sqlx::query_scalar(
            "SELECT after FROM audit_logs WHERE action = 'role.update' ORDER BY id DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(after["added"], json!(["users.create"]));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn in_use_and_system_roles_cannot_be_deleted(pool: PgPool) {
        let (ctx, admin_role) = setup(&pool).await;
        assert!(matches!(
            delete(&pool, &ctx, admin_role).await,
            Err(AppError::Conflict("roles.system"))
        ));
        assert!(matches!(
            update(&pool, &ctx, admin_role, input("X", &[])).await,
            Err(AppError::Conflict("roles.system"))
        ));
        let used = create(&pool, &ctx, input("Used", &[])).await.unwrap();
        sqlx::query("INSERT INTO users (org_id, email, full_name, password_hash, role_id) VALUES ($1,'u@test.local','U','x',$2)")
            .bind(ctx.org_id)
            .bind(used)
            .execute(&pool)
            .await
            .unwrap();
        let free = create(&pool, &ctx, input("Free", &[])).await.unwrap();
        let r = delete_many(&pool, &ctx, vec![used, free]).await.unwrap();
        assert_eq!(r.done, 1);
        assert_eq!(r.skipped, vec![(used, "roles.in_use".to_string())]);
        assert!(
            create(&pool, &ctx, input("used", &[])).await.is_ok(),
            "names are case-sensitive"
        );
        assert!(matches!(
            create(&pool, &ctx, input("Used", &[])).await,
            Err(AppError::Conflict("roles.name_taken"))
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn no_escalation_through_roles(pool: PgPool) {
        let (mut ctx, _) = setup(&pool).await;
        if let Some(u) = ctx.user.as_mut() {
            u.is_super_admin = false;
            u.permissions = ["roles.view", "roles.create", "roles.update", "users.view"]
                .iter()
                .map(|s| s.to_string())
                .collect();
        }
        assert!(
            create(&pool, &ctx, input("Viewer", &["users.view"]))
                .await
                .is_ok()
        );
        assert!(matches!(
            create(&pool, &ctx, input("Boss", &["users.delete"])).await,
            Err(AppError::Forbidden)
        ));
    }
}
