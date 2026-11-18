use crate::t;
use dioxus::prelude::*;

use super::TableState;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, Menu, MenuLabel, MenuSeparator};

/// `data-table/view-options.tsx`: show/hide columns. `columns` = (id, label) of hideable columns.
#[component]
pub fn ViewOptions(state: TableState, columns: Vec<(&'static str, String)>) -> Element {
    let mut hidden = state.hidden;
    rsx! {
        Menu {
            align: Align::End,
            close_on_click: false,
            class: "w-37.5",
            trigger_class: "ms-auto hidden lg:flex",
            trigger: rsx! {
                button { r#type: "button", class: "inline-flex h-8 items-center justify-center gap-1.5 rounded-md border bg-background px-3 text-sm font-medium shadow-xs transition-all hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30",
                    Icon { icon: IconType::Settings2, class: "size-4" }
                    {t!("dt-view")}
                }
            },
            MenuLabel { {t!("dt-toggle-columns")} }
            MenuSeparator {}
            for (id, label) in columns {
                button {
                    r#type: "button",
                    role: "menuitemcheckbox",
                    "aria-checked": "{!hidden().contains(id)}",
                    class: "relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 ps-8 pe-2 text-sm capitalize outline-hidden select-none hover:bg-accent hover:text-accent-foreground",
                    onclick: move |_| {
                        let mut h = hidden.write();
                        if !h.remove(id) { h.insert(id); }
                    },
                    span { class: "pointer-events-none absolute start-2 flex size-3.5 items-center justify-center",
                        if !hidden().contains(id) { Icon { icon: IconType::Check, class: "size-4" } }
                    }
                    "{label}"
                }
            }
        }
    }
}
