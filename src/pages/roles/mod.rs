//! Role page (§9.3): roles of the organization with user and permission counts, the role
//! editor (permission grid, M6.4), delete (refused while users hold the role), bulk delete
//! of unused roles and CSV export.
mod editor;

use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;
use uuid::Uuid;

use crate::api::roles::{delete_roles, list_roles};
use crate::components::data_table::{ICON_BTN_DESTRUCTIVE, TD, TH, TR};
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user,
};
use crate::components::permission_gate::PermissionGate;
use crate::components::skeletons::TableSkeleton;
use crate::components::ui_extra::dialog::ConfirmDialog;
use crate::components::ui_extra::form::CheckBox;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, ITEM, Menu, MenuSeparator, MenuShortcut};
use crate::domain::roles::RoleSummary;
use crate::pages::errors::{ErrorKind, ErrorView};
use crate::ui::button::Button;
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

const BADGE: &str = "inline-flex w-fit shrink-0 items-center justify-center rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap";

#[derive(Clone, Copy)]
pub struct RolesCtx {
    pub refresh: Signal<u32>,
    /// `Some(None)` = new role, `Some(Some(id))` = edit.
    pub editing: Signal<Option<Option<Uuid>>>,
}

impl RolesCtx {
    pub fn reload(mut self) {
        *self.refresh.write() += 1;
    }
}

#[component]
pub fn Roles() -> Element {
    let me = use_current_user();
    if !me.as_ref().is_some_and(|u| u.can("roles.view")) {
        return rsx! { ErrorView { kind: ErrorKind::Forbidden, minimal: true, class: "h-[calc(100svh-4rem)]" } };
    }
    let mut ctx = use_context_provider(|| RolesCtx {
        refresh: Signal::new(0),
        editing: Signal::new(None),
    });
    let mut data = use_signal(|| None::<Vec<RoleSummary>>);
    let refresh = ctx.refresh;
    let _load = use_resource(move || async move {
        let _ = refresh();
        match list_roles().await {
            Ok(r) => data.set(Some(r)),
            Err(e) => toast_error(&e),
        }
    });
    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { class: "flex flex-1 flex-col gap-4 sm:gap-6",
            div { class: "flex flex-wrap items-end justify-between gap-2",
                div {
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("roles-title")} }
                    p { class: "text-muted-foreground", {t!("roles-desc")} }
                }
                div { class: "flex gap-2",
                    PermissionGate { perm: "roles.export",
                        crate::components::export_menu::ExportMenu {
                            on_export: move |f: crate::components::export_menu::Format| crate::util::download(&format!("/api/export/roles.{}", f.ext()), &format!("roles.{}", f.ext())),
                        }
                    }
                    PermissionGate { perm: "roles.create",
                        Button { class: "space-x-1", onclick: move |_| ctx.editing.set(Some(None)),
                            span { {t!("roles-add")} }
                            Icon { icon: IconType::Plus, class: "size-[18px]" }
                        }
                    }
                }
            }
            match data() {
                None => rsx! { TableSkeleton { cols: 4, rows: 4 } },
                Some(roles) => rsx! { RoleTable { roles } },
            }
        }
        if let Some(id) = (ctx.editing)() {
            editor::RoleEditor { key: "{id:?}", id }
        }
    }
}

#[component]
fn RoleTable(roles: Vec<RoleSummary>) -> Element {
    let ctx = use_context::<RolesCtx>();
    let mut selected = use_signal(Vec::<Uuid>::new);
    let mut confirm = use_signal(|| false);
    let can_delete = use_current_user().is_some_and(|u| u.can("roles.delete"));
    let deletable: Vec<Uuid> = roles
        .iter()
        .filter(|r| !r.is_system && r.user_count == 0)
        .map(|r| r.id)
        .collect();
    let sel = selected();
    let all = !deletable.is_empty() && deletable.iter().all(|id| sel.contains(id));
    let count = sel.len();
    let run_delete = use_callback(move |ids: Vec<Uuid>| {
        spawn(async move {
            match delete_roles(ids).await {
                Ok(r) => {
                    if r.done > 0 {
                        toast(
                            ToastType::Success,
                            crate::ta!("roles-deleted", count: r.done as usize),
                        );
                    }
                    if let Some((_, code)) = r.skipped.first() {
                        toast(
                            ToastType::Warning,
                            crate::i18n::error_text(&crate::domain::errors::coded(409, code)),
                        );
                    }
                    selected.write().clear();
                    ctx.reload();
                }
                Err(e) => toast_error(&e),
            }
        });
    });
    rsx! {
        div { class: "flex flex-1 flex-col gap-4",
            div { class: "overflow-hidden rounded-md border",
                div { class: "relative w-full overflow-x-auto",
                    table { class: "w-full caption-bottom text-sm",
                        thead { class: "[&_tr]:border-b",
                            tr { class: TR,
                                if can_delete {
                                    th { class: tw_merge!(TH, "w-10"),
                                        CheckBox {
                                            checked: all,
                                            indeterminate: !sel.is_empty() && !all,
                                            aria_label: t!("dt-select-all"),
                                            onchange: move |on: bool| selected.set(if on { deletable.clone() } else { vec![] }),
                                        }
                                    }
                                }
                                th { class: TH, {t!("roles-col-name")} }
                                th { class: TH, {t!("roles-col-description")} }
                                th { class: TH, {t!("roles-col-users")} }
                                th { class: TH, {t!("roles-col-permissions")} }
                                th { class: TH, {t!("roles-col-2fa")} }
                                th { class: TH }
                            }
                        }
                        tbody { class: "[&_tr:last-child]:border-0",
                            for role in roles {
                                RoleLine { key: "{role.id}", role: role.clone(), can_delete, selected, run_delete }
                            }
                        }
                    }
                }
            }
            if count > 0 {
                div { class: "fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-xl",
                    div { class: "flex items-center gap-x-2 rounded-xl border bg-background/95 p-2 shadow-xl backdrop-blur-lg",
                        span { class: "inline-flex min-w-8 items-center justify-center rounded-lg bg-primary px-2 py-0.5 text-xs font-medium text-primary-foreground", "{count}" }
                        span { class: "text-sm", {t!("roles-selected")} }
                        div { class: "h-5 w-px bg-border" }
                        button { r#type: "button", class: ICON_BTN_DESTRUCTIVE, title: t!("roles-delete-selected"), "aria-label": t!("roles-delete-selected"),
                            onclick: move |_| confirm.set(true),
                            Icon { icon: IconType::Trash2 }
                        }
                    }
                }
            }
            ConfirmDialog {
                open: confirm,
                title: crate::ta!("roles-delete-n", count: count),
                title_destructive: true,
                desc: t!("roles-delete-desc"),
                confirm_text: t!("dt-delete"),
                cancel_text: t!("action-cancel"),
                destructive: true,
                on_confirm: move |_| {
                    confirm.set(false);
                    run_delete.call(selected());
                },
            }
        }
    }
}

