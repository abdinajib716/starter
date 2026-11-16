//! Request guards that run before the Dioxus app (§7):
//! - CSRF: unsafe methods must come from our own origin.
//! - Page guard: SSR redirects for unauthenticated users, `must_change_pw`, and `/login`.
//!
//! The server-function checks stay the real security; these only shape navigation.
use dioxus::server::axum::{
    extract::{Request, State},
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

use crate::server::ctx::Ctx;
use crate::server::state::AppState;

/// Reject cross-site state-changing requests (M1.9).
pub async fn csrf(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }
    let h = req.headers();
    let text = |name| h.get(name).and_then(|v| v.to_str().ok());
    let origin = text(header::ORIGIN.as_str());
    let host = text("x-forwarded-host").or_else(|| text(header::HOST.as_str()));
    let fetch_site = text("sec-fetch-site");
    if !allowed(fetch_site, origin, state.config.app_url.as_deref(), host) {
        tracing::warn!(?origin, ?fetch_site, "blocked cross-origin request");
        return (StatusCode::FORBIDDEN, "cross-origin request blocked").into_response();
    }
    next.run(req).await
}

/// The decision, kept pure for testing.
///
/// 1. `Sec-Fetch-Site` (sent by every current browser, not settable by page scripts) decides
///    when present: `same-origin` and `none` (typed URL / bookmark) pass, `same-site` and
///    `cross-site` are refused. This works behind any proxy or tunnel, even when it rewrites
///    `Host` (the `dx serve` dev proxy does, and a forwarded port or tunnel changes the origin).
/// 2. Older browsers without it: `Origin` must equal `APP_URL`, or the request host when
///    `APP_URL` is unset.
/// 3. No `Origin` and no `Sec-Fetch-Site`: not a browser, so not a CSRF vector.
fn allowed(
    fetch_site: Option<&str>,
    origin: Option<&str>,
    app_url: Option<&str>,
    host: Option<&str>,
) -> bool {
    if let Some(site) = fetch_site {
        return matches!(site, "same-origin" | "none");
    }
    match (origin, app_url) {
        (None, _) => true,
        (Some(o), Some(app_url)) => same_origin(o, app_url),
        (Some(o), None) => {
            host.is_some_and(|h| o.split_once("://").is_some_and(|(_, rest)| rest == h))
        }
    }
}

/// Scheme + host + port equality; trailing slash on `APP_URL` ignored.
fn same_origin(origin: &str, app_url: &str) -> bool {
    origin
        .trim_end_matches('/')
        .eq_ignore_ascii_case(app_url.trim_end_matches('/'))
}

/// Paths that are never page navigations or never need a session.
fn is_passthrough(path: &str) -> bool {
    path.starts_with("/api/") || path == "/healthz" || crate::server::ctx::is_asset(path)
}

/// Public pages (§12, §30.4). Everything else needs a session.
fn is_public_page(path: &str) -> bool {
    path == "/login" || path.starts_with("/auth/") || path == "/403"
}

pub async fn pages(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    if req.method() != Method::GET || is_passthrough(&path) {
        return next.run(req).await;
    }
    // Decide first (borrowing the request), then act (moving it).
    let redirect = match req.extensions().get::<Ctx>().and_then(|c| c.user.as_ref()) {
        None if !is_public_page(&path) => Some(if path == "/" {
            "/login".to_string()
        } else {
            format!("/login?next={path}")
        }),
        Some(u) if u.must_change_pw && path != "/profile" => Some("/profile".to_string()),
        Some(_) if path == "/login" => Some("/".to_string()),
        _ => None,
    };
    match redirect {
        Some(to) => Redirect::to(&to).into_response(),
        None => next.run(req).await,
    }
}

#[cfg(test)]
mod tests {
    use super::allowed;

    const APP: Option<&str> = Some("http://127.0.0.1:8080");

    #[test]
    fn fetch_metadata_decides_when_present() {
        // Same page reached through a tunnel or forwarded port: Origin differs, browser says same-origin.
        assert!(allowed(
            Some("same-origin"),
            Some("https://x.devtunnels.ms"),
            APP,
            None
        ));
        assert!(allowed(Some("none"), None, APP, None));
        assert!(!allowed(
            Some("cross-site"),
            Some("http://127.0.0.1:8080"),
            APP,
            None
        ));
        assert!(!allowed(
            Some("same-site"),
            Some("https://other.example.com"),
            APP,
            None
        ));
    }

    #[test]
    fn origin_fallback_for_older_browsers() {
        assert!(allowed(None, Some("http://127.0.0.1:8080/"), APP, None));
        assert!(!allowed(None, Some("https://evil.example"), APP, None));
        assert!(!allowed(None, Some("http://127.0.0.1:8081"), APP, None));
        assert!(allowed(
            None,
            Some("https://app.example.com"),
            None,
            Some("app.example.com")
        ));
        assert!(!allowed(
            None,
            Some("https://evil.example"),
            None,
            Some("app.example.com")
        ));
        // Non-browser client (curl, mobile app): neither header.
        assert!(allowed(None, None, APP, None));
    }
}
