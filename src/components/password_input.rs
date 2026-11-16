//! `password-input.tsx`: password field with a show/hide toggle.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::components::ui_extra::icon::{Icon, IconType};

#[component]
pub fn PasswordInput(
    value: Signal<String>,
    #[props(into, optional)] id: Option<String>,
    #[props(into, optional)] placeholder: Option<String>,
    #[props(default)] invalid: bool,
    #[props(default)] disabled: bool,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let mut value = value;
    let mut show = use_signal(|| false);
    rsx! {
        div { class: tw_merge!("relative rounded-md", class.as_deref().unwrap_or("")),
            input {
                id,
                r#type: if show() { "text" } else { "password" },
                placeholder,
                disabled,
                "aria-invalid": if invalid { "true" } else { "false" },
                class: "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs transition-colors placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive",
                value: value(),
                oninput: move |e| value.set(e.value()),
            }
            button {
                r#type: "button",
                disabled,
                class: "absolute end-1 top-1/2 inline-flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                onclick: move |_| { let v = show(); show.set(!v) },
                Icon { icon: if show() { IconType::Eye } else { IconType::EyeOff }, class: "size-[18px]" }
                span { class: "sr-only", if show() { {t!("pw-hide")} } else { {t!("pw-show")} } }
            }
        }
    }
}
