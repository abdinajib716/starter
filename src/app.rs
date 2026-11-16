//! Root component, route table and document head.
// The `Routable` derive generates an unreachable call (Dioxus 0.7.10); an attribute on the
// enum does not reach the generated impl, so the lint is allowed for this module.
#![allow(unreachable_code)]
use dioxus::prelude::*;
use dioxus_i18n::prelude::use_init_i18n;

use crate::components::layout::AppShell;
use crate::i18n;
use crate::pages::{dashboard::Dashboard, login::Login, not_found::NotFound, profile::Profile};
use crate::state::{Ui, UiPrefs};
use crate::ui::toast::ToastToaster;

/// Route table (§12, SPEC-02). Pages under `AppShell` need a session: the server redirects
/// page loads (`server::auth::guard::pages`), `AppShell` covers client-side navigation.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/login?:next")]
    Login { next: String },
    #[layout(AppShell)]
        #[route("/")]
        Dashboard {},
        #[route("/profile")]
        Profile {},
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
// Rust/UI toast runtime (copied from rust-ui/ui @ 4f5a516). The ApexCharts runtime is no
// longer loaded: dashboard charts are SVG components (`ui_extra::chart`).
const TOAST_RUNTIME_JS: Asset = asset!("/public/app_components/toast.js");
const TOAST_RUNTIME_CSS: Asset = asset!("/public/app_components/toast.css");

#[component]
pub fn App() -> Element {
    // Resolved on the server during SSR; the client reuses that value when hydrating.
    let initial = use_server_cached(i18n::request_locale);
    use_init_i18n(|| i18n::config(&initial));
    // Theme, sidebar, layout and direction from cookies (SSR), shared app-wide.
    let prefs = use_server_cached(UiPrefs::from_request);
    use_hook(|| Ui::provide(prefs));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: TOAST_RUNTIME_CSS }
        // `defer`: runs once after parsing (needs `document.body`). A plain `script {}` in the
        // body ran twice: once from the SSR HTML and again when hydration re-created it.
        document::Script { src: TOAST_RUNTIME_JS, defer: true }
        ToastToaster {}
        LocaleRoot { Router::<Route> {} }
    }
}

/// Sets `lang` and `dir`. A separate component so that switching language or direction
/// re-renders only this subtree, not `App`'s `<head>` elements. Direction is the user's
/// choice (appearance drawer), or RTL for an RTL locale.
#[component]
fn LocaleRoot(children: Element) -> Element {
    let lang = dioxus_i18n::prelude::i18n().language().to_string();
    let ui = Ui::get();
    let rtl = (ui.dir)() == crate::state::Dir::Rtl || i18n::is_rtl(&lang);
    let dir = if rtl { "rtl" } else { "ltr" };
    // Tailwind `rtl:` variants and logical utilities key off the nearest `dir`.
    rsx! {
        div { lang: "{lang}", dir: "{dir}", {children} }
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
