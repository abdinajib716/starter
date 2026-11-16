//! `tasks-dialogs.tsx`, `tasks-mutate-drawer.tsx`, `tasks-import-dialog.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;

use super::data::Task;
use super::{TaskDialog, TasksCtx};
use crate::components::ui_extra::dialog::{ConfirmDialog, Modal, use_dialog_flag};
use crate::components::ui_extra::form::{
    FormItem, FormLabel, FormMessage, Radio, SelectBox, TextInput,
};
use crate::components::ui_extra::sheet::SideSheet;
use crate::ui::button::{Button, ButtonVariant};
use crate::util::show_submitted_data;

#[component]
pub fn TasksDialogs() -> Element {
    let ctx = use_context::<TasksCtx>();
    let create = use_dialog_flag(ctx.open, TaskDialog::Create);
    let update = use_dialog_flag(ctx.open, TaskDialog::Update);
    let import = use_dialog_flag(ctx.open, TaskDialog::Import);
    let delete = use_dialog_flag(ctx.open, TaskDialog::Delete);
    let current = (ctx.current)();
    rsx! {
        if create() { MutateSheet { open: create } }
        if import() { ImportDialog { open: import } }
        if let Some(row) = current {
            if update() { MutateSheet { key: "{row.id}", open: update, current: row.clone() } }
            ConfirmDialog {
                open: delete,
                destructive: true,
                class: "max-w-md",
                title: crate::ta!("tasks-delete-title", id: row.id.clone()),
                desc: String::new(),
                body: rsx! {
                    {t!("tasks-delete-desc-1")}
                    " "
                    strong { "{row.id}" }
                    ". "
                    br {}
                    {t!("dt-cannot-undo")}
                },
                confirm_text: t!("dt-delete"),
                cancel_text: t!("action-cancel"),
                on_confirm: {
                    let row = row.clone();
                    let mut delete = delete;
                    move |_| {
                        delete.set(false);
                        show_submitted_data(&serde_json::to_value(&row).unwrap_or_default(), &t!("tasks-deleted-title"));
                    }
                },
            }
        }
    }
}

#[component]
fn MutateSheet(open: Signal<bool>, #[props(optional)] current: Option<Task>) -> Element {
    let mut open = open;
    let is_update = current.is_some();
    let c = current.clone();
    let title = use_signal(|| c.as_ref().map(|t| t.title.clone()).unwrap_or_default());
    let c = current.clone();
    let status = use_signal(|| c.as_ref().map(|t| t.status.clone()).unwrap_or_default());
    let c = current.clone();
    let label = use_signal(|| c.as_ref().map(|t| t.label.clone()).unwrap_or_default());
    let priority = use_signal(|| {
        current
            .as_ref()
            .map(|t| t.priority.clone())
            .unwrap_or_default()
    });
    let mut submitted = use_signal(|| false);

    let show = submitted();
    let e_title = (show && title().trim().is_empty()).then(|| t!("v-task-title"));
    let e_status = (show && status().is_empty()).then(|| t!("v-task-status"));
    let e_label = (show && label().is_empty()).then(|| t!("v-task-label"));
    let e_priority = (show && priority().is_empty()).then(|| t!("v-task-priority"));

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if title().trim().is_empty()
            || status().is_empty()
            || label().is_empty()
            || priority().is_empty()
        {
            return;
        }
        open.set(false);
        show_submitted_data(
            &json!({ "title": title(), "status": status(), "label": label(), "priority": priority() }),
            &t!("submitted-title"),
        );
    };
    let statuses = [
        ("in progress", "task-status-in-progress"),
        ("backlog", "task-status-backlog"),
        ("todo", "task-status-todo"),
        ("canceled", "task-status-canceled"),
        ("done", "task-status-done"),
    ]
    .iter()
    .map(|(v, k)| (v.to_string(), t!(*k)))
    .collect::<Vec<_>>();

    rsx! {
        SideSheet { open, class: "flex flex-col",
            div { class: "flex flex-col gap-1.5 p-4 text-start",
                h2 { class: "font-semibold text-foreground", {t!(if is_update { "tasks-update-title" } else { "tasks-create-title" })} }
                p { class: "text-sm text-muted-foreground",
                    {t!(if is_update { "tasks-update-desc" } else { "tasks-create-desc" })}
                    " "
                    {t!("tasks-save-hint")}
                }
            }
            form { id: "tasks-form", class: "flex flex-1 flex-col overflow-hidden", onsubmit, novalidate: true,
                div { class: "flex-1 space-y-6 overflow-y-auto px-4",
                    FormItem {
                        FormLabel { html_for: "task-title", invalid: e_title.is_some(), {t!("tasks-col-title")} }
                        TextInput { id: "task-title", value: title, placeholder: t!("tasks-title-ph"), invalid: e_title.is_some() }
                        FormMessage { message: e_title.clone() }
                    }
                    FormItem {
                        FormLabel { invalid: e_status.is_some(), {t!("tasks-col-status")} }
                        SelectBox { value: status, options: statuses, placeholder: t!("tasks-select-ph"), invalid: e_status.is_some() }
                        FormMessage { message: e_status.clone() }
                    }
                    FormItem { class: "relative",
                        FormLabel { invalid: e_label.is_some(), {t!("tasks-label")} }
                        div { role: "radiogroup", class: "flex flex-col space-y-1",
                            for (v, k) in [("documentation", "task-label-documentation"), ("feature", "task-label-feature"), ("bug", "task-label-bug")] {
                                RadioRow { value: label, item: v, text: t!(k), id: "task-label-{v}" }
                            }
                        }
                        FormMessage { message: e_label.clone() }
                    }
                    FormItem { class: "relative",
                        FormLabel { invalid: e_priority.is_some(), {t!("tasks-col-priority")} }
                        div { role: "radiogroup", class: "flex flex-col space-y-1",
                            for (v, k) in [("high", "task-priority-high"), ("medium", "task-priority-medium"), ("low", "task-priority-low")] {
                                RadioRow { value: priority, item: v, text: t!(k), id: "task-priority-{v}" }
                            }
                        }
                        FormMessage { message: e_priority.clone() }
                    }
                }
                div { class: "mt-auto flex flex-col gap-2 p-4",
                    Button { variant: ButtonVariant::Outline, class: "w-full", button_type: "button", onclick: move |_| open.set(false), {t!("action-close")} }
                    Button { class: "w-full", button_type: "submit", {t!("action-save-changes")} }
                }
            }
        }
    }
}

