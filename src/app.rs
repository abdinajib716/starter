//! Root component, route table and document head.
// The `Routable` derive generates an unreachable call (Dioxus 0.7.10); an attribute on the
// enum does not reach the generated impl, so the lint is allowed for this module.
#![allow(unreachable_code)]
use dioxus::prelude::*;
use dioxus_i18n::prelude::use_init_i18n;

use crate::components::layout::AppShell;
use crate::i18n;
use crate::pages::auth::{
    AcceptInvite, ForgotPassword, Login, ResetPassword, SignIn, TwoFactor, VerifyEmail,
};
use crate::pages::errors::{Error401, Error403, Error404, Error500, Error503, ShellError};
use crate::pages::{
    audit::Audit,
    backups::Backups,
    dashboard::Dashboard,
    not_found::NotFound,
    notifications::Notifications,
    organizations::Organizations,
    permissions::Permissions,
    profile::Profile,
    roles::Roles,
    system_settings::SystemSettings,
    users::{UserDetailPage, Users},
};
use crate::state::{Ui, UiPrefs};
use crate::ui::toast::ToastToaster;
use crate::util::query::Query;

/// Route table (§12, SPEC-02). Pages under `AppShell` need a session: the server redirects
/// page loads (`server::auth::guard::pages`), `AppShell` covers client-side navigation.
/// Public: the auth screens (`/sign-in` …) and the standalone error pages (`/401` … `/503`).
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    /// `reason=session`: the session ended (signed out elsewhere, password / role changed).
    #[route("/sign-in?:redirect&:reason")]
    SignIn { redirect: String, reason: String },
    /// Old login URL, kept as a redirect to `/sign-in`.
    /// Second sign-in step (§21): TOTP or recovery code.
    #[route("/auth/2fa?:redirect")]
    TwoFactor { redirect: String },
    #[route("/auth/forgot-password")]
    ForgotPassword {},
    #[route("/auth/reset-password?:token")]
    ResetPassword { token: String },
    #[route("/auth/accept-invite?:token")]
    AcceptInvite { token: String },
    #[route("/auth/verify-email?:token")]
    VerifyEmail { token: String },
    #[route("/login?:next")]
    Login { next: String },
    #[route("/401")]
    Error401 {},
    #[route("/403")]
    Error403 {},
    #[route("/404")]
    Error404 {},
    #[route("/500")]
    Error500 {},
    #[route("/503")]
    Error503 {},
    #[layout(AppShell)]
        #[route("/")]
        Dashboard {},
        #[route("/profile?:tab")]
        Profile { tab: String },
        #[route("/organizations")]
        Organizations {},
        #[route("/notifications")]
        Notifications {},
        #[route("/users?:..query")]
        Users { query: Query },
        #[route("/users/:id")]
        UserDetailPage { id: uuid::Uuid },
        #[route("/system-settings")]
        SystemSettings {},
        #[route("/roles")]
        Roles {},
        #[route("/permissions")]
        Permissions {},
        #[route("/audit?:..query")]
        Audit { query: Query },
        #[route("/backups")]
        Backups {},
        #[route("/errors/:error")]
        ShellError { error: String },
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

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
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: TOAST_RUNTIME_CSS }
        // `defer`: runs once after parsing (needs `document.body`). A plain `script {}` in the
        // body ran twice: once from the SSR HTML and again when hydration re-created it.
        document::Script { src: TOAST_RUNTIME_JS, defer: true }
        ToastToaster {}
        // Without a boundary, a page waiting on a server future (e.g. the shell's
        // `current_user()`) suspends the whole app root, which tears down the toaster too.
        LocaleRoot {
            SuspenseBoundary { fallback: |_| rsx! { crate::components::skeletons::ShellSkeleton {} }, Router::<Route> {} }
        }
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

    /// `/sign-in?redirect=` must never lead off-site: anything the router parses is an in-app route.
    #[test]
    fn next_param_stays_in_app() {
        // External URLs do not parse: login falls back to `/`.
        assert!("https://evil.example".parse::<Route>().is_err());
        // A protocol-relative path is just an unknown in-app path (404 page), not a redirect.
        assert!(matches!(
            "//evil.example/x".parse::<Route>(),
            Ok(Route::NotFound { .. })
        ));
        assert_eq!(
            "/profile".parse::<Route>().ok(),
            Some(Route::Profile { tab: String::new() })
        );
    }
}
