//! Per-request context (§18 "Enforcement 1"): every service takes `&Ctx`.
use std::collections::HashSet;

use dioxus::fullstack::FullstackContext;
use dioxus::prelude::ServerFnError;
use dioxus::server::axum::{Extension, extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

use super::state::AppState;

/// The signed-in user; filled by the session middleware in M1.
#[allow(dead_code)] // read from M1 on
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role_id: Uuid,
    pub permissions: HashSet<String>,
}

#[allow(dead_code)] // fields read by services from M1 on
#[derive(Debug, Clone)]
pub struct Ctx {
    pub org_id: Uuid,
    pub user: Option<CurrentUser>,
    pub request_id: Option<String>,
}

impl Ctx {
    /// Read the `Ctx` the middleware attached to the current request (inside a server fn).
    pub async fn current() -> Result<Ctx, ServerFnError> {
        let Extension(ctx) = FullstackContext::extract::<Extension<Ctx>, _>()
            .await
            .map_err(|_| ServerFnError::new("request context missing"))?;
        Ok(ctx)
    }
}

/// Builds `Ctx` for every request. Single mode: always the default org.
/// Multi mode (M16) resolves the org from the `Host` header instead.
pub async fn ctx_layer(
    Extension(state): Extension<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    req.extensions_mut().insert(Ctx {
        org_id: state.default_org_id,
        user: None,
        request_id,
    });
    next.run(req).await
}
