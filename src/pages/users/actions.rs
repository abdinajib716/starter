//! `data-table-row-actions.tsx`, `data-table-bulk-actions.tsx`, `users-multi-delete-dialog.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;

use super::data::User;
use super::{UserDialog, UsersCtx};
use crate::components::data_table::{
    BulkActions, ICON_BTN, ICON_BTN_DESTRUCTIVE, MultiDeleteBody, TableState,
};
use crate::components::ui_extra::dialog::ConfirmDialog;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, ITEM, Menu, MenuSeparator, MenuShortcut};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_promise};

#[component]
pub fn RowActions(user: User) -> Element {
    let mut ctx = use_context::<UsersCtx>();
    let edit = user.clone();
    rsx! {
        Menu {
            align: Align::End,
            class: "w-40",
            trigger: rsx! {
                button { r#type: "button", class: "flex h-8 w-8 items-center justify-center rounded-md p-0 hover:bg-accent hover:text-accent-foreground in-data-[state=open]:bg-muted",
                    Icon { icon: IconType::Ellipsis, class: "h-4 w-4" }
                    span { class: "sr-only", {t!("dt-open-menu")} }
                }
            },
            button { r#type: "button", class: ITEM, onclick: move |_| { ctx.current.set(Some(edit.clone())); ctx.open.set(Some(UserDialog::Edit)); },
                {t!("dt-edit")}
                MenuShortcut { Icon { icon: IconType::UserPen, class: "size-4" } }
            }
            MenuSeparator {}
            button { r#type: "button", class: "{ITEM} text-red-500!", onclick: move |_| { ctx.current.set(Some(user.clone())); ctx.open.set(Some(UserDialog::Delete)); },
                {t!("dt-delete")}
                MenuShortcut { Icon { icon: IconType::Trash2, class: "size-4" } }
            }
        }
    }
}

fn plural(n: usize) -> String {
    t!(if n > 1 {
        "users-entity-plural"
    } else {
        "users-entity"
    })
}

#[component]
pub fn UsersBulkActions(state: TableState, rows: Vec<User>) -> Element {
    let mut selected = state.selected;
    let count = rows.len();
    let mut confirm = use_signal(|| false);
    let mut word = use_signal(String::new);
    let mut done = move |loading: String, success: String| {
        toast_promise(loading, success, 2000);
        selected.write().clear();
    };
    let ok = word().trim() == "DELETE";
    rsx! {
        BulkActions { state, count, entity: t!("users-entity"),
            button { r#type: "button", class: ICON_BTN, "aria-label": t!("users-bulk-invite"), title: t!("users-bulk-invite"),
                onclick: move |_| done(t!("users-inviting"), crate::ta!("users-invited", count: count, entity: plural(count))),
                Icon { icon: IconType::Mail }
                span { class: "sr-only", {t!("users-bulk-invite")} }
            }
            button { r#type: "button", class: ICON_BTN, "aria-label": t!("users-bulk-activate"), title: t!("users-bulk-activate"),
                onclick: move |_| done(t!("users-activating"), crate::ta!("users-activated", count: count, entity: plural(count))),
                Icon { icon: IconType::UserCheck }
                span { class: "sr-only", {t!("users-bulk-activate")} }
            }
            button { r#type: "button", class: ICON_BTN, "aria-label": t!("users-bulk-deactivate"), title: t!("users-bulk-deactivate"),
                onclick: move |_| done(t!("users-deactivating"), crate::ta!("users-deactivated", count: count, entity: plural(count))),
                Icon { icon: IconType::UserX }
                span { class: "sr-only", {t!("users-bulk-deactivate")} }
            }
            button { r#type: "button", class: ICON_BTN_DESTRUCTIVE, "aria-label": t!("users-bulk-delete"), title: t!("users-bulk-delete"),
                onclick: move |_| confirm.set(true),
                Icon { icon: IconType::Trash2 }
                span { class: "sr-only", {t!("users-bulk-delete")} }
            }
        }
        ConfirmDialog {
            open: confirm,
            title: crate::ta!("users-delete-n", count: count, entity: plural(count)),
            title_destructive: true,
            desc: String::new(),
            confirm_text: t!("dt-delete"),
            cancel_text: t!("action-cancel"),
            destructive: true,
            disabled: !ok,
            on_confirm: move |_| {
                if word().trim() != "DELETE" {
                    toast(ToastType::Error, crate::ta!("dt-type-to-confirm-error", word: "DELETE"));
                    return;
                }
                confirm.set(false);
                word.set(String::new());
                done(t!("users-deleting"), crate::ta!("users-deleted", count: count, entity: plural(count)));
            },
            body: rsx! { MultiDeleteBody { word, entity_plural: t!("users-entity-plural") } },
        }
    }
}
