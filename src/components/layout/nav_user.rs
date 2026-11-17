use dioxus::prelude::*;
use dioxus_i18n::t;

use super::sign_out_dialog::SignOutDialog;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{
    Align, ITEM, Menu, MenuGroup, MenuItem, MenuLabel, MenuSeparator, Side,
};
use crate::components::ui_extra::sidebar::{
    MenuButtonSize, SidebarCtx, SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};
use crate::domain::UserInfo;
use crate::ui::avatar::{Avatar, AvatarFallback};

/// `nav-user.tsx`: the signed-in user at the bottom of the sidebar.
#[component]
pub fn NavUser(user: UserInfo) -> Element {
    let ctx = SidebarCtx::get();
    let mut sign_out = use_signal(|| false);
    let initials = user.initials();
    let card = rsx! {
        Avatar { class: "h-8 w-8 rounded-lg", AvatarFallback { class: "rounded-lg text-foreground", "{initials}" } }
        div { class: "grid flex-1 text-start text-sm leading-tight",
            span { class: "truncate font-semibold", "{user.full_name}" }
            span { class: "truncate text-xs", "{user.email}" }
        }
    };
    rsx! {
        SidebarMenu {
            SidebarMenuItem {
                Menu {
                    side: if (ctx.is_mobile)() { Side::Bottom } else { Side::Right },
                    align: Align::End,
                    match_trigger_width: true,
                    trigger_class: "w-full",
                    class: "min-w-56 rounded-lg",
                    trigger: rsx! {
                        SidebarMenuButton {
                            size: MenuButtonSize::Lg,
                            class: "in-data-[state=open]:bg-sidebar-accent in-data-[state=open]:text-sidebar-accent-foreground",
                            {card.clone()}
                            Icon { icon: IconType::ChevronsUpDown, class: "ms-auto size-4" }
                        }
                    },
                    MenuLabel { class: "p-0 font-normal",
                        div { class: "flex items-center gap-2 px-1 py-1.5 text-start text-sm", {card} }
                    }
                    MenuSeparator {}
                    MenuGroup {
                        Link { to: "/profile", class: ITEM, Icon { icon: IconType::BadgeCheck } {t!("nav-profile")} }
                    }
                    MenuSeparator {}
                    MenuItem { destructive: true, onclick: move |_| sign_out.set(true),
                        Icon { icon: IconType::LogOut }
                        {t!("menu-sign-out")}
                    }
                }
            }
        }
        SignOutDialog { open: sign_out }
    }
}
