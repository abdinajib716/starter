//! `features/tasks/*` (demo page, flag `demo.tasks` in M7): data table with URL-synced
//! search / status / priority filters and pagination, row + bulk actions, create / update
//! sheet, CSV import dialog and delete confirmations.
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
use data::Task;

/// `tasks-provider.tsx`: which dialog is open and for which row.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TaskDialog {
    Create,
    Update,
    Import,
    Delete,
}

#[derive(Clone, Copy)]
pub struct TasksCtx {
    pub open: Signal<Option<TaskDialog>>,
    pub current: Signal<Option<Task>>,
}

#[component]
pub fn Tasks(query: Query) -> Element {
    let mut ctx = use_context_provider(|| TasksCtx {
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
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("tasks-title")} }
                    p { class: "text-muted-foreground", {t!("tasks-desc")} }
                }
                // `tasks-primary-buttons.tsx`
                div { class: "flex gap-2",
                    Button { variant: ButtonVariant::Outline, class: "space-x-1", onclick: move |_| ctx.open.set(Some(TaskDialog::Import)),
                        span { {t!("tasks-import")} }
                        Icon { icon: IconType::Download, class: "size-[18px]" }
                    }
                    Button { class: "space-x-1", onclick: move |_| ctx.open.set(Some(TaskDialog::Create)),
                        span { {t!("tasks-create")} }
                        Icon { icon: IconType::Plus, class: "size-[18px]" }
                    }
                }
            }
            table::TasksTable { query }
        }
        dialogs::TasksDialogs {}
    }
}
