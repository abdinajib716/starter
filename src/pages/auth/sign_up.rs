//! `sign-up/*`. Users are invited (§9.2); self sign-up needs the `auth.self_signup` setting
//! (off by default, M7). Until then the form validates as in the reference and then says so.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::sign_in::SUBMIT;
use super::{AuthLayout, SocialButtons, TermsNote, valid_email};
use crate::app::Route;
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::form::{FormItem, FormLabel, FormMessage, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::ui::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::toast as show_toast;

#[component]
pub fn SignUp() -> Element {
    let email = use_signal(String::new);
    let password = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut submitted = use_signal(|| false);

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
    let e_confirm = if confirm().is_empty() {
        Some(t!("v-password-confirm"))
    } else if confirm() != password() {
        Some(t!("v-password-match"))
    } else {
        None
    };
    let show = submitted();
    let valid = e_email.is_none() && e_pw.is_none() && e_confirm.is_none();

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if valid {
            show_toast(ToastType::Info, t!("auth-signup-disabled"));
        }
    };

    rsx! {
        AuthLayout {
            Card { class: "max-w-sm gap-4",
                CardHeader {
                    CardTitle { class: "text-lg tracking-tight", {t!("auth-signup-title")} }
                    CardDescription {
                        {t!("auth-signup-desc")}
                        br {}
                        {t!("auth-have-account")}
                        " "
                        Link { to: Route::SignIn { redirect: String::new() }, class: "underline underline-offset-4 hover:text-primary", {t!("nav-sign-in")} }
                    }
                }
                CardContent {
                    form { class: "grid gap-3", onsubmit,
                        FormItem {
                            FormLabel { html_for: "email", invalid: show && e_email.is_some(), {t!("field-email")} }
                            TextInput { id: "email", value: email, placeholder: "name@example.com", invalid: show && e_email.is_some() }
                            FormMessage { message: e_email.filter(|_| show) }
                        }
                        FormItem {
                            FormLabel { html_for: "password", invalid: show && e_pw.is_some(), {t!("field-password")} }
                            PasswordInput { id: "password", value: password, placeholder: "********", invalid: show && e_pw.is_some() }
                            FormMessage { message: e_pw.filter(|_| show) }
                        }
                        FormItem {
                            FormLabel { html_for: "confirm", invalid: show && e_confirm.is_some(), {t!("auth-confirm-password")} }
                            PasswordInput { id: "confirm", value: confirm, placeholder: "********", invalid: show && e_confirm.is_some() }
                            FormMessage { message: e_confirm.filter(|_| show) }
                        }
                        button { r#type: "submit", class: tw_merge!(SUBMIT, "mt-2"),
                            Icon { icon: IconType::UserPlus }
                            {t!("auth-create-account")}
                        }
                        SocialButtons { disabled: false }
                    }
                }
                CardFooter { TermsNote { lead: t!("auth-terms-lead-sign-up") } }
            }
        }
    }
}
