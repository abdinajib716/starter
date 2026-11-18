//! PDF reports: the lists (users, audit log, roles, backups) as printable HTML, rendered by
//! headless Chrome (`server::pdf`). Same filters, permissions and audit as the CSV exports;
//! dates in the viewer's time zone. The HTML is built here (no SPA, no network fetches), so a
//! report renders in a few hundred milliseconds.
use std::collections::HashMap;

use dioxus::server::axum::{
    Extension,
    body::Body,
    extract::Query,
    http::{HeaderValue, header},
    response::Response,
};
use serde_json::json;

use crate::server::audit::{self, Entry};
use crate::server::ctx::Ctx;
use crate::server::error::AppError;
use crate::server::export::error_response;
use crate::server::modules::Modules;
use crate::server::pdf::{PrintOptions, render};
use crate::util::time::{datetime, datetime_secs};

/// Upper bound per PDF (a report, not a data dump: use CSV for more).
const MAX_ROWS: i64 = 2000;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub struct Report {
    pub title: String,
    /// "Filters: status = active · role = Editor"
    pub subtitle: String,
    pub columns: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
    pub landscape: bool,
    pub truncated: bool,
}

/// Print stylesheet + table. System font stack: no font download, identical everywhere.
pub fn html(r: &Report, app: &str, generated: &str) -> String {
    let head: String = r
        .columns
        .iter()
        .map(|c| format!("<th>{}</th>", esc(c)))
        .collect();
    let body: String = if r.rows.is_empty() {
        format!(
            "<tr><td colspan=\"{}\" class=\"empty\">No rows match these filters.</td></tr>",
            r.columns.len()
        )
    } else {
        r.rows
            .iter()
            .map(|row| {
                format!(
                    "<tr>{}</tr>",
                    row.iter()
                        .map(|c| format!("<td>{}</td>", esc(c)))
                        .collect::<String>()
                )
            })
            .collect()
    };
    let note = if r.truncated {
        format!(
            "<p class=\"note\">Only the first {MAX_ROWS} rows are included. Use CSV export for the full list.</p>"
        )
    } else {
        String::new()
    };
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>{title}</title><style>
@page {{ size: A4 {orientation}; }}
* {{ box-sizing: border-box; }}
body {{ margin: 0; font: 10px/1.45 ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif; color: #0f172a; }}
.top {{ display: flex; justify-content: space-between; align-items: flex-end; border-bottom: 2px solid #0f172a; padding-bottom: 8px; margin-bottom: 10px; }}
.brand {{ font-size: 11px; font-weight: 600; color: #475569; }}
h1 {{ margin: 2px 0 0; font-size: 18px; }}
.meta {{ text-align: right; color: #475569; font-size: 9px; }}
.sub {{ color: #475569; margin: 0 0 10px; }}
table {{ width: 100%; border-collapse: collapse; }}
thead {{ display: table-header-group; }}
tr {{ break-inside: avoid; }}
th {{ text-align: left; font-weight: 600; background: #f1f5f9; border-bottom: 1px solid #cbd5e1; padding: 5px 6px; }}
td {{ border-bottom: 1px solid #e2e8f0; padding: 4px 6px; vertical-align: top; word-break: break-word; }}
tbody tr:nth-child(even) td {{ background: #f8fafc; }}
.empty {{ text-align: center; color: #64748b; padding: 24px; }}
.note {{ margin-top: 8px; color: #b45309; }}
</style></head><body>
<div class="top"><div><div class="brand">{app}</div><h1>{title}</h1></div>
<div class="meta">Generated {generated}<br>{count} rows</div></div>
<p class="sub">{subtitle}</p>
<table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>{note}
</body></html>"#,
        title = esc(&r.title),
        orientation = if r.landscape { "landscape" } else { "portrait" },
        app = esc(app),
        generated = esc(generated),
        count = r.rows.len(),
        subtitle = esc(&r.subtitle),
    )
}

fn footer(app: &str, title: &str) -> String {
    format!(
        r#"<div style="font-size:8px;color:#64748b;width:100%;padding:0 0.4in;display:flex;justify-content:space-between;">
<span>{} · {}</span><span>Page <span class="pageNumber"></span> / <span class="totalPages"></span></span></div>"#,
        esc(app),
        esc(title)
    )
}

/// Viewer's zone: own, else the organization default.
fn zone(ctx: &Ctx) -> String {
    ctx.user
        .as_ref()
        .and_then(|u| u.timezone.clone())
        .unwrap_or_else(Modules::app_timezone)
}

async fn respond(
    ctx: &Ctx,
    report: Report,
    filename: &str,
    action: &'static str,
    entity: &'static str,
    filters: &HashMap<String, String>,
) -> Result<Response, AppError> {
    let st = crate::server::state::get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let actor = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    let app = Modules::get().app_name();
    let tz = zone(ctx);
    let now = time::OffsetDateTime::now_utc();
    let generated = format!(
        "{} ({})",
        datetime(now, &tz),
        crate::util::time::zone_label(&tz)
    );
    let doc = html(&report, &app, &generated);
    let started = std::time::Instant::now();
    let pdf = render(
        &st.config.chrome_url,
        &doc,
        &PrintOptions {
            landscape: report.landscape,
            header: "<span></span>".into(),
            footer: footer(&app, &report.title),
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, "PDF render failed");
        AppError::Conflict("pdf.unavailable")
    })?;
    tracing::info!(
        ms = started.elapsed().as_millis() as u64,
        bytes = pdf.len(),
        rows = report.rows.len(),
        "PDF rendered"
    );

    let mut tx = st.pool.begin().await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action,
            entity_type: Some(entity),
            after: Some(json!({ "format": "pdf", "filters": filters, "rows": report.rows.len(), "truncated": report.truncated })),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;

    let mut res = Response::new(Body::from(pdf));
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    Ok(res)
}

fn filters_text(p: &HashMap<String, String>, keys: &[(&str, &str)]) -> String {
    let parts: Vec<String> = keys
        .iter()
        .filter_map(|(k, label)| {
            p.get(*k)
                .filter(|v| !v.is_empty())
                .map(|v| format!("{label}: {v}"))
        })
        .collect();
    if parts.is_empty() {
        "All records".into()
    } else {
        format!("Filters — {}", parts.join(" · "))
    }
}

macro_rules! route {
    ($name:ident, $build:ident) => {
        pub async fn $name(
            Extension(ctx): Extension<Ctx>,
            Query(p): Query<HashMap<String, String>>,
        ) -> Response {
            match $build(&ctx, &p).await {
                Ok(r) => r,
                Err(e) => error_response(e),
            }
        }
    };
}
route!(users_pdf, users);
route!(audit_pdf, audit_log);
route!(roles_pdf, roles);
route!(backups_pdf, backups);

async fn users(ctx: &Ctx, p: &HashMap<String, String>) -> Result<Response, AppError> {
    ctx.require("users.export")?;
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let tz = zone(ctx);
    let (q, ids) = super::users::query_from_params(p);
    let rows = super::users::report_rows(pool, ctx.org_id, &q, &ids, MAX_ROWS + 1).await?;
    let truncated = rows.len() as i64 > MAX_ROWS;
    let report = Report {
        title: if q.deleted {
            "Deleted users".into()
        } else {
            "Users".into()
        },
        subtitle: filters_text(
            p,
            &[("q", "Search"), ("status", "Status"), ("role", "Role id")],
        ),
        columns: vec!["Name", "Email", "Role", "Status", "Last sign-in", "Created"],
        rows: rows
            .iter()
            .take(MAX_ROWS as usize)
            .map(|u| {
                vec![
                    u.full_name.clone(),
                    u.email.clone(),
                    u.role_name.clone(),
                    u.status.as_str().to_string(),
                    u.last_login_at
                        .map(|t| datetime(t, &tz))
                        .unwrap_or_else(|| "Never".into()),
                    datetime(u.created_at, &tz),
                ]
            })
            .collect(),
        landscape: false,
        truncated,
    };
    respond(ctx, report, "users.pdf", "export.users", "user", p).await
}

async fn audit_log(ctx: &Ctx, p: &HashMap<String, String>) -> Result<Response, AppError> {
    ctx.require("audit.export")?;
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let tz = zone(ctx);
    let q = super::audit_log::query_from_params(p);
    let rows = super::audit_log::report_rows(pool, ctx.org_id, &q, MAX_ROWS + 1).await?;
    let truncated = rows.len() as i64 > MAX_ROWS;
    let report = Report {
        title: "Audit log".into(),
        subtitle: filters_text(
            p,
            &[
                ("q", "Search"),
                ("module", "Module"),
                ("result", "Result"),
                ("from", "From"),
                ("to", "To"),
            ],
        ),
        columns: vec!["Time", "User", "Action", "Record", "Result", "IP"],
        rows: rows
            .iter()
            .take(MAX_ROWS as usize)
            .map(|a| {
                vec![
                    datetime_secs(a.at, &tz),
                    a.actor_email.clone().unwrap_or_else(|| "System".into()),
                    a.action.clone(),
                    match (&a.entity_type, &a.entity_id) {
                        (Some(t), Some(i)) => format!("{t} {i}"),
                        (Some(t), None) => t.clone(),
                        _ => String::new(),
                    },
                    if a.success {
                        "Success".into()
                    } else {
                        "Failed".into()
                    },
                    a.ip.clone().unwrap_or_default(),
                ]
            })
            .collect(),
        landscape: true,
        truncated,
    };
    respond(ctx, report, "audit.pdf", "export.audit", "audit_log", p).await
}

/// A context that may also read the list behind an export permission.
fn with_view(ctx: &Ctx, key: &str) -> Ctx {
    let mut c = ctx.clone();
    if let Some(u) = c.user.as_mut() {
        u.permissions.insert(key.into());
    }
    c
}

async fn roles(ctx: &Ctx, p: &HashMap<String, String>) -> Result<Response, AppError> {
    ctx.require("roles.export")?;
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let list = super::roles::list(pool, &with_view(ctx, "roles.view")).await?;
    let report = Report {
        title: "Roles".into(),
        subtitle: "All roles of the organization".into(),
        columns: vec![
            "Name",
            "Description",
            "System",
            "Two-factor",
            "Users",
            "Permissions",
        ],
        rows: list
            .iter()
            .map(|r| {
                vec![
                    r.name.clone(),
                    r.description.clone(),
                    if r.is_system {
                        "Yes".into()
                    } else {
                        "No".into()
                    },
                    if r.require_2fa {
                        "Required".into()
                    } else {
                        String::new()
                    },
                    r.user_count.to_string(),
                    r.permission_count.to_string(),
                ]
            })
            .collect(),
        landscape: false,
        truncated: false,
    };
    respond(ctx, report, "roles.pdf", "export.roles", "role", p).await
}

async fn backups(ctx: &Ctx, p: &HashMap<String, String>) -> Result<Response, AppError> {
    ctx.require("backup.export")?;
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let tz = zone(ctx);
    let list = super::backups::list(pool, &with_view(ctx, "backup.view")).await?;
    let report = Report {
        title: "Backups".into(),
        subtitle: "Database backups".into(),
        columns: vec!["Created", "File", "Status", "Type", "Size", "SHA-256", "By"],
        rows: list
            .iter()
            .map(|b| {
                vec![
                    datetime(b.created_at, &tz),
                    b.filename.clone(),
                    b.status.clone(),
                    b.kind.clone(),
                    crate::domain::backups::human_size(b.size_bytes),
                    b.sha256
                        .clone()
                        .map(|s| s.chars().take(16).collect())
                        .unwrap_or_default(),
                    b.created_by.clone().unwrap_or_else(|| "System".into()),
                ]
            })
            .collect(),
        landscape: true,
        truncated: false,
    };
    respond(ctx, report, "backups.pdf", "export.backups", "backup", p).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_is_escaped() {
        let r = Report {
            title: "Users".into(),
            subtitle: "All".into(),
            columns: vec!["Name"],
            rows: vec![vec!["<script>alert(1)</script>".into()]],
            landscape: false,
            truncated: false,
        };
        let h = html(&r, "Starter", "now");
        assert!(!h.contains("<script>alert"));
        assert!(h.contains("&lt;script&gt;"));
    }

    /// Needs the Chrome service (`docker compose up -d chrome`); run with `--ignored`.
    #[tokio::test]
    #[ignore]
    async fn renders_a_pdf_quickly() {
        let r = Report {
            title: "T".into(),
            subtitle: String::new(),
            columns: vec!["A"],
            rows: vec![vec!["1".into()]],
            landscape: false,
            truncated: false,
        };
        let opt = PrintOptions {
            landscape: false,
            header: "<span></span>".into(),
            footer: footer("S", "T"),
        };
        let doc = html(&r, "S", "now");
        render("http://127.0.0.1:9223", &doc, &opt).await.unwrap(); // warm up the connection
        let t = std::time::Instant::now();
        let pdf = render("http://127.0.0.1:9223", &doc, &opt).await.unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(t.elapsed().as_millis() < 1500, "took {:?}", t.elapsed());
    }
}
