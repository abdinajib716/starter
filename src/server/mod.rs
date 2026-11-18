//! Server-only code (`feature = "server"`): config, state, DB, request context, plain routes.
#[cfg(feature = "public-api")]
pub mod api_v1;
pub mod audit;
pub mod auth;
pub mod backup_crypto;
pub mod cli;
pub mod config;
pub mod ctx;
pub mod db;
pub mod email;
pub mod error;
pub mod export;
pub mod flags;
pub mod gdrive;
pub mod modules;
pub mod notify;
pub mod pdf;
pub mod rbac;
pub mod realtime;
pub mod remote;
pub mod request;
pub mod routes;
pub mod security;
pub mod seed;
pub mod services;
pub mod settings;
pub mod state;
pub mod storage;
pub mod system_state;

use dioxus::server::axum::{self, Extension, Router, routing::get};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

/// Build the full Axum router: Dioxus app (SSR + server fns + assets) plus plain routes.
pub async fn router() -> anyhow::Result<Router> {
    let state = state::init().await?.clone();

    // Layers wrap everything above them; the last one listed sees the request first:
    // request-id → trace → headers/limits → CSRF → state → ctx (session) → page guard → app.
    let router = dioxus::server::router(crate::app::App);
    #[cfg(feature = "public-api")]
    let router = router.merge(api_v1::router());
    let router = router
        .route("/healthz", get(routes::healthz))
        .route("/readyz", get(routes::readyz))
        .route("/api/stream", get(realtime::stream_route))
        .route("/files/avatars/{id}", get(services::files::avatar_route))
        .route(
            "/api/webhooks/resend",
            axum::routing::post(routes::resend_webhook),
        )
        .route("/api/export/users.csv", get(services::users::export_route))
        .route(
            "/api/export/audit.csv",
            get(services::audit_log::export_route),
        )
        .route("/api/export/roles.csv", get(services::roles::export_route))
        .route(
            "/api/export/backups.csv",
            get(services::backups::export_route),
        )
        .route(
            "/api/backups/{id}/download",
            get(services::backups::download_route),
        )
        .route("/api/export/users.pdf", get(services::reports::users_pdf))
        .route("/api/export/audit.pdf", get(services::reports::audit_pdf))
        .route("/api/export/roles.pdf", get(services::reports::roles_pdf))
        .route(
            "/api/export/backups.pdf",
            get(services::reports::backups_pdf),
        )
        .layer(axum::middleware::from_fn(auth::guard::pages))
        .layer(axum::middleware::from_fn(system_state::layer))
        .layer(axum::middleware::from_fn(ctx::ctx_layer))
        .layer(Extension(state.clone()))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::guard::csrf,
        ));
    let router = security::apply(router, &state.config)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http().make_span_with(routes::make_span))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    Ok(router)
}
