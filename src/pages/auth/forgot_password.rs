//! `forgot-password/*`. Reset links need email (Resend, M13); until then the form explains
//! that an administrator resets passwords, then continues to the OTP screen as the reference does.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::sign_in::SUBMIT;
use super::{AuthLayout, valid_email};
use crate::app::Route;
use crate::components::ui_extra::form::{FormItem, FormLabel, FormMessage, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::ui::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::toast as show_toast;

#[component]
pub fn ForgotPassword() -> Element {
    let email = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    let e_email = if email().trim().is_empty() {
        Some(t!("v-email-enter"))
    } else if !valid_email(&email()) {
        Some(t!("v-email-invalid"))
    } else {
        None
    };
    let show = submitted();
    let valid = e_email.is_none();

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if valid {
            show_toast(ToastType::Info, t!("auth-reset-unavailable"));
            navigator().push(Route::Otp {});
        }
    };

    rsx! {
        AuthLayout {
            Card { class: "max-w-sm gap-4 sm:min-w-sm",
                CardHeader {
                    CardTitle { class: "text-lg tracking-tight", {t!("nav-forgot-password")} }
                    CardDescription { {t!("auth-forgot-desc-1")} br {} {t!("auth-forgot-desc-2")} }
                }
                CardContent {
                    form { class: "grid gap-2", onsubmit,
                        FormItem { class: "space-y-1",
                            FormLabel { html_for: "email", invalid: show && e_email.is_some(), {t!("field-email")} }
                            TextInput { id: "email", value: email, placeholder: "name@example.com", invalid: show && e_email.is_some() }
                            FormMessage { message: e_email.filter(|_| show) }
                        }
                        button { r#type: "submit", class: tw_merge!(SUBMIT, "mt-2"),
                            {t!("auth-continue")}
                            Icon { icon: IconType::ArrowRight }
                        }
                    }
                }
                CardFooter {
                    p { class: "mx-auto px-8 text-center text-sm text-balance text-muted-foreground",
                        {t!("auth-no-account")}
                        " "
                        Link { to: Route::SignUp {}, class: "underline underline-offset-4 hover:text-primary", {t!("auth-sign-up-lower")} }
                        "."
                    }
                }
            }
        }
    }
}
