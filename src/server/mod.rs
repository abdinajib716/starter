//! Server-only code (`feature = "server"`): config, state, DB, request context, plain routes.
pub mod audit;
pub mod auth;
pub mod config;
pub mod ctx;
pub mod db;
pub mod error;
pub mod export;
pub mod modules;
pub mod rbac;
pub mod request;
pub mod routes;
pub mod security;
pub mod seed;
pub mod services;
pub mod settings;
pub mod state;

use dioxus::server::axum::{self, Extension, Router, routing::get};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

/// Build the full Axum router: Dioxus app (SSR + server fns + assets) plus plain routes.
pub async fn router() -> anyhow::Result<Router> {
    let state = state::init().await?.clone();

    // Layers wrap everything above them; the last one listed sees the request first:
    // request-id → trace → headers/limits → CSRF → state → ctx (session) → page guard → app.
    let router = dioxus::server::router(crate::app::App)
        .route("/healthz", get(routes::healthz))
        .route("/api/export/users.csv", get(services::users::export_route))
        .route(
            "/api/export/audit.csv",
            get(services::audit_log::export_route),
        )
        .layer(axum::middleware::from_fn(auth::guard::pages))
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
