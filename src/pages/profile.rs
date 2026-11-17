//! `/profile` (§9.1): own name, password (required when `must_change_pw`, M1.13) and
//! appearance (theme, font). Sessions, 2FA, API tokens and language arrive with M9 / M15.
use dioxus::prelude::*;
use dioxus_i18n::t;

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
pub fn Profile() -> Element {
    let must_change = use_current_user().is_some_and(|u| u.must_change_pw);
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

#[component]
fn NameCard() -> Element {
    let mut user = use_context::<UserResource>();
    let me = use_current_user();
    let email = me.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let role = me.as_ref().map(|u| u.role_name.clone()).unwrap_or_default();
    let initial = me.map(|u| u.full_name).unwrap_or_default();
    let name = use_signal(|| initial);
    let mut busy = use_signal(|| false);
    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        async move {
            if busy() {
                return;
            }
            busy.set(true);
            match update_profile(name()).await {
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
