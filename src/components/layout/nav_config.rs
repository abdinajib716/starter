//! Menu tree (§8 sidebar, shadcn-admin `sidebar-data.ts` shape). Titles are i18n keys; every
//! item needs its permission and a group without visible items is hidden. The reference's
//! demo entries (Tasks, Apps, Chats, Pages, Help Center) were removed at the owner's request.
use crate::components::ui_extra::icon::IconType;
use crate::domain::UserInfo;

#[derive(Clone, PartialEq)]
pub struct NavLink {
    pub title: &'static str,
    pub url: &'static str,
    pub icon: Option<IconType>,
    pub badge: Option<&'static str>,
    pub permission: Option<&'static str>,
}

#[derive(Clone, PartialEq)]
pub enum NavItem {
    Link(NavLink),
    Collapsible {
        title: &'static str,
        icon: Option<IconType>,
        badge: Option<&'static str>,
        items: Vec<NavLink>,
    },
}

#[derive(Clone, PartialEq)]
pub struct NavGroup {
    pub title: &'static str,
    pub items: Vec<NavItem>,
}

const fn link(title: &'static str, url: &'static str, icon: Option<IconType>) -> NavLink {
    NavLink {
        title,
        url,
        icon,
        badge: None,
        permission: None,
    }
}

pub fn nav_groups() -> Vec<NavGroup> {
    use IconType as I;
    let link_p = |title, url, icon, permission| NavLink {
        permission: Some(permission),
        ..link(title, url, Some(icon))
    };
    vec![NavGroup {
        title: "nav-group-general",
        items: vec![
            NavItem::Link(link_p(
                "nav-dashboard",
                "/",
                I::LayoutDashboard,
                "dashboard.view",
            )),
            // §8: "Access Control" shows when the user may open any of its children.
            NavItem::Collapsible {
                title: "nav-access-control",
                icon: Some(I::ShieldCheck),
                badge: None,
                items: vec![
                    link_p(
                        "nav-system-settings",
                        "/system-settings",
                        I::Settings,
                        "settings.view",
                    ),
                    link_p("nav-users", "/users", I::Users, "users.view"),
                    link_p("nav-roles", "/roles", I::UserCog, "roles.view"),
                    link_p(
                        "nav-permissions",
                        "/permissions",
                        I::KeyRound,
                        "permissions.view",
                    ),
                    link_p("nav-audit", "/audit", I::ScrollText, "audit.view"),
                    link_p("nav-backup", "/backups", I::DatabaseBackup, "backup.view"),
                ],
            },
        ],
    }]
}

pub fn visible_groups(user: &UserInfo) -> Vec<NavGroup> {
    let allowed = |l: &NavLink| l.permission.is_none_or(|p| user.can(p));
    nav_groups()
        .into_iter()
        .filter_map(|g| {
            let items: Vec<NavItem> = g
                .items
                .into_iter()
                .filter_map(|item| match item {
                    NavItem::Link(l) => allowed(&l).then_some(NavItem::Link(l)),
                    NavItem::Collapsible {
                        title,
                        icon,
                        badge,
                        items,
                    } => {
                        let items: Vec<NavLink> =
                            items.into_iter().filter(|l| allowed(l)).collect();
                        (!items.is_empty()).then_some(NavItem::Collapsible {
                            title,
                            icon,
                            badge,
                            items,
                        })
                    }
                })
                .collect();
            (!items.is_empty()).then_some(NavGroup {
                title: g.title,
                items,
            })
        })
        .collect()
}

/// `checkIsActive` from nav-group.tsx. `main_nav` also matches the first path segment.
pub fn is_active(path: &str, url: Option<&str>, children: &[NavLink], main_nav: bool) -> bool {
    let bare = path.split('?').next().unwrap_or(path);
    let first = |p: &str| p.split('/').nth(1).unwrap_or("").to_string();
    url.is_some_and(|u| path == u || bare == u)
        || children.iter().any(|c| c.url == path || c.url == bare)
        || (main_nav
            && !first(path).is_empty()
            && children.iter().any(|c| first(c.url) == first(path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(perms: &[&str]) -> UserInfo {
        UserInfo {
            id: uuid::Uuid::nil(),
            email: "a@b.c".into(),
            full_name: "A".into(),
            role_name: "R".into(),
            is_super_admin: false,
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            must_change_pw: false,
            modules: Default::default(),
        }
    }

    #[test]
    fn access_control_follows_permissions() {
        let titles = |p: &[&str]| {
            visible_groups(&user(p))
                .into_iter()
                .flat_map(|g| g.items)
                .map(|i| match i {
                    NavItem::Link(l) => l.title,
                    NavItem::Collapsible { title, items, .. } => {
                        if items.len() == 1 {
                            items[0].title
                        } else {
                            title
                        }
                    }
                })
                .collect::<Vec<_>>()
        };
        assert!(titles(&[]).is_empty(), "nothing without permissions");
        assert_eq!(titles(&["dashboard.view"]), ["nav-dashboard"]);
        assert_eq!(
            titles(&["audit.view"]),
            ["nav-audit"],
            "group shows with one child"
        );
    }

    #[test]
    fn users_link_needs_permission() {
        let has = |u: &UserInfo| {
            visible_groups(u)
                .iter()
                .flat_map(|g| g.items.clone())
                .any(|i| matches!(i, NavItem::Collapsible { items, .. } if items.iter().any(|l| l.url == "/users")))
        };
        assert!(!has(&user(&[])));
        assert!(has(&user(&["users.view"])));
    }

    #[test]
    fn active_matching() {
        let kids = [
            link("x", "/settings", None),
            link("y", "/settings/account", None),
        ];
        assert!(is_active("/", Some("/"), &[], false));
        assert!(is_active("/settings/account", None, &kids, false));
        assert!(is_active("/settings/display", None, &kids, true));
        assert!(!is_active("/users", None, &kids, true));
    }
}
