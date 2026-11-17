//! Role editor (M6.4): name, description, "require 2FA" (M6.5, used by M9) and the
//! permission grid — one row per module with "all" plus each action. Keys the signed-in user
//! does not hold are disabled (no escalation, §8); system roles open read-only.
use std::collections::BTreeSet;

use dioxus::prelude::*;
use dioxus_i18n::t;
use uuid::Uuid;

use super::RolesCtx;
use crate::api::roles::{create_role, get_role, list_permissions, update_role};
use crate::components::layout::use_current_user;
use crate::components::skeletons::FormSkeleton;
use crate::components::ui_extra::dialog::Modal;
use crate::components::ui_extra::form::{
    CheckBox, FormDescription, FormItem, FormLabel, FormMessage, Switch, TextArea, TextInput,
};
use crate::domain::roles::{PermissionInfo, RoleInput, group_by_module, validate_role};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

/// Translated module / action label, falling back to the code.
fn label(prefix: &str, code: &str) -> String {
    dioxus_i18n::te!(&format!("{prefix}-{}", code.replace('_', "-")))
        .unwrap_or_else(|_| code.replace('_', " "))
}

#[component]
pub fn RoleEditor(id: Option<Uuid>) -> Element {
    let mut ctx = use_context::<RolesCtx>();
    let mut open = use_signal(|| true);
    use_effect(move || {
        if !open() {
            ctx.editing.set(None);
        }
    });
    let me = use_current_user();
    let held: BTreeSet<String> = me
        .as_ref()
        .map(|u| u.permissions.iter().cloned().collect())
        .unwrap_or_default();
    let is_super = me.as_ref().is_some_and(|u| u.is_super_admin);
    let can_save = me.as_ref().is_some_and(|u| {
        u.can(if id.is_some() {
            "roles.update"
        } else {
            "roles.create"
        })
    });

    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut require_2fa = use_signal(|| false);
    let mut keys = use_signal(BTreeSet::<String>::new);
    let mut system = use_signal(|| false);
    let mut catalog = use_signal(|| None::<Vec<PermissionInfo>>);
    let mut submitted = use_signal(|| false);
    let mut busy = use_signal(|| false);

    let _load = use_resource(move || async move {
        let perms = match list_permissions().await {
            Ok(c) => c.permissions,
            Err(e) => return toast_error(&e),
        };
        if let Some(id) = id {
            match get_role(id).await {
                Ok(d) => {
                    name.set(d.role.name);
                    description.set(d.role.description);
                    require_2fa.set(d.role.require_2fa);
                    system.set(d.role.is_system);
                    keys.set(d.keys.into_iter().collect());
                }
                Err(e) => return toast_error(&e),
            }
        }
        catalog.set(Some(perms));
    });

    let read_only = system() || !can_save;
    let input = move || RoleInput {
        name: name(),
        description: description(),
        require_2fa: require_2fa(),
        keys: keys().into_iter().collect(),
    };
    let error = submitted()
        .then(|| validate_role(&input()))
        .flatten()
        .map(|c| crate::i18n::error_text(&crate::domain::errors::coded(400, c)));
    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if read_only || busy() || validate_role(&input()).is_some() {
            return;
        }
        busy.set(true);
        let data = input();
        spawn(async move {
            let res = match id {
                Some(id) => update_role(id, data).await.map(|n| {
                    if n > 0 {
                        crate::ta!("roles-saved-signed-out", count: n as usize)
                    } else {
                        t!("roles-saved")
                    }
                }),
                None => create_role(data).await.map(|_| t!("roles-created")),
            };
            busy.set(false);
            match res {
                Ok(msg) => {
                    toast(ToastType::Success, msg);
                    ctx.reload();
                    open.set(false);
                }
                Err(e) => toast_error(&e),
            }
        });
    };

    let title = t!(match (id, read_only) {
        (None, _) => "roles-add",
        (Some(_), true) => "roles-view",
        (Some(_), false) => "roles-edit",
    });
    rsx! {
        Modal { open, title, description: t!("roles-editor-desc"), class: "sm:max-w-3xl",
            match catalog() {
                None => rsx! { FormSkeleton { fields: 3 } },
                Some(perms) => rsx! {
                    form { id: "role-form", class: "flex max-h-[70vh] flex-col gap-4 overflow-y-auto pe-1", novalidate: true, onsubmit,
                        div { class: "grid gap-4 sm:grid-cols-2",
                            FormItem {
                                FormLabel { html_for: "role-name", invalid: error.is_some(), {t!("roles-col-name")} }
                                TextInput { id: "role-name", value: name, disabled: read_only, invalid: error.is_some(), placeholder: t!("roles-name-ph") }
                                FormMessage { message: error.clone() }
                            }
                            FormItem {
                                FormLabel { html_for: "role-2fa", {t!("roles-require-2fa")} }
                                div { class: "flex h-9 items-center gap-3",
                                    Switch { id: "role-2fa", checked: require_2fa, disabled: read_only }
                                    FormDescription { {t!("roles-require-2fa-desc")} }
                                }
                            }
                        }
                        FormItem {
                            FormLabel { html_for: "role-desc", {t!("roles-col-description")} }
                            TextArea { id: "role-desc", value: description, placeholder: t!("roles-desc-ph"), class: "min-h-16 resize-none" }
                        }
                        div { class: "space-y-2",
                            div { class: "flex items-center justify-between",
                                span { class: "text-sm font-medium", {t!("roles-col-permissions")} }
                                span { class: "text-xs text-muted-foreground tabular-nums", {crate::ta!("roles-n-of-m", n: keys().len(), m: perms.len())} }
                            }
                            div { class: "divide-y rounded-md border",
                                for (module, items) in group_by_module(&perms) {
                                    ModuleRow { key: "{module}", module, items, keys, held: held.clone(), read_only, is_super }
                                }
                            }
                        }
                    }
                    div { class: "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
                        Button { variant: ButtonVariant::Outline, button_type: "button", onclick: move |_| open.set(false), {t!("action-close")} }
                        if !read_only {
                            Button { button_type: "button", disabled: busy(), onclick: move |_| {
                                let _ = dioxus::document::eval("document.getElementById('role-form')?.requestSubmit()");
                            }, {t!("action-save-changes")} }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn ModuleRow(
    module: String,
    items: Vec<PermissionInfo>,
    keys: Signal<BTreeSet<String>>,
    held: BTreeSet<String>,
    read_only: bool,
    is_super: bool,
) -> Element {
    let mut keys = keys;
    let can = move |k: &str| !read_only && (is_super || held.contains(k));
    let grantable: Vec<String> = items
        .iter()
        .filter(|p| can(&p.key))
        .map(|p| p.key.clone())
        .collect();
    let current = keys();
    let on = items.iter().filter(|p| current.contains(&p.key)).count();
    let all = on == items.len();
    rsx! {
        div { class: "grid gap-2 p-3 sm:grid-cols-[10rem_1fr] sm:items-start",
            label { class: "flex items-center gap-2 text-sm font-medium capitalize",
                CheckBox {
                    checked: all,
                    indeterminate: on > 0 && !all,
                    aria_label: crate::ta!("roles-all-in", module: label("module", &module)),
                    onchange: move |v: bool| {
                        let mut k = keys.write();
                        for key in &grantable { if v { k.insert(key.clone()); } else { k.remove(key); } }
                    },
                }
                "{label(\"module\", &module)}"
            }
            div { class: "flex flex-wrap gap-x-4 gap-y-2",
                for p in items {
                    PermissionBox { key: "{p.key}", enabled: can(&p.key), p: p.clone(), keys }
                }
            }
        }
    }
}

#[component]
fn PermissionBox(p: PermissionInfo, enabled: bool, keys: Signal<BTreeSet<String>>) -> Element {
    let mut keys = keys;
    let checked = keys().contains(&p.key);
    let key = p.key.clone();
    rsx! {
        label { class: if enabled { "flex items-center gap-2 text-sm" } else { "flex items-center gap-2 text-sm opacity-50" }, title: "{p.description}",
            CheckBox {
                checked,
                aria_label: p.key.clone(),
                onchange: move |v: bool| {
                    if !enabled { return; }
                    let mut k = keys.write();
                    if v { k.insert(key.clone()); } else { k.remove(&key); }
                },
            }
            "{label(\"action\", &p.action)}"
        }
    }
}
