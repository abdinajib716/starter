//! Confirm dialog (shadcn `alert-dialog.tsx` + shadcn-admin `confirm-dialog.tsx`).
use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::ui::button::{Button, ButtonVariant};

#[component]
pub fn ConfirmDialog(
    open: Signal<bool>,
    title: String,
    desc: String,
    confirm_text: String,
    cancel_text: String,
    #[props(default)] destructive: bool,
    on_confirm: EventHandler<()>,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let mut open = open;
    if !open() {
        return rsx! {};
    }
    let content = tw_merge!(
        "fixed top-[50%] left-[50%] z-[81] grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border bg-background p-6 shadow-lg duration-200 animate-in fade-in-0 zoom-in-95 sm:max-w-lg",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div { class: "fixed inset-0 z-[80] bg-black/50 animate-in fade-in-0" }
        div {
            role: "alertdialog",
            "aria-modal": "true",
            tabindex: "-1",
            class: "{content}",
            onmounted: move |e| async move { let _ = e.data().set_focus(true).await; },
            onkeydown: move |e| if e.key() == Key::Escape { open.set(false) },
            div { class: "flex flex-col gap-2 text-start",
                h2 { class: "text-lg font-semibold", "{title}" }
                div { class: "text-sm text-muted-foreground", "{desc}" }
            }
            div { class: "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
                Button { variant: ButtonVariant::Outline, onclick: move |_| open.set(false), "{cancel_text}" }
                Button {
                    id: "confirm-dialog-confirm",
                    variant: if destructive { ButtonVariant::Destructive } else { ButtonVariant::Default },
                    onclick: move |_| on_confirm.call(()),
                    "{confirm_text}"
                }
            }
        }
    }
}
