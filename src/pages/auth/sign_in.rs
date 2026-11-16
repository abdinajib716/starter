//! `sign-in/index.tsx`, `sign-in-2.tsx`, `user-auth-form.tsx`, wired to our login (M1).
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::{AuthLayout, Logo, SocialButtons, TermsNote, valid_email};
use crate::api::auth::login;
use crate::app::Route;
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::form::{FormItem, FormLabel, FormMessage, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::AfterLogin;
use crate::i18n::error_text;
use crate::ui::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::toast as show_toast;

const DASHBOARD_LIGHT: Asset = asset!("/assets/auth/dashboard-light.png");
const DASHBOARD_DARK: Asset = asset!("/assets/auth/dashboard-dark.png");

/// `/login?next=` from earlier versions → `/sign-in?redirect=`.
#[component]
pub fn Login(next: String) -> Element {
    use_effect(move || {
        navigator().replace(Route::SignIn {
            redirect: next.clone(),
        });
    });
    rsx! {}
}

#[component]
pub fn SignIn(redirect: String) -> Element {
    rsx! {
        AuthLayout {
            Card { class: "max-w-sm gap-4",
                CardHeader {
                    CardTitle { class: "text-lg tracking-tight", {t!("login-title")} }
                    CardDescription { SignInBlurb {} }
                }
                CardContent { UserAuthForm { redirect } }
                CardFooter { TermsNote { lead: t!("auth-terms-lead-sign-in") } }
            }
        }
    }
}

#[component]
pub fn SignIn2() -> Element {
    rsx! {
        div { class: "relative container grid h-svh flex-col items-center justify-center lg:max-w-none lg:grid-cols-2 lg:px-0",
            div { class: "lg:p-8",
                div { class: "mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-120 sm:p-8",
                    div { class: "mb-4 flex items-center justify-center",
                        Logo { class: "me-2" }
                        h1 { class: "text-xl font-medium", {t!("app-name")} }
                    }
                }
                div { class: "mx-auto flex w-full max-w-sm flex-col justify-center space-y-2",
                    div { class: "flex flex-col space-y-2 text-start",
                        h2 { class: "text-lg font-semibold tracking-tight", {t!("login-title")} }
                        p { class: "text-sm text-muted-foreground", SignInBlurb {} }
                    }
                    UserAuthForm { redirect: String::new() }
                    TermsNote { lead: t!("auth-terms-lead-sign-in") }
                }
            }
            div { class: "relative h-full overflow-hidden bg-muted max-lg:hidden [&>img]:absolute [&>img]:top-[15%] [&>img]:left-20 [&>img]:h-full [&>img]:w-full [&>img]:object-cover [&>img]:object-top-left [&>img]:select-none",
                img { src: DASHBOARD_LIGHT, class: "dark:hidden", width: "1024", height: "1151", alt: t!("app-name") }
                img { src: DASHBOARD_DARK, class: "hidden dark:block", width: "1024", height: "1138", alt: t!("app-name") }
            }
        }
    }
}

#[component]
fn SignInBlurb() -> Element {
    rsx! {
        {t!("auth-sign-in-desc-1")}
        " "
        br { class: "max-sm:hidden" }
        {t!("auth-sign-in-desc-2")}
        " "
        Link { to: Route::SignUp {}, class: "text-nowrap underline underline-offset-4 hover:text-primary", {t!("nav-sign-up")} }
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
                    show_toast(ToastType::Success, t!("auth-welcome", email: email()));
                    let target = match next {
                        AfterLogin::ChangePassword => Route::Profile {},
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
                Link {
                    to: Route::ForgotPassword {},
                    class: "absolute end-0 -top-0.5 text-sm font-medium text-muted-foreground hover:opacity-75",
                    {t!("auth-forgot")}
                }
            }
            button {
                id: "login-submit",
                r#type: "submit",
                disabled: busy(),
                class: tw_merge!(SUBMIT, "mt-2"),
                if busy() { Icon { icon: IconType::LoaderCircle, class: "animate-spin" } } else { Icon { icon: IconType::LogIn } }
                {t!("login-submit")}
            }
            SocialButtons { disabled: busy() }
        }
    }
}

/// Default `Button` classes (`button.tsx`), for submit buttons that hold an icon.
pub(super) const SUBMIT: &str = "inline-flex h-9 items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium whitespace-nowrap text-primary-foreground shadow-xs transition-all hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50 has-[>svg]:px-3 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";
