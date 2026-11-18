//! Per-request context (§18 "Enforcement 1"): every service takes `&Ctx`.
use std::collections::HashSet;

use dioxus::fullstack::FullstackContext;
use dioxus::prelude::ServerFnError;
use dioxus::server::axum::{
    Extension,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
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
    pub org_id: Uuid,
    pub email: String,
    pub full_name: String,
    #[allow(dead_code)] // M5 (role changes)
    pub role_id: Uuid,
    pub role_name: String,
    pub is_super_admin: bool,
    pub permissions: HashSet<String>,
    pub must_change_pw: bool,
    pub locale: Option<String>,
    /// Own time zone; `None` = the organization default (`app.timezone`).
    pub timezone: Option<String>,
    /// `files.id` of the avatar.
    pub avatar: Option<String>,
    /// A confirmed TOTP exists (§21).
    pub twofa_enrolled: bool,
    pub role_require_2fa: bool,
    /// When 2FA became required for this user (later of: account created, mode changed).
    pub twofa_since: time::OffsetDateTime,
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

    /// The signed-in user holding permission `key` (§8), or 401 / 403.
    pub fn require(&self, key: &str) -> Result<&CurrentUser, super::error::AppError> {
        super::rbac::require(self, key)
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

    // Multi mode (§18): the organization comes from the subdomain.
    let org_id = match state.config.tenancy_mode {
        crate::domain::TenancyMode::Single => state.default_org_id,
        crate::domain::TenancyMode::Multi => {
            let host = headers
                .get("x-forwarded-host")
                .or_else(|| headers.get(dioxus::server::axum::http::header::HOST))
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            match resolve_org(&state, host).await {
                Some(id) => id,
                None => {
                    return (
                        dioxus::server::axum::http::StatusCode::NOT_FOUND,
                        "unknown organization",
                    )
                        .into_response();
                }
            }
        }
    };

    let mut user = None;
    if !is_asset(req.uri().path())
        && let Some(raw) = cookie::read(headers, state.config.session_cookie_name())
    {
        let hours = SecuritySettings::load(&state.pool)
            .await
            .unwrap_or_default()
            .session_hours;
        match session::load(&state.pool, raw, hours).await {
            // A session only counts on its own organization's host.
            Ok(u) => user = u.filter(|u| u.org_id == org_id),
            Err(e) => tracing::error!(error = %e, "session lookup failed"),
        }
    }

    req.extensions_mut().insert(Ctx {
        org_id,
        user,
        request_id,
        meta,
    });
    if state.config.tenancy_mode == crate::domain::TenancyMode::Multi {
        super::db::ORG.scope(org_id, next.run(req)).await
    } else {
        next.run(req).await
    }
}

/// `acme.app.example.com` → the active organization with slug `acme`; the bare base domain
/// (or a host outside it) → the default organization. Cached for a minute.
pub async fn resolve_org(state: &AppState, host: &str) -> Option<Uuid> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    type SlugCache = Option<HashMap<String, (Option<Uuid>, Instant)>>;
    static CACHE: Mutex<SlugCache> = Mutex::new(None);
    let host = host.split(':').next().unwrap_or("").to_ascii_lowercase();
    let slug = state
        .config
        .app_base_domain
        .as_deref()
        .and_then(|base| host.strip_suffix(&format!(".{}", base.to_ascii_lowercase())))
        .filter(|s| !s.is_empty() && !s.contains('.'))
        .map(str::to_owned);
    let Some(slug) = slug else {
        return Some(state.default_org_id);
    };
    if let Some((id, at)) = CACHE
        .lock()
        .ok()?
        .get_or_insert_with(HashMap::new)
        .get(&slug)
        .copied()
        && at.elapsed() < Duration::from_secs(60)
    {
        return id;
    }
    let id = sqlx::query_scalar!(
        "SELECT id FROM organizations WHERE slug = $1::text::citext AND is_active",
        slug
    )
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    if let Ok(mut c) = CACHE.lock() {
        c.get_or_insert_with(HashMap::new)
            .insert(slug, (id, Instant::now()));
    }
    id
}

impl CurrentUser {
    /// Second-factor state under the current `security.2fa_mode` (§21, M9.11).
    pub fn twofa_info(&self) -> crate::domain::security::TwoFaInfo {
        use crate::domain::TwoFaMode;
        let m = super::modules::Modules::get();
        let required = match m.twofa_mode() {
            TwoFaMode::RequiredAll => true,
            TwoFaMode::RequiredRoles => self.role_require_2fa,
            _ => false,
        };
        let deadline = (required && !self.twofa_enrolled)
            .then(|| self.twofa_since + time::Duration::days(m.twofa_grace_days()));
        crate::domain::security::TwoFaInfo {
            enrolled: self.twofa_enrolled,
            required,
            blocked: deadline.is_some_and(|d| d <= time::OffsetDateTime::now_utc()),
            deadline,
        }
    }
}
