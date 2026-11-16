use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::{FilterDef, TableState};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::Menu;

const BADGE: &str = "inline-flex items-center justify-center rounded-sm border border-transparent bg-secondary px-1 py-0.5 text-xs font-normal text-secondary-foreground";

/// `data-table/faceted-filter.tsx`: dashed button with selected badges; popover with search,
/// multi-select options, counts and "Clear filters".
#[component]
pub fn FacetedFilter(state: TableState, def: FilterDef) -> Element {
    let open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let selected = state.column_filter(def.column);
    let q = query().to_lowercase();
    let shown: Vec<_> = def
        .options
        .iter()
        .filter(|o| o.label.to_lowercase().contains(&q))
        .cloned()
        .collect();
    let column = def.column;
    let toggle = {
        let selected = selected.clone();
        move |value: &'static str| {
            let mut next = selected.clone();
            if let Some(i) = next.iter().position(|v| v == value) {
                next.remove(i);
            } else {
                next.push(value.to_string());
            }
            state.set_column_filter(column, next);
        }
    };
    let chosen: Vec<_> = def
        .options
        .iter()
        .filter(|o| selected.iter().any(|s| s == o.value))
        .cloned()
        .collect();
    rsx! {
        Menu {
            open_signal: open,
            close_on_click: false,
            class: "w-50 p-0",
            trigger: rsx! {
                button { r#type: "button", class: "inline-flex h-8 items-center justify-center gap-1.5 rounded-md border border-dashed bg-background px-3 text-sm font-medium shadow-xs transition-all hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30",
                    Icon { icon: IconType::CirclePlus, class: "size-4" }
                    "{def.title}"
                    if !selected.is_empty() {
                        div { role: "separator", class: "mx-2 h-4 w-px bg-border" }
                        span { class: "{BADGE} lg:hidden", "{selected.len()}" }
                        div { class: "hidden space-x-1 lg:flex",
                            if selected.len() > 2 {
                                span { class: BADGE, {crate::ta!("dt-n-selected", count: selected.len())} }
                            } else {
                                for o in chosen { span { class: BADGE, "{o.label}" } }
                            }
                        }
                    }
                }
            },
            div { class: "flex h-9 items-center gap-2 border-b px-3",
                Icon { icon: IconType::Search, class: "size-4 shrink-0 opacity-50" }
                input { class: "flex h-10 w-full bg-transparent py-3 text-sm outline-hidden placeholder:text-muted-foreground", placeholder: def.title.clone(), value: query(), oninput: move |e| query.set(e.value()) }
            }
            div { class: "max-h-[300px] overflow-y-auto p-1",
                if shown.is_empty() { div { class: "py-6 text-center text-sm", {t!("search-empty")} } }
                for o in shown {
                    button {
                        r#type: "button",
                        class: "relative flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none hover:bg-accent hover:text-accent-foreground",
                        onclick: { let toggle = toggle.clone(); let v = o.value; move |_| toggle(v) },
                        div { class: tw_merge!(
                                "flex size-4 items-center justify-center rounded-sm border border-primary",
                                if selected.iter().any(|s| s == o.value) { "bg-primary text-primary-foreground" } else { "opacity-50 [&_svg]:invisible" }
                            ),
                            Icon { icon: IconType::Check, class: "h-4 w-4 text-background" }
                        }
                        if let Some(icon) = o.icon { Icon { icon, class: "size-4 text-muted-foreground" } }
                        span { "{o.label}" }
                        if let Some((_, n)) = def.counts.iter().find(|(v, _)| *v == o.value) {
                            if *n > 0 { span { class: "ms-auto flex h-4 w-4 items-center justify-center font-mono text-xs", "{n}" } }
                        }
                    }
                }
                if !selected.is_empty() {
                    div { class: "-mx-1 my-1 h-px bg-border" }
                    button {
                        r#type: "button",
                        class: "relative flex w-full cursor-default items-center justify-center rounded-sm px-2 py-1.5 text-center text-sm hover:bg-accent hover:text-accent-foreground",
                        onclick: move |_| state.set_column_filter(column, Vec::new()),
                        {t!("dt-clear-filters")}
                    }
                }
            }
        }
    }
}
