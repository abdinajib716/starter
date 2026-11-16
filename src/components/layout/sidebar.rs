use dioxus::prelude::*;

use super::nav_config::visible_groups;
use super::nav_group::NavGroupView;
use super::nav_user::NavUser;
use super::team_switcher::TeamSwitcher;
use super::use_current_user;
use crate::components::ui_extra::sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarHeader, SidebarRail,
};
use crate::state::Ui;

/// `app-sidebar.tsx`: team switcher, permission-filtered groups, user menu, rail.
#[component]
pub fn AppSidebar() -> Element {
    let ui = Ui::get();
    rsx! {
        Sidebar {
            variant: (ui.variant)(),
            collapsible: (ui.collapsible)(),
            mobile: rsx! { SidebarBody { rail: false } },
            SidebarBody { rail: true }
        }
    }
}

#[component]
fn SidebarBody(rail: bool) -> Element {
    let Some(user) = use_current_user() else {
        return rsx! {};
    };
    let groups = visible_groups(&user);
    rsx! {
        SidebarHeader { TeamSwitcher {} }
        SidebarContent {
            for g in groups {
                NavGroupView { group: g }
            }
        }
        SidebarFooter { NavUser { user } }
        if rail { SidebarRail {} }
    }
}
