//! Temporary home: signed-in greeting, server status, Rust/UI smoke test. Replaced by the
//! dashboard in M10.
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::system::server_info;
use crate::components::layout::UserResource;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::ui::toast::{ToastType, show_toast};

#[component]
pub fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let info = use_server_future(server_info)?;
    let user = use_context::<UserResource>();
    let name = user
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(|u| u.as_ref())
        .map(|u| u.full_name.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "flex justify-center",
            Card { class: "w-full max-w-md",
                CardHeader {
                    CardTitle { span { id: "title", {t!("home-welcome", name: name)} } }
                    CardDescription { {t!("home-subtitle")} }
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
