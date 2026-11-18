//! `nav-group.tsx`: links, collapsible sections, and dropdowns when the sidebar is icons-only.
use crate::t;
use dioxus::prelude::*;
use tw_merge::tw_merge;

use super::nav_config::{NavGroup, NavItem, NavLink, is_active};
use crate::app::Route;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{ITEM, Menu, MenuLabel, MenuSeparator, Side};
use crate::components::ui_extra::sidebar::{
    MenuButtonSize, SidebarCtx, SidebarGroup, SidebarGroupLabel, SidebarMenu, SidebarMenuButton,
    SidebarMenuItem, SidebarMenuLink, SidebarMenuSub, SidebarMenuSubItem, SidebarMenuSubLink,
    menu_button_class,
};
use crate::ui::badge::Badge;
use crate::ui::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};

#[component]
pub fn NavGroupView(group: NavGroup) -> Element {
    rsx! {
        SidebarGroup {
            SidebarGroupLabel { {t!(group.title)} }
            SidebarMenu {
                for item in group.items {
                    match item {
                        NavItem::Link(l) => rsx! { NavLinkItem { link: l } },
                        NavItem::Collapsible { title, icon, badge, items } => rsx! {
                            NavCollapsibleItem { title, icon, badge, items }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn NavBadge(text: &'static str) -> Element {
    rsx! { Badge { class: "rounded-full px-1 py-0 text-xs", "{text}" } }
}

fn current_path() -> String {
    use_route::<Route>().to_string()
}

#[component]
fn NavLinkItem(link: NavLink) -> Element {
    let path = current_path();
    let title = t!(link.title);
    rsx! {
        SidebarMenuItem {
            SidebarMenuLink {
                to: link.url,
                is_active: is_active(&path, Some(link.url), &[], false),
                tooltip: title.clone(),
                if let Some(icon) = link.icon { Icon { icon } }
                span { "{title}" }
                if let Some(b) = link.badge { NavBadge { text: b } }
            }
        }
    }
}

#[component]
fn NavCollapsibleItem(
    title: &'static str,
    icon: Option<IconType>,
    badge: Option<&'static str>,
    items: Vec<NavLink>,
) -> Element {
    let ctx = SidebarCtx::get();
    let path = current_path();
    let label = t!(title);

    // Icons-only desktop sidebar: a dropdown to the right instead of an inline list.
    if ctx.collapsed() && !(ctx.is_mobile)() {
        let active = is_active(&path, None, &items, false);
        return rsx! {
            SidebarMenuItem {
                Menu {
                    side: Side::Right,
                    trigger_class: "w-full",
                    trigger: rsx! {
                        SidebarMenuButton { tooltip: label.clone(), is_active: active,
                            if let Some(icon) = icon { Icon { icon } }
                            span { "{label}" }
                            if let Some(b) = badge { NavBadge { text: b } }
                            Icon { icon: IconType::ChevronRight, class: "ms-auto" }
                        }
                    },
                    MenuLabel { "{label}" if let Some(b) = badge { " ({b})" } }
                    MenuSeparator {}
                    for sub in items {
                        Link {
                            to: sub.url,
                            class: tw_merge!(ITEM, if is_active(&path, Some(sub.url), &[], false) { "bg-secondary" } else { "" }),
                            if let Some(icon) = sub.icon { Icon { icon } }
                            span { class: "max-w-52 text-wrap", {t!(sub.title)} }
                            if let Some(b) = sub.badge { span { class: "ms-auto text-xs", "{b}" } }
                        }
                    }
                }
            }
        };
    }

    rsx! {
        SidebarMenuItem {
            Collapsible { default_open: is_active(&path, None, &items, true), class: "group/collapsible",
                CollapsibleTrigger { class: menu_button_class(MenuButtonSize::Default, ""),
                    if let Some(icon) = icon { Icon { icon } }
                    span { "{label}" }
                    if let Some(b) = badge { NavBadge { text: b } }
                    Icon {
                        icon: IconType::ChevronRight,
                        class: "ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90 rtl:rotate-180",
                    }
                }
                CollapsibleContent {
                    SidebarMenuSub {
                        for sub in items {
                            SidebarMenuSubItem {
                                SidebarMenuSubLink { to: sub.url, is_active: is_active(&path, Some(sub.url), &[], false),
                                    if let Some(icon) = sub.icon { Icon { icon } }
                                    span { {t!(sub.title)} }
                                    if let Some(b) = sub.badge { NavBadge { text: b } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
