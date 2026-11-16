//! `features/auth/*`. Sign in is wired to our session login (M1). Sign up, forgot password and
//! OTP are the reference screens; their back-ends arrive later (self sign-up setting, Resend
//! email M13, 2FA M9), so their submit messages say what actually happened.
mod forgot_password;
mod otp;
mod sign_in;
mod sign_up;

pub use forgot_password::ForgotPassword;
pub use otp::Otp;
pub use sign_in::{Login, SignIn, SignIn2};
pub use sign_up::SignUp;

use dioxus::prelude::*;
use dioxus_i18n::t;

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

/// "Or continue with" divider + GitHub / Facebook buttons (not wired in the reference either).
#[component]
pub fn SocialButtons(disabled: bool) -> Element {
    rsx! {
        div { class: "relative my-2",
            div { class: "absolute inset-0 flex items-center", span { class: "w-full border-t" } }
            div { class: "relative flex justify-center text-xs uppercase",
                span { class: "bg-card px-2 text-muted-foreground", {t!("auth-continue-with")} }
            }
        }
        div { class: "grid grid-cols-2 gap-2",
            for (name, path) in [
                ("GitHub", "M9 19c-4.3 1.4 -4.3 -2.5 -6 -3m12 5v-3.5c0 -1 .1 -1.4 -.5 -2c2.8 -.3 5.5 -1.4 5.5 -6a4.6 4.6 0 0 0 -1.3 -3.2a4.2 4.2 0 0 0 -.1 -3.2s-1.1 -.3 -3.5 1.3a12.3 12.3 0 0 0 -6.2 0c-2.4 -1.6 -3.5 -1.3 -3.5 -1.3a4.2 4.2 0 0 0 -.1 3.2a4.6 4.6 0 0 0 -1.3 3.2c0 4.6 2.7 5.7 5.5 6c-.6 .6 -.6 1.2 -.5 2v3.5"),
                ("Facebook", "M7 10v4h3v7h4v-7h3l1 -4h-4v-2a1 1 0 0 1 1 -1h3v-4h-3a5 5 0 0 0 -5 5v2h-3"),
            ] {
                button {
                    r#type: "button",
                    disabled,
                    class: "inline-flex h-9 items-center justify-center gap-2 rounded-md border bg-background px-4 py-2 text-sm font-medium whitespace-nowrap shadow-xs transition-all hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50 dark:border-input dark:bg-input/30 dark:hover:bg-input/50",
                    svg { view_box: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", class: "h-4 w-4",
                        path { d: path }
                    }
                    "{name}"
                }
            }
        }
    }
}

/// Terms / privacy line under the cards.
#[component]
pub fn TermsNote(lead: String) -> Element {
    rsx! {
        p { class: "px-8 text-center text-sm text-muted-foreground",
            "{lead} "
            a { href: "/terms", class: "underline underline-offset-4 hover:text-primary", {t!("auth-terms")} }
            " "
            {t!("auth-and")}
            " "
            a { href: "/privacy", class: "underline underline-offset-4 hover:text-primary", {t!("auth-privacy")} }
            "."
        }
    }
}

/// Email shape check used by the reference's `z.email()` fields.
pub fn valid_email(s: &str) -> bool {
    let s = s.trim();
    match s.split_once('@') {
        Some((user, host)) => {
            !user.is_empty()
                && host.contains('.')
                && !host.starts_with('.')
                && !host.ends_with('.')
                && !s.contains(' ')
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn email_check() {
        assert!(super::valid_email("a@b.co"));
        assert!(!super::valid_email("a@b"));
        assert!(!super::valid_email("@b.co"));
        assert!(!super::valid_email("a b@c.de"));
    }
}
