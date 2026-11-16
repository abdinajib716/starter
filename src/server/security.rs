//! Security headers, body limit and timeout (§13, M1.16).
use std::time::Duration;

use dioxus::server::axum::{
    Router,
    http::{HeaderName, HeaderValue, StatusCode, header},
};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

use super::config::Config;

/// CSP notes: Dioxus hydration injects inline scripts, Rust/UI calls `document::eval`
/// (needs `'unsafe-eval'`), wasm needs `'wasm-unsafe-eval'`. Everything is same-origin.
fn csp(cfg: &Config) -> String {
    // `dx serve` hot reload uses a websocket on the same host.
    let connect = if cfg.is_production() {
        "'self'"
    } else {
        "'self' ws: wss:"
    };
    [
        "default-src 'self'",
        "script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval'",
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' data: blob:",
        "font-src 'self' data:",
        &format!("connect-src {connect}"),
        "object-src 'none'",
        "base-uri 'self'",
        "form-action 'self'",
        "frame-ancestors 'none'",
    ]
    .join("; ")
}

pub fn apply(router: Router, cfg: &Config) -> Router {
    let set = |name: HeaderName, value: &str| {
        SetResponseHeaderLayer::if_not_present(
            name,
            HeaderValue::from_str(value).expect("static header value"),
        )
    };
    let mut router = router
        .layer(set(header::CONTENT_SECURITY_POLICY, &csp(cfg)))
        .layer(set(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(set(header::X_FRAME_OPTIONS, "DENY"))
        .layer(set(
            header::REFERRER_POLICY,
            "strict-origin-when-cross-origin",
        ))
        .layer(set(
            HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=()",
        ))
        // 2 MiB covers JSON and the 1 MiB avatar upload (§9.1); raise per route if needed.
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        // Applies until response headers are sent, so SSE bodies (M8) may stream longer.
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ));
    if cfg.cookie_secure {
        router = router.layer(set(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains",
        ));
    }
    router
}
