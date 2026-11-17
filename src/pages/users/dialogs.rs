//! `users-dialogs.tsx`, `users-action-dialog.tsx`, `users-delete-dialog.tsx` on real data,
//! plus the purge dialog of the Deleted tab (§25.3).
use dioxus::prelude::*;
use dioxus_i18n::t;
use uuid::Uuid;

use super::{UserDialog, UsersCtx};
use crate::api::users::{apply_user_action, create_user, purge_user, update_user};
use crate::components::data_table::DeleteWarning;
use crate::components::layout::use_current_user;
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::dialog::{ConfirmDialog, Modal, use_dialog_flag};
use crate::components::ui_extra::form::{FormItem, FormLabel, SelectBox, TextInput};
use crate::domain::users::{BulkUserAction, UserInput, UserRow, validate_input};
use crate::ui::button::Button;
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

/// Client-side minimum; the server applies `security.password_min_length` and answers with
/// `password.too_short` if it is higher.
const MIN_PW: usize = 8;

#[component]
pub fn UsersDialogs() -> Element {
    let ctx = use_context::<UsersCtx>();
    let add = use_dialog_flag(ctx.open, UserDialog::Add);
    let edit = use_dialog_flag(ctx.open, UserDialog::Edit);
    let delete = use_dialog_flag(ctx.open, UserDialog::Delete);
    let purge = use_dialog_flag(ctx.open, UserDialog::Purge);
    rsx! {
        if add() { ActionDialog { open: add } }
        if let Some(row) = (ctx.current)() {
            if edit() { ActionDialog { key: "edit-{row.id}", open: edit, current: row.clone() } }
            if delete() { DeleteDialog { key: "delete-{row.id}", open: delete, current: row.clone() } }
            if purge() { PurgeDialog { key: "purge-{row.id}", open: purge, current: row.clone() } }
        }
    }
}

const ROW: &str = "grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1";

#[component]
fn ActionDialog(open: Signal<bool>, #[props(optional)] current: Option<User>) -> Element {
    let mut open = open;
    let ctx = use_context::<UsersCtx>();
    let is_edit = current.is_some();
    let can_reset = use_current_user().is_some_and(|u| u.can("users.reset_password"));
    let c = current.clone();
    let name = use_signal(|| c.as_ref().map(|u| u.full_name.clone()).unwrap_or_default());
    let c = current.clone();
    let email = use_signal(|| c.as_ref().map(|u| u.email.clone()).unwrap_or_default());
    let c = current.clone();
    let role = use_signal(|| {
        c.as_ref()
            .map(|u| u.role_id.to_string())
            .unwrap_or_default()
    });
    let password = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    let mut busy = use_signal(|| false);

    let input = move || UserInput {
        full_name: name(),
        email: email(),
        role_id: role().parse().unwrap_or(Uuid::nil()),
        password: password().trim().to_string(),
    };
    let errors = validate_input(&input(), !is_edit, MIN_PW);
    let pw_mismatch = !password().trim().is_empty() && password().trim() != confirm().trim();
    let show = submitted();
    let err = |field: &str| {
        show.then(|| {
            errors
                .iter()
                .find(|(f, _)| *f == field)
                .map(|(_, c)| crate::i18n::error_text(&crate::domain::errors::coded(400, c)))
        })
        .flatten()
    };
    let (e_name, e_email, e_role, e_pw) = (
        err("full_name"),
        err("email"),
        err("role_id"),
        err("password"),
    );
    let e_confirm = (show && pw_mismatch).then(|| t!("v-password-match"));
    let valid = errors.is_empty() && !pw_mismatch;
    let id = current.as_ref().map(|u| u.id);

    let roles = (ctx.roles)();
    let role_options: Vec<(String, String)> = roles
        .iter()
        .map(|r| (r.id.to_string(), r.name.clone()))
        .collect();

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if !valid || busy() {
            return;
        }
        busy.set(true);
        let data = input();
        spawn(async move {
            let res = match id {
                Some(id) => update_user(id, data).await.map(|_| "users-updated"),
                None => create_user(data).await.map(|_| "users-created"),
            };
            busy.set(false);
            match res {
                Ok(key) => {
                    toast(ToastType::Success, t!(key));
                    open.set(false);
                    ctx.reload();
                }
                Err(e) => toast_error(&e),
            }
        });
    };

    rsx! {
        Modal {
            open,
            title: t!(if is_edit { "users-edit-title" } else { "users-add-title" }),
            description: format!("{} {}", t!(if is_edit { "users-edit-desc" } else { "users-add-desc" }), t!("tasks-save-hint")),
            class: "sm:max-w-lg",
            div { class: "overflow-x-hidden py-1 pe-1",
                form { id: "user-form", class: "space-y-4 px-0.5", novalidate: true, onsubmit,
                    Field { id: "user-name", label: t!("users-col-name"), error: e_name.clone(),
                        TextInput { id: "user-name", value: name, placeholder: t!("users-name-ph"), class: "col-span-4", invalid: e_name.is_some() }
                    }
                    Field { id: "user-email", label: t!("users-col-email"), error: e_email.clone(),
                        TextInput { id: "user-email", r#type: "email", value: email, placeholder: "john.doe@gmail.com", class: "col-span-4", invalid: e_email.is_some() }
                    }
                    Field { id: "user-role", label: t!("users-col-role"), error: e_role.clone(),
                        SelectBox { value: role, options: role_options, placeholder: t!("users-select-role"), class: "col-span-4 w-full", trigger_class: "w-full", invalid: e_role.is_some() }
                    }
                    if !is_edit || can_reset {
                        Field { id: "user-password", label: t!(if is_edit { "users-new-password" } else { "users-password" }), error: e_pw.clone(),
                            PasswordInput { id: "user-password", value: password, placeholder: t!(if is_edit { "users-password-keep" } else { "users-password-ph" }), class: "col-span-4", invalid: e_pw.is_some() }
                        }
                        Field { id: "user-confirm", label: t!("users-confirm-password"), error: e_confirm.clone(),
                            PasswordInput { id: "user-confirm", value: confirm, placeholder: t!("users-password-ph"), class: "col-span-4", disabled: password().is_empty(), invalid: e_confirm.is_some() }
                        }
                        p { class: "col-span-6 text-xs text-muted-foreground", {t!("users-temp-password-note")} }
                    }
                }
            }
            div { class: "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
                Button { button_type: "button", disabled: busy(), onclick: move |_| {
                    let _ = dioxus::document::eval("document.getElementById('user-form')?.requestSubmit()");
                }, {t!("action-save-changes")} }
            }
        }
    }
}

