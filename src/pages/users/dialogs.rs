//! `users-dialogs.tsx`, `users-action-dialog.tsx`, `users-invite-dialog.tsx`,
//! `users-delete-dialog.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;

use super::data::{ROLES, User};
use super::{UserDialog, UsersCtx};
use crate::components::data_table::DeleteWarning;
use crate::components::password_input::PasswordInput;
use crate::components::ui_extra::dialog::{ConfirmDialog, Modal, use_dialog_flag};
use crate::components::ui_extra::form::{
    FormItem, FormLabel, FormMessage, SelectBox, TextArea, TextInput,
};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::pages::auth::valid_email;
use crate::ui::button::{Button, ButtonVariant};
use crate::util::show_submitted_data;

const FOOTER: &str = "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end";

fn role_options() -> Vec<(String, String)> {
    ROLES
        .iter()
        .map(|(v, k, _)| (v.to_string(), t!(*k)))
        .collect()
}

#[component]
pub fn UsersDialogs() -> Element {
    let ctx = use_context::<UsersCtx>();
    let add = use_dialog_flag(ctx.open, UserDialog::Add);
    let edit = use_dialog_flag(ctx.open, UserDialog::Edit);
    let invite = use_dialog_flag(ctx.open, UserDialog::Invite);
    let delete = use_dialog_flag(ctx.open, UserDialog::Delete);
    rsx! {
        if add() { ActionDialog { open: add } }
        if invite() { InviteDialog { open: invite } }
        if let Some(row) = (ctx.current)() {
            if edit() { ActionDialog { key: "edit-{row.id}", open: edit, current: row.clone() } }
            if delete() { DeleteDialog { key: "delete-{row.id}", open: delete, current: row.clone() } }
        }
    }
}

/// Password rules in the reference's refine order; `None` when valid.
fn password_error(password: &str, is_edit: bool) -> Option<&'static str> {
    if is_edit && password.is_empty() {
        None
    } else if password.is_empty() {
        Some("v-password-required")
    } else if password.chars().count() < 8 {
        Some("v-password-min")
    } else if !password.chars().any(|c| c.is_ascii_lowercase()) {
        Some("v-password-lower")
    } else if !password.chars().any(|c| c.is_ascii_digit()) {
        Some("v-password-number")
    } else {
        None
    }
}

const ROW: &str = "grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1";

#[component]
fn ActionDialog(open: Signal<bool>, #[props(optional)] current: Option<User>) -> Element {
    let mut open = open;
    let is_edit = current.is_some();
    let init = |f: fn(&User) -> String| current.as_ref().map(f).unwrap_or_default();
    let first = use_signal(|| init(|u| u.first_name.clone()));
    let last = use_signal(|| init(|u| u.last_name.clone()));
    let username = use_signal(|| init(|u| u.username.clone()));
    let email = use_signal(|| init(|u| u.email.clone()));
    let phone = use_signal(|| init(|u| u.phone_number.clone()));
    let role = use_signal(|| init(|u| u.role.clone()));
    let password = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut submitted = use_signal(|| false);

    let show = submitted();
    let req = |s: Signal<String>, key: &str| (show && s().trim().is_empty()).then(|| t!(key));
    let e_first = req(first, "v-first-name");
    let e_last = req(last, "v-last-name");
    let e_username = req(username, "v-username");
    let e_phone = req(phone, "v-phone");
    let e_role = req(role, "v-role");
    let e_email = show
        .then(|| {
            if email().trim().is_empty() {
                Some(t!("v-email-required"))
            } else if !valid_email(&email()) {
                Some(t!("v-email-invalid"))
            } else {
                None
            }
        })
        .flatten();
    let pw = password().trim().to_string();
    let e_password = show
        .then(|| password_error(&pw, is_edit).map(|k| t!(k)))
        .flatten();
    let e_confirm = (show && !(is_edit && pw.is_empty()) && pw != confirm().trim())
        .then(|| t!("v-password-match"));
    let valid = move |pw: &str| {
        !first().trim().is_empty()
            && !last().trim().is_empty()
            && !username().trim().is_empty()
            && !phone().trim().is_empty()
            && !role().is_empty()
            && valid_email(&email())
            && password_error(pw, is_edit).is_none()
            && ((is_edit && pw.is_empty()) || pw == confirm().trim())
    };
    let current_row = current.clone();
    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        let pw = password().trim().to_string();
        if !valid(&pw) {
            return;
        }
        let mut values = current_row
            .as_ref()
            .and_then(|u| serde_json::to_value(u).ok())
            .unwrap_or_else(|| json!({}));
        for (k, v) in [
            ("firstName", first()),
            ("lastName", last()),
            ("username", username()),
            ("email", email()),
            ("role", role()),
            ("phoneNumber", phone()),
            ("password", pw.clone()),
            ("confirmPassword", confirm().trim().to_string()),
        ] {
            values[k] = json!(v);
        }
        values["isEdit"] = json!(is_edit);
        show_submitted_data(&values, &t!("submitted-title"));
        open.set(false);
    };

    rsx! {
        Modal {
            open,
            title: t!(if is_edit { "users-edit-title" } else { "users-add-title" }),
            description: format!("{} {}", t!(if is_edit { "users-edit-desc" } else { "users-add-desc" }), t!("tasks-save-hint")),
            class: "sm:max-w-lg",
            div { class: "h-105 w-[calc(100%+0.75rem)] overflow-y-auto py-1 pe-3",
                form { id: "user-form", class: "space-y-4 px-0.5", novalidate: true, onsubmit,
                    Field { id: "user-first", label: t!("users-first-name"), error: e_first,
                        TextInput { id: "user-first", value: first, placeholder: "John", class: "col-span-4", invalid: show && first().trim().is_empty() }
                    }
                    Field { id: "user-last", label: t!("users-last-name"), error: e_last,
                        TextInput { id: "user-last", value: last, placeholder: "Doe", class: "col-span-4", invalid: show && last().trim().is_empty() }
                    }
                    Field { id: "user-username", label: t!("users-col-username"), error: e_username,
                        TextInput { id: "user-username", value: username, placeholder: "john_doe", class: "col-span-4", invalid: show && username().trim().is_empty() }
                    }
                    Field { id: "user-email", label: t!("users-col-email"), error: e_email.clone(),
                        TextInput { id: "user-email", value: email, placeholder: "john.doe@gmail.com", class: "col-span-4", invalid: e_email.is_some() }
                    }
                    Field { id: "user-phone", label: t!("users-col-phone"), error: e_phone,
                        TextInput { id: "user-phone", value: phone, placeholder: "+123456789", class: "col-span-4", invalid: show && phone().trim().is_empty() }
                    }
                    Field { id: "user-role", label: t!("users-col-role"), error: e_role.clone(),
                        SelectBox { value: role, options: role_options(), placeholder: t!("users-select-role"), class: "col-span-4", trigger_class: "w-full", invalid: e_role.is_some() }
                    }
                    Field { id: "user-password", label: t!("users-password"), error: e_password.clone(),
                        PasswordInput { id: "user-password", value: password, placeholder: t!("users-password-ph"), class: "col-span-4", invalid: e_password.is_some() }
                    }
                    Field { id: "user-confirm", label: t!("users-confirm-password"), error: e_confirm.clone(),
                        PasswordInput { id: "user-confirm", value: confirm, placeholder: t!("users-password-ph"), class: "col-span-4", disabled: password().is_empty(), invalid: e_confirm.is_some() }
                    }
                }
            }
            div { class: FOOTER,
                Button { button_type: "button", onclick: move |_| {
                    // The submit button sits outside the scrolling form, like `form='user-form'`.
                    let _ = dioxus::document::eval("document.getElementById('user-form')?.requestSubmit()");
                }, {t!("action-save-changes")} }
            }
        }
    }
}

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

