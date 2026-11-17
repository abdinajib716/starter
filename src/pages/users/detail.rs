//! User details (`/users/:id`): account facts, sessions and the latest audit entries by or
//! about the user, with Edit / Activate / Deactivate / Delete actions (same dialogs and rules
//! as the list).
use dioxus::prelude::*;
use dioxus_i18n::t;
use uuid::Uuid;

use super::table::{status_class, status_key};
use super::{UserDialog, UsersCtx};
use crate::api::users::{apply_user_action, get_user, user_role_options};
use crate::app::Route;
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user, use_tz,
};
use crate::components::skeletons::FormSkeleton;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::users::{BulkUserAction, UserDetail, UserStatus};
use crate::pages::dashboard::action_label;
use crate::pages::errors::{ErrorKind, ErrorView};
use crate::ui::avatar::{Avatar, AvatarFallback};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::time::{date, datetime};
use crate::util::{toast, toast_error};

const BADGE: &str = "inline-flex w-fit shrink-0 items-center justify-center rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap text-foreground";

#[component]
pub fn UserDetailPage(id: Uuid) -> Element {
    let me = use_current_user();
    if !me.as_ref().is_some_and(|u| u.can("users.view")) {
        return rsx! { ErrorView { kind: ErrorKind::Forbidden, minimal: true, class: "h-[calc(100svh-4rem)]" } };
    }
    let ctx = use_context_provider(|| UsersCtx {
        open: Signal::new(None),
        current: Signal::new(None),
        refresh: Signal::new(0),
        roles: Signal::new(Vec::new()),
    });
    let mut roles = ctx.roles;
    use_future(move || async move {
        if let Ok(r) = user_role_options().await {
            roles.set(r);
        }
    });
    let mut data = use_signal(|| None::<UserDetail>);
    let mut missing = use_signal(|| false);
    let refresh = ctx.refresh;
    let _load = use_resource(move || async move {
        let _ = refresh();
        match get_user(id).await {
            Ok(d) => data.set(Some(d)),
            Err(e) => {
                if crate::domain::errors::i18n_key(&e) == "err-not-found" {
                    missing.set(true);
                } else {
                    toast_error(&e);
                }
            }
        }
    });
    if missing() {
        return rsx! { ErrorView { kind: ErrorKind::NotFound, minimal: true, class: "h-[calc(100svh-4rem)]" } };
    }
    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { class: "flex flex-1 flex-col gap-4 sm:gap-6",
            Link { to: Route::Users { query: Default::default() }, class: "inline-flex w-fit items-center gap-1 text-sm text-muted-foreground hover:text-foreground",
                Icon { icon: IconType::ChevronLeft, class: "size-4 rtl:rotate-180" }
                {t!("users-back")}
            }
            match data() {
                None => rsx! { FormSkeleton { fields: 3 } },
                Some(d) => rsx! { Detail { d } },
            }
        }
        super::dialogs::UsersDialogs {}
    }
}

