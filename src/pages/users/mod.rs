//! `features/users/*` on real data (§9.2, §25): server-side list with URL-synced search,
//! status / role filters, sort and paging; Active · Deleted tabs; row and bulk actions;
//! add / edit / delete / restore / purge dialogs; CSV export.
mod actions;
mod detail;
mod dialogs;
mod table;

pub use detail::UserDetailPage;

use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::users::user_role_options;
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user,
};
use crate::components::permission_gate::PermissionGate;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::users::{RoleOption, UserRow};
use crate::ui::button::{Button, ButtonVariant};
use crate::util::query::Query;

/// `users-provider.tsx`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UserDialog {
    Add,
    Edit,
    Delete,
    Purge,
}

#[derive(Clone, Copy)]
pub struct UsersCtx {
    pub open: Signal<Option<UserDialog>>,
    pub current: Signal<Option<UserRow>>,
    /// Bump to reload the list after a change.
    pub refresh: Signal<u32>,
    pub roles: Signal<Vec<RoleOption>>,
}

impl UsersCtx {
    pub fn reload(mut self) {
        *self.refresh.write() += 1;
    }
}

#[component]
pub fn Users(query: Query) -> Element {
    let mut ctx = use_context_provider(|| UsersCtx {
        open: Signal::new(None),
        current: Signal::new(None),
        refresh: Signal::new(0),
        roles: Signal::new(Vec::new()),
    });
    // Roles for the filter menu and the form (small list, loaded once).
    let mut roles = ctx.roles;
    use_future(move || async move {
        if let Ok(r) = user_role_options().await {
            roles.set(r);
        }
    });
    let deleted = query.get("tab") == Some("deleted");
    let can_restore = use_current_user().is_some_and(|u| u.can("users.restore"));

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
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("users-title")} }
                    p { class: "text-muted-foreground", {t!("users-desc")} }
                }
                // `users-primary-buttons.tsx` (+ export; invite needs the email module, M13)
                div { class: "flex gap-2",
                    PermissionGate { perm: "users.export",
                        Button { variant: ButtonVariant::Outline, class: "space-x-1",
                            onclick: move |_| table::download_csv(&query_for_export(), None),
                            span { {t!("users-export")} }
                            Icon { icon: IconType::Download, class: "size-[18px]" }
                        }
                    }
                    PermissionGate { perm: "users.create",
                        Button { class: "space-x-1", onclick: move |_| ctx.open.set(Some(UserDialog::Add)),
                            span { {t!("users-add")} }
                            Icon { icon: IconType::UserPlus, class: "size-[18px]" }
                        }
                    }
                }
            }
            if can_restore {
                Tabs { deleted }
            }
            // A keyed one-item loop: switching tabs mounts a fresh table (state, selection, data).
            for d in [deleted] {
                table::UsersTable { key: "{d}", query: query.clone(), deleted: d }
            }
        }
        dialogs::UsersDialogs {}
    }
}

/// The current URL's filters, for the Export button.
fn query_for_export() -> Query {
    let path = web_location();
    Query::from(path.split_once('?').map(|(_, q)| q).unwrap_or(""))
}

fn web_location() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

/// Active · Deleted (§25.3), shadcn `Tabs` look; the tab lives in the URL (`?tab=deleted`).
#[component]
fn Tabs(deleted: bool) -> Element {
    let tab = |on: bool| {
        if on {
            "inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center rounded-md border border-transparent bg-background px-3 py-1 text-sm font-medium text-foreground shadow-sm dark:border-input dark:bg-input/30"
        } else {
            "inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center rounded-md border border-transparent px-3 py-1 text-sm font-medium text-muted-foreground hover:text-foreground"
        }
    };
    rsx! {
        div { role: "tablist", class: "inline-flex h-9 w-fit items-center justify-center rounded-lg bg-muted p-[3px] text-muted-foreground",
            Link { role: "tab", "aria-selected": "{!deleted}", class: tab(!deleted), to: crate::app::Route::Users { query: Query::default() }, {t!("users-tab-active")} }
            Link { role: "tab", "aria-selected": "{deleted}", class: tab(deleted), to: crate::app::Route::Users { query: Query::default().with("tab", Some("deleted")) }, {t!("users-tab-deleted")} }
        }
    }
}
