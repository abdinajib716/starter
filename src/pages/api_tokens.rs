//! Profile → API tokens (§19, M15.8) and the organization-wide list (System Setting → API).
use dioxus::prelude::*;

use crate::api::tokens::{create_api_token, list_api_tokens, revoke_api_token};
use crate::components::layout::{use_current_user, use_tz};
use crate::components::ui_extra::form::{FormItem, FormLabel, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::api::TokenRow;
use crate::t;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

#[component]
pub fn TokensTab() -> Element {
    let me = use_current_user();
    let can_create = me.as_ref().is_some_and(|u| u.can("api_keys.create"));
    let perms: Vec<String> = me
        .as_ref()
        .map(|u| u.permissions.clone())
        .unwrap_or_default();
    let mut list = use_resource(|| list_api_tokens(false));
    let name = use_signal(String::new);
    let days = use_signal(|| "90".to_string());
    let mut scopes = use_signal(Vec::<String>::new);
    let mut shown = use_signal(|| None::<String>);
    let rows = list.read().clone().and_then(Result::ok).unwrap_or_default();
    rsx! {
        div { class: "grid max-w-5xl gap-6",
            if can_create {
                Card {
                    CardHeader {
                        CardTitle { {t!("tokens-new")} }
                        CardDescription { {t!("tokens-new-desc")} " " a { href: "/api/docs", target: "_blank", class: "underline underline-offset-4", {t!("tokens-docs")} } }
                    }
                    CardContent { class: "grid gap-4",
                        if let Some(tok) = shown() {
                            div { id: "token-once", class: "space-y-2 rounded-md border border-amber-500/40 bg-amber-500/10 p-3 text-sm",
                                p { class: "font-medium", {t!("tokens-copy-now")} }
                                code { class: "block break-all rounded bg-background px-2 py-1 font-mono text-xs", "{tok}" }
                            }
                        }
                        div { class: "grid gap-4 sm:grid-cols-[1fr_8rem]",
                            FormItem { FormLabel { html_for: "token-name", {t!("tokens-name")} } TextInput { id: "token-name", value: name, placeholder: "CI, mobile app …" } }
                            FormItem { FormLabel { html_for: "token-days", {t!("tokens-days")} } TextInput { id: "token-days", value: days, r#type: "number" } }
                        }
                        div {
                            p { class: "mb-2 text-sm font-medium", {t!("tokens-scopes")} }
                            div { class: "grid max-h-56 gap-1 overflow-y-auto rounded-md border p-2 sm:grid-cols-2",
                                for p in perms.clone() {
                                    label { key: "{p}", class: "flex items-center gap-2 text-sm",
                                        input { r#type: "checkbox", class: "size-4 accent-primary", checked: scopes().contains(&p),
                                            onchange: { let p = p.clone(); move |e: FormEvent| { if e.checked() { scopes.write().push(p.clone()) } else { scopes.write().retain(|x| *x != p) } } } }
                                        span { class: "font-mono text-xs", "{p}" }
                                    }
                                }
                            }
                        }
                        div { class: "flex justify-end",
                            Button { id: "token-create", onclick: move |_| async move {
                                let (mut n, d) = (name, days);
                                match create_api_token(n(), scopes(), d().trim().parse().ok()).await {
                                    Ok(t) => { shown.set(Some(t.token)); n.set(String::new()); scopes.set(vec![]); list.restart(); }
                                    Err(e) => toast_error(&e),
                                }
                            }, Icon { icon: IconType::KeyRound } {t!("tokens-create")} }
                        }
                    }
                }
            }
            Card {
                CardHeader { CardTitle { {t!("tokens-mine")} } }
                CardContent { TokenTable { rows, owner: false, on_change: move |_| list.restart() } }
            }
        }
    }
}

/// Every token in the organization (`api_keys.view_all`), on System Setting → API.
#[component]
pub fn AllTokens() -> Element {
    let mut list = use_resource(|| list_api_tokens(true));
    let rows = list.read().clone().and_then(Result::ok).unwrap_or_default();
    rsx! {
        div { class: "space-y-2",
            p { class: "text-sm font-medium", {t!("tokens-all")} }
            TokenTable { rows, owner: true, on_change: move |_| list.restart() }
        }
    }
}

#[component]
fn TokenTable(rows: Vec<TokenRow>, owner: bool, on_change: EventHandler<()>) -> Element {
    let tz = use_tz();
    let fmt = |t: Option<time::OffsetDateTime>| {
        t.map(|t| crate::util::time::date(t, &tz))
            .unwrap_or_else(|| "—".into())
    };
    rsx! {
        div { class: "overflow-x-auto rounded-md border",
            table { class: "w-full text-sm",
                thead { tr { class: "border-b bg-muted/40 text-muted-foreground",
                    th { class: "px-3 py-2 text-start font-medium", {t!("tokens-name")} }
                    if owner { th { class: "px-3 py-2 text-start font-medium", {t!("tokens-owner")} } }
                    th { class: "px-3 py-2 text-start font-medium", {t!("tokens-scopes")} }
                    th { class: "px-3 py-2 text-start font-medium", {t!("tokens-expires")} }
                    th { class: "px-3 py-2 text-start font-medium", {t!("tokens-last-used")} }
                    th { class: "px-3 py-2" }
                } }
                tbody {
                    if rows.is_empty() {
                        tr { td { colspan: "6", class: "px-3 py-6 text-center text-muted-foreground", {t!("tokens-none")} } }
                    }
                    for r in rows {
                        tr { key: "{r.id}", class: "border-b last:border-0",
                            td { class: "px-3 py-2", div { class: "font-medium", "{r.name}" } div { class: "font-mono text-xs text-muted-foreground", "{r.prefix}…" } }
                            if owner { td { class: "px-3 py-2", "{r.owner}" } }
                            td { class: "px-3 py-2 text-xs text-muted-foreground", {r.scopes.join(", ")} }
                            td { class: "px-3 py-2 whitespace-nowrap", {fmt(r.expires_at)} }
                            td { class: "px-3 py-2 whitespace-nowrap", {fmt(r.last_used_at)} }
                            td { class: "px-3 py-2 text-end",
                                if r.revoked {
                                    Badge { variant: BadgeVariant::Outline, {t!("tokens-revoked")} }
                                } else {
                                    Button { variant: ButtonVariant::Outline, class: "h-8", onclick: move |_| async move {
                                        match revoke_api_token(r.id).await {
                                            Ok(()) => { toast(ToastType::Success, t!("tokens-revoked")); on_change.call(()); }
                                            Err(e) => toast_error(&e),
                                        }
                                    }, {t!("tokens-revoke")} }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
