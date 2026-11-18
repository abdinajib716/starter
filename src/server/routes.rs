//! Plain Axum routes that live next to the Dioxus app (§31.1).
use dioxus::server::axum::{
    Extension, Json, extract::Request, http::StatusCode, response::IntoResponse,
};
use serde_json::json;
use tracing::Span;

use super::state::AppState;

/// Liveness + DB ping (§13).
pub async fn healthz(Extension(state): Extension<AppState>) -> impl IntoResponse {
    let db = state.db_ok().await;
    let code = if db {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(json!({
            "status": if db { "ok" } else { "degraded" },
            "db": if db { "ok" } else { "down" },
            "tenancy": state.config.tenancy_mode,
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

/// Tracing span per request carrying the `x-request-id`.
pub fn make_span(req: &Request) -> Span {
    let id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    tracing::info_span!("http", method = %req.method(), uri = %req.uri(), request_id = %id)
}

/// `POST /api/webhooks/resend` (M13.6): Svix-signed delivery events → outbox status +
/// suppression list. Needs `RESEND_WEBHOOK_SECRET`; without it every call is refused.
pub async fn resend_webhook(
    Extension(state): Extension<AppState>,
    headers: dioxus::server::axum::http::HeaderMap,
    body: dioxus::server::axum::body::Bytes,
) -> impl IntoResponse {
    let h = |n: &str| headers.get(n).and_then(|v| v.to_str().ok()).unwrap_or("");
    let Some(secret) = std::env::var("RESEND_WEBHOOK_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
    else {
        return StatusCode::NOT_FOUND;
    };
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if !super::email::verify_webhook(
        &secret,
        h("svix-id"),
        h("svix-timestamp"),
        &body,
        h("svix-signature"),
        now,
    ) {
        return StatusCode::UNAUTHORIZED;
    }
    let Ok(event) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return StatusCode::BAD_REQUEST;
    };
    match super::email::apply_event(&state.pool, &event).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!("resend webhook: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Readiness (M14.6): DB reachable and not in maintenance. Load balancers stop sending traffic
/// while it answers 503.
pub async fn readyz(Extension(state): Extension<AppState>) -> impl IntoResponse {
    let db = state.db_ok().await;
    let maintenance = super::system_state::maintenance();
    let ready = db && !maintenance;
    (
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(json!({ "ready": ready, "db": db, "maintenance": maintenance })),
    )
}