#[component]
fn RoleLine(
    role: RoleSummary,
    can_delete: bool,
    selected: Signal<Vec<Uuid>>,
    run_delete: Callback<Vec<Uuid>>,
) -> Element {
    let mut ctx = use_context::<RolesCtx>();
    let mut selected = selected;
    let id = role.id;
    let deletable = !role.is_system && role.user_count == 0;
    let mut confirm = use_signal(|| false);
    let can_update = use_current_user().is_some_and(|u| u.can("roles.update"));
    rsx! {
        tr { class: TR, "data-state": if selected().contains(&id) { "selected" } else { "" },
            if can_delete {
                td { class: TD,
                    if deletable {
                        CheckBox {
                            checked: selected().contains(&id),
                            aria_label: t!("dt-select-row"),
                            onchange: move |on: bool| { let mut s = selected.write(); if on { s.push(id) } else { s.retain(|x| *x != id) } },
                        }
                    }
                }
            }
            td { class: TD,
                div { class: "flex items-center gap-2 font-medium",
                    Icon { icon: if role.is_system { IconType::Shield } else { IconType::UserCheck }, class: "size-4 text-muted-foreground" }
                    "{role.name}"
                    if role.is_system { span { class: "{BADGE} text-muted-foreground", {t!("roles-system")} } }
                }
            }
            td { class: tw_merge!(TD, "max-w-80 truncate text-muted-foreground"), "{role.description}" }
            td { class: tw_merge!(TD, "tabular-nums"), "{role.user_count}" }
            td { class: tw_merge!(TD, "tabular-nums"), "{role.permission_count}" }
            td { class: TD,
                if role.require_2fa { span { class: "{BADGE} bg-sky-200/40 text-sky-900 dark:text-sky-100 border-sky-300", {t!("roles-2fa-required")} } }
            }
            td { class: TD,
                Menu {
                    align: Align::End,
                    class: "w-44",
                    trigger: rsx! {
                        button { r#type: "button", class: "flex h-8 w-8 items-center justify-center rounded-md p-0 hover:bg-accent in-data-[state=open]:bg-muted",
                            Icon { icon: IconType::Ellipsis, class: "h-4 w-4" }
                            span { class: "sr-only", {t!("dt-open-menu")} }
                        }
                    },
                    button { r#type: "button", class: ITEM, onclick: move |_| ctx.editing.set(Some(Some(id))),
                        {t!(if can_update && !role.is_system { "dt-edit" } else { "roles-view" })}
                        MenuShortcut { Icon { icon: IconType::Pencil, class: "size-4" } }
                    }
                    if can_delete && deletable {
                        MenuSeparator {}
                        button { r#type: "button", class: "{ITEM} text-red-500!", onclick: move |_| confirm.set(true),
                            {t!("dt-delete")}
                            MenuShortcut { Icon { icon: IconType::Trash2, class: "size-4" } }
                        }
                    }
                }
                ConfirmDialog {
                    open: confirm,
                    title: crate::ta!("roles-delete-one", name: role.name.clone()),
                    title_destructive: true,
                    desc: t!("roles-delete-desc"),
                    confirm_text: t!("dt-delete"),
                    cancel_text: t!("action-cancel"),
                    destructive: true,
                    on_confirm: move |_| { confirm.set(false); run_delete.call(vec![id]); },
                }
            }
        }
    }
}
