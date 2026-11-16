use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;

use super::ContentSection;
use crate::components::ui_extra::form::{
    CheckBox, FormDescription, FormItem, FormLabel, FormMessage,
};
use crate::ui::button::Button;
use crate::util::show_submitted_data;

const ITEMS: [(&str, &str); 6] = [
    ("recents", "Recents"),
    ("home", "Home"),
    ("applications", "Applications"),
    ("desktop", "Desktop"),
    ("downloads", "Downloads"),
    ("documents", "Documents"),
];

/// `display-form.tsx`: at least one item must stay selected.
#[component]
pub fn SettingsDisplay() -> Element {
    let mut items = use_signal(|| vec!["recents".to_string(), "home".to_string()]);
    let mut submitted = use_signal(|| false);
    let e_items = items().is_empty().then(|| t!("v-display-items"));
    let show = submitted();

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if !items().is_empty() {
            show_submitted_data(&json!({ "items": items() }), &t!("submitted-title"));
        }
    };

    rsx! {
        ContentSection { title: t!("nav-display"), desc: t!("settings-display-desc"),
            form { class: "space-y-8", onsubmit,
                FormItem {
                    div { class: "mb-4",
                        FormLabel { class: "text-base", {t!("f-display-sidebar")} }
                        FormDescription { {t!("f-display-sidebar-desc")} }
                    }
                    for (id, label) in ITEMS {
                        div { class: "flex flex-row items-start gap-3",
                            CheckBox {
                                id: "display-{id}",
                                checked: items().iter().any(|i| i == id),
                                onchange: move |on: bool| {
                                    let mut v = items.write();
                                    if on { v.push(id.to_string()) } else { v.retain(|i| i != id) }
                                },
                            }
                            FormLabel { html_for: "display-{id}", class: "font-normal", "{label}" }
                        }
                    }
                    FormMessage { message: e_items.filter(|_| show) }
                }
                Button { button_type: "submit", {t!("f-update-display")} }
            }
        }
    }
}
