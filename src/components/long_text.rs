//! `components/long-text.tsx`: truncated text that reveals the full value in a tooltip
//! (≥ sm) or a click popover (< sm), only when it actually overflows.
use dioxus::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn LongText(
    text: String,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] content_class: Option<String>,
) -> Element {
    let mut overflown = use_signal(|| false);
    let mut popover = use_signal(|| false);
    let class = tw_merge!("truncate", class.as_deref().unwrap_or(""));
    let content = content_class.unwrap_or_default();
    rsx! {
        div { class: "relative group/long",
            div {
                class: "{class}",
                onmounted: move |e| async move {
                    // scrollWidth > clientWidth  ⇔  text is cut off.
                    if let (Ok(size), Ok(rect)) = (e.data().get_scroll_size().await, e.data().get_client_rect().await) {
                        overflown.set(size.width > rect.width() + 0.5 || size.height > rect.height() + 0.5);
                    }
                },
                onclick: move |_| if overflown() { popover.toggle() },
                "{text}"
            }
            if overflown() {
                // Tooltip (hover, sm and up).
                div { role: "tooltip", class: "pointer-events-none absolute bottom-full start-0 z-[90] mb-1.5 hidden w-max max-w-xs rounded-md bg-foreground px-3 py-1.5 text-xs text-balance text-background group-hover/long:sm:block {content}",
                    p { "{text}" }
                }
                // Popover (click, below sm).
                if popover() {
                    div { class: "fixed inset-0 z-[70] sm:hidden", onclick: move |_| popover.set(false) }
                    div { class: "absolute top-full start-0 z-[71] mt-1 w-fit rounded-md border bg-popover p-4 text-popover-foreground shadow-md outline-hidden sm:hidden {content}",
                        p { "{text}" }
                    }
                }
            }
        }
    }
}
