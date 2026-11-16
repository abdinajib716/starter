//! Loading placeholders built from `Skeleton`, shaped like the pages they stand in for, so the
//! layout does not jump when data arrives. Used as `SuspenseBoundary` fallbacks: the app shell
//! (before the signed-in user is known) and the content area (while a page loads its data).
use dioxus::prelude::*;

use crate::components::layout::{ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch};
use crate::components::ui_extra::sidebar::SidebarMenuSkeleton;
use crate::ui::skeleton::Skeleton;

/// Page title + description, and an optional block of primary buttons on the right.
#[component]
pub fn TitleSkeleton(#[props(default)] actions: bool) -> Element {
    rsx! {
        div { class: "flex flex-wrap items-end justify-between gap-2",
            div { class: "space-y-2",
                Skeleton { class: "h-7 w-40" }
                Skeleton { class: "h-4 w-72 max-w-full" }
            }
            if actions {
                div { class: "flex gap-2",
                    Skeleton { class: "h-9 w-24" }
                    Skeleton { class: "h-9 w-24" }
                }
            }
        }
    }
}

/// A data table: toolbar (search + two filters + view), header row, `rows` rows, pagination.
#[component]
pub fn TableSkeleton(
    #[props(default = 10)] rows: usize,
    #[props(default = 5)] cols: usize,
) -> Element {
    rsx! {
        div { class: "flex flex-1 flex-col gap-4",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    Skeleton { class: "h-8 w-37.5 lg:w-62.5" }
                    Skeleton { class: "h-8 w-20" }
                    Skeleton { class: "h-8 w-20" }
                }
                Skeleton { class: "hidden h-8 w-20 lg:block" }
            }
            div { class: "overflow-hidden rounded-md border",
                div { class: "flex h-10 items-center gap-4 border-b px-2",
                    Skeleton { class: "size-4" }
                    for c in 0..cols { Skeleton { key: "{c}", class: "h-4 flex-1" } }
                }
                for r in 0..rows {
                    div { key: "{r}", class: "flex h-12 items-center gap-4 border-b px-2 last:border-0",
                        Skeleton { class: "size-4" }
                        for c in 0..cols {
                            Skeleton { key: "{c}", class: "h-4 flex-1", style: "max-width: {60 + (r * 7 + c * 13) % 40}%" }
                        }
                    }
                }
            }
            div { class: "flex items-center justify-between px-2",
                Skeleton { class: "h-8 w-32" }
                div { class: "flex gap-2",
                    for i in 0..5 { Skeleton { key: "{i}", class: "size-8" } }
                }
            }
        }
    }
}

/// Dashboard-style cards: a row of stat cards and two panels.
#[component]
pub fn CardsSkeleton() -> Element {
    rsx! {
        div { class: "space-y-4",
            div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
                for i in 0..4 {
                    div { key: "{i}", class: "space-y-3 rounded-xl border p-6",
                        Skeleton { class: "h-4 w-24" }
                        Skeleton { class: "h-7 w-32" }
                        Skeleton { class: "h-3 w-28" }
                    }
                }
            }
            div { class: "grid grid-cols-1 gap-4 lg:grid-cols-7",
                div { class: "col-span-1 rounded-xl border p-6 lg:col-span-4", Skeleton { class: "h-80 w-full" } }
                div { class: "col-span-1 space-y-5 rounded-xl border p-6 lg:col-span-3",
                    for i in 0..5 {
                        div { key: "{i}", class: "flex items-center gap-4",
                            Skeleton { class: "size-9 rounded-full" }
                            div { class: "flex-1 space-y-2",
                                Skeleton { class: "h-4 w-32" }
                                Skeleton { class: "h-3 w-44" }
                            }
                            Skeleton { class: "h-4 w-16" }
                        }
                    }
                }
            }
        }
    }
}

/// A form: label + input pairs and a submit button.
#[component]
pub fn FormSkeleton(#[props(default = 4)] fields: usize) -> Element {
    rsx! {
        div { class: "space-y-8 lg:max-w-xl",
            for i in 0..fields {
                div { key: "{i}", class: "space-y-2",
                    Skeleton { class: "h-4 w-24" }
                    Skeleton { class: "h-9 w-full" }
                    Skeleton { class: "h-3 w-64 max-w-full" }
                }
            }
            Skeleton { class: "h-9 w-32" }
        }
    }
}

/// Content-area fallback while a page suspends: the real header (it needs no data) and a
/// generic title + table body, the most common page shape in the app.
#[component]
pub fn PageSkeleton() -> Element {
    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { class: "flex flex-1 flex-col gap-4 sm:gap-6",
            div { class: "contents", "aria-busy": "true",
                TitleSkeleton { actions: true }
                TableSkeleton {}
            }
        }
    }
}

/// Whole-app fallback (before the signed-in user is known): sidebar rows + header bar +
/// page body, laid out like the inset shell.
#[component]
pub fn ShellSkeleton() -> Element {
    rsx! {
        div { class: "flex min-h-svh w-full", "aria-busy": "true",
            div { class: "hidden w-(--sidebar-width) shrink-0 flex-col gap-2 p-2 md:flex", style: "--sidebar-width: 16rem;",
                div { class: "flex h-12 items-center gap-2 p-2",
                    Skeleton { class: "size-8 rounded-lg" }
                    div { class: "flex-1 space-y-1.5",
                        Skeleton { class: "h-3.5 w-24" }
                        Skeleton { class: "h-3 w-32" }
                    }
                }
                div { class: "space-y-1 p-2",
                    Skeleton { class: "mb-3 h-3 w-16" }
                    for i in 0..5 { SidebarMenuSkeleton { key: "{i}", show_icon: true, seed: i } }
                    Skeleton { class: "mt-5 mb-3 h-3 w-16" }
                    for i in 5..7 { SidebarMenuSkeleton { key: "{i}", show_icon: true, seed: i } }
                    Skeleton { class: "mt-5 mb-3 h-3 w-16" }
                    for i in 7..9 { SidebarMenuSkeleton { key: "{i}", show_icon: true, seed: i } }
                }
            }
            div { class: "m-2 flex flex-1 flex-col rounded-xl bg-background shadow-sm md:ms-0",
                div { class: "flex h-16 items-center gap-4 p-4",
                    Skeleton { class: "size-7" }
                    Skeleton { class: "h-8 w-40 sm:w-60" }
                    div { class: "ms-auto flex gap-3",
                        Skeleton { class: "size-8 rounded-full" }
                        Skeleton { class: "size-8 rounded-full" }
                        Skeleton { class: "size-8 rounded-full" }
                    }
                }
                div { class: "flex flex-1 flex-col gap-6 px-4 py-6",
                    TitleSkeleton {}
                    CardsSkeleton {}
                }
            }
        }
    }
}
