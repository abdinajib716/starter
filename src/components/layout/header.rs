use dioxus::prelude::*;
use tw_merge::tw_merge;

use crate::components::ui_extra::sidebar::SidebarTrigger;
use crate::ui::button::ButtonVariant;

/// `header.tsx`: sidebar trigger, separator, then the page's header items. `fixed` makes it
/// sticky with a shadow and blur once the page scrolls.
#[component]
pub fn Header(
    #[props(default)] fixed: bool,
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    let mut offset = use_signal(|| 0.0_f64);
    use_effect(move || {
        if !fixed {
            return;
        }
        spawn(async move {
            let mut ev = document::eval(
                "document.addEventListener('scroll', () => dioxus.send(document.body.scrollTop || document.documentElement.scrollTop), { passive: true });",
            );
            while let Ok(v) = ev.recv::<f64>().await {
                if (v > 10.0) != (offset() > 10.0) {
                    offset.set(v);
                }
            }
        });
    });
    let scrolled = fixed && offset() > 10.0;
    let class = tw_merge!(
        "z-50 h-16",
        if fixed {
            "header-fixed peer/header sticky top-0 w-[inherit]"
        } else {
            ""
        },
        if scrolled { "shadow" } else { "shadow-none" },
        class.as_deref().unwrap_or("")
    );
    let inner = tw_merge!(
        "relative flex h-full items-center gap-3 p-4 sm:gap-4",
        if scrolled {
            "after:absolute after:inset-0 after:-z-10 after:bg-background/20 after:backdrop-blur-lg"
        } else {
            ""
        }
    );
    rsx! {
        header { class: "{class}",
            div { class: "{inner}",
                SidebarTrigger { variant: ButtonVariant::Outline, class: "max-md:scale-125" }
                div { role: "separator", "aria-orientation": "vertical", class: "h-6 w-px shrink-0 bg-border" }
                {children}
            }
        }
    }
}
