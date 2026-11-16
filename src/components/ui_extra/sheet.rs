//! Side sheet (shadcn `sheet.tsx` look), signal-controlled. Used for the mobile sidebar
//! and the appearance drawer.
use dioxus::prelude::*;
use tw_merge::tw_merge;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetSide {
    #[default]
    End,
    Start,
}

#[component]
pub fn SideSheet(
    open: Signal<bool>,
    #[props(default)] side: SheetSide,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] style: Option<String>,
    children: Element,
) -> Element {
    let mut open = open;
    if !open() {
        return rsx! {};
    }
    let side_class = match side {
        SheetSide::End => "inset-y-0 end-0 h-full w-3/4 border-s slide-in-from-end sm:max-w-sm",
        SheetSide::Start => {
            "inset-y-0 start-0 h-full w-3/4 border-e slide-in-from-start sm:max-w-sm"
        }
    };
    let class = tw_merge!(
        "fixed z-[61] flex flex-col gap-4 bg-background shadow-lg animate-in duration-500",
        side_class,
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div { class: "fixed inset-0 z-[60] bg-black/50 animate-in fade-in-0", onclick: move |_| open.set(false) }
        div {
            role: "dialog",
            "aria-modal": "true",
            tabindex: "-1",
            class: "{class}",
            style: style.unwrap_or_default(),
            onmounted: move |e| async move { let _ = e.data().set_focus(true).await; },
            onkeydown: move |e| if e.key() == Key::Escape { open.set(false) },
            {children}
        }
    }
}
