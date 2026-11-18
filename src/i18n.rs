//! i18n layer (§26) on `dioxus-i18n` (Fluent). Shared by server and wasm.
//!
//! Locale resolution (§26): `users.locale` → `locale` cookie → `Accept-Language`
//! (only enabled locales) → default. Resolved on the server during SSR and handed to
//! the client through `use_server_cached`, so hydration renders the same language.
use dioxus_i18n::prelude::I18nConfig;
use dioxus_i18n::unic_langid::{LanguageIdentifier, langid};

pub const DEFAULT_LOCALE: &str = "en";
/// Enabled locales. `i18n.enabled_locales` (settings, M7) will narrow this at runtime.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub const SUPPORTED: &[&str] = &["en", "so"];
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub const LOCALE_COOKIE: &str = "locale";

/// Fluent bundles, embedded at compile time. `en` is the fallback for missing keys.
pub fn config(locale: &str) -> I18nConfig {
    let selected: LanguageIdentifier = locale.parse().unwrap_or(langid!("en"));
    I18nConfig::new(selected)
        .with_locale((langid!("en"), include_str!("../locales/en.ftl")))
        .with_locale((langid!("so"), include_str!("../locales/so.ftl")))
        .with_fallback(langid!("en"))
}

/// Translated text for a server-function error (§26 rule 2: the server sends codes).
/// Never panics: an unknown code falls back to `err-generic`.
pub fn error_text(err: &dioxus::prelude::ServerFnError) -> String {
    let key = crate::domain::errors::i18n_key(err);
    dioxus_i18n::te!(&key).unwrap_or_else(|_| crate::t!("err-generic"))
}

/// Right-to-left scripts; drives `dir="rtl"` (§26 rule 7). No RTL locale ships today,
/// but adding one needs no further code.
pub fn is_rtl(locale: &str) -> bool {
    matches!(primary(locale).as_str(), "ar" | "he" | "fa" | "ur")
}

fn primary(tag: &str) -> String {
    tag.split(['-', '_'])
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

#[cfg_attr(not(feature = "server"), allow(dead_code))]
/// Pure resolution, unit-tested below. Each candidate is reduced to its primary subtag.
pub fn resolve(
    user: Option<&str>,
    cookie: Option<&str>,
    accept_language: Option<&str>,
    enabled: &[&str],
    default: &str,
) -> String {
    let pick = |tag: &str| {
        let p = primary(tag);
        enabled.contains(&p.as_str()).then_some(p)
    };
    if let Some(found) = user.and_then(pick).or_else(|| cookie.and_then(pick)) {
        return found;
    }
    if let Some(header) = accept_language {
        // "so-SO,so;q=0.9,en;q=0.8" → highest q first; q=0 means "not acceptable".
        let mut langs: Vec<(f32, &str)> = header
            .split(',')
            .filter_map(|part| {
                let mut it = part.trim().split(';');
                let tag = it.next()?.trim();
                let q = it
                    .find_map(|p| p.trim().strip_prefix("q="))
                    .and_then(|q| q.parse().ok())
                    .unwrap_or(1.0);
                (!tag.is_empty() && q > 0.0).then_some((q, tag))
            })
            .collect();
        langs.sort_by(|a, b| b.0.total_cmp(&a.0));
        if let Some(found) = langs.into_iter().find_map(|(_, tag)| pick(tag)) {
            return found;
        }
    }
    default.to_string()
}

/// Locale for the current request (server, during SSR). On the client this is never
/// called for the first render: `use_server_cached` returns the server's value.
pub fn request_locale() -> String {
    #[cfg(feature = "server")]
    if let Some(ctx) = dioxus::fullstack::FullstackContext::current() {
        use dioxus::fullstack::http::header::{ACCEPT_LANGUAGE, COOKIE};
        let parts = ctx.parts_mut();
        let cookie = parts
            .headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(';'))
            .find_map(|c| c.trim().strip_prefix(LOCALE_COOKIE)?.strip_prefix('='))
            .map(str::to_owned);
        let accept = parts
            .headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|v| v.to_str().ok());
        let user = parts
            .extensions
            .get::<crate::server::ctx::Ctx>()
            .and_then(|c| c.user.as_ref())
            .and_then(|u| u.locale.clone());
        let (enabled, default) = crate::server::modules::Modules::get().locales();
        let enabled: Vec<&str> = enabled.iter().map(String::as_str).collect();
        return resolve(
            user.as_deref(),
            cookie.as_deref(),
            accept,
            &enabled,
            &default,
        );
    }
    DEFAULT_LOCALE.to_string()
}

