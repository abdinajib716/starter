//! `otp/*`: six-digit code screen. Real verification arrives with 2FA (M9); like the reference,
//! submitting shows the values and goes home.
use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;
use tw_merge::tw_merge;

use super::AuthLayout;
use super::sign_in::SUBMIT;
use crate::app::Route;
use crate::ui::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::util::show_submitted_data;

#[component]
pub fn Otp() -> Element {
    let code = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        busy.set(true);
        show_submitted_data(&json!({ "otp": code() }), &t!("submitted-title"));
        navigator().push(Route::Dashboard {});
    };
    rsx! {
        AuthLayout {
            Card { class: "max-w-md gap-4",
                CardHeader {
                    CardTitle { class: "text-base tracking-tight", {t!("auth-otp-title")} }
                    CardDescription { {t!("auth-otp-desc-1")} br {} {t!("auth-otp-desc-2")} }
                }
                CardContent {
                    form { class: "grid gap-2", onsubmit,
                        label { class: "sr-only", r#for: "otp", {t!("auth-otp-label")} }
                        OtpInput { value: code }
                        button { r#type: "submit", disabled: code().len() < 6 || busy(), class: tw_merge!(SUBMIT, "mt-2"),
                            {t!("auth-verify")}
                        }
                    }
                }
                CardFooter {
                    p { class: "px-8 text-center text-sm text-muted-foreground",
                        {t!("auth-otp-resend-lead")}
                        " "
                        Link { to: Route::SignIn { redirect: String::new() }, class: "underline underline-offset-4 hover:text-primary", {t!("auth-otp-resend")} }
                        "."
                    }
                }
            }
        }
    }
}

/// `input-otp.tsx` (3 groups of 2 slots): one transparent input over the slots, so typing,
/// pasting and deleting behave like a normal field.
#[component]
fn OtpInput(value: Signal<String>) -> Element {
    let mut value = value;
    let mut focused = use_signal(|| false);
    let chars: Vec<char> = value().chars().collect();
    let active = chars.len().min(5);
    let slot = move |i: usize| {
        let ch = chars.get(i).map(|c| c.to_string()).unwrap_or_default();
        let first = i.is_multiple_of(2);
        rsx! {
            div {
                "data-active": "{focused() && i == active}",
                class: tw_merge!(
                    "relative flex h-9 w-9 items-center justify-center border-y border-e border-input text-sm shadow-xs transition-all outline-none dark:bg-input/30",
                    if first { "rounded-s-md border-s" } else { "rounded-e-md" },
                    if focused() && i == active { "z-10 border-ring ring-[3px] ring-ring/50" } else { "" }
                ),
                "{ch}"
            }
        }
    };
    rsx! {
        div { class: "relative flex items-center justify-between gap-2",
            for g in 0..3 {
                if g > 0 { div { role: "separator", class: "text-muted-foreground", "—" } }
                div { class: "flex items-center", {slot(g * 2)} {slot(g * 2 + 1)} }
            }
            input {
                id: "otp",
                class: "absolute inset-0 h-full w-full cursor-text bg-transparent text-transparent caret-transparent outline-none selection:bg-transparent",
                inputmode: "numeric",
                autocomplete: "one-time-code",
                maxlength: "6",
                value: value(),
                onfocus: move |_| focused.set(true),
                onblur: move |_| focused.set(false),
                oninput: move |e| value.set(e.value().chars().filter(char::is_ascii_digit).take(6).collect()),
            }
        }
    }
}
