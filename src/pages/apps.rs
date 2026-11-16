//! `features/apps/*` (demo page, feature flag `demo.apps` in M7): app integrations with
//! filter / type / sort kept in the URL like the reference (`?filter=&type=&sort=`).
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::app::Route;
use crate::components::icons::brand::{Brand, BrandIcon};
use crate::components::layout::{ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch};
use crate::components::ui_extra::form::{SelectBox, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, Menu};
use crate::util::query::Query;

/// `data/apps.tsx`.
const APPS: [(&str, Brand, bool, &str); 15] = [
    (
        "Telegram",
        Brand::Telegram,
        false,
        "Connect with Telegram for real-time communication.",
    ),
    (
        "Notion",
        Brand::Notion,
        true,
        "Effortlessly sync Notion pages for seamless collaboration.",
    ),
    (
        "Figma",
        Brand::Figma,
        true,
        "View and collaborate on Figma designs in one place.",
    ),
    (
        "Trello",
        Brand::Trello,
        false,
        "Sync Trello cards for streamlined project management.",
    ),
    (
        "Slack",
        Brand::Slack,
        false,
        "Integrate Slack for efficient team communication",
    ),
    (
        "Zoom",
        Brand::Zoom,
        true,
        "Host Zoom meetings directly from the dashboard.",
    ),
    (
        "Stripe",
        Brand::Stripe,
        false,
        "Easily manage Stripe transactions and payments.",
    ),
    (
        "Gmail",
        Brand::Gmail,
        true,
        "Access and manage Gmail messages effortlessly.",
    ),
    (
        "Medium",
        Brand::Medium,
        false,
        "Explore and share Medium stories on your dashboard.",
    ),
    (
        "Skype",
        Brand::Skype,
        false,
        "Connect with Skype contacts seamlessly.",
    ),
    (
        "Docker",
        Brand::Docker,
        false,
        "Effortlessly manage Docker containers on your dashboard.",
    ),
    (
        "GitHub",
        Brand::Github,
        false,
        "Streamline code management with GitHub integration.",
    ),
    (
        "GitLab",
        Brand::Gitlab,
        false,
        "Efficiently manage code projects with GitLab integration.",
    ),
    (
        "Discord",
        Brand::Discord,
        false,
        "Connect with Discord for seamless team communication.",
    ),
    (
        "WhatsApp",
        Brand::Whatsapp,
        false,
        "Easily integrate WhatsApp for direct messaging.",
    ),
];

#[component]
pub fn Apps(query: Query) -> Element {
    let search = use_signal(|| query.get("filter").unwrap_or("").to_string());
    let app_type = use_signal(|| query.get("type").unwrap_or("all").to_string());
    let sort = query.get("sort").unwrap_or("asc").to_string();

    let mut list: Vec<_> = APPS.to_vec();
    list.sort_by(|a, b| {
        if sort == "desc" {
            b.0.cmp(a.0)
        } else {
            a.0.cmp(b.0)
        }
    });
    let term = search().to_lowercase();
    let list: Vec<_> = list
        .into_iter()
        .filter(|a| match app_type().as_str() {
            "connected" => a.2,
            "notConnected" => !a.2,
            _ => true,
        })
        .filter(|a| a.0.to_lowercase().contains(&term))
        .collect();

    // Mirror the state into the URL (replace, so typing does not fill the history).
    let q = query.clone();
    let nav = move |key: &'static str, value: Option<String>| {
        navigator().replace(Route::Apps {
            query: q.clone().with(key, value),
        });
    };
    let nav_filter = nav.clone();
    let nav_type = nav.clone();

    rsx! {
        Header {
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { fixed: true,
            div {
                h1 { class: "text-2xl font-bold tracking-tight", {t!("apps-title")} }
                p { class: "text-muted-foreground", {t!("apps-desc")} }
            }
            div { class: "my-4 flex items-end justify-between sm:my-0 sm:items-center",
                div { class: "flex flex-col gap-4 sm:my-4 sm:flex-row",
                    TextInput {
                        value: search,
                        placeholder: t!("apps-filter"),
                        class: "h-9 w-40 lg:w-62.5",
                        oninput: move |v: String| nav_filter("filter", Some(v)),
                    }
                    SelectBox {
                        value: app_type,
                        options: vec![
                            ("all".to_string(), t!("apps-all")),
                            ("connected".to_string(), t!("apps-connected")),
                            ("notConnected".to_string(), t!("apps-not-connected")),
                        ],
                        placeholder: t!("apps-all"),
                        trigger_class: "w-36",
                        onchange: move |v: String| nav_type("type", (v != "all").then_some(v)),
                    }
                }
                Menu {
                    align: Align::End,
                    trigger: rsx! {
                        button { r#type: "button", class: "flex h-9 w-16 items-center justify-between gap-2 rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs dark:bg-input/30", "aria-label": t!("apps-sort"),
                            Icon { icon: IconType::SlidersHorizontal, class: "size-[18px]" }
                            Icon { icon: IconType::ChevronDown, class: "size-4 opacity-50" }
                        }
                    },
                    for (value, icon, key) in [("asc", IconType::ArrowUpAZ, "apps-asc"), ("desc", IconType::ArrowDownAZ, "apps-desc-sort")] {
                        button {
                            r#type: "button",
                            class: "relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 ps-2 pe-8 text-sm outline-hidden select-none hover:bg-accent hover:text-accent-foreground",
                            onclick: { let nav = nav.clone(); move |_| nav("sort", Some(value.to_string())) },
                            div { class: "flex items-center gap-4", Icon { icon, class: "size-4" } span { {t!(key)} } }
                            if sort == value { span { class: "absolute end-2 flex size-3.5 items-center justify-center", Icon { icon: IconType::Check, class: "size-4" } } }
                        }
                    }
                }
            }
            div { role: "separator", class: "h-px w-full shrink-0 bg-border shadow-sm" }
            ul { class: "faded-bottom no-scrollbar grid gap-4 overflow-auto pt-4 pb-16 md:grid-cols-2 lg:grid-cols-3",
                for (name, brand, connected, desc) in list {
                    li { class: "rounded-lg border p-4 hover:shadow-md",
                        div { class: "mb-8 flex items-center justify-between",
                            div { class: "flex size-10 items-center justify-center rounded-lg bg-muted p-2", BrandIcon { brand } }
                            button {
                                r#type: "button",
                                class: tw_merge!(
                                    "inline-flex h-8 items-center justify-center gap-1.5 rounded-md border bg-background px-3 text-sm font-medium shadow-xs transition-all hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 dark:hover:bg-input/50",
                                    if connected { "border border-blue-300 bg-blue-50 hover:bg-blue-100 dark:border-blue-700 dark:bg-blue-950 dark:hover:bg-blue-900" } else { "" }
                                ),
                                if connected { {t!("apps-connected")} } else { {t!("apps-connect")} }
                            }
                        }
                        div {
                            h2 { class: "mb-1 font-semibold", "{name}" }
                            p { class: "line-clamp-2 text-gray-500", "{desc}" }
                        }
                    }
                }
            }
        }
    }
}
