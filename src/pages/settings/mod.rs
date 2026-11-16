//! `features/settings/*`: layout with sub-navigation + the five settings forms. The forms are
//! the reference's demo forms (submit shows the values in a toast); real profile/account
//! settings arrive with M9 (profile, sessions, 2FA, language).
mod account;
mod appearance;
mod display;
mod notifications;
mod profile;

pub use account::SettingsAccount;
pub use appearance::SettingsAppearance;
pub use display::SettingsDisplay;
pub use notifications::SettingsNotifications;
pub use profile::SettingsProfile;

use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::app::Route;
use crate::components::layout::{ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch};
use crate::components::ui_extra::form::SelectBox;
use crate::components::ui_extra::icon::{Icon, IconType};

const ITEMS: [(&str, &str, IconType); 5] = [
    ("nav-profile", "/settings", IconType::UserCog),
    ("nav-account", "/settings/account", IconType::Wrench),
    ("nav-appearance", "/settings/appearance", IconType::Palette),
    (
        "nav-notifications",
        "/settings/notifications",
        IconType::Bell,
    ),
    ("nav-display", "/settings/display", IconType::Monitor),
];

#[component]
pub fn SettingsLayout() -> Element {
    rsx! {
        Header {
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { fixed: true,
            div { class: "space-y-0.5",
                h1 { class: "text-2xl font-bold tracking-tight md:text-3xl", {t!("nav-settings")} }
                p { class: "text-muted-foreground", {t!("settings-desc")} }
            }
            div { role: "separator", class: "my-4 h-px w-full shrink-0 bg-border lg:my-6" }
            div { class: "flex flex-1 flex-col space-y-2 overflow-hidden md:space-y-2 lg:flex-row lg:space-y-0 lg:space-x-12",
                aside { class: "top-0 lg:sticky lg:w-1/5", SidebarNav {} }
                div { class: "flex w-full overflow-y-hidden p-1", Outlet::<Route> {} }
            }
        }
    }
}

/// Current settings path without a trailing slash (`/settings/` → `/settings`).
fn settings_path() -> String {
    let p = use_route::<Route>().to_string();
    let p = p.trim_end_matches('/');
    if p.is_empty() {
        "/settings".to_string()
    } else {
        p.to_string()
    }
}

/// `sidebar-nav.tsx`: a select on phones, a horizontal (md) / vertical (lg) list otherwise.
#[component]
fn SidebarNav() -> Element {
    let path = settings_path();
    let mut selected = use_signal(|| path.clone());
    // Follow navigation done by other means (links, back button).
    if *selected.peek() != path {
        selected.set(path.clone());
    }
    let options = ITEMS
        .iter()
        .map(|(k, href, _)| (href.to_string(), t!(*k)))
        .collect::<Vec<_>>();
    rsx! {
        div { class: "p-1 md:hidden",
            SelectBox {
                value: selected,
                options,
                placeholder: t!("theme-label"),
                class: "sm:w-48",
                trigger_class: "h-12",
                onchange: move |href: String| { navigator().push(href); },
            }
        }
        div { class: "hidden w-full min-w-40 overflow-x-auto bg-background px-1 py-2 md:block",
            nav { class: "flex space-x-2 py-1 lg:flex-col lg:space-y-1 lg:space-x-0",
                for (key, href, icon) in ITEMS {
                    Link {
                        to: href,
                        class: tw_merge!(
                            "inline-flex h-9 items-center justify-start gap-2 rounded-md px-4 py-2 text-sm font-medium whitespace-nowrap transition-all",
                            if path == href { "bg-muted hover:bg-accent" } else { "hover:bg-accent hover:underline" }
                        ),
                        span { class: "me-2", Icon { icon, class: "size-[18px]" } }
                        {t!(key)}
                    }
                }
            }
        }
    }
}

/// `content-section.tsx`.
#[component]
pub fn ContentSection(title: String, desc: String, children: Element) -> Element {
    rsx! {
        div { class: "flex flex-1 flex-col",
            div { class: "flex-none",
                h3 { class: "text-lg font-medium", "{title}" }
                p { class: "text-sm text-muted-foreground", "{desc}" }
            }
            div { role: "separator", class: "my-4 h-px w-full flex-none shrink-0 bg-border" }
            div { class: "faded-bottom h-full w-full overflow-y-auto scroll-smooth pe-4 pb-12",
                div { class: "-mx-1 px-1.5 lg:max-w-xl", {children} }
            }
        }
    }
}
