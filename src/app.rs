//! Root component, route table and document head.
// The `Routable` derive generates an unreachable call (Dioxus 0.7.10); an attribute on the
// enum does not reach the generated impl, so the lint is allowed for this module.
#![allow(unreachable_code)]
use dioxus::prelude::*;
use dioxus_i18n::prelude::use_init_i18n;

use crate::components::layout::AppLayout;
use crate::i18n;
use crate::pages::{home::Home, login::Login, not_found::NotFound, profile::Profile};
use crate::ui::toast::ToastToaster;

/// Route table (§12). Pages under `AppLayout` need a session: the server redirects page
/// loads (`server::auth::guard::pages`), `AppLayout` covers client-side navigation.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/login?:next")]
    Login { next: String },
    #[layout(AppLayout)]
        #[route("/")]
        Home {},
        #[route("/profile")]
        Profile {},
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
// Rust/UI runtimes (copied from rust-ui/ui @ 4f5a516)
const TOAST_RUNTIME_JS: Asset = asset!("/public/app_components/toast.js");
const TOAST_RUNTIME_CSS: Asset = asset!("/public/app_components/toast.css");
const APEXCHARTS_JS: Asset = asset!("/public/cdn/apexcharts.5.3.6.min.js");
const CHART_INIT_JS: Asset = asset!("/public/app_components/chart_init.js");

#[component]
pub fn App() -> Element {
    // Resolved on the server during SSR; the client reuses that value when hydrating.
    let initial = use_server_cached(i18n::request_locale);
    use_init_i18n(|| i18n::config(&initial));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        // Dioxus mounts the app inside <div id="main">; give it full height so
        // Rust/UI's html(100dvh) -> body(h-full) chain keeps working.
        document::Style { "#main {{ height: 100%; overflow-y: auto; }}" }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: TOAST_RUNTIME_CSS }
        document::Script { src: APEXCHARTS_JS }
        document::Script { src: CHART_INIT_JS }
        // `defer`: runs once after parsing (needs `document.body`). A plain `script {}` in the
        // body ran twice: once from the SSR HTML and again when hydration re-created it.
        document::Script { src: TOAST_RUNTIME_JS, defer: true }
        ToastToaster {}
        LocaleRoot { Router::<Route> {} }
    }
}

/// Sets `lang`/`dir` for the current locale. A separate component so that a language
/// switch re-renders only this subtree, not `App`'s `<head>` elements.
#[component]
fn LocaleRoot(children: Element) -> Element {
    let lang = dioxus_i18n::prelude::i18n().language().to_string();
    let dir = if i18n::is_rtl(&lang) { "rtl" } else { "ltr" };
    // Tailwind `rtl:` variants and logical utilities key off the nearest `dir`.
    rsx! {
        div { class: "h-full", lang: "{lang}", dir: "{dir}", {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::Route;

    /// `/login?next=` must never lead off-site: anything the router parses is an in-app route.
    #[test]
    fn next_param_stays_in_app() {
        // External URLs do not parse: login falls back to `/`.
        assert!("https://evil.example".parse::<Route>().is_err());
        // A protocol-relative path is just an unknown in-app path (404 page), not a redirect.
        assert!(matches!(
            "//evil.example/x".parse::<Route>(),
            Ok(Route::NotFound { .. })
        ));
        assert_eq!("/profile".parse::<Route>().ok(), Some(Route::Profile {}));
    }
}
