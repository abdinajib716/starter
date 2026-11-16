use dioxus::prelude::*;
use dioxus_i18n::t;

use super::{FacetedFilter, FilterDef, TableState, ViewOptions};
use crate::components::ui_extra::icon::{Icon, IconType};

/// `data-table/toolbar.tsx`: search, faceted filters, reset, view options.
#[component]
pub fn Toolbar(
    state: TableState,
    placeholder: String,
    filters: Vec<FilterDef>,
    columns: Vec<(&'static str, String)>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between",
            div { class: "flex flex-1 flex-col-reverse items-start gap-y-2 sm:flex-row sm:items-center sm:space-x-2",
                input {
                    class: "flex h-8 w-37.5 min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 md:text-sm lg:w-62.5 dark:bg-input/30",
                    placeholder,
                    value: (state.filter)(),
                    oninput: move |e| state.set_filter(e.value()),
                }
                div { class: "flex gap-x-2",
                    for def in filters { FacetedFilter { state, def } }
                }
                if state.is_filtered() {
                    button { r#type: "button", class: "inline-flex h-8 items-center justify-center rounded-md px-2 text-sm font-medium hover:bg-accent hover:text-accent-foreground lg:px-3", onclick: move |_| state.reset_filters(),
                        {t!("dt-reset")}
                        Icon { icon: IconType::X, class: "ms-2 h-4 w-4" }
                    }
                }
            }
            ViewOptions { state, columns }
        }
    }
}
