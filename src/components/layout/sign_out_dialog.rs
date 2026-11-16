use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::auth::logout;
use crate::app::Route;
use crate::components::ui_extra::dialog::ConfirmDialog;

/// `sign-out-dialog.tsx`: confirm, end the session, go to the login page keeping the
/// current path as `next`.
#[component]
pub fn SignOutDialog(open: Signal<bool>) -> Element {
    let path = use_route::<Route>().to_string();
    rsx! {
        ConfirmDialog {
            open,
            title: t!("signout-title"),
            desc: t!("signout-desc"),
            confirm_text: t!("signout-confirm"),
            cancel_text: t!("action-cancel"),
            destructive: true,
            class: "sm:max-w-sm",
            on_confirm: move |_| {
                let next = path.clone();
                async move {
                    let _ = logout().await;
                    navigator().replace(Route::Login { next });
                }
            },
        }
    }
}
