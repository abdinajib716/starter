//! System Setting page (§9.6, §17, §27, §30.3): tabs General · Security · Email ·
//! Storage & Backup · API · Localization · Feature Flags. Each tab has its master switch on
//! top, the secrets it needs (configured ✓ / missing ✗), its fields and a Save button. The
//! server validates types, ranges and the dependency rules; refusals come back as toasts.
use std::collections::HashMap;

use crate::t;
use dioxus::prelude::*;

use crate::api::roles::list_roles;
use crate::api::settings::{list_flags, list_settings, update_flag, update_settings};
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user,
};
use crate::components::skeletons::FormSkeleton;
use crate::components::ui_extra::form::{
    CheckBox, FormDescription, FormItem, FormLabel, SelectBox, Switch, TextInput,
};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::TenancyMode;
use crate::domain::roles::RoleSummary;
use crate::domain::settings::{
    FlagItem, FlagUpdate, SettingItem, SettingsView, TABS, master_switch,
};
use crate::pages::errors::{ErrorKind, ErrorView};
use crate::ui::button::Button;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

/// Translation for a key built at runtime (falls back to the key).
fn tr(key: &str) -> String {
    dioxus_i18n::te!(key).unwrap_or_else(|_| key.to_string())
}

const TAB_CONTENT: &str = "space-y-4 text-base group-data-[orientation=Horizontal]/tabs:mt-0";
const TAB_TRIGGER: &str = "px-2 py-1 text-foreground";

/// Secrets each tab depends on.
fn tab_secrets(tab: &str) -> &'static [&'static str] {
    match tab {
        "email" => &["RESEND_API_KEY"],
        "storage" => &[
            "BACKUP_PUBLIC_KEY",
            "BACKUP_ENCRYPTION_KEY",
            "R2_ACCOUNT_ID",
            "R2_ACCESS_KEY_ID",
            "R2_SECRET_ACCESS_KEY",
            "R2_BUCKET",
            "GDRIVE_SERVICE_ACCOUNT_JSON",
            "GDRIVE_FOLDER_ID",
        ],
        _ => &[],
    }
}

#[derive(Clone, Copy)]
struct SettingsCtx {
    /// key → JSON text, as edited.
    values: Signal<HashMap<String, String>>,
    /// key → JSON text, as saved.
    saved: Signal<HashMap<String, String>>,
    refresh: Signal<u32>,
}

