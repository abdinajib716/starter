use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Menu, MenuItem, MenuLabel, MenuShortcut, Side};
use crate::components::ui_extra::sidebar::{
    MenuButtonSize, SidebarCtx, SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};

/// `team-switcher.tsx`. Single-tenant mode: the one organization (our branding). Multi-tenant
/// mode (M16) lists the user's organizations here.
#[component]
pub fn TeamSwitcher() -> Element {
    let ctx = SidebarCtx::get();
    let name = t!("app-name");
    let plan = t!("app-plan");
    rsx! {
        SidebarMenu {
            SidebarMenuItem {
                Menu {
                    side: if (ctx.is_mobile)() { Side::Bottom } else { Side::Right },
                    match_trigger_width: true,
                    trigger_class: "w-full",
                    class: "min-w-56 rounded-lg",
                    trigger: rsx! {
                        SidebarMenuButton {
                            size: MenuButtonSize::Lg,
                            class: "in-data-[state=open]:bg-sidebar-accent in-data-[state=open]:text-sidebar-accent-foreground",
                            div { class: "flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground",
                                Icon { icon: IconType::Command, class: "size-4" }
                            }
                            div { class: "grid flex-1 text-start text-sm leading-tight",
                                span { class: "truncate font-semibold", "{name}" }
                                span { class: "truncate text-xs", "{plan}" }
                            }
                            Icon { icon: IconType::ChevronsUpDown, class: "ms-auto" }
                        }
                    },
                    MenuLabel { class: "text-xs text-muted-foreground", {t!("teams-label")} }
                    MenuItem { class: "gap-2 p-2",
                        div { class: "flex size-6 items-center justify-center rounded-sm border",
                            Icon { icon: IconType::Command, class: "size-4 shrink-0" }
                        }
                        "{name}"
                        MenuShortcut { "⌘1" }
                    }
                }
            }
        }
    }
}
