//! Backup page (§9.6.1, M11.8): list with live status (polls while a backup runs), create,
//! download, delete, CSV export. Schedule and retention live in System Setting →
//! Storage & Backup. Restore arrives with M12.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;
use uuid::Uuid;

use crate::api::backups::{create_backup, delete_backup, list_backups};
use crate::components::data_table::{TD, TH, TR};
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user,
};
use crate::components::permission_gate::PermissionGate;
use crate::components::skeletons::TableSkeleton;
use crate::components::ui_extra::dialog::ConfirmDialog;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, ITEM, Menu, MenuSeparator, MenuShortcut};
use crate::domain::backups::{BackupRow, human_size};
use crate::pages::dashboard::overview::when;
use crate::pages::errors::{ErrorKind, ErrorView};
use crate::ui::button::Button;
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

const BADGE: &str = "inline-flex w-fit shrink-0 items-center justify-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap";

#[component]
pub fn Backups() -> Element {
    if !use_current_user().is_some_and(|u| u.can("backup.view")) {
        return rsx! { ErrorView { kind: ErrorKind::Forbidden, minimal: true, class: "h-[calc(100svh-4rem)]" } };
    }
    let mut rows = use_signal(|| None::<Vec<BackupRow>>);
    let mut refresh = use_signal(|| 0u32);
    let mut busy = use_signal(|| false);
    // Load, then poll every 2 s while something is running (live status, M11.8).
    let _load = use_resource(move || async move {
        let _ = refresh();
        loop {
            match list_backups().await {
                Ok(r) => {
                    let running = r.iter().any(|b| b.status == "running");
                    rows.set(Some(r));
                    if !running {
                        break;
                    }
                }
                Err(e) => {
                    toast_error(&e);
                    break;
                }
            }
            crate::util::sleep_ms(2000).await;
        }
    });
    let create = move |_| {
        spawn(async move {
            busy.set(true);
            match create_backup().await {
                Ok(_) => {
                    toast(ToastType::Success, t!("backup-started"));
                    *refresh.write() += 1;
                }
                Err(e) => toast_error(&e),
            }
            busy.set(false);
        });
    };
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
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("backup-title")} }
                    p { class: "text-muted-foreground", {t!("backup-desc")} }
                }
                div { class: "flex gap-2",
                    PermissionGate { perm: "backup.export",
                        crate::components::export_menu::ExportMenu {
                            on_export: move |f: crate::components::export_menu::Format| crate::util::download(&format!("/api/export/backups.{}", f.ext()), &format!("backups.{}", f.ext())),
                        }
                    }
                    PermissionGate { perm: "backup.create",
                        Button { class: "space-x-1", disabled: busy(), onclick: create,
                            span { {t!("backup-create")} }
                            Icon { icon: IconType::DatabaseBackup, class: "size-[18px]" }
                        }
                    }
                }
            }
            match rows() {
                None => rsx! { TableSkeleton { cols: 5, rows: 4 } },
                Some(list) => rsx! {
                    div { class: "overflow-hidden rounded-md border",
                        div { class: "relative w-full overflow-x-auto",
                            table { class: "w-full caption-bottom text-sm",
                                thead { class: "[&_tr]:border-b",
                                    tr { class: TR,
                                        th { class: TH, {t!("backup-col-created")} }
                                        th { class: TH, {t!("backup-col-status")} }
                                        th { class: TH, {t!("backup-col-kind")} }
                                        th { class: TH, {t!("backup-col-size")} }
                                        th { class: TH, {t!("backup-col-checksum")} }
                                        th { class: TH, {t!("backup-col-by")} }
                                        th { class: TH }
                                    }
                                }
                                tbody { class: "[&_tr:last-child]:border-0",
                                    if list.is_empty() {
                                        tr { class: TR, td { class: tw_merge!(TD, "h-24 text-center"), colspan: "7", {t!("backup-none")} } }
                                    }
                                    for b in list {
                                        BackupLine { key: "{b.id}", b: b.clone(), on_change: move |_| *refresh.write() += 1 }
                                    }
                                }
                            }
                        }
                    }
                },
            }
            p { class: "text-sm text-muted-foreground", {t!("backup-note")} }
        }
    }
}

