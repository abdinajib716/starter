use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;

use crate::api::i18n::set_locale;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};

/// Language buttons (§26 UI). Hidden in M7 when `i18n.enabled = false`.
#[component]
pub fn LocaleSwitch() -> Element {
    let mut i18n = i18n();
    let current = i18n.language().to_string();
    rsx! {
        div { class: "flex items-center gap-1",
            for code in crate::i18n::SUPPORTED.iter().copied() {
                Button {
                    id: "lang-{code}",
                    size: ButtonSize::Sm,
                    variant: if code == current { ButtonVariant::Secondary } else { ButtonVariant::Ghost },
                    onclick: move |_| async move {
                        i18n.set_language(code.parse().expect("valid locale"));
                        let _ = set_locale(code.to_string()).await;
                    },
                    "{code}"
                }
            }
        }
    }
}
