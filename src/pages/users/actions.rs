//! `data-table-row-actions.tsx`, `data-table-bulk-actions.tsx`, `users-multi-delete-dialog.tsx`
//! on real data. Every action is also checked on the server; the menus only hide what the
//! signed-in user may not do.
use dioxus::prelude::*;
use dioxus_i18n::t;
use uuid::Uuid;

use super::{UserDialog, UsersCtx};
use crate::api::users::{apply_user_action, bulk_users};
use crate::components::data_table::{
    BulkActions, ICON_BTN, ICON_BTN_DESTRUCTIVE, MultiDeleteBody, TableState,
};
use crate::components::layout::use_current_user;
use crate::components::ui_extra::dialog::ConfirmDialog;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{
    Align, ITEM, ITEM_DESTRUCTIVE, Menu, MenuSeparator, MenuShortcut,
};
use crate::domain::users::{BulkResult, BulkUserAction, UserRow, UserStatus};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

fn can(perm: &str) -> bool {
    use_current_user().is_some_and(|u| u.can(perm))
}

/// Run one action on one user, toast the outcome, reload the list.
fn run(ctx: UsersCtx, action: BulkUserAction, id: Uuid, done_key: &'static str) {
    spawn(async move {
        match apply_user_action(action, id).await {
            Ok(()) => {
                toast(ToastType::Success, t!(done_key));
                ctx.reload();
            }
            Err(e) => toast_error(&e),
        }
    });
}

#[component]
pub fn RowActions(user: UserRow, deleted: bool) -> Element {
    let mut ctx = use_context::<UsersCtx>();
    let (can_update, can_delete, can_restore, can_purge) = (
        can("users.update"),
        can("users.delete"),
        can("users.restore"),
        can("users.purge"),
    );
    let me = use_current_user().map(|u| u.id) == Some(user.id);
    let id = user.id;
    let active = !matches!(user.status, UserStatus::Inactive);
    let edit = user.clone();
    let del = user.clone();
    let purge = user.clone();
    let nothing = if deleted {
        !(can_restore || can_purge)
    } else {
        !(can_update || can_delete)
    };
    if nothing {
        return rsx! {};
    }
    rsx! {
        Menu {
            align: Align::End,
            class: "w-44",
            trigger: rsx! {
                button { r#type: "button", class: "flex h-8 w-8 items-center justify-center rounded-md p-0 hover:bg-accent hover:text-accent-foreground in-data-[state=open]:bg-muted",
                    Icon { icon: IconType::Ellipsis, class: "h-4 w-4" }
                    span { class: "sr-only", {t!("dt-open-menu")} }
                }
            },
            if deleted {
                if can_restore {
                    button { r#type: "button", class: ITEM, onclick: move |_| run(ctx, BulkUserAction::Restore, id, "users-restored"),
                        {t!("users-restore")}
                        MenuShortcut { Icon { icon: IconType::RotateCcw, class: "size-4" } }
                    }
                }
                if can_purge {
                    MenuSeparator {}
                    button { r#type: "button", class: "{ITEM} {ITEM_DESTRUCTIVE}",
                        onclick: move |_| { ctx.current.set(Some(purge.clone())); ctx.open.set(Some(UserDialog::Purge)); },
                        {t!("users-purge")}
                        MenuShortcut { Icon { icon: IconType::Trash2, class: "size-4" } }
                    }
                }
            } else {
                if can_update {
                    button { r#type: "button", class: ITEM, onclick: move |_| { ctx.current.set(Some(edit.clone())); ctx.open.set(Some(UserDialog::Edit)); },
                        {t!("dt-edit")}
                        MenuShortcut { Icon { icon: IconType::UserPen, class: "size-4" } }
                    }
                    if !me {
                        if active {
                            button { r#type: "button", class: ITEM, onclick: move |_| run(ctx, BulkUserAction::Deactivate, id, "users-deactivated-one"),
                                {t!("users-deactivate")}
                                MenuShortcut { Icon { icon: IconType::UserX, class: "size-4" } }
                            }
                        } else {
                            button { r#type: "button", class: ITEM, onclick: move |_| run(ctx, BulkUserAction::Activate, id, "users-activated-one"),
                                {t!("users-activate")}
                                MenuShortcut { Icon { icon: IconType::UserCheck, class: "size-4" } }
                            }
                        }
                    }
                }
                if can_delete && !me {
                    MenuSeparator {}
                    button { r#type: "button", class: "{ITEM} text-red-500!",
                        onclick: move |_| { ctx.current.set(Some(del.clone())); ctx.open.set(Some(UserDialog::Delete)); },
                        {t!("dt-delete")}
                        MenuShortcut { Icon { icon: IconType::Trash2, class: "size-4" } }
                    }
                }
            }
        }
    }
}

