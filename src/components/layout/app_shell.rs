//! `authenticated-layout.tsx`: session guard, search state, sidebar, inset content.
use dioxus::prelude::*;

use super::search::CommandMenu;
use super::sidebar::AppSidebar;
use super::skip_to_main::SkipToMain;
use crate::api::auth::current_user;
use crate::app::Route;
use crate::components::ui_extra::sidebar::{SidebarInset, SidebarProvider};
use crate::domain::UserInfo;
use crate::state::SearchOpen;

/// The signed-in user, shared with every page under the shell. Pages call `.restart()`
/// after changing something `current_user()` reports (e.g. `must_change_pw`).
pub type UserResource = Resource<Result<Option<UserInfo>, ServerFnError>>;

/// The current user inside the shell (always `Some` there; the shell redirects otherwise).
pub fn use_current_user() -> Option<UserInfo> {
    let user = use_context::<UserResource>();
    user.read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .cloned()
        .flatten()
}

#[component]
pub fn AppShell() -> Element {
    // Fetched during SSR and reused on hydration; the server guard already redirected
    // anonymous page loads, so `None` here means the session ended in this tab.
    let user: UserResource = use_server_future(current_user)?;
    use_context_provider(|| user);
    let search = use_context_provider(|| SearchOpen(Signal::new(false)));

    // Ctrl/Cmd+K toggles the command palette (search-provider.tsx).
    use_effect(move || {
        let mut open = search.0;
        spawn(async move {
            let mut ev = document::eval(
                "window.addEventListener('keydown', e => {
                   if (e.key === 'k' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); dioxus.send(1); }
                 });",
            );
            while ev.recv::<i32>().await.is_ok() {
                let v = open();
                open.set(!v);
            }
        });
    });

    let signed_in = user
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .is_some_and(|u| u.is_some());
    if !signed_in {
        return rsx! { GoToLogin {} };
    }

    rsx! {
        SidebarProvider {
            SkipToMain {}
            AppSidebar {}
            SidebarInset {
                // Container queries for <Main>; fixed layouts take the viewport height.
                class: "@container/content has-data-[layout=fixed]:h-svh peer-data-[variant=inset]:has-data-[layout=fixed]:h-[calc(100svh-(var(--spacing)*4))]",
                // Pages that wait for data suspend only the content area; the sidebar stays.
                SuspenseBoundary { fallback: |_| rsx! {}, Outlet::<Route> {} }
            }
        }
        CommandMenu {}
    }
}

/// Client-side guard: navigate after render, never during it.
#[component]
fn GoToLogin() -> Element {
    use_effect(|| {
        navigator().replace(Route::SignIn {
            redirect: String::new(),
        });
    });
    rsx! {}
}
