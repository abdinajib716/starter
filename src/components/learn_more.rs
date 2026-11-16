//! `components/learn-more.tsx`: a small "?" button that opens a popover with help text. The
//! reference uses it only on its Clerk pages (removed, §5); kept for feature pages that need
//! inline help.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::Menu;

#[allow(dead_code)]
#[component]
pub fn LearnMore(
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] content_class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        Menu {
            close_on_click: false,
            class: tw_merge!("w-72 p-4 text-sm text-muted-foreground", content_class.as_deref().unwrap_or("")),
            trigger: rsx! {
                button {
                    r#type: "button",
                    class: tw_merge!("inline-flex size-5 items-center justify-center rounded-full border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30", class.as_deref().unwrap_or("")),
                    span { class: "sr-only", {t!("learn-more")} }
                    Icon { icon: IconType::CircleQuestionMark, class: "size-4 [&>circle]:hidden" }
                }
            },
            {children}
        }
    }
}
