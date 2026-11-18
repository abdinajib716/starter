//! Audit log page (§9.5): filtered list with keyset pagination on `(at, id)` and a streamed
//! CSV export. Reading needs `audit.view`, exporting `audit.export` (and is itself audited).
use std::collections::HashMap;

use dioxus::server::axum::{Extension, extract::Query, response::Response};
use serde_json::json;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::domain::audit::{AuditCursor, AuditPage, AuditQuery, AuditRow};
use crate::server::audit::{self, Entry};
use crate::server::ctx::Ctx;
use crate::server::error::AppError;
use crate::server::export::{csv_response, error_response, line};
use crate::server::settings;

const MAX_LIMIT: u32 = 100;

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// `FROM audit_logs WHERE …` for the filters (org-scoped; `org_id IS NULL` = system rows).
fn push_where(b: &mut QueryBuilder<Postgres>, org: uuid::Uuid, q: &AuditQuery) {
    b.push(" FROM audit_logs WHERE (org_id = ")
        .push_bind(org)
        .push(" OR org_id IS NULL)");
    let term = q.q.trim();
    if !term.is_empty() {
        let like = format!("%{}%", escape_like(term));
        b.push(" AND (action ILIKE ").push_bind(like.clone());
        b.push(" OR actor_email ILIKE ").push_bind(like.clone());
        b.push(" OR entity_id ILIKE ").push_bind(like).push(")");
    }
    if !q.modules.is_empty() {
        b.push(" AND split_part(action, '.', 1) = ANY(")
            .push_bind(q.modules.clone())
            .push(")");
    }
    if let Some(ok) = q.success {
        b.push(" AND success = ").push_bind(ok);
    }
    if let Some(d) = q.from {
        b.push(" AND at >= ").push_bind(d.midnight().assume_utc());
    }
    if let Some(d) = q.to {
        b.push(" AND at < ")
            .push_bind(d.midnight().assume_utc() + time::Duration::days(1));
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    at: time::OffsetDateTime,
    actor_email: Option<String>,
    channel: String,
    action: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
    ip: Option<String>,
    user_agent: Option<String>,
    success: bool,
}

const COLUMNS: &str = "SELECT id, at, actor_email, channel, action, entity_type, entity_id, before, after, host(ip) AS ip, user_agent, success";

impl From<Row> for AuditRow {
    fn from(r: Row) -> Self {
        AuditRow {
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
        }
    }
}

pub async fn list(pool: &PgPool, ctx: &Ctx, q: &AuditQuery) -> Result<AuditPage, AppError> {
    ctx.require("audit.view")?;
    let limit = q.limit.clamp(1, MAX_LIMIT);
    let mut b = QueryBuilder::new(COLUMNS);
    push_where(&mut b, ctx.org_id, q);
    if let Some(c) = q.before {
        b.push(" AND (at, id) < (")
            .push_bind(c.at)
            .push(", ")
            .push_bind(c.id)
            .push(")");
    }
    b.push(" ORDER BY at DESC, id DESC LIMIT ")
        .push_bind(i64::from(limit) + 1);
    let mut rows: Vec<Row> = b.build_query_as().fetch_all(pool).await?;
    let more = rows.len() > limit as usize;
    rows.truncate(limit as usize);
    let next = more
        .then(|| rows.last().map(|r| AuditCursor { at: r.at, id: r.id }))
        .flatten();
    let modules = sqlx::query_scalar!(
        r#"SELECT DISTINCT split_part(action, '.', 1) AS "m!" FROM audit_logs
           WHERE org_id = $1 OR org_id IS NULL ORDER BY 1"#,
        ctx.org_id
    )
    .fetch_all(pool)
    .await?;
    Ok(AuditPage {
        rows: rows.into_iter().map(AuditRow::from).collect(),
        next,
        modules,
    })
}

/// Rows for a PDF report: the page's filters, newest first, at most `max`.
pub async fn report_rows(
    pool: &PgPool,
    org: uuid::Uuid,
    q: &AuditQuery,
    max: i64,
) -> Result<Vec<AuditRow>, AppError> {
    let mut b = QueryBuilder::new(COLUMNS);
    push_where(&mut b, org, q);
    b.push(" ORDER BY at DESC, id DESC LIMIT ").push_bind(max);
    let rows: Vec<Row> = b.build_query_as().fetch_all(pool).await?;
    Ok(rows.into_iter().map(AuditRow::from).collect())
}

pub fn query_from_params(p: &HashMap<String, String>) -> AuditQuery {
    let date = |k: &str| {
        p.get(k).and_then(|v| {
            time::Date::parse(
                v,
                &time::macros::format_description!("[year]-[month]-[day]"),
            )
            .ok()
        })
    };
    AuditQuery {
        q: p.get("q").cloned().unwrap_or_default(),
        modules: p
            .get("module")
            .map(|v| {
                v.split(',')
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        success: p.get("result").and_then(|v| match v.as_str() {
            "success" => Some(true),
            "failed" => Some(false),
            _ => None,
        }),
        from: date("from"),
        to: date("to"),
        before: None,
        limit: MAX_LIMIT,
    }
}

/// `GET /api/export/audit.csv?<same filters as the page>` (`audit.export`, audited).
pub async fn export_route(
    Extension(ctx): Extension<Ctx>,
    Query(p): Query<HashMap<String, String>>,
) -> Response {
    match export(&ctx, &p).await {
        Ok(r) => r,
        Err(e) => error_response(e),
    }
}

async fn export(ctx: &Ctx, p: &HashMap<String, String>) -> Result<Response, AppError> {
    let actor = ctx.require("audit.export")?.clone();
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let q = query_from_params(p);
    let max = settings::value(pool, "export.max_rows")
        .await
        .as_i64()
        .unwrap_or(50_000);
    let mut c = QueryBuilder::new("SELECT count(*)");
    push_where(&mut c, ctx.org_id, &q);
    let total: i64 = c.build_query_scalar().fetch_one(pool).await?;

    let mut tx = pool.begin().await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "export.audit",
            entity_type: Some("audit_log"),
            after: Some(json!({ "filters": p, "rows": total.min(max), "truncated": total > max })),
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
        use futures_util::TryStreamExt;
        let header = line(&[
            "at_utc",
            "actor",
            "channel",
            "action",
            "entity_type",
            "entity_id",
            "success",
            "ip",
            "before",
            "after",
        ]);
        if send.send(format!("\u{feff}{header}")).await.is_err() {
            return;
        }
        let mut b = QueryBuilder::new(COLUMNS);
        push_where(&mut b, org, &q);
        b.push(" ORDER BY at DESC, id DESC LIMIT ").push_bind(max);
        let mut rows = b.build_query_as::<Row>().fetch(pool);
        while let Ok(Some(r)) = rows.try_next().await {
            let at =
                r.at.format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default();
            let (before, after) = (
                r.before.map(|v| v.to_string()),
                r.after.map(|v| v.to_string()),
            );
            let l = line(&[
                &at,
                r.actor_email.as_deref().unwrap_or(""),
                &r.channel,
                &r.action,
                r.entity_type.as_deref().unwrap_or(""),
                r.entity_id.as_deref().unwrap_or(""),
                if r.success { "true" } else { "false" },
                r.ip.as_deref().unwrap_or(""),
                before.as_deref().unwrap_or(""),
                after.as_deref().unwrap_or(""),
            ]);
            if send.send(l).await.is_err() {
                return;
            }
        }
    });
    Ok(csv_response("audit.csv", total > max, recv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{rbac, request::RequestMeta};

    fn ctx(org: uuid::Uuid) -> Ctx {
        Ctx {
            org_id: org,
            user: Some(crate::server::ctx::CurrentUser {
                id: uuid::Uuid::new_v4(),
                session_id: uuid::Uuid::new_v4(),
                org_id: org,
                email: "a@test.local".into(),
                full_name: "A".into(),
                role_id: uuid::Uuid::new_v4(),
                role_name: "Super Admin".into(),
                is_super_admin: true,
                permissions: rbac::all_keys(),
                must_change_pw: false,
                locale: None,
                timezone: None,
                twofa_enrolled: false,
                role_require_2fa: false,
                twofa_since: time::OffsetDateTime::UNIX_EPOCH,
            }),
            request_id: None,
            meta: RequestMeta::default(),
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn keyset_pages_and_filters(pool: PgPool) {
        let org = crate::server::seed::ensure_default_org(&pool)
            .await
            .unwrap();
        for i in 0..7 {
            sqlx::query("INSERT INTO audit_logs (org_id, action, success, at) VALUES ($1, $2, $3, now() - make_interval(mins => $4))")
                .bind(org)
                .bind(if i % 2 == 0 { "auth.login" } else { "user.update" })
                .bind(i != 3)
                .bind(i)
                .execute(&pool)
                .await
                .unwrap();
        }
        let c = ctx(org);
        let mut q = AuditQuery {
            limit: 3,
            ..Default::default()
        };
        let p1 = list(&pool, &c, &q).await.unwrap();
        assert_eq!(p1.rows.len(), 3);
        q.before = p1.next;
        let p2 = list(&pool, &c, &q).await.unwrap();
        q.before = p2.next;
        let p3 = list(&pool, &c, &q).await.unwrap();
        assert_eq!((p2.rows.len(), p3.rows.len(), p3.next), (3, 1, None));
        let ids: Vec<i64> = [&p1, &p2, &p3]
            .iter()
            .flat_map(|p| p.rows.iter().map(|r| r.id))
            .collect();
        let mut sorted = ids.clone();
        sorted.dedup();
        assert_eq!(sorted.len(), 7, "no row repeated or skipped across pages");
        let f = list(
            &pool,
            &c,
            &AuditQuery {
                limit: 50,
                modules: vec!["user".into()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(f.rows.len(), 3);
        let failed = list(
            &pool,
            &c,
            &AuditQuery {
                limit: 50,
                success: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(failed.rows.len(), 1);
        assert_eq!(f.modules, vec!["auth".to_string(), "user".to_string()]);
    }
}
