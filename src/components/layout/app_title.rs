//! `layout/app-title.tsx`: the reference's alternative sidebar header (app name + a sidebar
//! toggle) for apps without a team switcher. Like the reference, the sidebar uses
//! `TeamSwitcher`; swap it for `AppTitle` in `sidebar.rs` to use this one.
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::app::Route;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::sidebar::{
    MenuButtonSize, SidebarCtx, SidebarMenu, SidebarMenuItem, menu_button_class,
};

#[allow(dead_code)]
#[component]
pub fn AppTitle() -> Element {
    let ctx = SidebarCtx::get();
    let class = menu_button_class(
        MenuButtonSize::Lg,
        "gap-0 py-0 hover:bg-transparent active:bg-transparent",
    );
    rsx! {
        SidebarMenu {
            SidebarMenuItem {
                div { "data-sidebar": "menu-button", "data-size": "lg", class: "{class}",
                    Link {
                        to: Route::Dashboard {},
                        class: "grid flex-1 text-start text-sm leading-tight",
                        onclick: move |_| ctx.close_mobile(),
                        span { class: "truncate font-bold", {t!("app-name")} }
                        span { class: "truncate text-xs", {t!("app-plan")} }
                    }
                    button {
                        r#type: "button",
                        "data-sidebar": "trigger",
                        class: "inline-flex aspect-square size-8 items-center justify-center rounded-md hover:bg-accent hover:text-accent-foreground max-md:scale-125 [&_svg]:size-4",
                        onclick: move |_| ctx.toggle(),
                        Icon { icon: IconType::X, class: "md:hidden" }
                        Icon { icon: IconType::Menu, class: "max-md:hidden" }
                        span { class: "sr-only", {t!("sidebar-toggle")} }
                    }
                }
            }
        }
    }
}
