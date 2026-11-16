use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{ITEM, Menu};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};

#[derive(Clone, PartialEq)]
pub struct TopNavLink {
    pub title: String,
    pub href: &'static str,
    pub is_active: bool,
    pub disabled: bool,
}

/// `top-nav.tsx`: inline links on large screens, a menu button below `lg`.
#[component]
pub fn TopNav(links: Vec<TopNavLink>, #[props(into, optional)] class: Option<String>) -> Element {
    let class = class.unwrap_or_default();
    let menu_links = links.clone();
    rsx! {
        Menu {
            trigger_class: tw_merge!("lg:hidden", class.clone()),
            trigger: rsx! {
                Button { size: ButtonSize::Icon, variant: ButtonVariant::Outline, class: "md:size-7",
                    Icon { icon: IconType::Menu }
                    span { class: "sr-only", "Toggle navigation menu" }
                }
            },
            for l in menu_links {
                Link {
                    to: l.href,
                    class: tw_merge!(ITEM, if l.is_active { "" } else { "text-muted-foreground" }, if l.disabled { "pointer-events-none opacity-50" } else { "" }),
                    "{l.title}"
                }
            }
        }
        nav { class: tw_merge!("hidden items-center space-x-4 lg:flex lg:space-x-4 xl:space-x-6", class),
            for l in links {
                Link {
                    to: l.href,
                    class: tw_merge!(
                        "text-sm font-medium transition-colors hover:text-primary",
                        if l.is_active { "" } else { "text-muted-foreground" },
                        if l.disabled { "pointer-events-none" } else { "" }
                    ),
                    "{l.title}"
                }
            }
        }
    }
}