fn plural(n: usize) -> String {
    t!(if n == 1 {
        "users-entity"
    } else {
        "users-entity-plural"
    })
}

/// Toast for a bulk result: how many were done, and the first skip reason if any.
fn report(r: &BulkResult, done_key: &'static str) {
    let n = r.done as usize;
    toast(
        ToastType::Success,
        crate::ta!(done_key, count: n, entity: plural(n)),
    );
    if let Some((_, code)) = r.skipped.first() {
        let reason = crate::i18n::error_text(&crate::domain::errors::coded(400, code));
        toast(
            ToastType::Warning,
            crate::ta!("users-skipped", count: r.skipped.len(), reason: reason),
        );
    }
}

#[component]
pub fn UsersBulkActions(state: TableState, ids: Vec<Uuid>, deleted: bool) -> Element {
    let ctx = use_context::<UsersCtx>();
    let mut selected = state.selected;
    let count = ids.len();
    let mut confirm = use_signal(|| false);
    let mut word = use_signal(String::new);
    let ok = word().trim() == "DELETE";
    let (can_update, can_delete, can_restore, can_export) = (
        can("users.update"),
        can("users.delete"),
        can("users.restore"),
        can("users.export"),
    );

    let run_bulk = {
        let ids = ids.clone();
        move |action: BulkUserAction, done_key: &'static str| {
            let ids = ids.clone();
            spawn(async move {
                match bulk_users(action, ids).await {
                    Ok(r) => {
                        report(&r, done_key);
                        selected.write().clear();
                        ctx.reload();
                    }
                    Err(e) => toast_error(&e),
                }
            });
        }
    };
    let (rb1, rb2, rb3, rb4) = (
        run_bulk.clone(),
        run_bulk.clone(),
        run_bulk.clone(),
        run_bulk,
    );
    let export_ids = ids.clone();

    rsx! {
        BulkActions { state, count, entity: t!("users-entity"),
            if deleted {
                if can_restore {
                    button { r#type: "button", class: ICON_BTN, "aria-label": t!("users-bulk-restore"), title: t!("users-bulk-restore"),
                        onclick: move |_| rb1(BulkUserAction::Restore, "users-restored-n"),
                        Icon { icon: IconType::RotateCcw }
                        span { class: "sr-only", {t!("users-bulk-restore")} }
                    }
                }
            } else {
                if can_update {
                    button { r#type: "button", class: ICON_BTN, "aria-label": t!("users-bulk-activate"), title: t!("users-bulk-activate"),
                        onclick: move |_| rb2(BulkUserAction::Activate, "users-activated"),
                        Icon { icon: IconType::UserCheck }
                        span { class: "sr-only", {t!("users-bulk-activate")} }
                    }
                    button { r#type: "button", class: ICON_BTN, "aria-label": t!("users-bulk-deactivate"), title: t!("users-bulk-deactivate"),
                        onclick: move |_| rb3(BulkUserAction::Deactivate, "users-deactivated"),
                        Icon { icon: IconType::UserX }
                        span { class: "sr-only", {t!("users-bulk-deactivate")} }
                    }
                }
            }
            if can_export {
                button { r#type: "button", class: ICON_BTN, "aria-label": t!("users-bulk-export"), title: t!("users-bulk-export"),
                    onclick: move |_| super::table::download_csv(&super::query_for_export(), Some(export_ids.clone())),
                    Icon { icon: IconType::Download }
                    span { class: "sr-only", {t!("users-bulk-export")} }
                }
            }
            if !deleted && can_delete {
                button { r#type: "button", class: ICON_BTN_DESTRUCTIVE, "aria-label": t!("users-bulk-delete"), title: t!("users-bulk-delete"),
                    onclick: move |_| confirm.set(true),
                    Icon { icon: IconType::Trash2 }
                    span { class: "sr-only", {t!("users-bulk-delete")} }
                }
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
                rb4(BulkUserAction::Delete, "users-deleted");
            },
            body: rsx! { MultiDeleteBody { word, entity_plural: t!("users-entity-plural") } },
        }
    }
}
