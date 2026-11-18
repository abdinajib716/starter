//! Shared system state (§28, M14.3): the maintenance flag. Cached per instance, refreshed on
//! `NOTIFY system_state` and every 2 s as a fallback, so every instance agrees quickly.
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use dioxus::server::axum::{
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

static MAINTENANCE: AtomicBool = AtomicBool::new(false);

pub fn maintenance() -> bool {
    MAINTENANCE.load(Ordering::Relaxed)
}

async fn load(pool: &PgPool) -> sqlx::Result<()> {
    let on = sqlx::query_scalar!(
        r#"SELECT COALESCE((value->>'on')::boolean, false) AS "on!" FROM system_state WHERE key = 'maintenance'"#
    )
    .fetch_optional(pool)
    .await?
    .unwrap_or(false);
    MAINTENANCE.store(on, Ordering::Relaxed);
    Ok(())
}

pub async fn set_maintenance(pool: &PgPool, on: bool, reason: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO system_state (key, value, updated_at) VALUES ('maintenance', $1, now())
         ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = now()",
        json!({ "on": on, "reason": reason })
    )
    .execute(pool)
    .await?;
    MAINTENANCE.store(on, Ordering::Relaxed);
    Ok(())
}

pub async fn start(pool: PgPool) -> sqlx::Result<()> {
    load(&pool).await?;
    tokio::spawn(async move {
        let mut listener = sqlx::postgres::PgListener::connect_with(&pool).await.ok();
        if let Some(l) = listener.as_mut() {
            let _ = l.listen("system_state").await;
        }
        loop {
            match listener.as_mut() {
                Some(l) => {
                    let _ = tokio::time::timeout(Duration::from_secs(2), l.recv()).await;
                }
                None => tokio::time::sleep(Duration::from_secs(2)).await,
            }
            if let Err(e) = load(&pool).await {
                tracing::warn!("system_state refresh: {e}");
            }
        }
    });
    Ok(())
}

/// While maintenance is on: pages show 503, API calls answer `503 system.maintenance`.
/// Health checks, static assets and the 503 page itself keep working.
pub async fn layer(req: Request, next: Next) -> Response {
    if !maintenance() {
        return next.run(req).await;
    }
    let path = req.uri().path();
    if matches!(path, "/healthz" | "/readyz" | "/503") || super::ctx::is_asset(path) {
        return next.run(req).await;
    }
    if path.starts_with("/api/") || req.method() != Method::GET {
        let body = json!({ "code": "system.maintenance" }).to_string();
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [
                (header::CONTENT_TYPE, "application/json"),
                (header::RETRY_AFTER, "30"),
            ],
            body,
        )
            .into_response();
    }
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::RETRY_AFTER, "30"),
        ],
        MAINTENANCE_PAGE,
    )
        .into_response()
}

const MAINTENANCE_PAGE: &str = r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="refresh" content="15"><title>Maintenance</title><style>body{margin:0;min-height:100vh;display:grid;place-items:center;font-family:system-ui,sans-serif;background:#fafafa;color:#18181b}@media(prefers-color-scheme:dark){body{background:#09090b;color:#fafafa}}div{text-align:center;padding:24px}h1{font-size:4rem;margin:0}p{color:#71717a}</style></head><body><div><h1>503</h1><h2>Under maintenance</h2><p>The system is being updated or restored. This page reloads by itself.</p></div></body></html>"#;
