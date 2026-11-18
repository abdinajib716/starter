//! Users (§9.2, §25): server-side list with filters, facets, sort and paging; create, edit,
//! activate/deactivate, reset password, soft delete, restore, purge; bulk versions with
//! per-item guards. Every change runs in a transaction with its audit row.
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

use crate::domain::users::{
    BulkResult, BulkUserAction, RoleOption, UserInput, UserPage, UserQuery, UserRow, UserSort,
    UserStatus, validate_input,
};
use crate::server::audit::{self, Entry};
use crate::server::auth::password;
use crate::server::ctx::{Ctx, CurrentUser};
use crate::server::error::AppError;
use crate::server::rbac;
use crate::server::settings::{self, SecuritySettings};

pub const PAGE_SIZES: [u32; 5] = [10, 20, 30, 40, 50];

const STATUS_SQL: &str = "CASE WHEN u.deleted_at IS NOT NULL THEN 'deleted'
     WHEN u.locked_until > now() THEN 'suspended'
     WHEN NOT u.is_active THEN 'inactive'
     WHEN u.last_login_at IS NULL THEN 'invited'
     ELSE 'active' END";

/// `WITH base AS (...)`: the org's users (live or deleted tab) matching the search text.
fn push_base(b: &mut QueryBuilder<Postgres>, org: Uuid, q: &UserQuery) {
    b.push("WITH base AS (SELECT u.id, u.email::text AS email, u.full_name, u.role_id, r.name AS role_name, ");
    b.push(STATUS_SQL);
    b.push(" AS status, u.must_change_pw, u.last_login_at, u.created_at, u.deleted_at, d.email::text AS deleted_by_email FROM users u JOIN roles r ON r.id = u.role_id LEFT JOIN users d ON d.id = u.deleted_by WHERE u.org_id = ");
    b.push_bind(org);
    b.push(if q.deleted {
        " AND u.deleted_at IS NOT NULL"
    } else {
        " AND u.deleted_at IS NULL"
    });
    let term = q.q.trim();
    if !term.is_empty() {
        let like = format!("%{}%", escape_like(term));
        b.push(" AND (u.full_name ILIKE ");
        b.push_bind(like.clone());
        b.push(" OR u.email::text ILIKE ");
        b.push_bind(like);
        b.push(")");
    }
    b.push(") ");
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// `WHERE` over `base` for the status / role filters (skipping one for facet counts).
fn push_filters(b: &mut QueryBuilder<Postgres>, q: &UserQuery, skip_status: bool, skip_role: bool) {
    b.push(" WHERE true");
    if !skip_status && !q.statuses.is_empty() {
        let s: Vec<String> = q.statuses.iter().map(|s| s.as_str().to_string()).collect();
        b.push(" AND status = ANY(");
        b.push_bind(s);
        b.push(")");
    }
    if !skip_role && !q.role_ids.is_empty() {
        b.push(" AND role_id = ANY(");
        b.push_bind(q.role_ids.clone());
        b.push(")");
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    email: String,
    full_name: String,
    role_id: Uuid,
    role_name: String,
    status: String,
    must_change_pw: bool,
    last_login_at: Option<time::OffsetDateTime>,
    created_at: time::OffsetDateTime,
    deleted_at: Option<time::OffsetDateTime>,
    deleted_by_email: Option<String>,
}

impl From<Row> for UserRow {
    fn from(r: Row) -> Self {
        UserRow {
            id: r.id,
            email: r.email,
            full_name: r.full_name,
            role_id: r.role_id,
            role_name: r.role_name,
            status: UserStatus::parse(&r.status).unwrap_or(UserStatus::Active),
            must_change_pw: r.must_change_pw,
            last_login_at: r.last_login_at,
            created_at: r.created_at,
            deleted_at: r.deleted_at,
            deleted_by_email: r.deleted_by_email,
        }
    }
}

pub async fn list(pool: &PgPool, ctx: &Ctx, q: &UserQuery) -> Result<UserPage, AppError> {
    let size = if PAGE_SIZES.contains(&q.page_size) {
        q.page_size
    } else {
        10
    };

    let mut b = QueryBuilder::new("");
    push_base(&mut b, ctx.org_id, q);
    b.push("SELECT * FROM base");
    push_filters(&mut b, q, false, false);
    let order = match q.sort {
        Some((UserSort::Name, d)) => format!("lower(full_name) {}, id", dir(d)),
        Some((UserSort::Email, d)) => format!("lower(email) {}, id", dir(d)),
        Some((UserSort::LastLogin, d)) => format!("last_login_at {} NULLS LAST, id", dir(d)),
        Some((UserSort::Created, d)) => format!("created_at {}, id", dir(d)),
        None if q.deleted => "deleted_at DESC, id".into(),
        None => "created_at DESC, id".into(),
    };
    b.push(" ORDER BY ").push(order); // whitelisted above, never user text
    b.push(" LIMIT ").push_bind(i64::from(size));
    b.push(" OFFSET ")
        .push_bind(i64::from(q.page) * i64::from(size));
    let rows: Vec<Row> = b.build_query_as().fetch_all(pool).await?;

    let mut c = QueryBuilder::new("");
    push_base(&mut c, ctx.org_id, q);
    c.push("SELECT count(*) FROM base");
    push_filters(&mut c, q, false, false);
    let total: i64 = c.build_query_scalar().fetch_one(pool).await?;

    let mut s = QueryBuilder::new("");
    push_base(&mut s, ctx.org_id, q);
    s.push("SELECT status, count(*) FROM base");
    push_filters(&mut s, q, true, false);
    s.push(" GROUP BY status");
    let status_counts: Vec<(String, i64)> = s.build_query_as().fetch_all(pool).await?;

    let mut r = QueryBuilder::new("");
    push_base(&mut r, ctx.org_id, q);
    r.push("SELECT role_id, count(*) FROM base");
    push_filters(&mut r, q, false, true);
    r.push(" GROUP BY role_id");
    let role_counts: Vec<(Uuid, i64)> = r.build_query_as().fetch_all(pool).await?;

    Ok(UserPage {
        rows: rows.into_iter().map(UserRow::from).collect(),
        total: total as u64,
        status_counts: status_counts
            .into_iter()
            .filter_map(|(s, n)| Some((UserStatus::parse(&s)?, n as u64)))
            .collect(),
        role_counts: role_counts
            .into_iter()
            .map(|(id, n)| (id, n as u64))
            .collect(),
    })
}

fn dir(desc: bool) -> &'static str {
    if desc { "DESC" } else { "ASC" }
}

/// One user with sessions and recent audit activity (`users.view`). Deleted users included.
pub async fn detail(
    pool: &PgPool,
    ctx: &Ctx,
    id: Uuid,
) -> Result<crate::domain::users::UserDetail, AppError> {
    ctx.require("users.view")?;
    let both = UserQuery {
        page_size: 10,
        ..Default::default()
    };
    let mut found = None;
    for deleted in [false, true] {
        let q = UserQuery {
            deleted,
            ..both.clone()
        };
        let mut b = QueryBuilder::new("");
        push_base(&mut b, ctx.org_id, &q);
        b.push("SELECT * FROM base WHERE id = ").push_bind(id);
        if let Some(r) = b.build_query_as::<Row>().fetch_optional(pool).await? {
            found = Some(r);
            break;
        }
    }
    let user = UserRow::from(found.ok_or(AppError::NotFound)?);
    let extra = sqlx::query!(
        r#"SELECT timezone, failed_attempts, locked_until, updated_at,
                  (SELECT count(*) FROM sessions s WHERE s.user_id = u.id AND s.kind = 'web'
                     AND s.expires_at > now()) AS "sessions!"
           FROM users u WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await?;
    let activity = sqlx::query!(
        r#"SELECT id, at, actor_email, channel, action, entity_type, entity_id, before, after,
                  host(ip) AS ip, user_agent, success
           FROM audit_logs
           WHERE (org_id = $1 OR org_id IS NULL)
             AND (actor_id = $2 OR (entity_type = 'user' AND entity_id = $3))
           ORDER BY at DESC, id DESC LIMIT 20"#,
        ctx.org_id,
        id,
        id.to_string()
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| crate::domain::audit::AuditRow {
        id: r.id,
        at: r.at,
        actor_email: r.actor_email,
        channel: r.channel,
        action: r.action,
        entity_type: r.entity_type,
        entity_id: r.entity_id,
        before: r.before.map(|v| v.to_string()),
        after: r.after.map(|v| v.to_string()),
        ip: r.ip,
        user_agent: r.user_agent,
        success: r.success,
    })
    .collect();
    Ok(crate::domain::users::UserDetail {
        user,
        timezone: extra.timezone,
        failed_attempts: extra.failed_attempts,
        locked_until: extra.locked_until,
        updated_at: extra.updated_at,
        active_sessions: extra.sessions,
        activity,
    })
}

/// Roles for the filter menu and the user form: the org's roles plus the platform Super Admin.
pub async fn role_options(pool: &PgPool, ctx: &Ctx) -> Result<Vec<RoleOption>, AppError> {
    Ok(sqlx::query_as!(
        RoleOption,
        r#"SELECT id, name, is_system FROM roles
           WHERE org_id = $1 OR org_id IS NULL ORDER BY is_system DESC, name"#,
        ctx.org_id
    )
    .fetch_all(pool)
    .await?)
}

// ---------- changes ----------

fn entry<'a>(ctx: &'a Ctx, actor: &'a CurrentUser, action: &'a str, id: Uuid) -> Entry<'a> {
    Entry {
        org_id: Some(ctx.org_id),
        actor_id: Some(actor.id),
        actor_email: Some(actor.email.as_str()),
        action,
        entity_type: Some("user"),
        entity_id: Some(id.to_string()),
        ip: ctx.meta.ip,
        user_agent: ctx.meta.user_agent.as_deref(),
        success: true,
        ..Default::default()
    }
}

