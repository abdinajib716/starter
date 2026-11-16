use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;
use tw_merge::tw_merge;

use super::ContentSection;
use crate::components::ui_extra::form::{FormDescription, FormItem, FormLabel};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::state::{Font, Theme, Ui};
use crate::ui::button::Button;
use crate::util::show_submitted_data;

/// `appearance-form.tsx`: font + light/dark, applied on submit.
#[component]
pub fn SettingsAppearance() -> Element {
    let ui = Ui::get();
    let mut font = use_signal(|| (ui.font)());
    let theme = use_signal(|| {
        if (ui.theme)() == Theme::Dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    });

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        if font() != (ui.font)() {
            ui.set_font(font());
        }
        if theme() != (ui.theme)() {
            ui.set_theme(theme());
        }
        show_submitted_data(
            &json!({ "theme": theme().as_str(), "font": font().as_str() }),
            &t!("submitted-title"),
        );
    };

    rsx! {
        ContentSection { title: t!("nav-appearance"), desc: t!("settings-appearance-desc"),
            form { class: "space-y-8", onsubmit,
                FormItem {
                    FormLabel { html_for: "font", {t!("f-font")} }
                    div { class: "relative w-max",
                        select {
                            id: "font",
                            class: "inline-flex h-9 w-50 appearance-none items-center rounded-md border bg-background px-4 py-2 text-sm font-normal capitalize shadow-xs transition-all hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-background dark:hover:bg-background",
                            value: font().as_str(),
                            onchange: move |e| font.set(Font::parse(&e.value())),
                            for f in Font::ALL {
                                option { value: f.as_str(), selected: f == font(), "{f.as_str()}" }
                            }
                        }
                        Icon { icon: IconType::ChevronDown, class: "absolute end-3 top-2.5 h-4 w-4 opacity-50" }
                    }
                    FormDescription { class: "font-manrope", {t!("f-font-desc")} }
                }
                FormItem {
                    FormLabel { {t!("theme-label")} }
                    FormDescription { {t!("f-theme-desc")} }
                    div { role: "radiogroup", class: "grid max-w-md grid-cols-2 gap-8 pt-2",
                        ThemeCard { theme, value: Theme::Light, label: t!("theme-light") }
                        ThemeCard { theme, value: Theme::Dark, label: t!("theme-dark") }
                    }
                }
                Button { button_type: "submit", {t!("f-update-preferences")} }
            }
        }
    }
}

/// The illustrated light/dark choice of the reference.
#[component]
fn ThemeCard(theme: Signal<Theme>, value: Theme, label: String) -> Element {
    let mut theme = theme;
    let on = theme() == value;
    let dark = value == Theme::Dark;
    let (outer, frame, block, bar) = if dark {
        (
            "bg-popover hover:bg-accent hover:text-accent-foreground",
            "bg-slate-950",
            "bg-slate-800",
            "bg-slate-400",
        )
    } else {
        (
            "hover:border-accent",
            "bg-[#ecedef]",
            "bg-white",
            "bg-[#ecedef]",
        )
    };
    rsx! {
        button {
            r#type: "button",
            role: "radio",
            "aria-checked": "{on}",
            class: "text-start",
            onclick: move |_| theme.set(value),
            div { class: tw_merge!("items-center rounded-md border-2 border-muted p-1", outer, if on { "border-primary" } else { "" }),
                div { class: "space-y-2 rounded-sm p-2 {frame}",
                    div { class: "space-y-2 rounded-md p-2 shadow-xs {block}",
                        div { class: "h-2 w-20 rounded-lg {bar}" }
                        div { class: "h-2 w-25 rounded-lg {bar}" }
                    }
                    for _ in 0..2 {
                        div { class: "flex items-center space-x-2 rounded-md p-2 shadow-xs {block}",
                            div { class: "h-4 w-4 rounded-full {bar}" }
                            div { class: "h-2 w-25 rounded-lg {bar}" }
                        }
                    }
                }
            }
            span { class: "block w-full p-2 text-center text-sm font-normal", "{label}" }
        }
    }
}