#[component]
fn Detail(d: UserDetail) -> Element {
    let mut ctx = use_context::<UsersCtx>();
    let tz = use_tz();
    let me = use_current_user();
    let can = |p: &str| me.as_ref().is_some_and(|u| u.can(p));
    let (can_update, can_delete) = (can("users.update"), can("users.delete"));
    let is_me = me.as_ref().map(|u| u.id) == Some(d.user.id);
    let u = d.user.clone();
    let deleted = u.status == UserStatus::Deleted;
    let initials: String = u
        .full_name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let id = u.id;
    let run = move |action: BulkUserAction, key: &'static str| {
        spawn(async move {
            match apply_user_action(action, id).await {
                Ok(()) => {
                    toast(ToastType::Success, t!(key));
                    ctx.reload();
                }
                Err(e) => toast_error(&e),
            }
        });
    };
    let edit_row = u.clone();
    let del_row = u.clone();
    let fact = |label: String, value: String| {
        rsx! {
            div { class: "grid grid-cols-3 gap-2 py-2 text-sm",
                dt { class: "text-muted-foreground", "{label}" }
                dd { class: "col-span-2 break-words", "{value}" }
            }
        }
    };
    let never = t!("users-never");
    rsx! {
        div { class: "flex flex-wrap items-center justify-between gap-4",
            div { class: "flex items-center gap-4",
                Avatar { class: "size-14", AvatarFallback { class: "text-lg text-foreground", "{initials}" } }
                div { class: "space-y-1",
                    h1 { class: "text-2xl font-bold tracking-tight", "{u.full_name}" }
                    div { class: "flex flex-wrap items-center gap-2 text-sm text-muted-foreground",
                        span { "{u.email}" }
                        span { class: "{BADGE} {status_class(u.status)}", {t!(status_key(u.status))} }
                    }
                }
            }
            if !deleted {
                div { class: "flex flex-wrap gap-2",
                    if can_update {
                        Button { variant: ButtonVariant::Outline, onclick: move |_| { ctx.current.set(Some(edit_row.clone())); ctx.open.set(Some(UserDialog::Edit)); },
                            Icon { icon: IconType::UserPen, class: "size-4" } {t!("dt-edit")}
                        }
                        if !is_me {
                            if u.status == UserStatus::Inactive {
                                Button { variant: ButtonVariant::Outline, onclick: move |_| run(BulkUserAction::Activate, "users-activated-one"),
                                    Icon { icon: IconType::UserCheck, class: "size-4" } {t!("users-activate")}
                                }
                            } else {
                                Button { variant: ButtonVariant::Outline, onclick: move |_| run(BulkUserAction::Deactivate, "users-deactivated-one"),
                                    Icon { icon: IconType::UserX, class: "size-4" } {t!("users-deactivate")}
                                }
                            }
                        }
                    }
                    if can_delete && !is_me {
                        Button { variant: ButtonVariant::Destructive, onclick: move |_| { ctx.current.set(Some(del_row.clone())); ctx.open.set(Some(UserDialog::Delete)); },
                            Icon { icon: IconType::Trash2, class: "size-4" } {t!("dt-delete")}
                        }
                    }
                }
            }
        }
        div { class: "grid gap-4 lg:grid-cols-5",
            Card { class: "lg:col-span-2",
                CardHeader { CardTitle { {t!("users-detail-account")} } }
                CardContent {
                    dl { class: "divide-y",
                        {fact(t!("users-col-role"), u.role_name.clone())}
                        {fact(t!("users-detail-created"), datetime(u.created_at, &tz))}
                        {fact(t!("users-detail-updated"), datetime(d.updated_at, &tz))}
                        {fact(t!("users-col-last-login"), u.last_login_at.map(|t| datetime(t, &tz)).unwrap_or_else(|| never.clone()))}
                        {fact(t!("users-detail-sessions"), d.active_sessions.to_string())}
                        {fact(t!("profile-timezone"), d.timezone.clone().unwrap_or_else(|| t!("profile-timezone-ph")))}
                        {fact(t!("users-detail-must-change"), t!(if u.must_change_pw { "users-yes" } else { "users-no" }))}
                        {fact(t!("users-detail-failed"), d.failed_attempts.to_string())}
                        if let Some(l) = d.locked_until { {fact(t!("users-detail-locked"), datetime(l, &tz))} }
                        if let Some(at) = u.deleted_at {
                            {fact(t!("users-col-deleted"), format!("{} · {}", date(at, &tz), u.deleted_by_email.clone().unwrap_or_default()))}
                        }
                    }
                }
            }
            Card { class: "lg:col-span-3",
                CardHeader {
                    CardTitle { {t!("dash-recent")} }
                    CardDescription { {t!("users-detail-activity-desc")} }
                }
                CardContent {
                    if d.activity.is_empty() {
                        p { class: "text-sm text-muted-foreground", {t!("dash-recent-empty")} }
                    }
                    ul { class: "space-y-4",
                        for a in d.activity.iter() {
                            li { key: "{a.id}", class: "flex items-start justify-between gap-4",
                                div { class: "min-w-0",
                                    p { class: if a.success { "text-sm font-medium" } else { "text-sm font-medium text-destructive" }, "{action_label(&a.action)}" }
                                    p { class: "truncate text-xs text-muted-foreground",
                                        {a.actor_email.clone().unwrap_or_else(|| t!("dash-system"))}
                                        if let Some(ip) = &a.ip { " · {ip}" }
                                    }
                                }
                                span { class: "shrink-0 text-xs text-muted-foreground tabular-nums", "{datetime(a.at, &tz)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
