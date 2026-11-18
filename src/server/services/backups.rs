//! Backups (§9.6.1, M11): `pg_dump --format=custom` into storage, with sha256 + size + status,
//! retention by `backup.retention_count`, download (audited), delete, CSV export and a
//! scheduler that only one instance runs (advisory-lock leader, §28). M12: `age` encryption,
//! off-site copies (R2/S3) with retries, weekly verification, restore under maintenance.
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
use sqlx::PgPool;
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
                  u.email::text AS created_by, b.created_at, b.finished_at,
                  b.encrypted, c.status AS "offsite?", c.error AS offsite_error,
                  b.verified_at, b.verify_error
           FROM backups b LEFT JOIN users u ON u.id = b.created_by
           LEFT JOIN backup_copies c ON c.backup_id = b.id
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
    let mode = crate::server::backup_crypto::Mode::parse(
        settings::value(pool, "backup.encrypt_mode")
            .await
            .as_str()
            .unwrap_or("none"),
    );
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
    ) + if mode == crate::server::backup_crypto::Mode::Off {
        ""
    } else {
        ".age"
    };
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
        let result = run_dump(&pool, &filename, mode).await;
        let done = match result {
            Ok((size, sha)) => {
                let r = sqlx::query!(
                    "UPDATE backups SET status = 'completed', size_bytes = $2, sha256 = $3, finished_at = now(),
                            encrypted = $4, enc_scheme = $5, verified_at = now() WHERE id = $1",
                    id,
                    size as i64,
                    sha,
                    mode.scheme().is_some(),
                    mode.scheme()
                )
                .execute(&pool)
                .await;
                if let Err(e) = queue_offsite(&pool, id, &filename).await {
                    tracing::error!(error = ?e, "queue off-site copy failed");
                }
                r
            }
            Err(e) => {
                tracing::error!(error = ?e, %id, "backup failed");
                notify_failure(&pool, "backup_failed", &filename, &e.to_string()).await;
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

// ---------------------------------------------------------------- M12: off-site, verify, restore

fn backup_dir() -> anyhow::Result<PathBuf> {
    Ok(crate::server::state::get()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .config
        .backup_dir
        .clone())
}

/// In-app notification to everyone who can see backups (+ email when on).
async fn notify_failure(pool: &PgPool, kind: &str, file: &str, error: &str) {
    let r: anyhow::Result<()> = async {
        let mut tx = pool.begin().await?;
        let org = sqlx::query_scalar!("SELECT id FROM organizations ORDER BY created_at LIMIT 1")
            .fetch_one(&mut *tx)
            .await?;
        let params = json!({ "file": file, "error": error.chars().take(200).collect::<String>() });
        crate::server::notify::send(&mut tx, org, crate::server::notify::Target::Permission("backup.view"), kind, params.clone(), Some("/backups")).await?;
        let ids = sqlx::query_scalar!(
            r#"SELECT user_id AS "u!" FROM notifications WHERE kind = $1 AND created_at > now() - interval '5 seconds'"#,
            kind
        )
        .fetch_all(&mut *tx)
        .await?;
        for u in ids {
            crate::server::email::queue_for_user(&mut tx, u, "backup_failed", params.clone()).await?;
        }
        tx.commit().await?;
        Ok(())
    }
    .await;
    if let Err(e) = r {
        tracing::error!("backup failure notification: {e:#}");
    }
}

/// After a successful backup: one pending copy per off-site target (M12.4).
async fn queue_offsite(pool: &PgPool, id: Uuid, filename: &str) -> sqlx::Result<()> {
    if !settings::value(pool, "backup.offsite_enabled")
        .await
        .as_bool()
        .unwrap_or(false)
    {
        return Ok(());
    }
    let target = settings::value(pool, "backup.offsite_driver")
        .await
        .as_str()
        .unwrap_or("r2")
        .to_string();
    sqlx::query!(
        "INSERT INTO backup_copies (backup_id, target, remote_key) VALUES ($1, $2, $3)
         ON CONFLICT (backup_id, target) DO NOTHING",
        id,
        target,
        format!("backups/{filename}")
    )
    .execute(pool)
    .await?;
    Ok(())
}

fn offsite_store(target: &str) -> anyhow::Result<crate::server::remote::Remote> {
    crate::server::remote::for_target(target)
}

const COPY_BACKOFF_MINS: [i32; 4] = [5, 30, 120, 360];

/// Upload due copies (claimed with `SKIP LOCKED`, so any number of instances can run this).
pub async fn copy_due(pool: &PgPool) -> anyhow::Result<usize> {
    let mut tx = pool.begin().await?;
    let rows = sqlx::query!(
        r#"SELECT c.id, c.target, c.remote_key, c.attempts, b.filename
           FROM backup_copies c JOIN backups b ON b.id = c.backup_id
           WHERE c.status = 'pending' AND c.next_attempt_at <= now()
           ORDER BY c.created_at LIMIT 3 FOR UPDATE OF c SKIP LOCKED"#
    )
    .fetch_all(&mut *tx)
    .await?;
    let n = rows.len();
    let dir = backup_dir()?;
    for r in rows {
        let res = async {
            let store = offsite_store(&r.target)?;
            let local = Local { root: dir.clone() }.path(&r.filename)?;
            store.upload(&r.remote_key, &local).await
        }
        .await;
        match res {
            Ok(size) => {
                sqlx::query!(
                    "UPDATE backup_copies SET status = 'done', size_bytes = $2, error = NULL,
                            attempts = attempts + 1, verified_at = now() WHERE id = $1",
                    r.id,
                    size as i64
                )
                .execute(&mut *tx)
                .await?;
            }
            Err(e) => {
                let attempts = r.attempts + 1;
                let failed = attempts >= 5;
                let wait = COPY_BACKOFF_MINS[usize::try_from(attempts - 1).unwrap_or(0).min(3)];
                sqlx::query!(
                    "UPDATE backup_copies SET attempts = $2, error = $3,
                            status = CASE WHEN $4 THEN 'failed' ELSE 'pending' END,
                            next_attempt_at = now() + make_interval(mins => $5) WHERE id = $1",
                    r.id,
                    attempts,
                    format!("{e:#}").chars().take(500).collect::<String>(),
                    failed,
                    wait
                )
                .execute(&mut *tx)
                .await?;
                if failed {
                    notify_failure(
                        pool,
                        "backup_offsite_failed",
                        &r.filename,
                        &format!("{e:#}"),
                    )
                    .await;
                }
            }
        }
    }
    tx.commit().await?;
    if n > 0 {
        offsite_retention(pool).await?;
    }
    Ok(n)
}

/// Keep the newest `backup.offsite_retention_count` remote copies (M12.6).
pub async fn offsite_retention(pool: &PgPool) -> anyhow::Result<u64> {
    let keep = settings::value(pool, "backup.offsite_retention_count")
        .await
        .as_i64()
        .unwrap_or(30);
    let old = sqlx::query!(
        "SELECT id, target, remote_key FROM backup_copies WHERE status = 'done'
         ORDER BY created_at DESC OFFSET $1",
        keep
    )
    .fetch_all(pool)
    .await?;
    for c in &old {
        offsite_store(&c.target)?.delete(&c.remote_key).await?;
        sqlx::query!("DELETE FROM backup_copies WHERE id = $1", c.id)
            .execute(pool)
            .await?;
    }
    Ok(old.len() as u64)
}

/// Off-site copier (every minute) + weekly verification (leader only).
pub fn start_offsite_worker(pool: PgPool) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(60));
        tick.tick().await; // the first tick is immediate: let the app state finish starting
        let mut last_verify = std::time::Instant::now();
        loop {
            tick.tick().await;
            if let Err(e) = copy_due(&pool).await {
                tracing::error!("off-site copy: {e:#}");
            }
            if last_verify.elapsed() > Duration::from_secs(7 * 24 * 3600) {
                last_verify = std::time::Instant::now();
                let Ok(cfg) = crate::server::config::Config::load() else {
                    continue;
                };
                if let Err(e) = verify_all(&pool, &cfg).await {
                    tracing::error!("backup verification: {e:#}");
                }
            }
        }
    });
}

