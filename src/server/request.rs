//! Request metadata for audit, sessions and rate limits.
use std::net::IpAddr;

use dioxus::fullstack::http::{HeaderMap, header::USER_AGENT};

use super::config::Config;

#[derive(Debug, Clone, Default)]
pub struct RequestMeta {
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

impl RequestMeta {
    pub fn from_headers(cfg: &Config, headers: &HeaderMap) -> Self {
        // Only trust the header the operator configured for their reverse proxy. With
        // `x-forwarded-for` the proxy appends the peer address last, so take the last entry.
        let ip = cfg.client_ip_header.as_deref().and_then(|name| {
            headers
                .get(name)?
                .to_str()
                .ok()?
                .rsplit(',')
                .next()?
                .trim()
                .parse()
                .ok()
        });
        let user_agent = headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.chars().take(512).collect());
        Self { ip, user_agent }
    }
}
