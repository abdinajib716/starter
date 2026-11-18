//! `/notifications` (§23, M8.9): everything I was notified about, paged; announcements for
//! holders of `notifications.broadcast`.
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::notifications::{list_notifications, mark_read, send_announcement};
use crate::components::layout::{
    ConfigDrawer, Header, Main, NotificationItem, ProfileDropdown, Search, ThemeSwitch, use_live,
    use_tz,
};
use crate::components::permission_gate::PermissionGate;
use crate::components::ui_extra::dialog::Modal;
use crate::components::ui_extra::form::{FormItem, FormLabel, TextArea, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

const PAGE: i64 = 20;

#[component]
pub fn Notifications() -> Element {
    let live = use_live();
    let mut unread_only = use_signal(|| false);
    let mut offset = use_signal(|| 0i64);
    let mut data = use_resource(move || async move {
        let _ = (live.notifications)();
        list_notifications(unread_only(), PAGE, offset()).await
    });
    let tz = use_tz();
    let mut announce = use_signal(|| false);
    let title = use_signal(String::new);
    let body = use_signal(String::new);
    let page = data.read().clone().and_then(Result::ok).unwrap_or_default();
    let total = if unread_only() {
        page.unread
    } else {
        page.total
    };
    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { class: "flex flex-1 flex-col gap-4",
            div { class: "flex flex-wrap items-end justify-between gap-2",
                div {
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("notif-title")} }
                    p { class: "text-muted-foreground", {t!("notif-desc")} }
                }
                div { class: "flex gap-2",
                    Button { variant: ButtonVariant::Outline, onclick: move |_| async move { let _ = mark_read(None).await; data.restart(); },
                        Icon { icon: IconType::CheckCheck } {t!("notif-mark-all")}
                    }
                    PermissionGate { perm: "notifications.broadcast",
                        Button { id: "announce-open", onclick: move |_| announce.set(true), Icon { icon: IconType::Megaphone } {t!("notif-announce")} }
                    }
                }
            }
            div { class: "flex gap-2",
                Button { variant: if unread_only() { ButtonVariant::Ghost } else { ButtonVariant::Secondary }, onclick: move |_| { unread_only.set(false); offset.set(0); }, {t!("notif-all")} }
                Button { variant: if unread_only() { ButtonVariant::Secondary } else { ButtonVariant::Ghost }, onclick: move |_| { unread_only.set(true); offset.set(0); }, {crate::ta!("notif-unread", count: page.unread)} }
            }
            div { class: "divide-y rounded-md border",
                if page.rows.is_empty() {
                    p { class: "px-4 py-10 text-center text-sm text-muted-foreground", {t!("notif-empty")} }
                }
                for n in page.rows.clone() {
                    NotificationItem { key: "{n.id}", n, tz: tz.clone(), on_read: move |_| data.restart() }
                }
            }
            if total > PAGE {
                div { class: "flex items-center justify-end gap-2 text-sm",
                    span { class: "text-muted-foreground", "{offset() + 1}–{(offset() + PAGE).min(total)} / {total}" }
                    Button { variant: ButtonVariant::Outline, disabled: offset() == 0, onclick: move |_| offset.set((offset() - PAGE).max(0)), Icon { icon: IconType::ChevronLeft } }
                    Button { variant: ButtonVariant::Outline, disabled: offset() + PAGE >= total, onclick: move |_| offset.set(offset() + PAGE), Icon { icon: IconType::ChevronRight } }
                }
            }
        }
        Modal { open: announce, title: t!("notif-announce"), description: t!("notif-announce-desc"),
            form { class: "grid gap-3",
                onsubmit: move |e: FormEvent| async move {
                    e.prevent_default();
                    let (mut ti, mut bo) = (title, body);
                    match send_announcement(ti(), bo()).await {
                        Ok(n) => {
                            toast(ToastType::Success, crate::ta!("notif-announced", count: n));
                            ti.set(String::new()); bo.set(String::new()); announce.set(false);
                        }
                        Err(e) => toast_error(&e),
                    }
                },
                FormItem { FormLabel { html_for: "ann-title", {t!("notif-ann-title")} } TextInput { id: "ann-title", value: title } }
                FormItem { FormLabel { html_for: "ann-body", {t!("notif-ann-body")} } TextArea { id: "ann-body", value: body } }
                Button { button_type: "submit", {t!("notif-send")} }
            }
        }
    }
}