#[component]
fn InviteDialog(open: Signal<bool>) -> Element {
    let mut open = open;
    let email = use_signal(String::new);
    let role = use_signal(String::new);
    let desc = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    let show = submitted();
    let e_email = show
        .then(|| {
            if email().trim().is_empty() {
                Some(t!("v-invite-email"))
            } else if !valid_email(&email()) {
                Some(t!("v-email-invalid"))
            } else {
                None
            }
        })
        .flatten();
    let e_role = (show && role().is_empty()).then(|| t!("v-role"));
    rsx! {
        Modal { open, title: t!("users-invite-title"), description: t!("users-invite-desc"), class: "sm:max-w-md",
            form {
                id: "user-invite-form",
                class: "space-y-4",
                novalidate: true,
                onsubmit: move |e: FormEvent| {
                    e.prevent_default();
                    submitted.set(true);
                    if valid_email(&email()) && !role().is_empty() {
                        show_submitted_data(&json!({ "email": email(), "role": role(), "desc": desc() }), &t!("submitted-title"));
                        open.set(false);
                    }
                },
                FormItem {
                    FormLabel { html_for: "invite-email", invalid: e_email.is_some(), {t!("users-col-email")} }
                    TextInput { id: "invite-email", r#type: "email", value: email, placeholder: t!("users-invite-email-ph"), invalid: e_email.is_some() }
                    FormMessage { message: e_email.clone() }
                }
                FormItem {
                    FormLabel { invalid: e_role.is_some(), {t!("users-col-role")} }
                    SelectBox { value: role, options: role_options(), placeholder: t!("users-select-role"), invalid: e_role.is_some() }
                    FormMessage { message: e_role.clone() }
                }
                FormItem {
                    FormLabel { html_for: "invite-desc", {t!("users-invite-note")} }
                    TextArea { id: "invite-desc", value: desc, placeholder: t!("users-invite-note-ph"), class: "resize-none" }
                }
                div { class: "{FOOTER} gap-y-2",
                    Button { variant: ButtonVariant::Outline, button_type: "button", onclick: move |_| open.set(false), {t!("action-cancel")} }
                    Button { button_type: "submit", {t!("users-invite-submit")} Icon { icon: IconType::Send } }
                }
            }
        }
    }
}

#[component]
fn DeleteDialog(open: Signal<bool>, current: User) -> Element {
    let mut open = open;
    let mut value = use_signal(String::new);
    let ok = value().trim() == current.username;
    let row = current.clone();
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
                if value().trim() != row.username {
                    return;
                }
                open.set(false);
                show_submitted_data(&serde_json::to_value(&row).unwrap_or_default(), &t!("users-deleted-title"));
            },
            body: rsx! {
                div { class: "space-y-4",
                    p { class: "mb-2",
                        {t!("users-delete-sure")}
                        " "
                        span { class: "font-bold", "{current.username}" }
                        "?"
                        br {}
                        {t!("users-delete-role")}
                        " "
                        span { class: "font-bold", "{current.role.to_uppercase()}" }
                        " "
                        {t!("users-delete-tail")}
                    }
                    label { class: "my-2 flex items-center gap-2 text-sm leading-none font-medium text-foreground select-none",
                        {t!("users-username-label")}
                        input {
                            class: "flex h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base font-normal shadow-xs outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 md:text-sm dark:bg-input/30",
                            value: value(),
                            placeholder: t!("users-username-confirm-ph"),
                            autofocus: true,
                            oninput: move |e| value.set(e.value()),
                        }
                    }
                    DeleteWarning {}
                }
            },
        }
    }
}
