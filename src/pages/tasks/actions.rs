//! `data-table-row-actions.tsx`, `data-table-bulk-actions.tsx`, `tasks-multi-delete-dialog.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;

use super::data::{LABELS, PRIORITIES, STATUSES, Task};
use super::{TaskDialog, TasksCtx};
use crate::components::data_table::{
    BulkActions, ICON_BTN, ICON_BTN_DESTRUCTIVE, MultiDeleteBody, TableState,
};
use crate::components::ui_extra::dialog::ConfirmDialog;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, ITEM, Menu, MenuSeparator, MenuShortcut};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_promise};

const DISABLED: &str = "relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm opacity-50 outline-hidden select-none pointer-events-none";

#[component]
pub fn RowActions(task: Task) -> Element {
    let mut ctx = use_context::<TasksCtx>();
    let edit = task.clone();
    let del = task.clone();
    rsx! {
        Menu {
            align: Align::End,
            class: "w-40 overflow-x-visible overflow-y-visible",
            trigger: rsx! {
                button { r#type: "button", class: "flex h-8 w-8 items-center justify-center rounded-md p-0 hover:bg-accent hover:text-accent-foreground in-data-[state=open]:bg-muted",
                    Icon { icon: IconType::Ellipsis, class: "h-4 w-4" }
                    span { class: "sr-only", {t!("dt-open-menu")} }
                }
            },
            button { r#type: "button", class: ITEM, onclick: move |_| { ctx.current.set(Some(edit.clone())); ctx.open.set(Some(TaskDialog::Update)); }, {t!("dt-edit")} }
            div { class: DISABLED, "aria-disabled": "true", {t!("tasks-make-copy")} }
            div { class: DISABLED, "aria-disabled": "true", {t!("tasks-favorite")} }
            MenuSeparator {}
            // DropdownMenuSub: opens on hover / focus to the side.
            div { class: "group/sub relative",
                button { r#type: "button", onclick: move |e| e.stop_propagation(), class: "flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none hover:bg-accent focus:bg-accent group-focus-within/sub:bg-accent",
                    {t!("tasks-labels")}
                    Icon { icon: IconType::ChevronRight, class: "ms-auto size-4 rtl:rotate-180" }
                }
                div { role: "menu", class: "invisible absolute end-full top-0 z-50 min-w-[8rem] rounded-md border bg-popover p-1 text-popover-foreground shadow-lg group-focus-within/sub:visible group-hover/sub:visible",
                    for (value, key) in LABELS {
                        div { role: "menuitemradio", "aria-checked": "{task.label == value}", class: "relative flex cursor-default items-center gap-2 rounded-sm py-1.5 ps-8 pe-2 text-sm outline-hidden select-none hover:bg-accent",
                            span { class: "pointer-events-none absolute start-2 flex size-3.5 items-center justify-center",
                                if task.label == value { Icon { icon: IconType::Circle, class: "size-2 fill-current" } }
                            }
                            {t!(key)}
                        }
                    }
                }
            }
            MenuSeparator {}
            button { r#type: "button", class: ITEM, onclick: move |_| { ctx.current.set(Some(del.clone())); ctx.open.set(Some(TaskDialog::Delete)); },
                {t!("dt-delete")}
                MenuShortcut { Icon { icon: IconType::Trash2, class: "size-4" } }
            }
        }
    }
}

fn plural(n: usize) -> String {
    t!(if n > 1 {
        "tasks-entity-plural"
    } else {
        "tasks-entity"
    })
}

#[component]
pub fn TasksBulkActions(state: TableState, rows: Vec<Task>) -> Element {
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
        BulkActions { state, count, entity: t!("tasks-entity"),
            Menu {
                side_offset: 14.0,
                trigger: rsx! {
                    button { r#type: "button", class: ICON_BTN, "aria-label": t!("tasks-bulk-status"), title: t!("tasks-bulk-status"),
                        Icon { icon: IconType::CircleArrowUp }
                        span { class: "sr-only", {t!("tasks-bulk-status")} }
                    }
                },
                for (value, key, icon) in STATUSES {
                    button { r#type: "button", class: ITEM,
                        onclick: move |_| done(t!("tasks-updating-status"), crate::ta!("tasks-status-updated", status: value, count: count, entity: plural(count))),
                        Icon { icon, class: "size-4 text-muted-foreground" }
                        {t!(key)}
                    }
                }
            }
            Menu {
                side_offset: 14.0,
                trigger: rsx! {
                    button { r#type: "button", class: ICON_BTN, "aria-label": t!("tasks-bulk-priority"), title: t!("tasks-bulk-priority"),
                        Icon { icon: IconType::ArrowUpDown }
                        span { class: "sr-only", {t!("tasks-bulk-priority")} }
                    }
                },
                for (value, key, icon) in PRIORITIES {
                    button { r#type: "button", class: ITEM,
                        onclick: move |_| done(t!("tasks-updating-priority"), crate::ta!("tasks-priority-updated", priority: value, count: count, entity: plural(count))),
                        Icon { icon, class: "size-4 text-muted-foreground" }
                        {t!(key)}
                    }
                }
            }
            button { r#type: "button", class: ICON_BTN, "aria-label": t!("tasks-bulk-export"), title: t!("tasks-bulk-export"),
                onclick: move |_| done(t!("tasks-exporting"), crate::ta!("tasks-exported", count: count, entity: plural(count))),
                Icon { icon: IconType::Download }
                span { class: "sr-only", {t!("tasks-bulk-export")} }
            }
            button { r#type: "button", class: ICON_BTN_DESTRUCTIVE,
                "aria-label": t!("tasks-bulk-delete"), title: t!("tasks-bulk-delete"),
                onclick: move |_| confirm.set(true),
                Icon { icon: IconType::Trash2 }
                span { class: "sr-only", {t!("tasks-bulk-delete")} }
            }
        }
        ConfirmDialog {
            open: confirm,
            title: crate::ta!("tasks-delete-n", count: count, entity: plural(count)),
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
                done(t!("tasks-deleting"), crate::ta!("tasks-deleted", count: count, entity: plural(count)));
            },
            body: rsx! { MultiDeleteBody { word, entity_plural: t!("tasks-entity-plural") } },
        }
    }
}
