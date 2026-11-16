use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, Menu, MenuItem};
use crate::state::{Theme, Ui};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};

/// `theme-switch.tsx`: Light / Dark / System.
#[component]
pub fn ThemeSwitch() -> Element {
    let ui = Ui::get();
    let current = (ui.theme)();
    let check =
        move |t: Theme| tw_merge!("ms-auto size-3.5", if current == t { "" } else { "hidden" });
    rsx! {
        Menu {
            align: Align::End,
            trigger: rsx! {
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Icon, class: "relative scale-95 rounded-full",
                    Icon { icon: IconType::Sun, class: "size-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90" }
                    Icon { icon: IconType::Moon, class: "absolute size-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0" }
                    span { class: "sr-only", {t!("theme-toggle")} }
                }
            },
            MenuItem { onclick: move |_| ui.set_theme(Theme::Light), {t!("theme-light")} Icon { icon: IconType::Check, class: check(Theme::Light) } }
            MenuItem { onclick: move |_| ui.set_theme(Theme::Dark), {t!("theme-dark")} Icon { icon: IconType::Check, class: check(Theme::Dark) } }
            MenuItem { onclick: move |_| ui.set_theme(Theme::System), {t!("theme-system")} Icon { icon: IconType::Check, class: check(Theme::System) } }
        }
    }
}
