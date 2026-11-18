//! Header bell (§23, M8.9): unread badge + the latest 10; live through `Live::notifications`.
use crate::t;
use dioxus::prelude::*;

use super::live::use_live;
use super::{use_current_user, use_tz};
use crate::api::notifications::{list_notifications, mark_read};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, Menu, MenuSeparator};
use crate::domain::notifications::NotificationRow;

/// The translated message of a notification (`notif-<kind>` with its params).
pub fn message(n: &NotificationRow) -> String {
    crate::i18n::tr(&format!("notif-{}", n.kind.replace('_', "-")), &n.params)
}

#[component]
pub fn NotificationBell() -> Element {
    let live = use_live();
    let enabled = use_current_user().is_some_and(|u| u.modules.notifications);
    let mut page = use_resource(move || async move {
        let _ = (live.notifications)();
        if !enabled {
            return None;
        }
        list_notifications(false, 10, 0).await.ok()
    });
    let tz = use_tz();
    if !enabled {
        return rsx! {};
    }
    let data = page.read().clone().flatten().unwrap_or_default();
    let unread = data.unread;
    rsx! {
        Menu {
            align: Align::End,
            class: "w-80 p-0",
            trigger: rsx! {
                button { r#type: "button", id: "notification-bell", "aria-label": t!("notif-title"),
                    class: "relative inline-flex size-8 items-center justify-center rounded-full hover:bg-accent hover:text-accent-foreground",
                    Icon { icon: IconType::Bell, class: "size-[1.15rem]" }
                    if unread > 0 {
                        span { class: "absolute -top-0.5 -end-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-red-500 px-1 text-[10px] font-semibold text-white",
                            if unread > 99 { "99+" } else { "{unread}" }
                        }
                    }
                }
            },
            div { class: "flex items-center justify-between px-3 py-2",
                span { class: "text-sm font-semibold", {t!("notif-title")} }
                if unread > 0 {
                    button { r#type: "button", class: "text-xs text-muted-foreground hover:text-foreground",
                        onclick: move |_| async move { let _ = mark_read(None).await; page.restart(); },
                        {t!("notif-mark-all")}
                    }
                }
            }
            MenuSeparator {}
            div { class: "max-h-96 overflow-y-auto",
                if data.rows.is_empty() {
                    p { class: "px-3 py-6 text-center text-sm text-muted-foreground", {t!("notif-empty")} }
                }
                for n in data.rows.clone() {
                    NotificationItem { key: "{n.id}", n, tz: tz.clone(), on_read: move |_| page.restart() }
                }
            }
            MenuSeparator {}
            Link { to: "/notifications", class: "block px-3 py-2 text-center text-sm font-medium hover:bg-accent", {t!("notif-view-all")} }
        }
    }
}

#[component]
pub fn NotificationItem(n: NotificationRow, tz: String, on_read: EventHandler<()>) -> Element {
    let id = n.id;
    let link = n.link.clone();
    let text = message(&n);
    let when = crate::util::time::datetime(n.created_at, &tz);
    rsx! {
        button { r#type: "button",
            class: "flex w-full items-start gap-2 px-3 py-2.5 text-start text-sm hover:bg-accent",
            onclick: move |_| {
                let link = link.clone();
                async move {
                    let _ = mark_read(Some(vec![id])).await;
                    on_read.call(());
                    if let Some(l) = link { navigator().push(l); }
                }
            },
            span { class: if n.read { "mt-1.5 size-2 shrink-0 rounded-full bg-transparent" } else { "mt-1.5 size-2 shrink-0 rounded-full bg-primary" } }
            span { class: "flex flex-col gap-0.5",
                span { class: if n.read { "text-muted-foreground" } else { "" }, "{text}" }
                span { class: "text-xs text-muted-foreground", "{when}" }
            }
        }
    }
}
