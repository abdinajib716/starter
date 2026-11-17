//! Permission page (§9.4): the read-only catalog. Permissions are defined in code
//! (`server::rbac::PERMISSIONS`); this shows each module's keys, what they allow and which
//! roles hold them. Assign them on the Role page.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::api::roles::list_permissions;
use crate::components::data_table::{TD, TH, TR};
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user,
};
use crate::components::skeletons::TableSkeleton;
use crate::domain::roles::{PermissionInfo, group_by_module};
use crate::pages::errors::{ErrorKind, ErrorView};

const BADGE: &str = "inline-flex w-fit shrink-0 items-center justify-center rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap text-foreground";

pub fn module_label(code: &str) -> String {
    dioxus_i18n::te!(&format!("module-{}", code.replace('_', "-")))
        .unwrap_or_else(|_| code.replace('_', " "))
}

#[component]
pub fn Permissions() -> Element {
    if !use_current_user().is_some_and(|u| u.can("permissions.view")) {
        return rsx! { ErrorView { kind: ErrorKind::Forbidden, minimal: true, class: "h-[calc(100svh-4rem)]" } };
    }
    let mut data = use_signal(|| None::<Vec<PermissionInfo>>);
    let mut filter = use_signal(String::new);
    let _load = use_resource(move || async move {
        match list_permissions().await {
            Ok(c) => data.set(Some(c.permissions)),
            Err(e) => crate::util::toast_error(&e),
        }
    });
    let term = filter().to_lowercase();
    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { class: "flex flex-1 flex-col gap-4 sm:gap-6",
            div {
                h2 { class: "text-2xl font-bold tracking-tight", {t!("perms-title")} }
                p { class: "text-muted-foreground", {t!("perms-desc")} }
            }
            input {
                class: "flex h-8 w-full max-w-62.5 min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 dark:bg-input/30",
                placeholder: t!("perms-search"),
                value: filter(),
                oninput: move |e| filter.set(e.value()),
            }
            match data() {
                None => rsx! { TableSkeleton { cols: 3 } },
                Some(perms) => {
                    let shown: Vec<PermissionInfo> = perms
                        .into_iter()
                        .filter(|p| term.is_empty() || p.key.contains(&term) || p.description.to_lowercase().contains(&term)
                            || p.roles.iter().any(|r| r.to_lowercase().contains(&term)))
                        .collect();
                    rsx! {
                        div { class: "overflow-hidden rounded-md border",
                            div { class: "relative w-full overflow-x-auto",
                                table { class: "w-full caption-bottom text-sm",
                                    thead { class: "[&_tr]:border-b",
                                        tr { class: TR,
                                            th { class: TH, {t!("perms-col-permission")} }
                                            th { class: TH, {t!("perms-col-description")} }
                                            th { class: TH, {t!("perms-col-roles")} }
                                        }
                                    }
                                    tbody {
                                        if shown.is_empty() {
                                            tr { class: TR, td { class: tw_merge!(TD, "h-24 text-center"), colspan: "3", {t!("dt-no-results")} } }
                                        }
                                        for (module, items) in group_by_module(&shown) {
                                            tr { key: "m-{module}", class: "border-b bg-muted/50",
                                                td { class: "px-2 py-1.5 text-xs font-semibold tracking-wide text-muted-foreground uppercase", colspan: "3",
                                                    "{module_label(&module)} · {items.len()}"
                                                }
                                            }
                                            for p in items {
                                                tr { key: "{p.key}", class: TR,
                                                    td { class: TD, span { class: "font-mono text-xs", "{p.key}" } }
                                                    td { class: tw_merge!(TD, "whitespace-normal text-muted-foreground"), "{p.description}" }
                                                    td { class: tw_merge!(TD, "whitespace-normal"),
                                                        div { class: "flex flex-wrap gap-1",
                                                            for r in p.roles.iter() { span { class: BADGE, "{r}" } }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
