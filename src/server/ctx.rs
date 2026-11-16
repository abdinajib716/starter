//! Per-request context (§18 "Enforcement 1"): every service takes `&Ctx`.
use std::collections::HashSet;

use dioxus::fullstack::FullstackContext;
use dioxus::prelude::ServerFnError;
use dioxus::server::axum::{Extension, extract::Request, middleware::Next, response::Response};
use uuid::Uuid;

use super::auth::{cookie, session};
use super::request::RequestMeta;
use super::settings::SecuritySettings;
use super::state::AppState;

/// The signed-in user, loaded from the session cookie on every request (§7).
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub session_id: Uuid,
    #[allow(dead_code)] // M16 (multi-tenant checks)
    pub org_id: Uuid,
    pub email: String,
    pub full_name: String,
    #[allow(dead_code)] // M2.4 escalation guard
    pub role_id: Uuid,
    pub role_name: String,
    pub is_super_admin: bool,
    pub permissions: HashSet<String>,
    pub must_change_pw: bool,
    #[allow(dead_code)] // M3.9 locale resolution
    pub locale: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Ctx {
    pub org_id: Uuid,
    pub user: Option<CurrentUser>,
    #[allow(dead_code)] // M4 audit / log correlation
    pub request_id: Option<String>,
    pub meta: RequestMeta,
}

impl Ctx {
    /// Read the `Ctx` the middleware attached to the current request (inside a server fn).
    pub async fn current() -> Result<Ctx, ServerFnError> {
        let Extension(ctx) = FullstackContext::extract::<Extension<Ctx>, _>()
            .await
            .map_err(|_| ServerFnError::new("request context missing"))?;
        Ok(ctx)
    }

    /// The signed-in user, or `401 auth.unauthenticated`.
    pub fn require_user(&self) -> Result<&CurrentUser, ServerFnError> {
        self.user
            .as_ref()
            .ok_or_else(|| crate::domain::errors::coded(401, "auth.unauthenticated"))
    }
}

/// Add a `Set-Cookie` header to the current server-function response.
pub fn set_cookie(value: &str) -> Result<(), ServerFnError> {
    use dioxus::fullstack::{HeaderValue, http::header::SET_COOKIE};
    let ctx = FullstackContext::current().ok_or_else(|| ServerFnError::new("no request"))?;
    let value = HeaderValue::from_str(value).map_err(|e| ServerFnError::new(e.to_string()))?;
    ctx.add_response_header(SET_COOKIE, value);
    Ok(())
}

/// Static files (dx assets, wasm, dev tooling, `public/` folders) never need a session.
pub const STATIC_PREFIXES: &[&str] = &[
    "/assets/", "/wasm/", "/_dioxus", "/favicon", "/fonts/", "/images/",
];

pub fn is_asset(path: &str) -> bool {
    STATIC_PREFIXES.iter().any(|p| path.starts_with(p))
}

/// Builds `Ctx` for every request: default org (single mode; multi mode resolves it from
/// `Host`, M16) plus the user behind the session cookie, if any.
pub async fn ctx_layer(
    Extension(state): Extension<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let headers = req.headers();
    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let meta = RequestMeta::from_headers(&state.config, headers);

    let mut user = None;
    if !is_asset(req.uri().path())
        && let Some(raw) = cookie::read(headers, state.config.session_cookie_name())
    {
        let hours = SecuritySettings::load(&state.pool)
            .await
            .unwrap_or_default()
            .session_hours;
        match session::load(&state.pool, raw, hours).await {
            Ok(u) => user = u,
            Err(e) => tracing::error!(error = %e, "session lookup failed"),
        }
    }

    req.extensions_mut().insert(Ctx {
        org_id: state.default_org_id,
        user,
        request_id,
        meta,
    });
    next.run(req).await
}
