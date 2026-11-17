use dioxus::prelude::*;
use dioxus_i18n::t;

use super::use_current_user;
use crate::app::Route;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::sidebar::{
    MenuButtonSize, SidebarCtx, SidebarMenu, SidebarMenuItem, menu_button_class,
};

/// `team-switcher.tsx` in single-tenant mode: our branding (`app.name` from System Settings)
/// linking home. Multi-tenant mode (M16) turns this into the organization switcher.
#[component]
pub fn TeamSwitcher() -> Element {
    let ctx = SidebarCtx::get();
    let name = use_current_user()
        .map(|u| u.modules.app_name)
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| t!("app-name"));
    let plan = t!("app-plan");
    let class = menu_button_class(MenuButtonSize::Lg, "");
    rsx! {
        SidebarMenu {
            SidebarMenuItem {
                Link { to: Route::Dashboard {}, class: "{class}", onclick: move |_| ctx.close_mobile(),
                    div { class: "flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground",
                        Icon { icon: IconType::Command, class: "size-4" }
                    }
                    div { class: "grid flex-1 text-start text-sm leading-tight",
                        span { class: "truncate font-semibold", "{name}" }
                        span { class: "truncate text-xs", "{plan}" }
                    }
                }
            }
        }
    }
}
