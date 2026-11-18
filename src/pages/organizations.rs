//! `/organizations` (multi-tenant mode, §18, M16.3): tenants, their users, on/off, limits.
use dioxus::prelude::*;

use crate::api::orgs::{create_org, list_orgs, update_org};
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user, use_tz,
};
use crate::components::permission_gate::PermissionGate;
use crate::components::ui_extra::dialog::Modal;
use crate::components::ui_extra::form::{FormItem, FormLabel, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::orgs::NewOrg;
use crate::t;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

#[component]
pub fn Organizations() -> Element {
    let mut list = use_resource(list_orgs);
    let tz = use_tz();
    let can_manage = use_current_user().is_some_and(|u| u.can("organizations.manage"));
    let mut open = use_signal(|| false);
    let (name, slug, admin_name, admin_email, admin_pw, max) = (
        use_signal(String::new),
        use_signal(String::new),
        use_signal(String::new),
        use_signal(String::new),
        use_signal(String::new),
        use_signal(String::new),
    );
    let rows = list.read().clone();
    rsx! {
        Header { fixed: true, Search { class: "me-auto" } ThemeSwitch {} ConfigDrawer {} ProfileDropdown {} }
        Main { class: "flex flex-1 flex-col gap-4",
            div { class: "flex flex-wrap items-end justify-between gap-2",
                div {
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("orgs-title")} }
                    p { class: "text-muted-foreground", {t!("orgs-desc")} }
                }
                PermissionGate { perm: "organizations.manage",
                    Button { id: "org-new", onclick: move |_| open.set(true), Icon { icon: IconType::Plus } {t!("orgs-new")} }
                }
            }
            match rows {
                None => rsx! { crate::components::skeletons::TableSkeleton { cols: 5 } },
                Some(Err(e)) => rsx! { p { class: "text-destructive", {crate::i18n::error_text(&e)} } },
                Some(Ok(rows)) => rsx! {
                    div { class: "overflow-x-auto rounded-md border",
                        table { class: "w-full text-sm",
                            thead { tr { class: "border-b bg-muted/40 text-muted-foreground",
                                th { class: "px-3 py-2 text-start font-medium", {t!("orgs-name")} }
                                th { class: "px-3 py-2 text-start font-medium", {t!("orgs-slug")} }
                                th { class: "px-3 py-2 text-start font-medium", {t!("orgs-users")} }
                                th { class: "px-3 py-2 text-start font-medium", {t!("orgs-created")} }
                                th { class: "px-3 py-2" }
                            } }
                            tbody {
                                for o in rows {
                                    tr { key: "{o.id}", class: "border-b last:border-0",
                                        td { class: "px-3 py-2 font-medium",
                                            "{o.name} "
                                            if o.is_default { Badge { variant: BadgeVariant::Secondary, {t!("orgs-platform")} } }
                                            if !o.is_active { Badge { variant: BadgeVariant::Outline, {t!("orgs-off")} } }
                                        }
                                        td { class: "px-3 py-2 font-mono text-xs", "{o.slug}" }
                                        td { class: "px-3 py-2 tabular-nums",
                                            "{o.users}"
                                            if let Some(m) = o.max_users { span { class: "text-muted-foreground", " / {m}" } }
                                        }
                                        td { class: "px-3 py-2", {crate::util::time::date(o.created_at, &tz)} }
                                        td { class: "px-3 py-2 text-end",
                                            if can_manage && !o.is_default {
                                                Button { variant: ButtonVariant::Outline, class: "h-8",
                                                    onclick: { let o = o.clone(); move |_| { let o = o.clone(); async move {
                                                        match update_org(o.id, o.name.clone(), !o.is_active, o.max_users).await {
                                                            Ok(()) => { toast(ToastType::Success, t!("orgs-saved")); list.restart(); }
                                                            Err(e) => toast_error(&e),
                                                        }
                                                    } } },
                                                    if o.is_active { {t!("orgs-switch-off")} } else { {t!("orgs-switch-on")} }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
        Modal { open, title: t!("orgs-new"), description: t!("orgs-new-desc"),
            form { class: "grid gap-3",
                onsubmit: move |e: FormEvent| async move {
                    e.prevent_default();
                    let n = NewOrg {
                        name: name(), slug: slug(), admin_name: admin_name(), admin_email: admin_email(),
                        admin_password: admin_pw(), max_users: max().trim().parse().ok(),
                    };
                    match create_org(n).await {
                        Ok(_) => { toast(ToastType::Success, t!("orgs-created-ok")); open.set(false); list.restart(); }
                        Err(e) => toast_error(&e),
                    }
                },
                FormItem { FormLabel { html_for: "org-name", {t!("orgs-name")} } TextInput { id: "org-name", value: name } }
                FormItem { FormLabel { html_for: "org-slug", {t!("orgs-slug")} } TextInput { id: "org-slug", value: slug, placeholder: "acme" } }
                FormItem { FormLabel { html_for: "org-admin-name", {t!("orgs-admin-name")} } TextInput { id: "org-admin-name", value: admin_name } }
                FormItem { FormLabel { html_for: "org-admin-email", {t!("orgs-admin-email")} } TextInput { id: "org-admin-email", value: admin_email } }
                FormItem { FormLabel { html_for: "org-admin-pw", {t!("orgs-admin-password")} } TextInput { id: "org-admin-pw", value: admin_pw } }
                FormItem { FormLabel { html_for: "org-max", {t!("orgs-max-users")} } TextInput { id: "org-max", value: max, r#type: "number" } }
                Button { button_type: "submit", {t!("orgs-create")} }
            }
        }
    }
}