#[component]
fn RadioRow(value: Signal<String>, item: String, text: String, id: String) -> Element {
    rsx! {
        div { "data-slot": "form-item", class: "flex items-center gap-2",
            Radio { value, item, id: id.clone() }
            FormLabel { html_for: id, class: "font-normal", "{text}" }
        }
    }
}

#[component]
fn ImportDialog(open: Signal<bool>) -> Element {
    let mut open = open;
    let mut file = use_signal(|| None::<(String, u64, String)>);
    let mut submitted = use_signal(|| false);
    let error = match (submitted(), file()) {
        (false, _) => None,
        (true, None) => Some(t!("v-file-required")),
        (true, Some((_, _, ty))) if ty != "text/csv" => Some(t!("v-file-csv")),
        _ => None,
    };
    rsx! {
        Modal { open, title: t!("tasks-import-title"), description: t!("tasks-import-desc"), class: "gap-2 sm:max-w-sm",
            form {
                id: "task-import-form",
                novalidate: true,
                onsubmit: move |e: FormEvent| {
                    e.prevent_default();
                    submitted.set(true);
                    if let Some((name, size, ty)) = file()
                        && ty == "text/csv"
                    {
                        show_submitted_data(&json!({ "name": name, "size": size, "type": ty }), &t!("tasks-imported-title"));
                        open.set(false);
                    }
                },
                FormItem { class: "my-2",
                    FormLabel { html_for: "task-import-file", invalid: error.is_some(), {t!("tasks-file")} }
                    input {
                        id: "task-import-file",
                        r#type: "file",
                        accept: "text/csv",
                        "aria-invalid": error.is_some(),
                        class: "flex h-8 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-0 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 md:text-sm dark:bg-input/30",
                        onchange: move |e| {
                            file.set(e.files().first().map(|f| (f.name(), f.size(), f.content_type().unwrap_or_default())));
                        },
                    }
                    FormMessage { message: error.clone() }
                }
                div { class: "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
                    Button { variant: ButtonVariant::Outline, button_type: "button", onclick: move |_| open.set(false), {t!("action-close")} }
                    Button { button_type: "submit", {t!("tasks-import")} }
                }
            }
        }
    }
}
