//! Temporary home: Rust/UI smoke test + server status. Replaced by the dashboard in M3/M10.
use dioxus::prelude::*;

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

    rsx! {
        div { class: "min-h-screen bg-background text-foreground p-8 flex items-start justify-center",
            Card { class: "w-full max-w-md",
                CardHeader {
                    div { class: "flex items-center gap-3",
                        Avatar { size: AvatarSize::Lg,
                            AvatarFallback { "AM" }
                        }
                        div { class: "flex-1",
                            CardTitle { "Rust/UI is working" }
                            CardDescription { "Dioxus 0.7 + Tailwind v4 theme" }
                        }
                        ThemeToggle {}
                    }
                }
                CardContent { class: "space-y-2",
                    p { "Clicked {count} times" }
                    match &*info.read() {
                        Some(Ok(i)) => rsx! {
                            div { class: "flex flex-wrap gap-2 text-sm",
                                Badge { variant: BadgeVariant::Secondary, "v{i.version}" }
                                Badge { variant: BadgeVariant::Secondary, "tenancy: {i.tenancy}" }
                                Badge { variant: BadgeVariant::Secondary, "db: {i.db}" }
                            }
                        },
                        Some(Err(e)) => rsx! { p { class: "text-destructive text-sm", "server: {e}" } },
                        None => rsx! { p { class: "text-muted-foreground text-sm", "loading…" } },
                    }
                }
                CardFooter { class: "gap-2",
                    Button { onclick: move |_| count += 1, "Increment" }
                    Button { variant: ButtonVariant::Outline, onclick: move |_| count.set(0), "Reset" }
                    Button { variant: ButtonVariant::Destructive, size: ButtonSize::Sm,
                        onclick: move |_| show_toast(ToastType::Error, "Delete clicked (toast works)"),
                        "Delete"
                    }
                }
            }
        }
    }
}
