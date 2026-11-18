//! Profile → Security (§21, §22): two-factor authentication and signed-in devices.
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::security::{
    my_sessions, revoke_my_session, twofa_begin, twofa_confirm, twofa_disable, twofa_regenerate,
    twofa_status,
};
use crate::components::layout::{UserResource, use_tz};
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::dialog::{ConfirmDialog, Modal};
use crate::components::ui_extra::form::{FormItem, FormLabel, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::security::{EnrollStart, SessionRow};
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

#[component]
pub fn SecurityTab() -> Element {
    rsx! {
        div { class: "grid max-w-5xl gap-6",
            TwoFaCard {}
            SessionsCard {}
        }
    }
}

/// Recovery codes, shown once, with copy-free download as a text file.
#[component]
fn RecoveryCodes(codes: Vec<String>) -> Element {
    let text = codes.join("\n");
    rsx! {
        div { id: "recovery-codes", class: "space-y-3 rounded-lg border bg-muted/40 p-4",
            p { class: "text-sm font-medium", {t!("twofa-codes-once")} }
            div { class: "grid grid-cols-2 gap-2 font-mono text-sm",
                for c in codes.iter() { span { key: "{c}", "{c}" } }
            }
            Button { variant: ButtonVariant::Outline,
                onclick: move |_| {
                    let url = format!("data:text/plain;charset=utf-8,{}", js_encode(&text));
                    crate::util::download(&url, "recovery-codes.txt");
                },
                Icon { icon: IconType::Download } {t!("twofa-codes-download")}
            }
        }
    }
}

fn js_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[component]
fn TwoFaCard() -> Element {
    let mut user = use_context::<UserResource>();
    let mut status = use_resource(twofa_status);
    let mut enroll = use_signal(|| None::<EnrollStart>);
    let mut codes = use_signal(|| None::<Vec<String>>);
    let code = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut disable_open = use_signal(|| false);
    let disable_pw = use_signal(String::new);
    let mut regen_open = use_signal(|| false);
    let regen_code = use_signal(String::new);

    let Some(Ok(st)) = status.read().clone() else {
        return rsx! { crate::components::skeletons::FormSkeleton { fields: 2 } };
    };
    let tz = use_tz();
    let deadline = st
        .info
        .deadline
        .map(|d| crate::util::time::datetime(d, &tz));

    let start = move |_| async move {
        busy.set(true);
        match twofa_begin().await {
            Ok(e) => enroll.set(Some(e)),
            Err(e) => toast_error(&e),
        }
        busy.set(false);
    };
    let confirm = move |evt: FormEvent| async move {
        evt.prevent_default();
        let mut c = code;
        busy.set(true);
        match twofa_confirm(c()).await {
            Ok(list) => {
                enroll.set(None);
                c.set(String::new());
                codes.set(Some(list));
                toast(ToastType::Success, t!("twofa-enabled"));
                status.restart();
                user.restart();
            }
            Err(e) => toast_error(&e),
        }
        busy.set(false);
    };

    rsx! {
        Card {
            CardHeader {
                CardTitle { class: "flex items-center gap-2",
                    {t!("twofa-card-title")}
                    if st.info.enrolled {
                        Badge { variant: BadgeVariant::Success, {t!("twofa-on")} }
                    } else {
                        Badge { variant: BadgeVariant::Outline, {t!("twofa-off")} }
                    }
                }
                CardDescription { {t!("twofa-card-desc")} }
            }
            CardContent { class: "space-y-4",
                if !st.available && !st.info.enrolled {
                    p { class: "text-sm text-muted-foreground", {t!("twofa-not-available")} }
                }
                if st.info.required && !st.info.enrolled {
                    div { role: "alert", class: "rounded-md border border-amber-500/50 bg-amber-500/10 px-3 py-2 text-sm",
                        if st.info.blocked { {t!("twofa-required-now")} }
                        else { {crate::ta!("twofa-required-by", date: deadline.clone().unwrap_or_default())} }
                    }
                }
                if let Some(list) = codes() {
                    RecoveryCodes { codes: list }
                }
                if st.info.enrolled {
                    p { class: "text-sm text-muted-foreground",
                        {crate::ta!("twofa-codes-left", count: st.recovery_left)}
                    }
                    div { class: "flex flex-wrap gap-2",
                        Button { variant: ButtonVariant::Outline, onclick: move |_| regen_open.set(true),
                            Icon { icon: IconType::RefreshCw } {t!("twofa-regenerate")}
                        }
                        if !st.info.required {
                            Button { variant: ButtonVariant::Destructive, onclick: move |_| disable_open.set(true),
                                {t!("twofa-disable")}
                            }
                        }
                    }
                } else if let Some(e) = enroll() {
                    div { class: "grid gap-4 md:grid-cols-[auto_1fr]",
                        div { id: "twofa-qr", class: "size-[200px] rounded-md border bg-white p-1", dangerous_inner_html: e.qr_svg.clone() }
                        div { class: "space-y-3 text-sm",
                            p { {t!("twofa-scan")} }
                            p { class: "text-muted-foreground", {t!("twofa-manual")} }
                            code { id: "twofa-secret", class: "block break-all rounded bg-muted px-2 py-1 font-mono text-xs", "{e.secret}" }
                            form { class: "flex items-end gap-2", onsubmit: confirm,
                                FormItem { class: "flex-1",
                                    FormLabel { html_for: "twofa-code", {t!("twofa-code")} }
                                    TextInput { id: "twofa-code", value: code, placeholder: "123456" }
                                }
                                Button { button_type: "submit", disabled: busy(), {t!("twofa-confirm")} }
                            }
                        }
                    }
                } else if st.available {
                    Button { id: "twofa-start", disabled: busy(), onclick: start,
                        Icon { icon: IconType::ShieldCheck } {t!("twofa-set-up")}
                    }
                }
            }
        }
        ConfirmDialog {
            open: disable_open,
            title: t!("twofa-disable"),
            desc: String::new(),
            confirm_text: t!("twofa-disable"),
            cancel_text: t!("action-cancel"),
            destructive: true,
            on_confirm: move |_| async move {
                let mut pw = disable_pw;
                match twofa_disable(pw()).await {
                    Ok(()) => { toast(ToastType::Success, t!("twofa-disabled")); disable_open.set(false); status.restart(); user.restart(); }
                    Err(e) => toast_error(&e),
                }
                pw.set(String::new());
            },
            body: rsx! {
                FormItem {
                    FormLabel { html_for: "twofa-disable-pw", {t!("field-current-password")} }
                    PasswordInput { id: "twofa-disable-pw", value: disable_pw }
                }
            },
        }
        Modal { open: regen_open, title: t!("twofa-regenerate"), description: t!("twofa-regenerate-desc"),
            form { class: "flex items-end gap-2",
                onsubmit: move |evt: FormEvent| async move {
                    evt.prevent_default();
                    let mut c = regen_code;
                    match twofa_regenerate(c()).await {
                        Ok(list) => { codes.set(Some(list)); regen_open.set(false); status.restart(); }
                        Err(e) => toast_error(&e),
                    }
                    c.set(String::new());
                },
                FormItem { class: "flex-1",
                    FormLabel { html_for: "regen-code", {t!("twofa-code")} }
                    TextInput { id: "regen-code", value: regen_code, placeholder: "123456" }
                }
                Button { button_type: "submit", {t!("twofa-regenerate")} }
            }
        }
    }
}

#[component]
pub fn SessionList(rows: Vec<SessionRow>, on_revoke: Option<EventHandler<uuid::Uuid>>) -> Element {
    let tz = use_tz();
    rsx! {
        ul { class: "divide-y rounded-md border",
            if rows.is_empty() {
                li { class: "px-4 py-6 text-center text-sm text-muted-foreground", {t!("sessions-none")} }
            }
            for s in rows {
                li { key: "{s.id}", class: "flex items-center gap-3 px-4 py-3",
                    Icon { icon: if s.device.contains("iOS") || s.device.contains("Android") { IconType::Smartphone } else { IconType::Monitor }, class: "size-5 text-muted-foreground" }
                    div { class: "flex-1 text-sm",
                        div { class: "flex items-center gap-2 font-medium",
                            "{s.device}"
                            if s.current { Badge { variant: BadgeVariant::Secondary, {t!("sessions-this-device")} } }
                            if s.mfa { Badge { variant: BadgeVariant::Outline, "2FA" } }
                        }
                        div { class: "text-xs text-muted-foreground",
                            if let Some(ip) = s.ip.clone().filter(|i| !i.is_empty()) { "{ip} · " }
                            {crate::ta!("sessions-last-seen", when: crate::util::time::datetime(s.last_seen_at, &tz))}
                        }
                    }
                    if let Some(h) = on_revoke.filter(|_| !s.current) {
                        Button { variant: ButtonVariant::Outline, class: "h-8", onclick: move |_| h.call(s.id), {t!("sessions-sign-out")} }
                    }
                }
            }
        }
    }
}

#[component]
fn SessionsCard() -> Element {
    let mut list = use_resource(my_sessions);
    let rows = list.read().clone().and_then(Result::ok);
    let others = rows.as_ref().is_some_and(|r| r.iter().any(|s| !s.current));
    rsx! {
        Card {
            CardHeader {
                CardTitle { {t!("sessions-title")} }
                CardDescription { {t!("sessions-desc")} }
            }
            CardContent { class: "space-y-4",
                match rows {
                    Some(rows) => rsx! {
                        SessionList { rows, on_revoke: move |id| async move {
                            match revoke_my_session(Some(id)).await {
                                Ok(_) => { toast(ToastType::Success, t!("sessions-revoked")); list.restart(); }
                                Err(e) => toast_error(&e),
                            }
                        } }
                    },
                    None => rsx! { crate::components::skeletons::FormSkeleton { fields: 2 } },
                }
                if others {
                    Button { id: "sessions-revoke-others", variant: ButtonVariant::Outline,
                        onclick: move |_| async move {
                            match revoke_my_session(None).await {
                                Ok(n) => { toast(ToastType::Success, crate::ta!("sessions-revoked-n", count: n)); list.restart(); }
                                Err(e) => toast_error(&e),
                            }
                        },
                        Icon { icon: IconType::LogOut } {t!("sessions-revoke-others")}
                    }
                }
            }
        }
    }
}
