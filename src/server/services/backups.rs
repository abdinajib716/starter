//! Backups (§9.6.1, M11): `pg_dump --format=custom` into storage, with sha256 + size + status,
//! retention by `backup.retention_count`, download (audited), delete, CSV export and a
//! scheduler that only one instance runs (advisory-lock leader, §28). Restore: M12.
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use dioxus::server::axum::{
    Extension,
    body::Body,
    extract::{Path, Query},
    http::{HeaderValue, header},
    response::Response,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

use crate::domain::backups::BackupRow;
use crate::server::audit::{self, Entry};
use crate::server::ctx::Ctx;
use crate::server::error::AppError;
use crate::server::export::{csv_response, error_response, line};
use crate::server::settings;
use crate::server::storage::{Local, Storage};

/// Advisory-lock ids (§28): one backup at a time across all instances; one scheduler leader.
const LOCK_BACKUP: i64 = 0x5354_4152_4241_4b31; // "STARBAK1"
const LOCK_SCHEDULER: i64 = 0x5354_4152_5343_4831; // "STARSCH1"

fn storage() -> Result<Local, AppError> {
    let st = crate::server::state::get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(Local {
        root: st.config.backup_dir.clone(),
    })
}

pub async fn list(pool: &PgPool, ctx: &Ctx) -> Result<Vec<BackupRow>, AppError> {
    ctx.require("backup.view")?;
    Ok(sqlx::query_as!(
        BackupRow,
        r#"SELECT b.id, b.filename, b.size_bytes, b.sha256, b.status, b.kind, b.error,
                  u.email::text AS created_by, b.created_at, b.finished_at
           FROM backups b LEFT JOIN users u ON u.id = b.created_by
           ORDER BY b.created_at DESC"#
    )
    .fetch_all(pool)
    .await?)
}

/// Start a manual backup; returns its id at once (status `running`), the dump runs in the
/// background. Refused while another backup runs on any instance.
pub async fn create(pool: &PgPool, ctx: &Ctx) -> Result<Uuid, AppError> {
    let actor = ctx.require("backup.create")?;
    let id = start(pool, "manual", Some(actor.id)).await?;
    let mut tx = pool.begin().await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "backup.create",
            entity_type: Some("backup"),
            entity_id: Some(id.to_string()),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

/// Insert the `running` row and spawn the dump. The advisory lock is taken on a dedicated
/// connection held by the task until it ends.
async fn start(pool: &PgPool, kind: &'static str, by: Option<Uuid>) -> Result<Uuid, AppError> {
    let mut conn = pool.acquire().await?.detach();
    let got: bool = sqlx::query_scalar!(r#"SELECT pg_try_advisory_lock($1) AS "l!""#, LOCK_BACKUP)
        .fetch_one(&mut conn)
        .await?;
    if !got {
        return Err(AppError::Conflict("backup.running"));
    }
    let now = time::OffsetDateTime::now_utc();
    let filename = format!(
        "backup_{:04}{:02}{:02}_{:02}{:02}{:02}_{}.dump",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        &Uuid::new_v4().simple().to_string()[..6]
    );
    let id = sqlx::query_scalar!(
        "INSERT INTO backups (filename, status, kind, created_by) VALUES ($1, 'running', $2, $3) RETURNING id",
        filename,
        kind,
        by
    )
    .fetch_one(pool)
    .await?;
    let pool = pool.clone();
    tokio::spawn(async move {
        let result = run_dump(&pool, &filename).await;
        let done = match result {
            Ok((size, sha)) => {
                sqlx::query!(
                    "UPDATE backups SET status = 'completed', size_bytes = $2, sha256 = $3, finished_at = now() WHERE id = $1",
                    id,
                    size as i64,
                    sha
                )
                .execute(&pool)
                .await
            }
            Err(e) => {
                tracing::error!(error = ?e, %id, "backup failed");
                sqlx::query!(
                    "UPDATE backups SET status = 'failed', error = $2, finished_at = now() WHERE id = $1",
                    id,
                    // The tool's message, without connection details.
                    e.to_string().chars().take(500).collect::<String>()
                )
                .execute(&pool)
                .await
            }
        };
        if let Err(e) = done {
            tracing::error!(error = %e, "recording backup status failed");
        }
        if let Err(e) = retention(&pool).await {
            tracing::error!(error = ?e, "backup retention failed");
        }
        let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(LOCK_BACKUP)
            .execute(&mut conn)
            .await;
    });
    Ok(id)
}

/// `postgres://user:pass@host/db` → (`postgres://user@host/db`, Some(pass)): the password goes
/// to `pg_dump` through `PGPASSWORD`, never on its command line (visible in `ps`).
pub fn split_password(url: &str) -> (String, Option<String>) {
    let Some((scheme, rest)) = url.split_once("://") else {
        return (url.into(), None);
    };
    let Some((auth, host)) = rest.split_once('@') else {
        return (url.into(), None);
    };
    match auth.split_once(':') {
        Some((user, pass)) => (format!("{scheme}://{user}@{host}"), Some(pass.to_string())),
        None => (url.into(), None),
    }
}

async fn run_dump(_pool: &PgPool, filename: &str) -> anyhow::Result<(u64, String)> {
    let st = crate::server::state::get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let store = Local {
        root: st.config.backup_dir.clone(),
    };
    tokio::fs::create_dir_all(&st.config.backup_dir).await?;
    let tmp: PathBuf = st.config.backup_dir.join(format!(".{filename}.part"));
    let (url, password) = split_password(&st.config.database_url);
    let mut cmd = tokio::process::Command::new("pg_dump");
    cmd.arg("--format=custom")
        .arg("--no-owner")
        .arg("--file")
        .arg(&tmp)
        .arg("--dbname")
        .arg(&url);
    if let Some(p) = password {
        cmd.env("PGPASSWORD", p);
    }
    let out = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("pg_dump not runnable: {e}"))?;
    if !out.status.success() {
        let _ = tokio::fs::remove_file(&tmp).await;
        let msg = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!(
            "pg_dump failed: {}",
            msg.lines().last().unwrap_or("unknown error")
        );
    }
    crate::server::storage::restrict(&tmp).await?;
    // sha256 while the file is still local.
    let mut f = tokio::fs::File::open(&tmp).await?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    let sha = h
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let size = store.put_file(filename, &tmp).await?;
    Ok((size, sha))
}

/// Keep the newest `backup.retention_count` completed backups; delete older files + rows.
pub async fn retention(pool: &PgPool) -> anyhow::Result<u64> {
    let keep = settings::value(pool, "backup.retention_count")
        .await
        .as_i64()
        .unwrap_or(14);
    let old = sqlx::query!(
        "SELECT id, filename FROM backups WHERE status = 'completed'
         ORDER BY created_at DESC OFFSET $1",
        keep
    )
    .fetch_all(pool)
    .await?;
    let store = storage().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    for b in &old {
        store.delete(&b.filename).await?;
        sqlx::query!("DELETE FROM backups WHERE id = $1", b.id)
            .execute(pool)
            .await?;
    }
    Ok(old.len() as u64)
}

pub async fn delete(pool: &PgPool, ctx: &Ctx, id: Uuid) -> Result<(), AppError> {
    let actor = ctx.require("backup.delete")?;
    let mut tx = pool.begin().await?;
    let b = sqlx::query!(
        "SELECT filename, status FROM backups WHERE id = $1 FOR UPDATE",
        id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;
    if b.status == "running" {
        return Err(AppError::Conflict("backup.running"));
    }
    sqlx::query!("DELETE FROM backups WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "backup.delete",
            entity_type: Some("backup"),
            entity_id: Some(id.to_string()),
            before: Some(json!({ "filename": b.filename })),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    storage()?.delete(&b.filename).await?;
    Ok(())
}

/// `GET /api/backups/:id/download` — looked up by id only (never a client path), streamed,
/// audited as `backup.download`.
pub async fn download_route(Extension(ctx): Extension<Ctx>, Path(id): Path<Uuid>) -> Response {
    match download(&ctx, id).await {
        Ok(r) => r,
        Err(e) => error_response(e),
    }
}

async fn download(ctx: &Ctx, id: Uuid) -> Result<Response, AppError> {
    let actor = ctx.require("backup.download")?.clone();
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let b = sqlx::query!("SELECT filename, status FROM backups WHERE id = $1", id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    if b.status != "completed" {
        return Err(AppError::NotFound);
    }
    let path = storage()?.path(&b.filename)?;
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|_| AppError::NotFound)?;
    let len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
    let mut tx = pool.begin().await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "backup.download",
            entity_type: Some("backup"),
            entity_id: Some(id.to_string()),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    let stream = futures_util::stream::unfold(
        (file, vec![0u8; 64 * 1024]),
        |(mut f, mut buf)| async move {
            match f.read(&mut buf).await {
                Ok(0) => None,
                Ok(n) => Some((Ok::<_, std::io::Error>(buf[..n].to_vec()), (f, buf))),
                Err(e) => Some((Err(e), (f, buf))),
            }
        },
    );
    let mut res = Response::new(Body::from_stream(stream));
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    h.insert(header::CONTENT_LENGTH, HeaderValue::from(len));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", b.filename)) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    Ok(res)
}

/// `GET /api/export/backups.csv` (`backup.export`, audited).
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
    let actor = ctx.require("backup.export")?.clone();
    let pool = &crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .pool;
    let mut viewer = ctx.clone();
    if let Some(u) = viewer.user.as_mut() {
        u.permissions.insert("backup.view".into());
    }
    let rows = list(pool, &viewer).await?;
    let mut tx = pool.begin().await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "export.backups",
            entity_type: Some("backup"),
            after: Some(json!({ "rows": rows.len() })),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    let (send, recv) = tokio::sync::mpsc::channel::<String>(16);
    tokio::spawn(async move {
        let fmt = |t: Option<time::OffsetDateTime>| {
            t.and_then(|t| {
                t.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            })
            .unwrap_or_default()
        };
        let _ = send
            .send(format!(
                "\u{feff}{}",
                line(&[
                    "filename",
                    "status",
                    "kind",
                    "size_bytes",
                    "sha256",
                    "created_by",
                    "created_utc",
                    "finished_utc"
                ])
            ))
            .await;
        for b in rows {
            let l = line(&[
                &b.filename,
                &b.status,
                &b.kind,
                &b.size_bytes.to_string(),
                b.sha256.as_deref().unwrap_or(""),
                b.created_by.as_deref().unwrap_or(""),
                &fmt(Some(b.created_at)),
                &fmt(b.finished_at),
            ]);
            if send.send(l).await.is_err() {
                return;
            }
        }
    });
    Ok(csv_response("backups.csv", false, recv))
}

/// Scheduler (M11.7): every 30 s, the instance holding the scheduler lock checks
/// `backup.schedule_enabled` / `backup.schedule_cron` (read fresh each time, so a settings
/// change applies without a restart, M7.6) and starts a `scheduled` backup when one is due.
pub fn start_scheduler(pool: PgPool) {
    tokio::spawn(async move {
        let mut conn = None;
        let mut last_fire: Option<time::OffsetDateTime> = None;
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            if conn.is_none()
                && let Ok(c) = pool.acquire().await
            {
                {
                    let mut c = c.detach();
                    let leader: bool = sqlx::query_scalar!(
                        r#"SELECT pg_try_advisory_lock($1) AS "l!""#,
                        LOCK_SCHEDULER
                    )
                    .fetch_one(&mut c)
                    .await
                    .unwrap_or(false);
                    if leader {
                        tracing::info!("backup scheduler: this instance is the leader");
                        conn = Some(c);
                    }
                }
            }
            if conn.is_none()
                || !settings::value(&pool, "backup.schedule_enabled")
                    .await
                    .as_bool()
                    .unwrap_or(false)
            {
                continue;
            }
            let expr = settings::value(&pool, "backup.schedule_cron")
                .await
                .as_str()
                .unwrap_or("")
                .to_string();
            let Ok(schedule) = expr.parse::<cron::Schedule>() else {
                continue;
            };
            let now = chrono_now();
            let since = last_fire.unwrap_or(now - time::Duration::seconds(30));
            if due(&schedule, since, now) {
                last_fire = Some(now);
                match start(&pool, "scheduled", None).await {
                    Ok(id) => tracing::info!(%id, "scheduled backup started"),
                    Err(e) => tracing::warn!(error = %e, "scheduled backup not started"),
                }
            }
        }
    });
}

fn chrono_now() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc()
}

/// Did the schedule fire in `(since, now]`?
pub fn due(
    schedule: &cron::Schedule,
    since: time::OffsetDateTime,
    now: time::OffsetDateTime,
) -> bool {
    let to_chrono = |t: time::OffsetDateTime| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(t.unix_timestamp(), 0)
    };
    match (to_chrono(since), to_chrono(now)) {
        (Some(s), Some(n)) => schedule.after(&s).next().is_some_and(|next| next <= n),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_never_on_the_command_line() {
        assert_eq!(
            split_password("postgres://app:s3cret@db:5432/starter?sslmode=disable"),
            (
                "postgres://app@db:5432/starter?sslmode=disable".into(),
                Some("s3cret".into())
            )
        );
        assert_eq!(
            split_password("postgres://app@db/x"),
            ("postgres://app@db/x".into(), None)
        );
    }

    #[test]
    fn schedule_due_window() {
        let s: cron::Schedule = "0 0 2 * * *".parse().unwrap();
        let at = |h: u8, m: u8, sec: u8| {
            time::macros::datetime!(2026-11-17 00:00 UTC)
                .replace_time(time::Time::from_hms(h, m, sec).unwrap())
        };
        assert!(due(&s, at(1, 59, 45), at(2, 0, 15)));
        assert!(!due(&s, at(2, 0, 15), at(2, 0, 45)), "fires once");
        assert!(!due(&s, at(12, 0, 0), at(12, 0, 30)));
    }
}