/// The audited view of a user (never the password hash).
async fn snapshot(
    tx: &mut Transaction<'_, Postgres>,
    org: Uuid,
    id: Uuid,
) -> Result<Snap, AppError> {
    sqlx::query_as!(
        Snap,
        r#"SELECT u.id, u.email::text AS "email!", u.full_name, u.role_id, r.name AS role_name,
                  r.is_system, u.is_active, u.must_change_pw, u.deleted_at
           FROM users u JOIN roles r ON r.id = u.role_id
           WHERE u.id = $1 AND u.org_id = $2 FOR UPDATE OF u"#,
        id,
        org
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(AppError::NotFound)
}

#[derive(Debug)]
struct Snap {
    id: Uuid,
    email: String,
    full_name: String,
    role_id: Uuid,
    role_name: String,
    is_system: bool,
    is_active: bool,
    must_change_pw: bool,
    deleted_at: Option<time::OffsetDateTime>,
}

impl Snap {
    fn json(&self) -> Value {
        json!({
            "email": self.email, "full_name": self.full_name, "role": self.role_name,
            "role_id": self.role_id, "is_active": self.is_active,
            "must_change_pw": self.must_change_pw, "deleted": self.deleted_at.is_some(),
        })
    }
    fn is_super_admin(&self) -> bool {
        self.is_system && self.role_name == crate::server::seed::SUPER_ADMIN
    }
}

async fn kill_sessions(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn first_error(errors: Vec<(&'static str, &'static str)>) -> Result<(), AppError> {
    match errors.first() {
        Some((_, code)) => Err(AppError::Validation(code)),
        None => Ok(()),
    }
}

fn unique_violation(e: sqlx::Error) -> AppError {
    match &e {
        sqlx::Error::Database(d) if d.is_unique_violation() => {
            AppError::Conflict("users.email_taken")
        }
        _ => e.into(),
    }
}

pub async fn create(pool: &PgPool, ctx: &Ctx, input: UserInput) -> Result<Uuid, AppError> {
    let actor = ctx.require("users.create")?;
    let min = SecuritySettings::load(pool).await?.password_min_length;
    first_error(validate_input(&input, true, min))?;
    password::check_policy(&input.password, min).map_err(|p| AppError::Validation(p.code()))?;
    let hash = password::hash(input.password.clone()).await?;

    let mut tx = pool.begin().await?;
    rbac::can_assign_role(&mut tx, actor, input.role_id).await?;
    let id = sqlx::query_scalar!(
        r#"INSERT INTO users (org_id, email, full_name, password_hash, role_id, must_change_pw)
           VALUES ($1, $2::text::citext, $3, $4, $5, true) RETURNING id"#,
        ctx.org_id,
        input.email.trim().to_lowercase(),
        input.full_name.trim(),
        hash,
        input.role_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(unique_violation)?;
    let after = snapshot(&mut tx, ctx.org_id, id).await?.json();
    audit::record(
        &mut tx,
        Entry {
            after: Some(after),
            ..entry(ctx, actor, "user.create", id)
        },
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

/// Name, email and role; a non-empty password also resets it (needs `users.reset_password`).
pub async fn update(pool: &PgPool, ctx: &Ctx, id: Uuid, input: UserInput) -> Result<(), AppError> {
    let actor = ctx.require("users.update")?;
    let min = SecuritySettings::load(pool).await?.password_min_length;
    first_error(validate_input(&input, false, min))?;

    let mut tx = pool.begin().await?;
    let before = snapshot(&mut tx, ctx.org_id, id).await?;
    if before.deleted_at.is_some() {
        return Err(AppError::NotFound);
    }
    let role_changed = before.role_id != input.role_id;
    if role_changed {
        rbac::can_assign_role(&mut tx, actor, input.role_id).await?;
        if before.is_super_admin() {
            if !actor.is_super_admin {
                return Err(AppError::Forbidden);
            }
            rbac::keep_last_super_admin(&mut tx, id).await?;
        }
    }
    sqlx::query!(
        r#"UPDATE users SET full_name = $2, email = $3::text::citext, role_id = $4, updated_at = now()
           WHERE id = $1"#,
        id,
        input.full_name.trim(),
        input.email.trim().to_lowercase(),
        input.role_id
    )
    .execute(&mut *tx)
    .await
    .map_err(unique_violation)?;
    if role_changed {
        kill_sessions(&mut tx, id).await?; // new permissions take effect now
    }
    let after = snapshot(&mut tx, ctx.org_id, id).await?;
    audit::record(
        &mut tx,
        Entry {
            before: Some(before.json()),
            after: Some(after.json()),
            ..entry(ctx, actor, "user.update", id)
        },
    )
    .await?;
    if !input.password.is_empty() {
        if !actor.permissions.contains("users.reset_password") {
            return Err(AppError::Forbidden);
        }
        set_password(&mut tx, ctx, actor, id, &input.password, min).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn set_password(
    tx: &mut Transaction<'_, Postgres>,
    ctx: &Ctx,
    actor: &CurrentUser,
    id: Uuid,
    pw: &str,
    min: usize,
) -> Result<(), AppError> {
    password::check_policy(pw, min).map_err(|p| AppError::Validation(p.code()))?;
    let hash = password::hash(pw.to_string()).await?;
    sqlx::query!(
        "UPDATE users SET password_hash = $2, must_change_pw = true, failed_attempts = 0,
                locked_until = NULL, updated_at = now() WHERE id = $1",
        id,
        hash
    )
    .execute(&mut **tx)
    .await?;
    kill_sessions(tx, id).await?;
    audit::record(tx, entry(ctx, actor, "user.reset_password", id)).await?;
    Ok(())
}

/// One user, one action, own transaction: the unit both single and bulk actions use.
pub async fn apply(
    pool: &PgPool,
    ctx: &Ctx,
    action: BulkUserAction,
    id: Uuid,
) -> Result<(), AppError> {
    let actor = ctx.require(match action {
        BulkUserAction::Activate | BulkUserAction::Deactivate => "users.update",
        BulkUserAction::Delete => "users.delete",
        BulkUserAction::Restore => "users.restore",
    })?;
    let mut tx = pool.begin().await?;
    let before = snapshot(&mut tx, ctx.org_id, id).await?;
    let deleted = before.deleted_at.is_some();
    let action_name = match action {
        BulkUserAction::Activate => {
            if deleted {
                return Err(AppError::NotFound);
            }
            if before.is_active {
                return Ok(()); // already active: nothing to do, nothing to audit
            }
            sqlx::query!(
                "UPDATE users SET is_active = true, failed_attempts = 0, locked_until = NULL,
                        updated_at = now() WHERE id = $1",
                id
            )
            .execute(&mut *tx)
            .await?;
            "user.activate"
        }
        BulkUserAction::Deactivate => {
            if deleted {
                return Err(AppError::NotFound);
            }
            rbac::not_self(actor, id)?;
            if !before.is_active {
                return Ok(());
            }
            protect_super_admin(&mut tx, actor, &before).await?;
            sqlx::query!(
                "UPDATE users SET is_active = false, updated_at = now() WHERE id = $1",
                id
            )
            .execute(&mut *tx)
            .await?;
            kill_sessions(&mut tx, id).await?;
            "user.deactivate"
        }
        BulkUserAction::Delete => {
            if deleted {
                return Ok(());
            }
            rbac::not_self(actor, id)?;
            protect_super_admin(&mut tx, actor, &before).await?;
            sqlx::query!(
                "UPDATE users SET deleted_at = now(), deleted_by = $2, is_active = false, updated_at = now()
                 WHERE id = $1",
                id,
                actor.id
            )
            .execute(&mut *tx)
            .await?;
            kill_sessions(&mut tx, id).await?;
            "user.delete"
        }
        BulkUserAction::Restore => {
            if !deleted {
                return Ok(());
            }
            // The email may have been reused while this user was deleted (§25.3).
            let taken: bool = sqlx::query_scalar!(
                r#"SELECT EXISTS (SELECT 1 FROM users WHERE org_id = $1 AND email = $2::text::citext
                                  AND deleted_at IS NULL AND id <> $3) AS "e!""#,
                ctx.org_id,
                before.email,
                id
            )
            .fetch_one(&mut *tx)
            .await?;
            if taken {
                return Err(AppError::Conflict("users.email_taken"));
            }
            sqlx::query!(
                "UPDATE users SET deleted_at = NULL, deleted_by = NULL, is_active = false,
                        must_change_pw = true, updated_at = now() WHERE id = $1",
                id
            )
            .execute(&mut *tx)
            .await?;
            "user.restore"
        }
    };
    let after = snapshot(&mut tx, ctx.org_id, id).await?;
    audit::record(
        &mut tx,
        Entry {
            before: Some(before.json()),
            after: Some(after.json()),
            ..entry(ctx, actor, action_name, id)
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Only a Super Admin may deactivate/delete a Super Admin, and never the last active one.
async fn protect_super_admin(
    tx: &mut Transaction<'_, Postgres>,
    actor: &CurrentUser,
    target: &Snap,
) -> Result<(), AppError> {
    if target.is_super_admin() {
        if !actor.is_super_admin {
            return Err(AppError::Forbidden);
        }
        rbac::keep_last_super_admin(tx, target.id).await?;
    }
    Ok(())
}

/// Admin sets a new temporary password (users.reset_password).
pub async fn reset_password(
    pool: &PgPool,
    ctx: &Ctx,
    id: Uuid,
    pw: String,
) -> Result<(), AppError> {
    let actor = ctx.require("users.reset_password")?;
    let min = SecuritySettings::load(pool).await?.password_min_length;
    let mut tx = pool.begin().await?;
    let s = snapshot(&mut tx, ctx.org_id, id).await?;
    if s.deleted_at.is_some() {
        return Err(AppError::NotFound);
    }
    if s.is_super_admin() && !actor.is_super_admin {
        return Err(AppError::Forbidden);
    }
    set_password(&mut tx, ctx, actor, id, &pw, min).await?;
    tx.commit().await?;
    Ok(())
}

/// Permanently remove a soft-deleted user; the caller re-types the email to confirm.
pub async fn purge(
    pool: &PgPool,
    ctx: &Ctx,
    id: Uuid,
    confirm_email: &str,
) -> Result<(), AppError> {
    let actor = ctx.require("users.purge")?;
    let mut tx = pool.begin().await?;
    let s = snapshot(&mut tx, ctx.org_id, id).await?;
    if s.deleted_at.is_none() {
        return Err(AppError::Conflict("users.purge_not_deleted"));
    }
    if !s.email.eq_ignore_ascii_case(confirm_email.trim()) {
        return Err(AppError::Validation("users.purge_confirm"));
    }
    // References without ON DELETE rules: keep the rows, drop the link.
    sqlx::query!(
        "UPDATE settings SET updated_by = NULL WHERE updated_by = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE backups SET created_by = NULL WHERE created_by = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    audit::record(
        &mut tx,
        Entry {
            before: Some(s.json()),
            ..entry(ctx, actor, "user.purge", id)
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Bulk (§25.1): capped by `bulk.max_items`; every item re-checks its own guards; failures are
/// reported per id with their code instead of aborting the batch; one summary audit row.
pub async fn bulk(
    pool: &PgPool,
    ctx: &Ctx,
    action: BulkUserAction,
    ids: Vec<Uuid>,
) -> Result<BulkResult, AppError> {
    let actor = ctx.user.clone().ok_or(AppError::Unauthenticated)?;
    let max = settings::value(pool, "bulk.max_items")
        .await
        .as_u64()
        .unwrap_or(1000) as usize;
    if ids.len() > max {
        return Err(AppError::Validation("bulk.too_many"));
    }
    let mut res = BulkResult::default();
    for id in ids.iter().copied() {
        match apply(pool, ctx, action, id).await {
            Ok(()) => res.done += 1,
            Err(AppError::Internal(e)) => return Err(AppError::Internal(e)),
            Err(e) => res.skipped.push((id, e.code().to_string())),
        }
    }
    let mut tx = pool.begin().await?;
    let name = match action {
        BulkUserAction::Activate => "user.bulk_activate",
        BulkUserAction::Deactivate => "user.bulk_deactivate",
        BulkUserAction::Delete => "user.bulk_delete",
        BulkUserAction::Restore => "user.bulk_restore",
    };
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: name,
            entity_type: Some("user"),
            after: Some(
                json!({ "requested": ids.len(), "done": res.done, "skipped": res.skipped.len() }),
            ),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(res)
}

/// Rows for a PDF report: the list's filters (and optional `ids`), newest first, at most `max`.
pub async fn report_rows(
    pool: &PgPool,
    org: Uuid,
    q: &UserQuery,
    ids: &[Uuid],
    max: i64,
) -> Result<Vec<UserRow>, AppError> {
    let mut b = QueryBuilder::new("");
    push_base(&mut b, org, q);
    b.push("SELECT * FROM base");
    push_filters(&mut b, q, false, false);
    if !ids.is_empty() {
        b.push(" AND id = ANY(").push_bind(ids.to_vec()).push(")");
    }
    b.push(" ORDER BY lower(full_name), id LIMIT ")
        .push_bind(max);
    let rows: Vec<Row> = b.build_query_as().fetch_all(pool).await?;
    Ok(rows.into_iter().map(UserRow::from).collect())
}

// ---------- CSV export (§25.2) ----------

/// `GET /api/export/users.csv?q=&status=a,b&role=<uuid>,..&deleted=1&sort=email&desc=1&ids=..`
/// — the same filters as the list. Needs `users.export`; audited as `export.users`.
pub async fn export_route(
    dioxus::server::axum::Extension(ctx): dioxus::server::axum::Extension<Ctx>,
    dioxus::server::axum::extract::Query(p): dioxus::server::axum::extract::Query<
        std::collections::HashMap<String, String>,
    >,
) -> dioxus::server::axum::response::Response {
    match export(&ctx, &p).await {
        Ok(r) => r,
        Err(e) => crate::server::export::error_response(e),
    }
}

pub fn query_from_params(p: &std::collections::HashMap<String, String>) -> (UserQuery, Vec<Uuid>) {
    let list = |k: &str| -> Vec<String> {
        p.get(k)
            .map(|v| {
                v.split(',')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let q = UserQuery {
        q: p.get("q").cloned().unwrap_or_default(),
        statuses: list("status")
            .iter()
            .filter_map(|s| UserStatus::parse(s))
            .collect(),
        role_ids: list("role").iter().filter_map(|s| s.parse().ok()).collect(),
        sort: p
            .get("sort")
            .and_then(|s| UserSort::parse(s))
            .map(|s| (s, p.get("desc").is_some_and(|d| d == "1"))),
        page: 0,
        page_size: 10,
        deleted: p.get("deleted").is_some_and(|d| d == "1"),
    };
    let ids = list("ids").iter().filter_map(|s| s.parse().ok()).collect();
    (q, ids)
}

async fn export(
    ctx: &Ctx,
    p: &std::collections::HashMap<String, String>,
) -> Result<dioxus::server::axum::response::Response, AppError> {
    let actor = ctx.require("users.export")?.clone();
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let (q, ids) = query_from_params(p);
    let max = settings::value(pool, "export.max_rows")
        .await
        .as_i64()
        .unwrap_or(50_000);

    let mut c = QueryBuilder::new("");
    push_base(&mut c, ctx.org_id, &q);
    c.push("SELECT count(*) FROM base");
    push_filters(&mut c, &q, false, false);
    if !ids.is_empty() {
        c.push(" AND id = ANY(").push_bind(ids.clone()).push(")");
    }
    let total: i64 = c.build_query_scalar().fetch_one(pool).await?;
    let rows = total.min(max);

    let mut tx = pool.begin().await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "export.users",
            entity_type: Some("user"),
            after: Some(json!({ "filters": p, "rows": rows, "truncated": total > max })),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;

    let (send, recv) = tokio::sync::mpsc::channel::<String>(64);
    let org = ctx.org_id;
    tokio::spawn(async move {
        use crate::server::export::line;
        use futures_util::TryStreamExt;
        let fmt = |t: Option<time::OffsetDateTime>| {
            t.and_then(|t| {
                t.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            })
            .unwrap_or_default()
        };
        let header = line(&[
            "name",
            "email",
            "role",
            "status",
            "must_change_password",
            "last_sign_in_utc",
            "created_utc",
            "deleted_utc",
            "deleted_by",
        ]);
        if send.send(format!("\u{feff}{header}")).await.is_err() {
            return;
        }
        let mut b = QueryBuilder::new("");
        push_base(&mut b, org, &q);
        b.push("SELECT * FROM base");
        push_filters(&mut b, &q, false, false);
        if !ids.is_empty() {
            b.push(" AND id = ANY(").push_bind(ids).push(")");
        }
        b.push(" ORDER BY created_at DESC, id LIMIT ")
            .push_bind(max);
        let mut stream = b.build_query_as::<Row>().fetch(pool);
        loop {
            match stream.try_next().await {
                Ok(Some(r)) => {
                    let l = line(&[
                        &r.full_name,
                        &r.email,
                        &r.role_name,
                        &r.status,
                        if r.must_change_pw { "true" } else { "false" },
                        &fmt(r.last_login_at),
                        &fmt(Some(r.created_at)),
                        &fmt(r.deleted_at),
                        r.deleted_by_email.as_deref().unwrap_or(""),
                    ]);
                    if send.send(l).await.is_err() {
                        return; // client went away
                    }
                }
                Ok(None) => return,
                Err(e) => {
                    tracing::error!(error = %e, "users export stream failed");
                    return;
                }
            }
        }
    });
    Ok(crate::server::export::csv_response(
        "users.csv",
        total > max,
        recv,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{rbac, request::RequestMeta, seed};

    struct Fx {
        pool: PgPool,
        org: Uuid,
        admin_role: Uuid,
        editor_role: Uuid,
    }

    async fn fx(pool: PgPool) -> Fx {
        rbac::sync_permissions(&pool).await.unwrap();
        settings::seed(&pool).await.unwrap();
        let org = seed::ensure_default_org(&pool).await.unwrap();
        let admin_role = seed::ensure_super_admin_role(&pool).await.unwrap();
        let editor_role: Uuid = sqlx::query_scalar(
            "INSERT INTO roles (org_id, name) VALUES ($1, 'Editor') RETURNING id",
        )
        .bind(org)
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO role_permissions SELECT $1, key FROM permissions WHERE key LIKE 'users.%'",
        )
        .bind(editor_role)
        .execute(&pool)
        .await
        .unwrap();
        Fx {
            pool,
            org,
            admin_role,
            editor_role,
        }
    }

    async fn insert(f: &Fx, email: &str, role: Uuid) -> Uuid {
        sqlx::query_scalar("INSERT INTO users (org_id, email, full_name, password_hash, role_id) VALUES ($1, $2, $3, 'x', $4) RETURNING id")
            .bind(f.org)
            .bind(email)
            .bind(email.split('@').next().unwrap())
            .bind(role)
            .fetch_one(&f.pool)
            .await
            .unwrap()
    }

    fn ctx_for(f: &Fx, id: Uuid, super_admin: bool) -> Ctx {
        let permissions = if super_admin {
            rbac::all_keys()
        } else {
            [
                "users.view",
                "users.create",
                "users.update",
                "users.delete",
                "users.restore",
                "users.purge",
                "users.reset_password",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect()
        };
        Ctx {
            org_id: f.org,
            user: Some(CurrentUser {
                id,
                session_id: Uuid::new_v4(),
                org_id: f.org,
                email: "actor@test.local".into(),
                full_name: "Actor".into(),
                role_id: if super_admin {
                    f.admin_role
                } else {
                    f.editor_role
                },
                role_name: if super_admin {
                    seed::SUPER_ADMIN.into()
                } else {
                    "Editor".into()
                },
                is_super_admin: super_admin,
                permissions,
                must_change_pw: false,
                locale: None,
                timezone: None,
            }),
            request_id: None,
            meta: RequestMeta::default(),
        }
    }

    fn input(email: &str, role: Uuid) -> UserInput {
        UserInput {
            full_name: "New Person".into(),
            email: email.into(),
            role_id: role,
            password: "a-long-password-1".into(),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn list_filters_facets_and_pages(pool: PgPool) {
        let f = fx(pool).await;
        let admin = insert(&f, "root@test.local", f.admin_role).await;
        for i in 0..12 {
            insert(&f, &format!("e{i}@test.local"), f.editor_role).await;
        }
        sqlx::query("UPDATE users SET is_active = false WHERE email = 'e0@test.local'")
            .execute(&f.pool)
            .await
            .unwrap();
        let ctx = ctx_for(&f, admin, true);
        let page = list(
            &f.pool,
            &ctx,
            &UserQuery {
                page_size: 10,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!((page.rows.len(), page.total), (10, 13));
        let q = UserQuery {
            page_size: 10,
            statuses: vec![UserStatus::Inactive],
            ..Default::default()
        };
        let p = list(&f.pool, &ctx, &q).await.unwrap();
        assert_eq!(p.total, 1);
        // the status facet ignores the status filter itself
        let invited = p
            .status_counts
            .iter()
            .find(|(s, _)| *s == UserStatus::Invited)
            .map(|x| x.1);
        assert_eq!(invited, Some(12));
        let q = UserQuery {
            q: "E1".into(),
            page_size: 10,
            ..Default::default()
        };
        assert_eq!(list(&f.pool, &ctx, &q).await.unwrap().total, 3); // e1, e10, e11
        let q = UserQuery {
            q: "%".into(),
            page_size: 10,
            ..Default::default()
        };
        assert_eq!(
            list(&f.pool, &ctx, &q).await.unwrap().total,
            0,
            "LIKE wildcards are escaped"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_update_audit_and_email_conflict(pool: PgPool) {
        let f = fx(pool).await;
        let admin = insert(&f, "root@test.local", f.admin_role).await;
        let ctx = ctx_for(&f, admin, true);
        let id = create(&f.pool, &ctx, input("x@test.local", f.editor_role))
            .await
            .unwrap();
        let dup = create(&f.pool, &ctx, input("X@test.local", f.editor_role)).await;
        assert!(matches!(dup, Err(AppError::Conflict("users.email_taken"))));
        update(
            &f.pool,
            &ctx,
            id,
            UserInput {
                full_name: "Renamed".into(),
                password: String::new(),
                ..input("x@test.local", f.editor_role)
            },
        )
        .await
        .unwrap();
        let actions: Vec<String> = sqlx::query_scalar("SELECT action FROM audit_logs ORDER BY id")
            .fetch_all(&f.pool)
            .await
            .unwrap();
        assert_eq!(actions, ["user.create", "user.update"]);
        let pw_logged: bool =
            sqlx::query_scalar("SELECT bool_or(after::text ILIKE '%password%') FROM audit_logs")
                .fetch_one(&f.pool)
                .await
                .unwrap();
        assert!(!pw_logged, "audit never contains passwords");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn editor_cannot_create_super_admin(pool: PgPool) {
        let f = fx(pool).await;
        let me = insert(&f, "ed@test.local", f.editor_role).await;
        let ctx = ctx_for(&f, me, false);
        let r = create(&f.pool, &ctx, input("y@test.local", f.admin_role)).await;
        assert!(matches!(r, Err(AppError::Forbidden)));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn guards_and_bulk_skip_reasons(pool: PgPool) {
        let f = fx(pool).await;
        let admin = insert(&f, "root@test.local", f.admin_role).await;
        let other = insert(&f, "o@test.local", f.editor_role).await;
        let ctx = ctx_for(&f, admin, true);
        let r = bulk(
            &f.pool,
            &ctx,
            BulkUserAction::Deactivate,
            vec![admin, other],
        )
        .await
        .unwrap();
        assert_eq!(r.done, 1);
        assert_eq!(r.skipped, vec![(admin, "users.self_action".to_string())]);
        // a second admin can't remove the only Super Admin either
        let admin2 = insert(&f, "root2@test.local", f.editor_role).await;
        let ctx2 = ctx_for(&f, admin2, true);
        let r = apply(&f.pool, &ctx2, BulkUserAction::Delete, admin).await;
        assert!(matches!(
            r,
            Err(AppError::Conflict("users.last_super_admin"))
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_restore_and_purge(pool: PgPool) {
        let f = fx(pool).await;
        let admin = insert(&f, "root@test.local", f.admin_role).await;
        let ctx = ctx_for(&f, admin, true);
        let gone = insert(&f, "gone@test.local", f.editor_role).await;
        apply(&f.pool, &ctx, BulkUserAction::Delete, gone)
            .await
            .unwrap();
        // email reused while deleted → restore refused
        let reuse = create(&f.pool, &ctx, input("gone@test.local", f.editor_role))
            .await
            .unwrap();
        let r = apply(&f.pool, &ctx, BulkUserAction::Restore, gone).await;
        assert!(matches!(r, Err(AppError::Conflict("users.email_taken"))));
        apply(&f.pool, &ctx, BulkUserAction::Delete, reuse)
            .await
            .unwrap();
        // the purged user had acted: their audit rows stay (actor_id → NULL, email kept)
        sqlx::query("INSERT INTO audit_logs (actor_id, actor_email, action) VALUES ($1, 'gone@test.local', 'x.y')")
            .bind(reuse)
            .execute(&f.pool)
            .await
            .unwrap();
        assert!(matches!(
            purge(&f.pool, &ctx, reuse, "wrong@test.local").await,
            Err(AppError::Validation("users.purge_confirm"))
        ));
        purge(&f.pool, &ctx, reuse, "gone@test.local")
            .await
            .unwrap();
        let kept: Option<String> = sqlx::query_scalar(
            "SELECT actor_email FROM audit_logs WHERE action = 'x.y' AND actor_id IS NULL",
        )
        .fetch_one(&f.pool)
        .await
        .unwrap();
        assert_eq!(kept.as_deref(), Some("gone@test.local"));
        apply(&f.pool, &ctx, BulkUserAction::Restore, gone)
            .await
            .unwrap();
        let (active, must): (bool, bool) =
            sqlx::query_as("SELECT is_active, must_change_pw FROM users WHERE id = $1")
                .bind(gone)
                .fetch_one(&f.pool)
                .await
                .unwrap();
        assert!(
            !active && must,
            "restored users come back inactive and must change password"
        );
    }
}
