//! `features/users/*`: user list with URL-synced username / status / role filters and
//! pagination, row + bulk actions, add / edit / invite / delete dialogs. Demo data for now;
//! M5 wires it to the real users API.
mod actions;
mod data;
mod dialogs;
mod table;

use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::components::layout::{ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::ui::button::{Button, ButtonVariant};
use crate::util::query::Query;
use data::User;

/// `users-provider.tsx`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UserDialog {
    Add,
    Edit,
    Invite,
    Delete,
}

#[derive(Clone, Copy)]
pub struct UsersCtx {
    pub open: Signal<Option<UserDialog>>,
    pub current: Signal<Option<User>>,
}

#[component]
pub fn Users(query: Query) -> Element {
    let mut ctx = use_context_provider(|| UsersCtx {
        open: Signal::new(None),
        current: Signal::new(None),
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
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("users-title")} }
                    p { class: "text-muted-foreground", {t!("users-desc")} }
                }
                // `users-primary-buttons.tsx`
                div { class: "flex gap-2",
                    Button { variant: ButtonVariant::Outline, class: "space-x-1", onclick: move |_| ctx.open.set(Some(UserDialog::Invite)),
                        span { {t!("users-invite")} }
                        Icon { icon: IconType::MailPlus, class: "size-[18px]" }
                    }
                    Button { class: "space-x-1", onclick: move |_| ctx.open.set(Some(UserDialog::Add)),
                        span { {t!("users-add")} }
                        Icon { icon: IconType::UserPlus, class: "size-[18px]" }
                    }
                }
            }
            table::UsersTable { query }
        }
        dialogs::UsersDialogs {}
    }
}
