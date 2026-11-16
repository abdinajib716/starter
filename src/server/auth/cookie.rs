//! Session cookie (§7 step 7, M1.17).
use dioxus::fullstack::http::{HeaderMap, header::COOKIE};

use crate::server::config::Config;

/// `HttpOnly; SameSite=Lax; Path=/` (+ `Secure` with `COOKIE_SECURE`). No `Domain`.
pub fn set(cfg: &Config, token: &str, max_age_secs: i64) -> String {
    let secure = if cfg.cookie_secure { "; Secure" } else { "" };
    format!(
        "{}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age_secs}{secure}",
        cfg.session_cookie_name()
    )
}

pub fn clear(cfg: &Config) -> String {
    set(cfg, "", 0)
}

/// Value of cookie `name` from the request's `Cookie` headers.
pub fn read<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|c| c.trim().strip_prefix(name)?.strip_prefix('='))
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::fullstack::HeaderValue;

    #[test]
    fn reads_named_cookie_only() {
        let mut h = HeaderMap::new();
        h.append(COOKIE, HeaderValue::from_static("a=1; sidx=2; sid=abc"));
        assert_eq!(read(&h, "sid"), Some("abc"));
        assert_eq!(read(&h, "nope"), None);
        h = HeaderMap::new();
        h.append(COOKIE, HeaderValue::from_static("sid="));
        assert_eq!(read(&h, "sid"), None);
    }
}
