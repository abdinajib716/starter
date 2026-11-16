//! Cookie helpers (port of shadcn-admin `src/lib/cookies.ts`).
//!
//! Preference cookies (theme, sidebar, layout, direction) are read on the server during
//! SSR, so the first paint already matches, and written from the browser when changed.

/// Value of cookie `name` in a `Cookie` header string (`a=1; b=2`).
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn find(header: &str, name: &str) -> Option<String> {
    header
        .split(';')
        .find_map(|c| c.trim().strip_prefix(name)?.strip_prefix('='))
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// Cookie of the current request (server, during SSR). `None` on the client.
pub fn request(name: &str) -> Option<String> {
    #[cfg(feature = "server")]
    if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
        let parts = ctx.parts_mut();
        return parts
            .headers
            .get_all(dioxus::fullstack::http::header::COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find_map(|h| find(h, name));
    }
    let _ = name;
    None
}

/// Set a cookie from the browser. Names and values are internal constants, never user input.
pub fn set(name: &str, value: &str, max_age_secs: u32) {
    let js = format!(
        "document.cookie = '{}'",
        cookie_string(name, value, max_age_secs)
    );
    let _ = dioxus::document::eval(&js);
}

/// The `document.cookie` assignment; `max-age=0` with an empty value deletes the cookie.
fn cookie_string(name: &str, value: &str, max_age_secs: u32) -> String {
    format!("{name}={value}; path=/; max-age={max_age_secs}; samesite=lax")
}

pub fn remove(name: &str) {
    set(name, "", 0);
}

#[cfg(test)]
mod tests {
    use super::{cookie_string, find};

    /// What the browser sends back for a cookie set with `cookie_string` (name=value only).
    fn sent(cookie: &str) -> String {
        cookie.split(';').next().unwrap_or_default().to_string()
    }

    // lib/cookies.test.ts
    #[test]
    fn stores_a_value_that_can_be_read_back() {
        let c = cookie_string("test_cookie_a", "hello-world", 60);
        assert_eq!(
            find(&sent(&c), "test_cookie_a").as_deref(),
            Some("hello-world")
        );
    }

    #[test]
    fn clears_a_value_so_it_is_no_longer_readable() {
        let c = cookie_string("test_cookie_b", "", 0);
        assert!(c.contains("max-age=0"));
        assert_eq!(find(&sent(&c), "test_cookie_b"), None);
    }

    #[test]
    fn finds_named_cookie() {
        assert_eq!(
            find("a=1; theme=dark; sidebar_state=false", "theme").as_deref(),
            Some("dark")
        );
        assert_eq!(find("theme_x=1; theme=", "theme"), None);
        assert_eq!(find("", "theme"), None);
    }
}