/// M12.5: every completed backup — local file present, same size and SHA-256; every done
/// copy — remote object present with the same size. Returns a one-line report.
pub async fn verify_all(
    pool: &PgPool,
    cfg: &crate::server::config::Config,
) -> anyhow::Result<String> {
    let rows = sqlx::query!(
        "SELECT id, filename, size_bytes, sha256 FROM backups WHERE status = 'completed' ORDER BY created_at"
    )
    .fetch_all(pool)
    .await?;
    let store = Local {
        root: cfg.backup_dir.clone(),
    };
    let (mut ok, mut bad) = (0, 0);
    for b in rows {
        let problem = async {
            let path = store.path(&b.filename)?;
            let meta = tokio::fs::metadata(&path)
                .await
                .map_err(|_| anyhow::anyhow!("file missing"))?;
            if meta.len() as i64 != b.size_bytes {
                anyhow::bail!("size {} ≠ {}", meta.len(), b.size_bytes);
            }
            let sha = crate::server::backup_crypto::sha256_file(&path).await?;
            if b.sha256.as_deref().is_some_and(|s| s != sha) {
                anyhow::bail!("SHA-256 mismatch");
            }
            Ok::<_, anyhow::Error>(())
        }
        .await
        .err()
        .map(|e| e.to_string());
        if problem.is_some() {
            bad += 1
        } else {
            ok += 1
        }
        sqlx::query!(
            "UPDATE backups SET verify_error = $2, verified_at = CASE WHEN $2::text IS NULL THEN now() ELSE verified_at END WHERE id = $1",
            b.id,
            problem
        )
        .execute(pool)
        .await?;
    }
    let copies = sqlx::query!(
        "SELECT id, target, remote_key, size_bytes FROM backup_copies WHERE status = 'done'"
    )
    .fetch_all(pool)
    .await?;
    let (mut cok, mut cbad) = (0, 0);
    for c in copies {
        let res = async {
            let s = offsite_store(&c.target)?;
            match s.size(&c.remote_key).await? {
                None => anyhow::bail!("remote copy missing"),
                Some(n) if Some(n as i64) != c.size_bytes => anyhow::bail!("remote size differs"),
                _ => Ok(()),
            }
        }
        .await;
        match res {
            Ok(()) => {
                cok += 1;
                sqlx::query!(
                    "UPDATE backup_copies SET verified_at = now(), error = NULL WHERE id = $1",
                    c.id
                )
                .execute(pool)
                .await?;
            }
            Err(e) => {
                cbad += 1;
                sqlx::query!(
                    "UPDATE backup_copies SET error = $2 WHERE id = $1",
                    c.id,
                    e.to_string()
                )
                .execute(pool)
                .await?;
            }
        }
    }
    Ok(format!(
        "backups: {ok} ok, {bad} with problems; off-site copies: {cok} ok, {cbad} with problems"
    ))
}

