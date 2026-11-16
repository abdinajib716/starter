use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::{PAGE_SIZES, TableState};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::Menu;

/// `lib/utils.ts` `getPageNumbers`: at most 5 buttons; `None` = "...".
pub fn page_numbers(current: usize, total: usize) -> Vec<Option<usize>> {
    if total <= 5 {
        return (1..=total).map(Some).collect();
    }
    let mut out = vec![Some(1)];
    if current <= 3 {
        out.extend((2..=4).map(Some));
        out.extend([None, Some(total)]);
    } else if current >= total - 2 {
        out.push(None);
        out.extend((total - 3..=total).map(Some));
    } else {
        out.push(None);
        out.extend((current - 1..=current + 1).map(Some));
        out.extend([None, Some(total)]);
    }
    out
}

const OUTLINE: &str = "inline-flex items-center justify-center rounded-md border bg-background text-sm font-medium shadow-xs transition-all hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50 dark:border-input dark:bg-input/30 dark:hover:bg-input/50";
const ACTIVE: &str = "inline-flex items-center justify-center rounded-md bg-primary text-sm font-medium text-primary-foreground shadow-xs hover:bg-primary/90";

/// `data-table/pagination.tsx`.
#[component]
pub fn Pagination(
    state: TableState,
    page_count: usize,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let mut page = state.page;
    let mut page_size = state.page_size;
    let current = page() + 1;
    let total = page_count.max(1);
    let can_prev = current > 1;
    let can_next = current < total;
    let label = crate::ta!("dt-page-of", page: current, total: total);
    rsx! {
        div {
            class: tw_merge!("flex items-center justify-between overflow-clip px-2 @max-2xl/content:flex-col-reverse @max-2xl/content:gap-4", class.as_deref().unwrap_or("")),
            style: "overflow-clip-margin: 1px",
            div { class: "flex w-full items-center justify-between",
                div { class: "flex w-25 items-center justify-center text-sm font-medium @2xl/content:hidden", "{label}" }
                div { class: "flex items-center gap-2 @max-2xl/content:flex-row-reverse",
                    Menu {
                        class: "min-w-0",
                        trigger: rsx! {
                            button { r#type: "button", class: "flex h-8 w-17.5 items-center justify-between gap-2 rounded-md border border-input bg-transparent px-3 text-sm shadow-xs dark:bg-input/30",
                                "{page_size()}"
                                Icon { icon: IconType::ChevronDown, class: "size-4 opacity-50" }
                            }
                        },
                        for size in PAGE_SIZES {
                            button {
                                r#type: "button",
                                class: "relative flex w-full cursor-default items-center rounded-sm py-1.5 ps-2 pe-8 text-sm hover:bg-accent hover:text-accent-foreground",
                                onclick: move |_| { page_size.set(size); page.set(0); },
                                "{size}"
                                if page_size() == size { span { class: "absolute end-2 flex size-3.5 items-center justify-center", Icon { icon: IconType::Check, class: "size-4" } } }
                            }
                        }
                    }
                    p { class: "hidden text-sm font-medium sm:block", {t!("dt-rows-per-page")} }
                }
            }
            div { class: "flex items-center sm:space-x-6 lg:space-x-8",
                div { class: "flex w-25 items-center justify-center text-sm font-medium @max-3xl/content:hidden", "{label}" }
                div { class: "flex items-center space-x-2",
                    button { r#type: "button", class: "{OUTLINE} size-8 p-0 @max-md/content:hidden", disabled: !can_prev, onclick: move |_| page.set(0),
                        span { class: "sr-only", {t!("dt-first")} }
                        Icon { icon: IconType::ChevronsLeft, class: "h-4 w-4" }
                    }
                    button { r#type: "button", class: "{OUTLINE} size-8 p-0", disabled: !can_prev, onclick: move |_| page.set(current.saturating_sub(2)),
                        span { class: "sr-only", {t!("dt-previous")} }
                        Icon { icon: IconType::ChevronLeft, class: "h-4 w-4" }
                    }
                    for n in page_numbers(current, total) {
                        div { class: "flex items-center",
                            match n {
                                None => rsx! { span { class: "px-1 text-sm text-muted-foreground", "..." } },
                                Some(n) => rsx! {
                                    button {
                                        r#type: "button",
                                        class: tw_merge!(if n == current { ACTIVE } else { OUTLINE }, "h-8 min-w-8 px-2"),
                                        onclick: move |_| page.set(n - 1),
                                        span { class: "sr-only", {crate::ta!("dt-go-page", page: n)} }
                                        "{n}"
                                    }
                                },
                            }
                        }
                    }
                    button { r#type: "button", class: "{OUTLINE} size-8 p-0", disabled: !can_next, onclick: move |_| page.set(current),
                        span { class: "sr-only", {t!("dt-next")} }
                        Icon { icon: IconType::ChevronRight, class: "h-4 w-4" }
                    }
                    button { r#type: "button", class: "{OUTLINE} size-8 p-0 @max-md/content:hidden", disabled: !can_next, onclick: move |_| page.set(total - 1),
                        span { class: "sr-only", {t!("dt-last")} }
                        Icon { icon: IconType::ChevronsRight, class: "h-4 w-4" }
                    }
                }
            }
        }
    }
}
