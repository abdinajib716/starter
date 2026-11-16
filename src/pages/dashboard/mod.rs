//! `features/dashboard/index.tsx`. UI only: figures are the reference's sample data;
//! M10 replaces them with `get_dashboard()` (real KPIs, §10).
mod analytics;
mod overview;
mod recent_sales;

use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, TopNav, TopNavLink,
};
use crate::components::ui_extra::tabs::TabsTriggerDisabled;
use crate::ui::button::Button;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};

/// Rust/UI → shadcn differences, fixed by class overrides (src/ui stays vendored):
/// its `TabsContent` adds `text-sm` and a top margin, its triggers dim inactive tabs.
const TAB_CONTENT: &str = "space-y-4 text-base group-data-[orientation=Horizontal]/tabs:mt-0";
const TAB_TRIGGER: &str = "px-2 py-1 text-foreground";
/// shadcn cards use `gap-6` (Rust/UI default `gap-4`).
pub(crate) const CARD: &str = "gap-6";

#[component]
pub fn Dashboard() -> Element {
    let top_nav = vec![
        TopNavLink {
            title: t!("dash-nav-overview"),
            href: "/",
            is_active: true,
            disabled: false,
        },
        TopNavLink {
            title: t!("dash-nav-customers"),
            href: "/",
            is_active: false,
            disabled: true,
        },
        TopNavLink {
            title: t!("dash-nav-products"),
            href: "/",
            is_active: false,
            disabled: true,
        },
        TopNavLink {
            title: t!("nav-settings"),
            href: "/",
            is_active: false,
            disabled: true,
        },
    ];
    rsx! {
        Header {
            TopNav { links: top_nav, class: "me-auto" }
            Search {}
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main {
            div { class: "mb-2 flex items-center justify-between space-y-2",
                h1 { class: "text-2xl font-bold tracking-tight", {t!("nav-dashboard")} }
                div { class: "flex items-center space-x-2", Button { {t!("dash-download")} } }
            }
            Tabs { default_value: "overview", class: "space-y-4",
                div { class: "w-full overflow-x-auto pb-2",
                    TabsList { class: "h-9 p-0.75",
                        TabsTrigger { value: "overview", class: TAB_TRIGGER, {t!("dash-tab-overview")} }
                        TabsTrigger { value: "analytics", class: TAB_TRIGGER, {t!("dash-tab-analytics")} }
                        TabsTriggerDisabled { {t!("dash-tab-reports")} }
                        TabsTriggerDisabled { {t!("dash-tab-notifications")} }
                    }
                }
                TabsContent { value: "overview", class: TAB_CONTENT,
                    div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
                        Kpi { title: t!("dash-revenue"), value: "$45,231.89", note: t!("dash-revenue-note"), icon: KpiIcon::Dollar }
                        Kpi { title: t!("dash-subscriptions"), value: "+2350", note: t!("dash-subscriptions-note"), icon: KpiIcon::Users }
                        Kpi { title: t!("dash-sales"), value: "+12,234", note: t!("dash-sales-note"), icon: KpiIcon::Card }
                        Kpi { title: t!("dash-active"), value: "+573", note: t!("dash-active-note"), icon: KpiIcon::Pulse }
                    }
                    div { class: "grid grid-cols-1 gap-4 lg:grid-cols-7",
                        Card { class: "col-span-1 gap-6 lg:col-span-4",
                            CardHeader { CardTitle { {t!("dash-overview")} } }
                            CardContent { class: "ps-2", overview::Overview {} }
                        }
                        Card { class: "col-span-1 gap-6 lg:col-span-3",
                            CardHeader {
                                CardTitle { {t!("dash-recent-sales")} }
                                CardDescription { {t!("dash-recent-sales-desc", count: 265)} }
                            }
                            CardContent { recent_sales::RecentSales {} }
                        }
                    }
                }
                TabsContent { value: "analytics", class: TAB_CONTENT, analytics::Analytics {} }
            }
        }
    }
}

/// The inline SVG icons of the reference's KPI cards.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum KpiIcon {
    Dollar,
    Users,
    Card,
    Pulse,
    Chart,
    User,
    Bounce,
    Clock,
}

#[component]
pub(crate) fn KpiSvg(icon: KpiIcon) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "stroke-width": "2",
            class: "h-4 w-4 text-muted-foreground",
            match icon {
                KpiIcon::Dollar => rsx! { path { d: "M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" } },
                KpiIcon::Users => rsx! {
                    path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                    circle { cx: "9", cy: "7", r: "4" }
                    path { d: "M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" }
                },
                KpiIcon::Card => rsx! {
                    rect { width: "20", height: "14", x: "2", y: "5", rx: "2" }
                    path { d: "M2 10h20" }
                },
                KpiIcon::Pulse => rsx! { path { d: "M22 12h-4l-3 9L9 3l-3 9H2" } },
                KpiIcon::Chart => rsx! { path { d: "M3 3v18h18" } path { d: "M7 15l4-4 4 4 4-6" } },
                KpiIcon::User => rsx! { circle { cx: "12", cy: "7", r: "4" } path { d: "M6 21v-2a6 6 0 0 1 12 0v2" } },
                KpiIcon::Bounce => rsx! { path { d: "M3 12h6l3 6 3-6h6" } },
                KpiIcon::Clock => rsx! { circle { cx: "12", cy: "12", r: "10" } path { d: "M12 6v6l4 2" } },
            }
        }
    }
}

/// One KPI card (title + icon, value, note).
#[component]
pub(crate) fn Kpi(title: String, value: String, note: String, icon: KpiIcon) -> Element {
    rsx! {
        Card { class: "gap-6",
            CardHeader { class: "flex flex-row items-center justify-between space-y-0 pb-2 sm:flex",
                CardTitle { class: "text-sm font-medium", "{title}" }
                KpiSvg { icon }
            }
            CardContent {
                div { class: "text-2xl font-bold", "{value}" }
                p { class: "text-xs text-muted-foreground", "{note}" }
            }
        }
    }
}