#[component]
pub fn SystemSettings() -> Element {
    let me = use_current_user();
    if !me.as_ref().is_some_and(|u| u.can("settings.view")) {
        return rsx! { ErrorView { kind: ErrorKind::Forbidden, minimal: true, class: "h-[calc(100svh-4rem)]" } };
    }
    let can_flags = me.as_ref().is_some_and(|u| u.can("flags.view"));
    let ctx = use_context_provider(|| SettingsCtx {
        values: Signal::new(HashMap::new()),
        saved: Signal::new(HashMap::new()),
        refresh: Signal::new(0),
    });
    let mut view = use_signal(|| None::<SettingsView>);
    let refresh = ctx.refresh;
    let _load = use_resource(move || async move {
        let _ = refresh();
        match list_settings().await {
            Ok(v) => {
                let map: HashMap<String, String> = v
                    .items
                    .iter()
                    .map(|i| (i.key.clone(), i.value.clone()))
                    .collect();
                let (mut values, mut saved) = (ctx.values, ctx.saved);
                values.set(map.clone());
                saved.set(map);
                view.set(Some(v));
            }
            Err(e) => toast_error(&e),
        }
    });

    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main {
            div { class: "space-y-0.5",
                h1 { class: "text-2xl font-bold tracking-tight md:text-3xl", {t!("sys-title")} }
                p { class: "text-muted-foreground", {t!("sys-desc")} }
            }
            div { role: "separator", class: "my-4 h-px w-full shrink-0 bg-border lg:my-6" }
            match view() {
                None => rsx! { FormSkeleton {} },
                Some(v) => rsx! {
                    Tabs { default_value: "general", class: "space-y-4",
                        div { class: "w-full overflow-x-auto pb-2",
                            TabsList { class: "h-9 p-0.75",
                                for (tab, _) in TABS.iter() {
                                    TabsTrigger { value: *tab, class: TAB_TRIGGER, {tr(&format!("sys-tab-{tab}"))} }
                                }
                                if can_flags {
                                    TabsTrigger { value: "flags", class: TAB_TRIGGER, {t!("sys-tab-flags")} }
                                }
                            }
                        }
                        for (tab, cats) in TABS.iter() {
                            TabsContent { value: *tab, class: TAB_CONTENT,
                                SettingsTab { tab: *tab, items: v.items.iter().filter(|i| cats.contains(&i.category.as_str())).cloned().collect::<Vec<_>>(), secrets: v.secrets.clone(), tenancy: v.tenancy, version: v.version.clone() }
                            }
                        }
                        if can_flags {
                            TabsContent { value: "flags", class: TAB_CONTENT, FlagsTab {} }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn SettingsTab(
    tab: &'static str,
    items: Vec<SettingItem>,
    secrets: Vec<(String, bool)>,
    tenancy: TenancyMode,
    version: String,
) -> Element {
    let ctx = use_context::<SettingsCtx>();
    let can_update = use_current_user().is_some_and(|u| u.can("settings.update"));
    let mut busy = use_signal(|| false);
    let master = master_switch(tab);
    let master_on = master.map(|k| {
        ctx.values
            .read()
            .get(k)
            .map(|v| v == "true")
            .unwrap_or(false)
    });
    let keys: Vec<String> = items.iter().map(|i| i.key.clone()).collect();
    let dirty: Vec<(String, String)> = {
        let (v, s) = (ctx.values.read(), ctx.saved.read());
        keys.iter()
            .filter(|k| v.get(*k) != s.get(*k))
            .map(|k| (k.clone(), v[k].clone()))
            .collect()
    };
    let n_dirty = dirty.len();
    let needed: Vec<(String, bool)> = secrets
        .into_iter()
        .filter(|(n, _)| tab_secrets(tab).contains(&n.as_str()))
        .collect();
    let save = move |_| {
        let changes = dirty.clone();
        let mut refresh = ctx.refresh;
        spawn(async move {
            busy.set(true);
            match update_settings(changes).await {
                Ok(n) => {
                    toast(ToastType::Success, crate::ta!("sys-saved", count: n));
                    *refresh.write() += 1;
                }
                Err(e) => toast_error(&e),
            }
            busy.set(false);
        });
    };
    let reset = move |_| {
        let (mut values, saved) = (ctx.values, ctx.saved);
        values.set(saved());
    };

    rsx! {
        Card {
            CardHeader {
                CardTitle { {tr(&format!("sys-tab-{tab}"))} }
                CardDescription { {tr(&format!("sys-tab-{tab}-desc"))} }
            }
            CardContent { class: "space-y-6",
                if let Some(k) = master {
                    if let Some(item) = items.iter().find(|i| i.key == k) {
                        div { class: "flex items-center justify-between gap-4 rounded-lg border p-4",
                            div { class: "space-y-0.5",
                                div { class: "text-base font-medium", "{item.label}" }
                                p { class: "text-sm text-muted-foreground", {if master_on == Some(true) { t!("sys-master-on") } else { t!("sys-master-off") }} }
                            }
                            BoolField { key: "{k}", setting: k.to_string(), disabled: !can_update }
                        }
                    }
                }
                if !needed.is_empty() {
                    div { class: "space-y-2",
                        div { class: "text-sm font-medium", {t!("sys-secrets")} }
                        p { class: "text-xs text-muted-foreground", {t!("sys-secrets-desc")} }
                        div { class: "flex flex-wrap gap-2",
                            for (name, ok) in needed {
                                span { class: if ok { "inline-flex items-center gap-1 rounded-md border border-teal-200 bg-teal-100/30 px-2 py-0.5 font-mono text-xs text-teal-900 dark:text-teal-200" } else { "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 font-mono text-xs text-muted-foreground" },
                                    Icon { icon: if ok { IconType::Check } else { IconType::X }, class: "size-3" }
                                    "{name}"
                                }
                            }
                        }
                    }
                }
                if tab == "general" {
                    div { class: "grid gap-2 rounded-lg border p-4 text-sm sm:grid-cols-2",
                        div { span { class: "text-muted-foreground", {t!("sys-tenancy")} } " " span { class: "font-medium", "{tenancy}" } }
                        div { span { class: "text-muted-foreground", {t!("sys-version")} } " " span { class: "font-mono", "{version}" } }
                        p { class: "text-xs text-muted-foreground sm:col-span-2", {t!("sys-tenancy-note")} }
                    }
                }
                div { class: "grid gap-6 md:grid-cols-2",
                    for item in items.into_iter().filter(|i| Some(i.key.as_str()) != master) {
                        SettingField { key: "{item.key}", item, disabled: !can_update }
                    }
                }
                if can_update {
                    div { class: "flex items-center justify-end gap-2 border-t pt-4",
                        if n_dirty > 0 {
                            span { class: "me-auto text-sm text-muted-foreground", {crate::ta!("sys-unsaved", count: n_dirty)} }
                            Button { variant: crate::ui::button::ButtonVariant::Outline, onclick: reset, {t!("dt-reset")} }
                        }
                        Button { disabled: busy() || n_dirty == 0, onclick: save, {t!("action-save-changes")} }
                    }
                }
                if tab == "email" {
                    EmailTools {}
                }
                if tab == "api" && crate::components::layout::use_current_user().is_some_and(|u| u.can("api_keys.view_all")) {
                    crate::pages::api_tokens::AllTokens {}
                }
                if tab == "storage" && can_update {
                    div { class: "flex flex-wrap items-center gap-2 rounded-lg border p-4",
                        p { class: "me-auto text-sm text-muted-foreground", {t!("offsite-test-desc")} }
                        Button { id: "offsite-test", variant: crate::ui::button::ButtonVariant::Outline,
                            onclick: move |_| async move {
                                match crate::api::backups::test_offsite().await {
                                    Ok(()) => crate::util::toast(crate::ui::toast::ToastType::Success, t!("offsite-test-ok")),
                                    Err(e) => crate::util::toast_error(&e),
                                }
                            },
                            Icon { icon: IconType::CloudUpload } {t!("offsite-test")}
                        }
                    }
                }
            }
        }
    }
}

/// Email tab extras (M13.7): send a test email, delivery log.
#[component]
fn EmailTools() -> Element {
    use crate::api::account::{email_log, send_test_email};
    let me = crate::components::layout::use_current_user();
    let (can_test, can_log) = (
        me.as_ref().is_some_and(|u| u.can("email.send_test")),
        me.as_ref().is_some_and(|u| u.can("email.view_log")),
    );
    let to = use_signal(|| me.as_ref().map(|u| u.email.clone()).unwrap_or_default());
    let mut offset = use_signal(|| 0i64);
    let mut log = use_resource(move || async move {
        if !can_log {
            return None;
        }
        email_log(offset()).await.ok()
    });
    let tz = crate::components::layout::use_tz();
    let mut busy = use_signal(|| false);
    let page = log.read().clone().flatten().unwrap_or_default();
    rsx! {
        if can_test {
            div { class: "space-y-2 rounded-lg border p-4",
                p { class: "text-sm font-medium", {t!("email-test-title")} }
                p { class: "text-xs text-muted-foreground", {t!("email-test-desc")} }
                div { class: "flex flex-wrap items-center gap-2",
                    crate::components::ui_extra::form::TextInput { id: "email-test-to", value: to, class: "max-w-xs" }
                    Button { id: "email-test-send", disabled: busy(), onclick: move |_| async move {
                        busy.set(true);
                        match send_test_email(to()).await {
                            Ok(()) => crate::util::toast(crate::ui::toast::ToastType::Success, t!("email-test-sent")),
                            Err(e) => crate::util::toast_error(&e),
                        }
                        busy.set(false);
                        log.restart();
                    }, Icon { icon: IconType::Send } {t!("email-test-send")} }
                }
            }
        }
        if can_log {
            div { class: "space-y-2",
                p { class: "text-sm font-medium", {t!("email-log-title")} }
                div { class: "overflow-x-auto rounded-md border",
                    table { class: "w-full text-sm",
                        thead { tr { class: "border-b bg-muted/40 text-muted-foreground",
                            th { class: "px-3 py-2 text-start font-medium", {t!("email-log-when")} }
                            th { class: "px-3 py-2 text-start font-medium", {t!("email-log-to")} }
                            th { class: "px-3 py-2 text-start font-medium", {t!("email-log-template")} }
                            th { class: "px-3 py-2 text-start font-medium", {t!("email-log-status")} }
                        } }
                        tbody {
                            if page.rows.is_empty() {
                                tr { td { colspan: "4", class: "px-3 py-6 text-center text-muted-foreground", {t!("email-log-empty")} } }
                            }
                            for r in page.rows.iter() {
                                tr { key: "{r.id}", class: "border-b last:border-0",
                                    td { class: "px-3 py-2 whitespace-nowrap tabular-nums", {crate::util::time::datetime(r.created_at, &tz)} }
                                    td { class: "px-3 py-2", "{r.to}" }
                                    td { class: "px-3 py-2 font-mono text-xs", "{r.template}" }
                                    td { class: "px-3 py-2",
                                        span { class: if matches!(r.status.as_str(), "sent" | "delivered") { "text-teal-700 dark:text-teal-300" } else if r.status == "pending" { "text-muted-foreground" } else { "text-destructive" },
                                            "{r.status}"
                                        }
                                        if let Some(e) = &r.error { span { class: "block max-w-xs truncate text-xs text-muted-foreground", title: "{e}", "{e}" } }
                                    }
                                }
                            }
                        }
                    }
                }
                if page.total > 25 {
                    div { class: "flex justify-end gap-2",
                        Button { variant: crate::ui::button::ButtonVariant::Outline, disabled: offset() == 0, onclick: move |_| offset.set((offset() - 25).max(0)), Icon { icon: IconType::ChevronLeft } }
                        Button { variant: crate::ui::button::ButtonVariant::Outline, disabled: offset() + 25 >= page.total, onclick: move |_| offset.set(offset() + 25), Icon { icon: IconType::ChevronRight } }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingField(item: SettingItem, disabled: bool) -> Element {
    let key = item.key.clone();
    let hint = match (item.min, item.max) {
        (Some(lo), Some(hi)) => format!("{} ({lo}–{hi})", item.description),
        _ => item.description.clone(),
    };
    rsx! {
        FormItem {
            FormLabel { html_for: "s-{key}", "{item.label}" }
            match item.kind.as_str() {
                "bool" => rsx! { BoolField { setting: key.clone(), disabled } },
                "enum" => rsx! { EnumField { setting: key.clone(), options: item.options.clone(), disabled } },
                kind => rsx! { TextField { setting: key.clone(), numeric: kind == "int", disabled } },
            }
            if !hint.is_empty() { FormDescription { "{hint}" } }
            span { class: "font-mono text-[11px] text-muted-foreground/70", "{key}" }
        }
    }
}

/// Reads/writes one key of the edited values as JSON.
fn current(ctx: SettingsCtx, key: &str) -> serde_json::Value {
    ctx.values
        .read()
        .get(key)
        .and_then(|v| serde_json::from_str(v).ok())
        .unwrap_or(serde_json::Value::Null)
}

fn set(ctx: SettingsCtx, key: &str, v: serde_json::Value) {
    let mut values = ctx.values;
    values.write().insert(key.to_string(), v.to_string());
}

/// shadcn `Switch` look, controlled by the edited values (no local copy to keep in sync).
#[component]
fn BoolField(setting: String, disabled: bool) -> Element {
    let ctx = use_context::<SettingsCtx>();
    let on = current(ctx, &setting).as_bool().unwrap_or(false);
    let state = if on { "checked" } else { "unchecked" };
    let k = setting.clone();
    rsx! {
        button {
            r#type: "button",
            role: "switch",
            id: "s-{setting}",
            disabled,
            "aria-checked": "{on}",
            "data-state": state,
            onclick: move |_| set(ctx, &k, serde_json::Value::Bool(!on)),
            class: "peer inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80",
            span {
                "data-state": state,
                class: "pointer-events-none block size-4 rounded-full bg-background ring-0 transition-transform data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0 rtl:data-[state=checked]:-translate-x-[calc(100%-2px)] dark:data-[state=checked]:bg-primary-foreground dark:data-[state=unchecked]:bg-foreground",
            }
        }
    }
}

#[component]
fn EnumField(setting: String, options: Vec<String>, disabled: bool) -> Element {
    let ctx = use_context::<SettingsCtx>();
    let mut local = use_signal(|| current(ctx, &setting).as_str().unwrap_or("").to_string());
    let s2 = setting.clone();
    // Follow reloads and Reset.
    use_effect(move || {
        let v = current(ctx, &s2).as_str().unwrap_or("").to_string();
        if *local.peek() != v {
            local.set(v);
        }
    });
    let k = setting.clone();
    let opts: Vec<(String, String)> = options
        .iter()
        .map(|o| {
            (
                o.clone(),
                dioxus_i18n::te!(&format!("sys-opt-{}", o.replace('_', "-")))
                    .unwrap_or_else(|_| o.clone()),
            )
        })
        .collect();
    if disabled {
        return rsx! { p { class: "text-sm", "{local}" } };
    }
    rsx! {
        SelectBox {
            value: local,
            options: opts,
            placeholder: setting.clone(),
            trigger_class: "w-full",
            onchange: move |v: String| set(ctx, &k, serde_json::Value::String(v)),
        }
    }
}

#[component]
fn TextField(setting: String, numeric: bool, disabled: bool) -> Element {
    let ctx = use_context::<SettingsCtx>();
    let raw = match current(ctx, &setting) {
        serde_json::Value::String(s) => s,
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    };
    let mut local = use_signal(|| raw.clone());
    let s2 = setting.clone();
    use_effect(move || {
        let v = match current(ctx, &s2) {
            serde_json::Value::String(s) => s,
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };
        if *local.peek() != v {
            local.set(v);
        }
    });
    let k = setting.clone();
    rsx! {
        TextInput {
            id: "s-{setting}",
            value: local,
            r#type: if numeric { "number" } else { "text" },
            disabled,
            oninput: move |v: String| {
                let json = if numeric {
                    v.trim().parse::<i64>().map(serde_json::Value::from).unwrap_or(serde_json::Value::String(v))
                } else {
                    serde_json::Value::String(v)
                };
                set(ctx, &k, json);
            },
        }
    }
}

// ---------- Feature Flags tab (§27) ----------

#[component]
fn FlagsTab() -> Element {
    let mut flags = use_signal(|| None::<Vec<FlagItem>>);
    let mut roles = use_signal(Vec::<RoleSummary>::new);
    let mut refresh = use_signal(|| 0u32);
    let _load = use_resource(move || async move {
        let _ = refresh();
        match list_flags().await {
            Ok(f) => flags.set(Some(f)),
            Err(e) => toast_error(&e),
        }
        if let Ok(r) = list_roles().await {
            roles.set(r);
        }
    });
    let can_update = use_current_user().is_some_and(|u| u.can("flags.update"));
    rsx! {
        match flags() {
            None => rsx! { FormSkeleton { fields: 2 } },
            Some(list) if list.is_empty() => rsx! { p { class: "text-sm text-muted-foreground", {t!("flags-none")} } },
            Some(list) => rsx! {
                div { class: "space-y-4",
                    for f in list {
                        FlagCard { key: "{f.key}", flag: f.clone(), roles: roles(), can_update, on_saved: move |_| *refresh.write() += 1 }
                    }
                }
            },
        }
    }
}

#[component]
fn FlagCard(
    flag: FlagItem,
    roles: Vec<RoleSummary>,
    can_update: bool,
    on_saved: EventHandler<()>,
) -> Element {
    let enabled = use_signal(|| flag.enabled);
    let pct = use_signal(|| flag.rollout_percent.to_string());
    let mut role_ids = use_signal(|| flag.allowed_role_ids.clone());
    let beta = use_signal(|| flag.beta_emails.join(", "));
    let mut busy = use_signal(|| false);
    let key = flag.key.clone();
    let save = move |_| {
        let update = FlagUpdate {
            key: key.clone(),
            enabled: enabled(),
            rollout_percent: pct().trim().parse().unwrap_or(-1),
            allowed_role_ids: role_ids(),
            beta_emails: beta()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        };
        spawn(async move {
            busy.set(true);
            match update_flag(update).await {
                Ok(()) => {
                    toast(ToastType::Success, t!("flags-saved"));
                    on_saved.call(());
                }
                Err(e) => toast_error(&e),
            }
            busy.set(false);
        });
    };
    let updated = flag
        .updated_by
        .clone()
        .map(|by| crate::ta!("flags-updated-by", by: by))
        .unwrap_or_default();
    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-start justify-between gap-4",
                    div { class: "space-y-1",
                        CardTitle { class: "font-mono text-base", "{flag.key}" }
                        CardDescription { "{flag.description}" }
                        if flag.stale { span { class: "inline-flex rounded-md border border-amber-300 bg-amber-100/40 px-2 py-0.5 text-xs text-amber-900 dark:text-amber-200", {t!("flags-stale")} } }
                    }
                    Switch { checked: enabled, disabled: !can_update }
                }
            }
            CardContent { class: "grid gap-4 md:grid-cols-3",
                FormItem {
                    FormLabel { {t!("flags-rollout")} }
                    TextInput { value: pct, r#type: "number", disabled: !can_update, class: "w-28" }
                    FormDescription { {t!("flags-rollout-desc")} }
                }
                FormItem {
                    FormLabel { {t!("flags-roles")} }
                    div { class: "flex flex-wrap gap-x-4 gap-y-2",
                        for r in roles {
                            label { class: "flex items-center gap-2 text-sm",
                                CheckBox {
                                    checked: role_ids().contains(&r.id),
                                    aria_label: r.name.clone(),
                                    onchange: move |on: bool| { if !can_update { return; } let mut v = role_ids.write(); if on { v.push(r.id) } else { v.retain(|x| *x != r.id) } },
                                }
                                "{r.name}"
                            }
                        }
                    }
                    FormDescription { {t!("flags-roles-desc")} }
                }
                FormItem {
                    FormLabel { {t!("flags-beta")} }
                    TextInput { value: beta, disabled: !can_update, placeholder: "a@example.com, b@example.com" }
                    FormDescription { {t!("flags-beta-desc")} }
                }
                div { class: "flex items-center justify-between gap-2 md:col-span-3",
                    span { class: "text-xs text-muted-foreground", "{updated}" }
                    if can_update { Button { disabled: busy(), onclick: save, {t!("action-save-changes")} } }
                }
            }
        }
    }
}
