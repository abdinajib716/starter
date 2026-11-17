//! Confirm dialog (shadcn `alert-dialog.tsx` + shadcn-admin `confirm-dialog.tsx`).
use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::ui::button::{Button, ButtonVariant};

/// General dialog (`dialog.tsx`): overlay, centred panel, title/description header, close ×.
/// Click outside or Escape closes it.
#[component]
pub fn Modal(
    open: Signal<bool>,
    title: String,
    #[props(into, optional)] description: Option<String>,
    #[props(into, optional)] class: Option<String>,
    /// Called whenever the dialog closes (reset form state here).
    #[props(optional)]
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let mut open = open;
    if !open() {
        return rsx! {};
    }
    let mut close = move || {
        open.set(false);
        if let Some(h) = &on_close {
            h.call(())
        }
    };
    let content = tw_merge!(
        "fixed top-[50%] left-[50%] z-[81] grid max-h-[calc(100dvh-2rem)] w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 overflow-y-auto rounded-lg border bg-background p-6 shadow-lg duration-200 animate-in fade-in-0 zoom-in-95 sm:max-w-lg",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div { class: "fixed inset-0 z-[80] bg-black/50 animate-in fade-in-0", onclick: move |_| close() }
        div {
            role: "dialog",
            "aria-modal": "true",
            tabindex: "-1",
            class: "{content}",
            onmounted: move |e| async move { let _ = e.data().set_focus(true).await; },
            onkeydown: move |e| if e.key() == Key::Escape { close() },
            div { class: "flex flex-col gap-2 text-start",
                h2 { class: "text-lg leading-none font-semibold", "{title}" }
                if let Some(d) = description { p { class: "text-sm text-muted-foreground", "{d}" } }
            }
            {children}
            button {
                r#type: "button",
                class: "absolute end-4 top-4 rounded-xs opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:outline-hidden [&_svg]:size-4",
                onclick: move |_| close(),
                super::icon::Icon { icon: super::icon::IconType::X }
                span { class: "sr-only", "Close" }
            }
        }
    }
}

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
    /// Rich body (replaces `desc`), e.g. the multi-delete "type DELETE" form.
    #[props(optional)]
    body: Option<Element>,
    /// Destructive title with a warning triangle (`text-destructive` + `AlertTriangle`).
    #[props(default)]
    title_destructive: bool,
    #[props(default)] disabled: bool,
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
                h2 { class: "text-lg font-semibold",
                    if title_destructive {
                        span { class: "text-destructive",
                            super::icon::Icon { icon: super::icon::IconType::TriangleAlert, class: "me-1 inline-block size-[18px] stroke-destructive" }
                            " {title}"
                        }
                    } else { "{title}" }
                }
                div { class: "text-sm text-muted-foreground",
                    if let Some(body) = body { {body} } else { "{desc}" }
                }
            }
            div { class: "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
                Button { variant: ButtonVariant::Outline, onclick: move |_| open.set(false), "{cancel_text}" }
                Button {
                    id: "confirm-dialog-confirm",
                    variant: if destructive { ButtonVariant::Destructive } else { ButtonVariant::Default },
                    disabled,
                    onclick: move |_| on_confirm.call(()),
                    "{confirm_text}"
                }
            }
        }
    }
}

/// `hooks/use-dialog-state.tsx` for Dioxus: the feature provider keeps one
/// `Signal<Option<D>>` ("which dialog is open"); each dialog gets a `bool` signal that follows
/// it, and closing the dialog clears the provider state.
pub fn use_dialog_flag<D: PartialEq + Copy + 'static>(
    open: Signal<Option<D>>,
    which: D,
) -> Signal<bool> {
    let mut open = open;
    let mut flag = use_signal(|| false);
    use_effect(move || flag.set(open() == Some(which)));
    use_effect(move || {
        if !flag() && *open.peek() == Some(which) {
            open.set(None);
        }
    });
    flag
}