/// "Test off-site connection" (settings, M12.6).
pub async fn test_offsite(pool: &PgPool, ctx: &Ctx) -> Result<(), AppError> {
    ctx.require("settings.update")?;
    let target = settings::value(pool, "backup.offsite_driver")
        .await
        .as_str()
        .unwrap_or("r2")
        .to_string();
    let store = offsite_store(&target).map_err(|e| {
        tracing::warn!("off-site test: {e:#}");
        AppError::Conflict("backup.offsite_not_configured")
    })?;
    store.test().await.map_err(|e| {
        tracing::warn!("off-site test: {e:#}");
        AppError::Conflict("backup.offsite_test_failed")
    })
}

/// M12.7: restore a backup. Checks (permission, typed file name, own password), then a
/// `pre_restore` backup, maintenance on, fetch (local or off-site), decrypt, `pg_restore`,
/// migrations, sessions cleared, caches reloaded, maintenance off; the audit row and the
/// backup rows written after the dump are put back.
pub async fn restore(
    pool: &PgPool,
    ctx: &Ctx,
    req: crate::domain::backups::RestoreRequest,
) -> Result<(), AppError> {
    let actor = ctx.require("backup.restore")?;
    let b = sqlx::query!(
        "SELECT id, filename, status, encrypted, enc_scheme, sha256 FROM backups WHERE id = $1",
        req.id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    if b.status != "completed" {
        return Err(AppError::Conflict("backup.not_completed"));
    }
    if req.confirm.trim() != b.filename {
        return Err(AppError::Validation("backup.confirm_mismatch"));
    }
    let stored = sqlx::query_scalar!("SELECT password_hash FROM users WHERE id = $1", actor.id)
        .fetch_one(pool)
        .await?;
    if !crate::server::auth::password::verify(req.password.clone(), Some(stored)).await {
        return Err(AppError::Validation("auth.wrong_password"));
    }
    if b.encrypted
        && b.enc_scheme.as_deref() == Some("age")
        && req.secret.as_deref().is_none_or(|s| s.trim().is_empty())
        && std::env::var("BACKUP_PRIVATE_KEY").is_err()
    {
        return Err(AppError::Validation("backup.key_required"));
    }
    // Same lock as backups: nothing else dumps while we restore.
    let mut conn = pool.acquire().await?.detach();
    let got: bool = sqlx::query_scalar!(r#"SELECT pg_try_advisory_lock($1) AS "l!""#, LOCK_BACKUP)
        .fetch_one(&mut conn)
        .await?;
    if !got {
        return Err(AppError::Conflict("backup.running"));
    }
    let actor_id = actor.id;
    let actor_email = actor.email.clone();
    let org = ctx.org_id;
    let (ip, ua) = (ctx.meta.ip, ctx.meta.user_agent.clone());
    let pool2 = pool.clone();
    tokio::spawn(async move {
        let r = run_restore(
            &pool2,
            &b.filename,
            b.encrypted,
            b.enc_scheme.as_deref(),
            b.sha256.as_deref(),
            &req,
        )
        .await;
        // Whatever happened: the app comes back.
        let _ = crate::server::system_state::set_maintenance(&pool2, false, "").await;
        let ok = r.is_ok();
        let error = r.as_ref().err().map(|e| format!("{e:#}"));
        if let Some(e) = &error {
            tracing::error!("restore failed: {e}");
        }
        let after = r.unwrap_or_default();
        let write = async || -> anyhow::Result<()> {
            let mut tx = pool2.begin().await?;
            // Rows created after the dump was taken (they're gone after --clean).
            for (id, filename, size, sha, kind) in &after {
                sqlx::query!(
                    "INSERT INTO backups (id, filename, size_bytes, sha256, status, kind, finished_at)
                     VALUES ($1, $2, $3, $4, 'completed', $5, now()) ON CONFLICT (id) DO NOTHING",
                    id, filename, size, sha.as_deref(), kind
                )
                .execute(&mut *tx)
                .await?;
            }
            // The actor may not exist in the restored data.
            let actor_still =
                sqlx::query_scalar!(r#"SELECT id AS "id!" FROM users WHERE id = $1"#, actor_id)
                    .fetch_optional(&mut *tx)
                    .await?;
            audit::record(
                &mut tx,
                Entry {
                    org_id: Some(org),
                    actor_id: actor_still,
                    actor_email: Some(actor_email.as_str()),
                    action: "backup.restore",
                    entity_type: Some("backup"),
                    entity_id: Some(req.id.to_string()),
                    after: Some(
                        json!({ "file": b.filename, "source": req.source, "error": error }),
                    ),
                    ip,
                    user_agent: ua.as_deref(),
                    success: ok,
                    ..Default::default()
                },
            )
            .await?;
            if ok {
                crate::server::notify::send(
                    &mut tx,
                    org,
                    crate::server::notify::Target::Permission("backup.restore"),
                    "restore_done",
                    json!({ "file": b.filename }),
                    Some("/backups"),
                )
                .await?;
            }
            tx.commit().await?;
            Ok(())
        };
        if let Err(e) = write().await {
            tracing::error!("restore bookkeeping: {e:#}");
        }
        let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(LOCK_BACKUP)
            .execute(&mut conn)
            .await;
    });
    Ok(())
}

type KeptRow = (Uuid, String, i64, Option<String>, String);

/// The backup list describes files on disk, not database content: keep it as it is now
/// across a restore (the dump holds an older list, with its own row still `running`).
async fn snapshot_backup_rows(
    pool: &PgPool,
) -> sqlx::Result<(serde_json::Value, serde_json::Value)> {
    let b = sqlx::query_scalar!(r#"SELECT COALESCE(jsonb_agg(b), '[]') AS "j!" FROM backups b"#)
        .fetch_one(pool)
        .await?;
    let c =
        sqlx::query_scalar!(r#"SELECT COALESCE(jsonb_agg(c), '[]') AS "j!" FROM backup_copies c"#)
            .fetch_one(pool)
            .await?;
    Ok((b, c))
}

async fn put_back_backup_rows(
    pool: &PgPool,
    rows: &(serde_json::Value, serde_json::Value),
) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM backup_copies")
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM backups")
        .execute(&mut *tx)
        .await?;
    // `created_by` may point at a user who doesn't exist in the restored data.
    sqlx::query!(
        r#"INSERT INTO backups (id, filename, size_bytes, sha256, status, kind, error, created_by,
                               created_at, finished_at, encrypted, enc_scheme, verified_at, verify_error)
           SELECT r.id, r.filename, r.size_bytes, r.sha256, r.status, r.kind, r.error,
                  (SELECT u.id FROM users u WHERE u.id = r.created_by),
                  r.created_at, r.finished_at, r.encrypted, r.enc_scheme, r.verified_at, r.verify_error
           FROM jsonb_populate_recordset(NULL::backups, $1) r"#,
        rows.0
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"INSERT INTO backup_copies SELECT r.* FROM jsonb_populate_recordset(NULL::backup_copies, $1) r
           WHERE r.backup_id IN (SELECT id FROM backups)"#,
        rows.1
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

async fn run_restore(
    pool: &PgPool,
    filename: &str,
    encrypted: bool,
    scheme: Option<&str>,
    sha: Option<&str>,
    req: &crate::domain::backups::RestoreRequest,
) -> anyhow::Result<Vec<KeptRow>> {
    let dir = backup_dir()?;
    let store = Local { root: dir.clone() };
    // 1. Safety net: dump the current database first (unencrypted is fine: same server).
    let pre_name = format!(
        "pre_restore_{}.dump",
        time::OffsetDateTime::now_utc().unix_timestamp()
    );
    let (pre_size, pre_sha) =
        run_dump(pool, &pre_name, crate::server::backup_crypto::Mode::Off).await?;
    // 2. Maintenance on: every instance answers 503.
    let pre_id = sqlx::query_scalar!(
        "INSERT INTO backups (filename, status, kind, size_bytes, sha256, finished_at)
         VALUES ($1, 'completed', 'pre_restore', $2, $3, now()) RETURNING id",
        pre_name,
        pre_size as i64,
        pre_sha
    )
    .fetch_one(pool)
    .await?;
    let kept = snapshot_backup_rows(pool).await?;
    crate::server::system_state::set_maintenance(pool, true, "restore").await?;
    // 3. The file: local copy or the off-site one.
    let work = dir.join(format!(".restore-{}", Uuid::new_v4().simple()));
    let source = if req.source == "offsite" {
        let c = sqlx::query!(
            "SELECT target, remote_key FROM backup_copies WHERE backup_id = $1 AND status = 'done' LIMIT 1",
            req.id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no off-site copy of this backup"))?;
        offsite_store(&c.target)?
            .download(&c.remote_key, &work)
            .await?;
        work.clone()
    } else {
        store.path(filename)?
    };
    let got = crate::server::backup_crypto::sha256_file(&source).await?;
    if sha.is_some_and(|s| s != got) {
        let _ = tokio::fs::remove_file(&work).await;
        anyhow::bail!("the backup file is damaged (SHA-256 differs)");
    }
    let plain = if encrypted {
        let out = dir.join(format!(".restore-plain-{}", Uuid::new_v4().simple()));
        let r = crate::server::backup_crypto::decrypt(
            scheme.unwrap_or("age"),
            req.secret.clone(),
            source.clone(),
            out.clone(),
        )
        .await;
        let _ = tokio::fs::remove_file(&work).await;
        r?;
        out
    } else {
        source
    };
    // 4. pg_restore over the live database.
    let cfg = crate::server::config::Config::load()?;
    let (url, password) = split_password(&cfg.database_url);
    let mut cmd = tokio::process::Command::new("pg_restore");
    cmd.args([
        "--clean",
        "--if-exists",
        "--no-owner",
        "--single-transaction",
        "--dbname",
    ])
    .arg(&url)
    .arg(&plain);
    if let Some(p) = password {
        cmd.env("PGPASSWORD", p);
    }
    let out = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("pg_restore not runnable: {e}"));
    if plain != store.path(filename)? {
        let _ = tokio::fs::remove_file(&plain).await;
    }
    let out = out?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!(
            "pg_restore failed: {}",
            msg.lines().last().unwrap_or("unknown error")
        );
    }
    // The restored system_state says "off": keep maintenance on until we're done.
    crate::server::system_state::set_maintenance(pool, true, "restore").await?;
    // 5. Schema up to date, nobody signed in with a stale session, caches fresh.
    crate::server::db::migrate(pool).await?;
    put_back_backup_rows(pool, &kept).await?;
    sqlx::query!("DELETE FROM sessions").execute(pool).await?;
    crate::server::settings::reload_now(pool).await?;
    crate::server::flags::reload(pool).await?;
    let _ = pre_id;
    Ok(vec![])
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

async fn run_dump(
    _pool: &PgPool,
    filename: &str,
    mode: crate::server::backup_crypto::Mode,
) -> anyhow::Result<(u64, String)> {
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
    // Encrypt before the file reaches storage; the plain dump is removed at once.
    let tmp = if mode == crate::server::backup_crypto::Mode::Off {
        tmp
    } else {
        let enc = st.config.backup_dir.join(format!(".{filename}.enc.part"));
        let r = crate::server::backup_crypto::encrypt(mode, tmp.clone(), enc.clone()).await;
        let _ = tokio::fs::remove_file(&tmp).await;
        r?;
        crate::server::storage::restrict(&enc).await?;
        enc
    };
    // sha256 of the stored bytes (what verification and restore check against).
    let sha = crate::server::backup_crypto::sha256_file(&tmp).await?;
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
            match tokio::io::AsyncReadExt::read(&mut f, &mut buf).await {
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
