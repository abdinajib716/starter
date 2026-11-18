//! Email-link screens (§20, M13.5): forgot password, reset password, accept invite, verify
//! email. Public routes under `/auth/`.
use crate::t;
use dioxus::prelude::*;
use tw_merge::tw_merge;

use super::AuthLayout;
use super::sign_in::SUBMIT;
use crate::api::account::{forgot_password, set_password_with_token, verify_email};
use crate::app::Route;
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::form::{FormItem, FormLabel, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

fn back_to_sign_in() -> Element {
    rsx! {
        Link { to: Route::SignIn { redirect: String::new(), reason: String::new() },
            class: "text-center text-sm text-muted-foreground underline-offset-4 hover:underline",
            {t!("auth-back-to-sign-in")}
        }
    }
}

#[component]
pub fn ForgotPassword() -> Element {
    let email = use_signal(String::new);
    let mut sent = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let onsubmit = move |e: FormEvent| async move {
        e.prevent_default();
        if !crate::domain::valid_email(&email()) || busy() {
            toast(ToastType::Error, t!("v-email-invalid"));
            return;
        }
        busy.set(true);
        match forgot_password(email()).await {
            Ok(()) => sent.set(true),
            Err(e) => toast_error(&e),
        }
        busy.set(false);
    };
    rsx! {
        AuthLayout {
            Card { class: "max-w-sm gap-4",
                CardHeader {
                    CardTitle { class: "text-lg tracking-tight", {t!("forgot-title")} }
                    CardDescription { {t!("forgot-desc")} }
                }
                CardContent { class: "grid gap-3",
                    if sent() {
                        p { id: "forgot-sent", class: "text-sm", {t!("forgot-sent")} }
                    } else {
                        form { class: "grid gap-3", onsubmit,
                            FormItem {
                                FormLabel { html_for: "email", {t!("field-email")} }
                                TextInput { id: "email", value: email, placeholder: "name@example.com" }
                            }
                            button { id: "forgot-submit", r#type: "submit", disabled: busy(), class: tw_merge!(SUBMIT, "mt-2"),
                                Icon { icon: IconType::Mail } {t!("forgot-submit")}
                            }
                        }
                    }
                    {back_to_sign_in()}
                }
            }
        }
    }
}

/// Reset (`purpose = password_reset`) and accept invite (`invite`) share the form.
#[component]
fn SetPassword(token: String, purpose: &'static str) -> Element {
    let pw = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        let token = token.clone();
        async move {
            if pw() != confirm() {
                toast(ToastType::Error, t!("password-mismatch"));
                return;
            }
            busy.set(true);
            match set_password_with_token(token, purpose.to_string(), pw()).await {
                Ok(()) => {
                    toast(ToastType::Success, t!("set-password-done"));
                    navigator().replace(Route::SignIn {
                        redirect: String::new(),
                        reason: String::new(),
                    });
                }
                Err(e) => toast_error(&e),
            }
            busy.set(false);
        }
    };
    let invite = purpose == "invite";
    rsx! {
        AuthLayout {
            Card { class: "max-w-sm gap-4",
                CardHeader {
                    CardTitle { class: "text-lg tracking-tight", if invite { {t!("invite-title")} } else { {t!("reset-title")} } }
                    CardDescription { {t!("password-hint")} }
                }
                CardContent { class: "grid gap-3",
                    form { class: "grid gap-3", onsubmit,
                        FormItem {
                            FormLabel { html_for: "new-password", {t!("field-new-password")} }
                            PasswordInput { id: "new-password", value: pw }
                        }
                        FormItem {
                            FormLabel { html_for: "confirm-password", {t!("field-confirm-password")} }
                            PasswordInput { id: "confirm-password", value: confirm }
                        }
                        button { id: "set-password-submit", r#type: "submit", disabled: busy(), class: tw_merge!(SUBMIT, "mt-2"),
                            Icon { icon: IconType::KeyRound } {t!("set-password-submit")}
                        }
                    }
                    {back_to_sign_in()}
                }
            }
        }
    }
}

#[component]
pub fn ResetPassword(token: String) -> Element {
    rsx! { SetPassword { token, purpose: "password_reset" } }
}

#[component]
pub fn AcceptInvite(token: String) -> Element {
    rsx! { SetPassword { token, purpose: "invite" } }
}

#[component]
pub fn VerifyEmail(token: String) -> Element {
    let t2 = token.clone();
    let result = use_resource(move || {
        let t = t2.clone();
        async move { verify_email(t).await }
    });
    rsx! {
        AuthLayout {
            Card { class: "max-w-sm gap-4",
                CardHeader { CardTitle { class: "text-lg tracking-tight", {t!("verify-title")} } }
                CardContent { class: "grid gap-3 text-sm",
                    match &*result.read() {
                        None => rsx! { p { {t!("verify-checking")} } },
                        Some(Ok(())) => rsx! { p { id: "verify-ok", {t!("verify-done")} } },
                        Some(Err(e)) => rsx! { p { class: "text-destructive", {crate::i18n::error_text(e)} } },
                    }
                    {back_to_sign_in()}
                }
            }
        }
    }
}