type User = UserRow;

#[component]
fn Field(id: String, label: String, error: Option<String>, children: Element) -> Element {
    rsx! {
        FormItem { class: ROW,
            FormLabel { html_for: id, invalid: error.is_some(), class: "col-span-2 justify-end text-end", "{label}" }
            {children}
            if let Some(m) = error { p { class: "col-span-4 col-start-3 text-sm text-destructive", "{m}" } }
        }
    }
}

const CONFIRM_INPUT: &str = "flex h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base font-normal shadow-xs outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 md:text-sm dark:bg-input/30";

/// `users-delete-dialog.tsx`: type the email to confirm a (soft) delete.
#[component]
fn DeleteDialog(open: Signal<bool>, current: User) -> Element {
    let mut open = open;
    let ctx = use_context::<UsersCtx>();
    let mut value = use_signal(String::new);
    let ok = value().trim().eq_ignore_ascii_case(&current.email);
    let id = current.id;
    rsx! {
        ConfirmDialog {
            open,
            title: t!("users-delete-title"),
            title_destructive: true,
            desc: String::new(),
            confirm_text: t!("dt-delete"),
            cancel_text: t!("action-cancel"),
            destructive: true,
            disabled: !ok,
            on_confirm: move |_| {
                if !ok { return; }
                // Close after the answer: the task belongs to this dialog and ends with it.
                spawn(async move {
                    match apply_user_action(BulkUserAction::Delete, id).await {
                        Ok(()) => { toast(ToastType::Success, t!("users-deleted-one")); ctx.reload(); }
                        Err(e) => toast_error(&e),
                    }
                    open.set(false);
                });
            },
            body: rsx! {
                div { class: "space-y-4",
                    p { class: "mb-2",
                        {t!("users-delete-sure")}
                        " "
                        span { class: "font-bold", "{current.full_name}" }
                        "?"
                        br {}
                        {t!("users-delete-role")}
                        " "
                        span { class: "font-bold", "{current.role_name.to_uppercase()}" }
                        " "
                        {t!("users-delete-soft-tail")}
                    }
                    label { class: "my-2 flex flex-col items-start gap-1.5 text-sm leading-none font-medium text-foreground select-none",
                        {t!("users-email-label")}
                        input { class: CONFIRM_INPUT, value: value(), placeholder: t!("users-email-confirm-ph"), autofocus: true, oninput: move |e| value.set(e.value()) }
                    }
                    DeleteWarning {}
                }
            },
        }
    }
}

/// Deleted tab → "Delete permanently" (`users.purge`, type the email).
#[component]
fn PurgeDialog(open: Signal<bool>, current: User) -> Element {
    let mut open = open;
    let ctx = use_context::<UsersCtx>();
    let mut value = use_signal(String::new);
    let ok = value().trim().eq_ignore_ascii_case(&current.email);
    let id = current.id;
    rsx! {
        ConfirmDialog {
            open,
            title: t!("users-purge-title"),
            title_destructive: true,
            desc: String::new(),
            confirm_text: t!("users-purge"),
            cancel_text: t!("action-cancel"),
            destructive: true,
            disabled: !ok,
            on_confirm: move |_| {
                if !ok { return; }
                let email = value();
                spawn(async move {
                    match purge_user(id, email).await {
                        Ok(()) => { toast(ToastType::Success, t!("users-purged")); ctx.reload(); }
                        Err(e) => toast_error(&e),
                    }
                    open.set(false);
                });
            },
            body: rsx! {
                div { class: "space-y-4",
                    p { class: "mb-2",
                        {t!("users-purge-sure")}
                        " "
                        span { class: "font-bold", "{current.email}" }
                        br {}
                        {t!("users-purge-tail")}
                    }
                    label { class: "my-2 flex flex-col items-start gap-1.5 text-sm leading-none font-medium text-foreground select-none",
                        {t!("users-email-label")}
                        input { class: CONFIRM_INPUT, value: value(), placeholder: t!("users-email-confirm-ph"), autofocus: true, oninput: move |e| value.set(e.value()) }
                    }
                    DeleteWarning {}
                }
            },
        }
    }
}
