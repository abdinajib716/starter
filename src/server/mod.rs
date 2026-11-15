//! Server-only code (`feature = "server"`): config, state, DB, request context, plain routes.
pub mod config;
pub mod ctx;
pub mod db;
pub mod routes;
pub mod seed;
pub mod state;

use dioxus::server::axum::{self, Extension, Router, routing::get};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

/// Build the full Axum router: Dioxus app (SSR + server fns + assets) plus plain routes.
pub async fn router() -> anyhow::Result<Router> {
    let state = state::init().await?.clone();

    let router = dioxus::server::router(crate::app::App)
        .route("/healthz", get(routes::healthz))
        .layer(axum::middleware::from_fn(ctx::ctx_layer))
        .layer(Extension(state))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http().make_span_with(routes::make_span))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    Ok(router)
}
