//! Temporary home: Rust/UI smoke test + server status + i18n spike. Replaced in M3/M10.
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;
use dioxus_i18n::t;

use crate::api::i18n::set_locale;
use crate::api::system::server_info;
use crate::ui::avatar::{Avatar, AvatarFallback, AvatarSize};
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::ui::theme_toggle::ThemeToggle;
use crate::ui::toast::{ToastType, show_toast};

#[component]
pub fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let info = use_server_future(server_info)?;
    let mut i18n = i18n();
    let current = i18n.language().to_string();

    rsx! {
        div { class: "min-h-screen bg-background text-foreground p-8 flex items-start justify-center",
            Card { class: "w-full max-w-md",
                CardHeader {
                    div { class: "flex items-center gap-3",
                        Avatar { size: AvatarSize::Lg,
                            AvatarFallback { "AM" }
                        }
                        div { class: "flex-1",
                            CardTitle { span { id: "title", {t!("home-title")} } }
                            CardDescription { {t!("home-subtitle")} }
                        }
                        ThemeToggle {}
                    }
                }
                CardContent { class: "space-y-2",
                    p { id: "clicks", {t!("home-clicks", count: count())} }
                    match &*info.read() {
                        Some(Ok(i)) => rsx! {
                            div { class: "flex flex-wrap gap-2 text-sm",
                                Badge { variant: BadgeVariant::Secondary, "v{i.version}" }
                                Badge { variant: BadgeVariant::Secondary, "tenancy: {i.tenancy}" }
                                Badge { variant: BadgeVariant::Secondary, "db: {i.db}" }
                            }
                        },
                        Some(Err(e)) => rsx! { p { class: "text-destructive text-sm", "server: {e}" } },
                        None => rsx! { p { class: "text-muted-foreground text-sm", {t!("status-loading")} } },
                    }
                    div { class: "flex items-center gap-2 text-sm",
                        span { class: "text-muted-foreground", {t!("lang-label")} }
                        for code in crate::i18n::SUPPORTED.iter().copied() {
                            Button {
                                id: "lang-{code}",
                                size: ButtonSize::Sm,
                                variant: if code == current { ButtonVariant::Default } else { ButtonVariant::Outline },
                                onclick: move |_| async move {
                                    i18n.set_language(code.parse().expect("valid locale"));
                                    let _ = set_locale(code.to_string()).await;
                                },
                                "{code}"
                            }
                        }
                    }
                }
                CardFooter { class: "gap-2",
                    Button { id: "inc", onclick: move |_| count += 1, {t!("action-increment")} }
                    Button { variant: ButtonVariant::Outline, onclick: move |_| count.set(0), {t!("action-reset")} }
                    Button { variant: ButtonVariant::Destructive, size: ButtonSize::Sm,
                        onclick: move |_| show_toast(ToastType::Error, t!("toast-delete")),
                        {t!("action-delete")}
                    }
                }
            }
        }
    }
}
