//! `/profile` (§9.1, M9.1): tabs Profile (name, time zone, password, appearance) · Security
//! (2FA, sessions) · Notifications (preferences) · Language · API tokens (when the public API
//! is on).
use crate::t;
use dioxus::prelude::*;

use crate::api::auth::{change_password, update_profile};
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, UserResource,
    use_current_user,
};
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::form::{
    FormDescription, FormItem, FormLabel, SelectBox, TextInput,
};
use crate::state::{Font, Theme, Ui};
use crate::ui::alert::Alert;
use crate::ui::button::Button;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

#[component]
pub fn Profile(tab: String) -> Element {
    let me = use_current_user();
    let must_change = me.as_ref().is_some_and(|u| u.must_change_pw);
    let api = me.as_ref().is_some_and(|u| u.modules.public_api);
    let tab = match tab.as_str() {
        "security" | "notifications" | "language" => tab,
        "tokens" if api => tab,
        _ => "profile".to_string(),
    };
    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main {
            div { class: "space-y-0.5",
                h1 { class: "text-2xl font-bold tracking-tight md:text-3xl", {t!("nav-profile")} }
                p { class: "text-muted-foreground", {t!("profile-desc")} }
            }
            div { role: "separator", class: "my-4 h-px w-full shrink-0 bg-border lg:my-6" }
            ProfileTabs { tab: tab.clone(), api }
            if tab == "security" {
                super::profile_security::SecurityTab {}
            } else if tab == "notifications" {
                super::profile_prefs::NotificationsTab {}
            } else if tab == "language" {
                super::profile_prefs::LanguageTab {}
            } else if tab == "tokens" {
                super::api_tokens::TokensTab {}
            } else {
                div { class: "grid max-w-5xl gap-6 lg:grid-cols-2",
                    if must_change {
                        div { class: "lg:col-span-2", Alert { span { id: "must-change", {t!("profile-must-change")} } } }
                    }
                    NameCard {}
                    PasswordCard {}
                    AppearanceCard {}
                }
            }
        }
    }
}

#[component]
fn NameCard() -> Element {
    let mut user = use_context::<UserResource>();
    let me = use_current_user();
    let email = me.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let role = me.as_ref().map(|u| u.role_name.clone()).unwrap_or_default();
    let org_tz = me
        .as_ref()
        .map(|u| {
            if u.own_timezone.is_some() {
                String::new()
            } else {
                u.timezone.clone()
            }
        })
        .unwrap_or_default();
    let own_tz = me
        .as_ref()
        .and_then(|u| u.own_timezone.clone())
        .unwrap_or_default();
    let initial = me.map(|u| u.full_name).unwrap_or_default();
    let name = use_signal(|| initial);
    let tz = use_signal(|| own_tz);
    let zones = use_hook(|| {
        jiff::tz::db()
            .available()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
    });
    let mut busy = use_signal(|| false);
    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        async move {
            if busy() {
                return;
            }
            busy.set(true);
            match update_profile(name(), tz()).await {
                Ok(()) => {
                    toast(ToastType::Success, t!("profile-saved"));
                    user.restart(); // sidebar and menus show the new name
                }
                Err(e) => toast_error(&e),
            }
            busy.set(false);
        }
    };
    rsx! {
        Card {
            CardHeader {
                CardTitle { {t!("profile-card-title")} }
                CardDescription { {t!("profile-card-desc")} }
            }
            CardContent {
                AvatarField {}
                form { class: "flex flex-col gap-4", onsubmit,
                    FormItem {
                        FormLabel { html_for: "profile-name", {t!("users-col-name")} }
                        TextInput { id: "profile-name", value: name }
                    }
                    FormItem {
                        FormLabel { {t!("users-col-email")} }
                        p { class: "text-sm", "{email}" }
                        FormDescription { {t!("profile-email-note")} }
                    }
                    FormItem {
                        FormLabel { {t!("users-col-role")} }
                        p { class: "text-sm", "{role}" }
                    }
                    FormItem {
                        FormLabel { html_for: "profile-tz", {t!("profile-timezone")} }
                        TextInput { id: "profile-tz", value: tz, placeholder: if org_tz.is_empty() { t!("profile-timezone-ph") } else { crate::ta!("profile-timezone-org", zone: org_tz.clone()) }, list: "tz-list" }
                        datalist { id: "tz-list",
                            for z in zones.iter() { option { key: "{z}", value: "{z}" } }
                        }
                        FormDescription { {t!("profile-timezone-desc")} }
                    }
                    Button { button_type: "submit", class: "w-fit", disabled: busy(), {t!("action-save-changes")} }
                }
            }
        }
    }
}

#[component]
fn PasswordCard() -> Element {
    let mut user = use_context::<UserResource>();
    let current = use_signal(String::new);
    let new = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        let (mut current, mut new, mut confirm) = (current, new, confirm);
        async move {
            if busy() {
                return;
            }
            if new() != confirm() {
                toast(ToastType::Error, t!("password-mismatch"));
                return;
            }
            busy.set(true);
            match change_password(current(), new()).await {
                Ok(()) => {
                    current.set(String::new());
                    new.set(String::new());
                    confirm.set(String::new());
                    toast(ToastType::Success, t!("profile-password-changed"));
                    user.restart(); // `must_change_pw` is now false
                }
                Err(e) => toast_error(&e),
            }
            busy.set(false);
        }
    };
    rsx! {
        Card {
            CardHeader {
                CardTitle { {t!("profile-change-password")} }
                CardDescription { {t!("password-hint")} }
            }
            CardContent {
                form { class: "flex flex-col gap-4", onsubmit: submit,
                    FormItem {
                        FormLabel { html_for: "current-password", {t!("field-current-password")} }
                        PasswordInput { id: "current-password", value: current }
                    }
                    FormItem {
                        FormLabel { html_for: "new-password", {t!("field-new-password")} }
                        PasswordInput { id: "new-password", value: new }
                    }
                    FormItem {
                        FormLabel { html_for: "confirm-password", {t!("field-confirm-password")} }
                        PasswordInput { id: "confirm-password", value: confirm }
                    }
                    Button { id: "profile-save", button_type: "submit", class: "w-fit", disabled: busy(), {t!("profile-save")} }
                }
            }
        }
    }
}

