use dioxus::prelude::*;
use dioxus_i18n::t;

use super::LocaleSwitch;
use crate::api::auth::{current_user, logout};
use crate::app::Route;
use crate::domain::UserInfo;
use crate::ui::avatar::{Avatar, AvatarFallback, AvatarSize};
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::theme_toggle::ThemeToggle;

/// The signed-in user, shared with every page under the layout. Pages call `.restart()`
/// after changing something `current_user()` reports (e.g. `must_change_pw`).
pub type UserResource = Resource<Result<Option<UserInfo>, ServerFnError>>;

/// Layout for every page that needs a session (§12).
#[component]
pub fn AppLayout() -> Element {
    // Fetched during SSR and reused on hydration; the server guard already redirected
    // anonymous page loads, so `None` here means the session ended in this tab.
    let user: UserResource = use_server_future(current_user)?;
    use_context_provider(|| user);

    let current = user
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .cloned()
        .flatten();
    let Some(me) = current else {
        return rsx! { GoToLogin {} };
    };
    let initials = me.initials();

    rsx! {
        div { class: "min-h-full bg-background text-foreground",
            header { class: "flex items-center gap-3 border-b px-4 py-2",
                Link { to: Route::Home {}, class: "font-semibold", {t!("app-name")} }
                div { class: "flex-1" }
                LocaleSwitch {}
                ThemeToggle {}
                Link { to: Route::Profile {}, class: "flex items-center gap-2",
                    Avatar { size: AvatarSize::Sm, AvatarFallback { "{initials}" } }
                    span { class: "hidden text-sm sm:inline", "{me.full_name}" }
                    Badge { variant: BadgeVariant::Secondary, "{me.role_name}" }
                }
                Button {
                    id: "logout",
                    size: ButtonSize::Sm,
                    variant: ButtonVariant::Ghost,
                    onclick: move |_| async move {
                        // Even if the call fails, leave the signed-in area.
                        let _ = logout().await;
                        navigator().replace(Route::Login { next: String::new() });
                    },
                    {t!("nav-logout")}
                }
            }
            main { class: "p-4 sm:p-8", Outlet::<Route> {} }
        }
    }
}

/// Client-side guard: navigate after render, never during it.
#[component]
fn GoToLogin() -> Element {
    use_effect(|| {
        navigator().replace(Route::Login {
            next: String::new(),
        });
    });
    rsx! {}
}