#[component]
fn BackupLine(b: BackupRow, on_change: EventHandler<()>) -> Element {
    let tz = crate::components::layout::use_tz();
    let me = use_current_user();
    let (can_download, can_delete) = (
        me.as_ref().is_some_and(|u| u.can("backup.download")),
        me.as_ref().is_some_and(|u| u.can("backup.delete")),
    );
    let mut confirm = use_signal(|| false);
    let id: Uuid = b.id;
    let done = b.status == "completed";
    let status = match b.status.as_str() {
        "completed" => {
            rsx! { span { class: "{BADGE} bg-teal-100/30 text-teal-900 dark:text-teal-200 border-teal-200", {t!("dash-backup-completed")} } }
        }
        "running" => {
            rsx! { span { class: "{BADGE} bg-sky-200/40 text-sky-900 dark:text-sky-100 border-sky-300",
            Icon { icon: IconType::LoaderCircle, class: "size-3 animate-spin" } {t!("dash-backup-running")} } }
        }
        _ => {
            rsx! { span { class: "{BADGE} bg-destructive/10 text-destructive border-destructive/10", title: b.error.clone().unwrap_or_default(), {t!("dash-backup-failed")} } }
        }
    };
    let filename = b.filename.clone();
    rsx! {
        tr { class: TR,
            td { class: TD,
                div { class: "font-medium tabular-nums", "{when(b.created_at, &tz)}" }
                div { class: "font-mono text-xs text-muted-foreground", "{b.filename}" }
            }
            td { class: TD, {status} }
            td { class: TD, {dioxus_i18n::te!(&format!("backup-kind-{}", b.kind.replace('_', "-"))).unwrap_or(b.kind.clone())} }
            td { class: tw_merge!(TD, "tabular-nums"), if done { "{human_size(b.size_bytes)}" } }
            td { class: tw_merge!(TD, "font-mono text-xs text-muted-foreground"), title: b.sha256.clone().unwrap_or_default(),
                {b.sha256.as_deref().map(|s| s.chars().take(12).collect::<String>()).unwrap_or_default()}
            }
            td { class: tw_merge!(TD, "text-muted-foreground"), {b.created_by.clone().unwrap_or_else(|| t!("dash-system"))} }
            td { class: TD,
                if (can_download && done) || (can_delete && b.status != "running") {
                    Menu {
                        align: Align::End,
                        class: "w-44",
                        trigger: rsx! {
                            button { r#type: "button", class: "flex h-8 w-8 items-center justify-center rounded-md p-0 hover:bg-accent in-data-[state=open]:bg-muted",
                                Icon { icon: IconType::Ellipsis, class: "h-4 w-4" }
                                span { class: "sr-only", {t!("dt-open-menu")} }
                            }
                        },
                        if can_download && done {
                            button { r#type: "button", class: ITEM, onclick: move |_| crate::util::download(&format!("/api/backups/{id}/download"), &filename),
                                {t!("backup-download")}
                                MenuShortcut { Icon { icon: IconType::Download, class: "size-4" } }
                            }
                        }
                        if can_delete && b.status != "running" {
                            MenuSeparator {}
                            button { r#type: "button", class: "{ITEM} text-red-500!", onclick: move |_| confirm.set(true),
                                {t!("dt-delete")}
                                MenuShortcut { Icon { icon: IconType::Trash2, class: "size-4" } }
                            }
                        }
                    }
                }
                ConfirmDialog {
                    open: confirm,
                    title: t!("backup-delete-title"),
                    title_destructive: true,
                    desc: t!("backup-delete-desc"),
                    confirm_text: t!("dt-delete"),
                    cancel_text: t!("action-cancel"),
                    destructive: true,
                    on_confirm: move |_| {
                        spawn(async move {
                            match delete_backup(id).await {
                                Ok(()) => { toast(ToastType::Success, t!("backup-deleted")); on_change.call(()); }
                                Err(e) => toast_error(&e),
                            }
                            confirm.set(false);
                        });
                    },
                }
            }
        }
    }
}
