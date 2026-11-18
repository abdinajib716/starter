//! `features/auth/*`: sign in, wired to our session login (M1). Forgot password / invites
//! (email, M13) and the 2FA step (M9).
mod account;
mod sign_in;
mod two_factor;

pub use account::{AcceptInvite, ForgotPassword, ResetPassword, VerifyEmail};
pub use sign_in::{Login, SignIn};
pub use two_factor::TwoFactor;

use crate::t;
use dioxus::prelude::*;

use crate::components::ui_extra::icon::{Icon, IconType};

/// `assets/logo.tsx` → our branding (the command glyph used in the sidebar).
#[component]
pub fn Logo(#[props(into, optional)] class: Option<String>) -> Element {
    rsx! { Icon { icon: IconType::Command, class: tw_merge::tw_merge!("size-6", class.as_deref().unwrap_or("")) } }
}

/// `auth-layout.tsx`.
#[component]
pub fn AuthLayout(children: Element) -> Element {
    rsx! {
        crate::components::layout::NavigationProgress {}
        div { class: "container grid h-svh max-w-none items-center justify-center",
            div { class: "mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:p-8",
                div { class: "mb-4 flex items-center justify-center",
                    Logo { class: "me-2" }
                    h1 { class: "text-xl font-medium", {t!("app-name")} }
                }
                {children}
            }
        }
    }
}
