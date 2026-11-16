//! Menu tree (shadcn-admin `layout/data/sidebar-data.ts` + `types.ts`). Titles are i18n
//! keys. Items with `permission` are hidden from users without it (§8); Clerk entries
//! are removed (SPEC-02 §5). `demo` items become feature flags (`demo.*`, default off)
//! in M7; until flags exist they are shown so the port can be compared with the demo.
use crate::components::ui_extra::icon::IconType;
use crate::domain::UserInfo;

#[derive(Clone, PartialEq)]
pub struct NavLink {
    pub title: &'static str,
    pub url: &'static str,
    pub icon: Option<IconType>,
    pub badge: Option<&'static str>,
    pub permission: Option<&'static str>,
    pub demo: Option<&'static str>,
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
        demo: None,
    }
}

pub fn nav_groups() -> Vec<NavGroup> {
    use IconType as I;
    vec![
        NavGroup {
            title: "nav-group-general",
            items: vec![
                NavItem::Link(link("nav-dashboard", "/", Some(I::LayoutDashboard))),
                NavItem::Link(NavLink {
                    demo: Some("demo.tasks"),
                    ..link("nav-tasks", "/tasks", Some(I::ListTodo))
                }),
                NavItem::Link(NavLink {
                    demo: Some("demo.apps"),
                    ..link("nav-apps", "/apps", Some(I::Package))
                }),
                NavItem::Link(NavLink {
                    badge: Some("3"),
                    demo: Some("demo.chats"),
                    ..link("nav-chats", "/chats", Some(I::MessagesSquare))
                }),
                NavItem::Link(NavLink {
                    permission: Some("users.view"),
                    ..link("nav-users", "/users", Some(I::Users))
                }),
            ],
        },
        NavGroup {
            title: "nav-group-pages",
            items: vec![
                NavItem::Collapsible {
                    title: "nav-auth",
                    icon: Some(I::ShieldCheck),
                    badge: None,
                    items: vec![
                        link("nav-sign-in", "/sign-in", None),
                        link("nav-sign-in-2", "/sign-in-2", None),
                        link("nav-sign-up", "/sign-up", None),
                        link("nav-forgot-password", "/forgot-password", None),
                        link("nav-otp", "/otp", None),
                    ],
                },
                NavItem::Collapsible {
                    title: "nav-errors",
                    icon: Some(I::Bug),
                    badge: None,
                    items: vec![
                        link("nav-unauthorized", "/errors/unauthorized", Some(I::Lock)),
                        link("nav-forbidden", "/errors/forbidden", Some(I::UserX)),
                        link("nav-not-found", "/errors/not-found", Some(I::FileX)),
                        link(
                            "nav-internal-error",
                            "/errors/internal-server-error",
                            Some(I::ServerOff),
                        ),
                        link(
                            "nav-maintenance",
                            "/errors/maintenance-error",
                            Some(I::Construction),
                        ),
                    ],
                },
            ],
        },
        NavGroup {
            title: "nav-group-other",
            items: vec![
                NavItem::Collapsible {
                    title: "nav-settings",
                    icon: Some(I::Settings),
                    badge: None,
                    items: vec![
                        link("nav-profile", "/settings", Some(I::UserCog)),
                        link("nav-account", "/settings/account", Some(I::Wrench)),
                        link("nav-appearance", "/settings/appearance", Some(I::Palette)),
                        link(
                            "nav-notifications",
                            "/settings/notifications",
                            Some(I::Bell),
                        ),
                        link("nav-display", "/settings/display", Some(I::Monitor)),
                    ],
                },
                NavItem::Link(NavLink {
                    demo: Some("demo.help_center"),
                    ..link("nav-help-center", "/help-center", Some(I::CircleHelp))
                }),
            ],
        },
    ]
}

/// Permission filter (§8): hides links the user may not open, and empty collapsibles/groups.
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
        }
    }

    #[test]
    fn users_link_needs_permission() {
        let has = |u: &UserInfo| {
            visible_groups(u)
                .iter()
                .flat_map(|g| g.items.clone())
                .any(|i| matches!(i, NavItem::Link(l) if l.url == "/users"))
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
