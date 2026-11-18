use crate::t;
use dioxus::prelude::*;

use super::sign_out_dialog::SignOutDialog;
use super::use_current_user;
use crate::components::ui_extra::menu::{
    Align, ITEM, Menu, MenuGroup, MenuItem, MenuLabel, MenuSeparator, MenuShortcut,
};
use crate::ui::avatar::{Avatar, AvatarFallback, AvatarImage};
use crate::ui::button::{Button, ButtonVariant};

/// `profile-dropdown.tsx`: avatar button in the header.
#[component]
pub fn ProfileDropdown() -> Element {
    let mut sign_out = use_signal(|| false);
    let Some(user) = use_current_user() else {
        return rsx! {};
    };
    let initials = user.initials();
    rsx! {
        super::notification_bell::NotificationBell {}
        Menu {
            align: Align::End,
            class: "w-56",
            trigger: rsx! {
                Button { variant: ButtonVariant::Ghost, class: "relative h-8 w-8 rounded-full p-0",
                    Avatar { class: "h-8 w-8",
                        if let Some(src) = user.avatar_url.clone() { AvatarImage { src } }
                        AvatarFallback { class: "text-foreground", "{initials}" }
                    }
                }
            },
            MenuLabel { class: "font-normal",
                div { class: "flex flex-col gap-1.5",
                    p { class: "text-sm leading-none font-medium", "{user.full_name}" }
                    p { class: "text-xs leading-none text-muted-foreground", "{user.email}" }
                }
            }
            MenuSeparator {}
            MenuGroup {
                Link { to: "/profile", class: ITEM, {t!("nav-profile")} }
            }
            MenuSeparator {}
            MenuItem { destructive: true, onclick: move |_| sign_out.set(true),
                {t!("menu-sign-out")}
                MenuShortcut { class: "text-current", "⇧⌘Q" }
            }
        }
        SignOutDialog { open: sign_out }
    }
}
