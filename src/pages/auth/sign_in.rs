//! `sign-in/index.tsx` + `user-auth-form.tsx`, wired to our login (M1).
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::AuthLayout;
use crate::api::auth::login;
use crate::app::Route;
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::form::{FormItem, FormLabel, FormMessage, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::AfterLogin;
use crate::domain::valid_email;
use crate::i18n::error_text;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::toast as show_toast;

/// `/login?next=` from earlier versions → `/sign-in?redirect=`.
#[component]
pub fn Login(next: String) -> Element {
    use_effect(move || {
        navigator().replace(Route::SignIn {
            redirect: next.clone(),
            reason: String::new(),
        });
    });
    rsx! {}
}

#[component]
pub fn SignIn(redirect: String, reason: String) -> Element {
    rsx! {
        AuthLayout {
            if reason == "session" {
                div { role: "status", class: "mx-auto mb-2 w-full max-w-sm rounded-lg border bg-card px-4 py-3 text-sm",
                    {t!("auth-session-ended")}
                }
            }
            Card { class: "max-w-sm gap-4",
                CardHeader {
                    CardTitle { class: "text-lg tracking-tight", {t!("login-title")} }
                    CardDescription { SignInBlurb {} }
                }
                CardContent { UserAuthForm { redirect } }
            }
        }
    }
}

#[component]
fn SignInBlurb() -> Element {
    rsx! {
        {t!("auth-sign-in-desc-1")}
    }
}

/// `user-auth-form.tsx`: the reference's checks, then our server login.
#[component]
fn UserAuthForm(redirect: String) -> Element {
    let email = use_signal(String::new);
    let password = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    let mut busy = use_signal(|| false);

    let e_email = if email().trim().is_empty() {
        Some(t!("v-email-enter"))
    } else if !valid_email(&email()) {
        Some(t!("v-email-invalid"))
    } else {
        None
    };
    let e_pw = if password().is_empty() {
        Some(t!("v-password-enter"))
    } else if password().chars().count() < 7 {
        Some(t!("v-password-min7"))
    } else {
        None
    };
    let show = submitted();
    let valid = e_email.is_none() && e_pw.is_none();

    let onsubmit = move |evt: FormEvent| {
        evt.prevent_default();
        let redirect = redirect.clone();
        async move {
            submitted.set(true);
            if !valid || busy() {
                return;
            }
            busy.set(true);
            let mut pw = password;
            match login(email(), pw()).await {
                Ok(next) => {
                    if next != AfterLogin::TwoFactor {
                        show_toast(
                            ToastType::Success,
                            crate::ta!("auth-welcome", email: email()),
                        );
                    }
                    let target = match next {
                        AfterLogin::ChangePassword => Route::Profile { tab: String::new() },
                        AfterLogin::TwoFactor => Route::TwoFactor {
                            redirect: redirect.clone(),
                        },
                        AfterLogin::Home => {
                            redirect.parse::<Route>().unwrap_or(Route::Dashboard {})
                        }
                    };
                    navigator().replace(target);
                }
                Err(e) => {
                    pw.set(String::new());
                    show_toast(ToastType::Error, error_text(&e));
                }
            }
            busy.set(false);
        }
    };

    rsx! {
        form { class: "grid gap-3", onsubmit,
            FormItem {
                FormLabel { html_for: "email", invalid: show && e_email.is_some(), {t!("field-email")} }
                TextInput { id: "email", value: email, placeholder: "name@example.com", invalid: show && e_email.is_some() }
                FormMessage { message: e_email.filter(|_| show) }
            }
            FormItem { class: "relative",
                FormLabel { html_for: "password", invalid: show && e_pw.is_some(), {t!("field-password")} }
                PasswordInput { id: "password", value: password, placeholder: "********", invalid: show && e_pw.is_some() }
                FormMessage { message: e_pw.filter(|_| show) }
            }
            button {
                id: "login-submit",
                r#type: "submit",
                disabled: busy(),
                class: tw_merge!(SUBMIT, "mt-2"),
                if busy() { Icon { icon: IconType::LoaderCircle, class: "animate-spin" } } else { Icon { icon: IconType::LogIn } }
                {t!("login-submit")}
            }
        }
    }
}

/// Default `Button` classes (`button.tsx`), for submit buttons that hold an icon.
pub(super) const SUBMIT: &str = "inline-flex h-9 items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium whitespace-nowrap text-primary-foreground shadow-xs transition-all hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50 has-[>svg]:px-3 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";
