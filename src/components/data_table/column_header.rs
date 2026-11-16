use dioxus::prelude::*;
use dioxus_i18n::t;

use super::TableState;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{ITEM, Menu, MenuSeparator};

/// `data-table/column-header.tsx`: title with Asc / Desc / Hide menu, or plain text when the
/// column cannot sort.
#[component]
pub fn ColumnHeader(
    state: TableState,
    column: &'static str,
    title: String,
    #[props(default = true)] sortable: bool,
    #[props(default = true)] hideable: bool,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let mut sort = state.sort;
    let mut hidden = state.hidden;
    if !sortable {
        return rsx! { div { class: class.unwrap_or_default(), "{title}" } };
    }
    let icon = match sort() {
        Some((c, true)) if c == column => IconType::ArrowDown,
        Some((c, false)) if c == column => IconType::ArrowUp,
        _ => IconType::ChevronsUpDown,
    };
    rsx! {
        div { class: tw_merge::tw_merge!("flex items-center space-x-2", class.as_deref().unwrap_or("")),
            Menu {
                trigger: rsx! {
                    button { r#type: "button", class: "inline-flex h-8 items-center justify-center gap-1.5 rounded-md px-2.5 text-sm font-medium hover:bg-accent hover:text-accent-foreground in-data-[state=open]:bg-accent",
                        span { "{title}" }
                        Icon { icon, class: "ms-2 h-4 w-4" }
                    }
                },
                button { r#type: "button", class: ITEM, onclick: move |_| sort.set(Some((column, false))),
                    Icon { icon: IconType::ArrowUp, class: "size-3.5 text-muted-foreground/70" }
                    {t!("dt-asc")}
                }
                button { r#type: "button", class: ITEM, onclick: move |_| sort.set(Some((column, true))),
                    Icon { icon: IconType::ArrowDown, class: "size-3.5 text-muted-foreground/70" }
                    {t!("dt-desc")}
                }
                if hideable {
                    MenuSeparator {}
                    button { r#type: "button", class: ITEM, onclick: move |_| { hidden.write().insert(column); },
                        Icon { icon: IconType::EyeOff, class: "size-3.5 text-muted-foreground/70" }
                        {t!("dt-hide")}
                    }
                }
            }
        }
    }
}
