//! Plain Axum routes that live next to the Dioxus app (§31.1).
use dioxus::server::axum::{
    Extension, Json, extract::Request, http::StatusCode, response::IntoResponse,
};
use serde_json::json;
use tracing::Span;

use super::state::AppState;

/// Liveness + DB ping (§13). `/readyz` (maintenance aware) comes in M14.
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