/// Theme and font (cookies, applied before first paint; see `state::prefs`).
#[component]
fn AppearanceCard() -> Element {
    let ui = Ui::get();
    let font = use_signal(|| (ui.font)().as_str().to_string());
    let theme = use_signal(|| (ui.theme)().as_str().to_string());
    rsx! {
        Card {
            CardHeader {
                CardTitle { {t!("nav-appearance")} }
                CardDescription { {t!("profile-appearance-desc")} }
            }
            CardContent { class: "flex flex-col gap-4",
                FormItem {
                    FormLabel { {t!("f-font")} }
                    SelectBox {
                        value: font,
                        options: Font::ALL.iter().map(|f| (f.as_str().to_string(), f.label().to_string())).collect(),
                        placeholder: t!("f-font"),
                        trigger_class: "w-48",
                        onchange: move |v: String| ui.set_font(Font::parse(&v)),
                    }
                    FormDescription { {t!("f-font-desc")} }
                }
                FormItem {
                    FormLabel { {t!("theme-label")} }
                    SelectBox {
                        value: theme,
                        options: vec![
                            ("system".to_string(), t!("theme-system")),
                            ("light".to_string(), t!("theme-light")),
                            ("dark".to_string(), t!("theme-dark")),
                        ],
                        placeholder: t!("theme-label"),
                        trigger_class: "w-48",
                        onchange: move |v: String| ui.set_theme(Theme::parse(&v)),
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileTabs(tab: String, api: bool) -> Element {
    let mut tabs = vec![
        ("profile", t!("profile-tab-profile")),
        ("security", t!("profile-tab-security")),
        ("notifications", t!("profile-tab-notifications")),
        ("language", t!("profile-tab-language")),
    ];
    if api {
        tabs.push(("tokens", t!("profile-tab-tokens")));
    }
    rsx! {
        div { role: "tablist", class: "mb-6 inline-flex h-9 w-fit max-w-full items-center overflow-x-auto rounded-lg bg-muted p-[3px] text-muted-foreground",
            for (key, label) in tabs {
                Link { key: "{key}", role: "tab", "aria-selected": "{tab == key}",
                    to: crate::app::Route::Profile { tab: if key == "profile" { String::new() } else { key.to_string() } },
                    class: if tab == key { "inline-flex h-[calc(100%-1px)] items-center rounded-md bg-background px-3 py-1 text-sm font-medium text-foreground shadow-sm dark:bg-input/30" } else { "inline-flex h-[calc(100%-1px)] items-center rounded-md px-3 py-1 text-sm font-medium hover:text-foreground" },
                    "{label}"
                }
            }
        }
    }
}

/// Avatar (M11.3): pick an image (≤ 1 MB); the server re-encodes it to a 256 px WebP.
#[component]
fn AvatarField() -> Element {
    use crate::ui::avatar::{Avatar, AvatarFallback, AvatarImage};
    let mut user = use_context::<UserResource>();
    let me = use_current_user();
    let initials = me.as_ref().map(|u| u.initials()).unwrap_or_default();
    let src = me.as_ref().and_then(|u| u.avatar_url.clone());
    let has = src.is_some();
    let mut busy = use_signal(|| false);
    let pick = move |_| async move {
        busy.set(true);
        // Read the chosen file in the browser as base64 (no upload widget needed).
        let mut ev = document::eval(
            "const i = document.createElement('input'); i.type = 'file'; i.accept = 'image/png,image/jpeg,image/webp,image/gif';
             i.onchange = () => { const f = i.files[0]; if (!f) { dioxus.send(''); return; }
               if (f.size > 1048576) { dioxus.send('!big'); return; }
               const r = new FileReader(); r.onload = () => dioxus.send(String(r.result).split(',')[1] || ''); r.readAsDataURL(f); };
             i.addEventListener('cancel', () => dioxus.send('')); i.click();",
        );
        match ev.recv::<String>().await {
            Ok(b) if b == "!big" => toast(ToastType::Error, t!("err-files-too-large")),
            Ok(b) if !b.is_empty() => match crate::api::auth::upload_avatar(b).await {
                Ok(_) => {
                    toast(ToastType::Success, t!("avatar-saved"));
                    user.restart();
                }
                Err(e) => toast_error(&e),
            },
            _ => {}
        }
        busy.set(false);
    };
    rsx! {
        div { class: "mb-6 flex items-center gap-4",
            Avatar { class: "size-16",
                if let Some(src) = src { AvatarImage { src } }
                AvatarFallback { class: "text-lg text-foreground", "{initials}" }
            }
            div { class: "flex flex-wrap gap-2",
                Button { id: "avatar-upload", variant: crate::ui::button::ButtonVariant::Outline, disabled: busy(), onclick: pick, {t!("avatar-change")} }
                if has {
                    Button { variant: crate::ui::button::ButtonVariant::Ghost, disabled: busy(),
                        onclick: move |_| async move {
                            match crate::api::auth::remove_avatar().await {
                                Ok(()) => user.restart(),
                                Err(e) => toast_error(&e),
                            }
                        },
                        {t!("avatar-remove")}
                    }
                }
                p { class: "basis-full text-xs text-muted-foreground", {t!("avatar-hint")} }
            }
        }
    }
}
