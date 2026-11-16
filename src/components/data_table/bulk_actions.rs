use dioxus::prelude::*;
use dioxus_i18n::t;

use super::TableState;
use crate::components::ui_extra::icon::{Icon, IconType};

/// `data-table/bulk-actions.tsx`: floating toolbar while rows are selected; Escape clears the
/// selection. `count` = selected rows that pass the current filters.
#[component]
pub fn BulkActions(state: TableState, count: usize, entity: String, children: Element) -> Element {
    let mut selected = state.selected;
    if count == 0 {
        return rsx! {};
    }
    let plural = if count > 1 {
        format!("{entity}s")
    } else {
        entity.clone()
    };
    rsx! {
        div { "aria-live": "polite", "aria-atomic": "true", class: "sr-only", role: "status",
            {crate::ta!("dt-bulk-announce", count: count, entity: plural.clone())}
        }
        div {
            role: "toolbar",
            "aria-label": crate::ta!("dt-bulk-label", count: count, entity: plural.clone()),
            tabindex: "-1",
            class: "fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-xl transition-all delay-100 duration-300 ease-out hover:scale-105 focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:outline-none",
            onkeydown: move |e| if e.key() == Key::Escape { selected.write().clear() },
            div { class: "flex items-center gap-x-2 rounded-xl border bg-background/95 p-2 shadow-xl backdrop-blur-lg supports-backdrop-filter:bg-background/60",
                button {
                    r#type: "button",
                    class: "inline-flex size-6 items-center justify-center rounded-full border bg-background shadow-xs hover:bg-accent [&_svg]:size-4",
                    "aria-label": t!("dt-clear-selection"),
                    title: t!("dt-clear-selection-esc"),
                    onclick: move |_| selected.write().clear(),
                    Icon { icon: IconType::X }
                    span { class: "sr-only", {t!("dt-clear-selection")} }
                }
                div { role: "separator", "aria-hidden": "true", class: "h-5 w-px bg-border" }
                div { class: "flex items-center gap-x-1 text-sm",
                    span { class: "inline-flex min-w-8 items-center justify-center rounded-lg border border-transparent bg-primary px-2 py-0.5 text-xs font-medium text-primary-foreground", "{count}" }
                    " "
                    span { class: "hidden sm:inline", "{plural}" }
                    " "
                    {t!("dt-selected")}
                }
                div { role: "separator", "aria-hidden": "true", class: "h-5 w-px bg-border" }
                {children}
            }
        }
    }
}
