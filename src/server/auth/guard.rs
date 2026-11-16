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

/// Reject cross-site state-changing requests (M1.9). Browsers always send `Origin` on
/// cross-origin POSTs, and `Sec-Fetch-Site` on modern ones; non-browser clients send
/// neither and are not a CSRF vector.
///
/// The `Origin` must equal `APP_URL` when configured. Otherwise it must match the request's
/// host (`X-Forwarded-Host` from a proxy, else `Host`). `APP_URL` is needed whenever a
/// proxy rewrites `Host`, as the `dx serve` dev proxy does.
pub async fn csrf(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }
    let h = req.headers();
    let text = |name| h.get(name).and_then(|v| v.to_str().ok());
    let origin = text(header::ORIGIN.as_str());
    let allowed = match (origin, state.config.app_url.as_deref()) {
        (None, _) => true,
        (Some(o), Some(app_url)) => same_origin(o, app_url),
        (Some(o), None) => {
            let host = text("x-forwarded-host").or_else(|| text(header::HOST.as_str()));
            host.is_some_and(|h| o.split_once("://").is_some_and(|(_, rest)| rest == h))
        }
    };
    let cross_site = text("sec-fetch-site") == Some("cross-site");
    if !allowed || cross_site {
        tracing::warn!(?origin, "blocked cross-origin request");
        return (StatusCode::FORBIDDEN, "cross-origin request blocked").into_response();
    }
    next.run(req).await
}

/// Scheme + host + port equality; trailing slash on `APP_URL` ignored.
fn same_origin(origin: &str, app_url: &str) -> bool {
    origin
        .trim_end_matches('/')
        .eq_ignore_ascii_case(app_url.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
    #[test]
    fn origin_matching() {
        assert!(super::same_origin(
            "http://127.0.0.1:8080",
            "http://127.0.0.1:8080/"
        ));
        assert!(!super::same_origin(
            "https://evil.example",
            "http://127.0.0.1:8080"
        ));
        assert!(!super::same_origin(
            "http://127.0.0.1:8081",
            "http://127.0.0.1:8080"
        ));
    }
}

/// Paths that are never page navigations or never need a session.
fn is_passthrough(path: &str) -> bool {
    path.starts_with("/api/")
        || path.starts_with("/assets/")
        || path.starts_with("/wasm/")
        || path.starts_with("/_dioxus")
        || path.starts_with("/favicon")
        || path == "/healthz"
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