/// Fluent wraps every placeable in Unicode isolation marks (U+2068 … U+2069). They are
/// invisible but end up in `aria-label`s and copied text, and `dioxus-i18n` has no switch
/// for `use_isolating`, so `ta!` (t! with arguments) strips them.
pub fn strip_isolation(s: String) -> String {
    s.replace(['\u{2068}', '\u{2069}'], "")
}

/// `t!` with arguments, without Fluent's isolation marks.
#[macro_export]
macro_rules! ta {
    ($($arg:tt)*) => { $crate::i18n::strip_isolation($crate::t!($($arg)*)) };
}

/// `t!` that never panics (M3.8): a key missing even in `en` logs once and renders the key.
#[macro_export]
macro_rules! t {
    ($key:expr $(, $($rest:tt)*)?) => {
        match dioxus_i18n::te!($key $(, $($rest)*)?) {
            Ok(s) => s,
            Err(_) => $crate::i18n::missing(&$key),
        }
    };
}

/// Translate a key chosen at run time with string arguments (notifications, audit labels).
pub fn tr(key: &str, args: &std::collections::HashMap<String, String>) -> String {
    use dioxus_i18n::fluent::FluentArgs;
    let mut a = FluentArgs::new();
    for (k, v) in args {
        a.set(k.clone(), v.clone());
    }
    strip_isolation(dioxus_i18n::prelude::i18n().translate_with_args(key, Some(&a)))
}

/// Missing translation: warn once per key, show the key.
pub fn missing(key: &str) -> String {
    use std::sync::Mutex;
    static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
    if let Ok(mut seen) = SEEN.lock()
        && !seen.iter().any(|k| k == key)
    {
        seen.push(key.to_string());
        #[cfg(feature = "server")]
        tracing::warn!(key, "missing translation");
        #[cfg(not(feature = "server"))]
        web_sys::console::warn_1(&format!("missing translation: {key}").into());
    }
    key.to_string()
}

#[cfg(test)]
mod tests {
    /// A broken `.ftl` file panics the app at the first render; catch it here instead.
    /// Each line is a comment, a `key = …` message, or part of a `{ $x -> … }` select.
    #[test]
    fn locale_files_are_well_formed() {
        for (name, text) in [
            ("en", include_str!("../locales/en.ftl")),
            ("so", include_str!("../locales/so.ftl")),
        ] {
            let mut in_select = false;
            let mut keys = std::collections::HashSet::new();
            for (i, line) in text.lines().enumerate() {
                let ok = if in_select {
                    if line.trim() == "}" {
                        in_select = false;
                    }
                    true
                } else if line.is_empty() || line.starts_with('#') {
                    true
                } else if let Some((key, value)) = line.split_once(" = ") {
                    in_select = value.trim_end().ends_with("->");
                    assert!(
                        keys.insert(key.trim().to_string()),
                        "{name}.ftl:{}: duplicate key {key}",
                        i + 1
                    );
                    key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                } else {
                    false
                };
                assert!(ok, "{name}.ftl:{}: unexpected line {line:?}", i + 1);
            }
            assert!(!in_select, "{name}.ftl: unclosed select expression");
        }
    }

    use super::*;

    const ENABLED: &[&str] = &["en", "so"];

    #[test]
    fn order_is_user_cookie_header_default() {
        assert_eq!(
            resolve(Some("so"), Some("en"), Some("en"), ENABLED, "en"),
            "so"
        );
        assert_eq!(resolve(None, Some("so"), Some("en"), ENABLED, "en"), "so");
        assert_eq!(
            resolve(None, None, Some("so-SO,en;q=0.5"), ENABLED, "en"),
            "so"
        );
        assert_eq!(resolve(None, None, None, ENABLED, "en"), "en");
    }

    #[test]
    fn accept_language_respects_q_and_enabled_list() {
        assert_eq!(
            resolve(
                None,
                None,
                Some("fr;q=1, so;q=0.4, en;q=0.6"),
                ENABLED,
                "en"
            ),
            "en"
        );
        assert_eq!(
            resolve(None, None, Some("en;q=0, so;q=0.1"), ENABLED, "en"),
            "so"
        );
        assert_eq!(resolve(None, None, Some("fr, de"), ENABLED, "en"), "en");
        assert_eq!(
            resolve(None, Some("xx"), Some("SO-so"), ENABLED, "en"),
            "so"
        );
    }

    #[test]
    fn rtl_detection() {
        assert!(is_rtl("he-IL"));
        assert!(!is_rtl("so"));
        assert!(!is_rtl("en"));
    }
}
