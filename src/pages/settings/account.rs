use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;

use super::ContentSection;
use crate::components::ui_extra::form::{
    Combobox, DatePicker, FormDescription, FormItem, FormLabel, FormMessage, TextInput,
};
use crate::ui::button::Button;
use crate::util::show_submitted_data;

const LANGUAGES: [(&str, &str); 9] = [
    ("en", "English"),
    ("fr", "French"),
    ("de", "German"),
    ("es", "Spanish"),
    ("pt", "Portuguese"),
    ("ru", "Russian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("zh", "Chinese"),
];

/// `account-form.tsx`.
#[component]
pub fn SettingsAccount() -> Element {
    let name = use_signal(String::new);
    let dob = use_signal(|| None::<time::Date>);
    let language = use_signal(String::new);
    let mut submitted = use_signal(|| false);

    let e_name = {
        let n = name().chars().count();
        if n == 0 {
            Some(t!("v-name-required"))
        } else if n < 2 {
            Some(t!("v-name-min"))
        } else if n > 30 {
            Some(t!("v-name-max"))
        } else {
            None
        }
    };
    let e_dob = dob().is_none().then(|| t!("v-dob"));
    let e_lang = language().is_empty().then(|| t!("v-language"));
    let show = submitted();
    let valid = e_name.is_none() && e_dob.is_none() && e_lang.is_none();

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if valid {
            show_submitted_data(
                &json!({ "name": name(), "dob": dob().map(|d| d.to_string()), "language": language() }),
                &t!("submitted-title"),
            );
        }
    };

    rsx! {
        ContentSection { title: t!("nav-account"), desc: t!("settings-account-desc"),
            form { class: "space-y-8", onsubmit,
                FormItem {
                    FormLabel { html_for: "name", invalid: show && e_name.is_some(), {t!("f-name")} }
                    TextInput { id: "name", value: name, placeholder: t!("f-name-placeholder"), invalid: show && e_name.is_some() }
                    FormDescription { {t!("f-name-desc")} }
                    FormMessage { message: e_name.filter(|_| show) }
                }
                FormItem { class: "flex flex-col",
                    FormLabel { invalid: show && e_dob.is_some(), {t!("f-dob")} }
                    DatePicker { selected: dob, placeholder: t!("f-dob-placeholder") }
                    FormDescription { {t!("f-dob-desc")} }
                    FormMessage { message: e_dob.filter(|_| show) }
                }
                FormItem { class: "flex flex-col",
                    FormLabel { invalid: show && e_lang.is_some(), {t!("f-language")} }
                    Combobox {
                        value: language,
                        options: LANGUAGES.map(|(v, l)| (v.to_string(), l.to_string())).to_vec(),
                        placeholder: t!("f-language-placeholder"),
                        search_placeholder: t!("f-language-search"),
                        empty_text: t!("f-language-empty"),
                        class: "w-50",
                    }
                    FormDescription { {t!("f-language-desc")} }
                    FormMessage { message: e_lang.filter(|_| show) }
                }
                Button { button_type: "submit", {t!("f-update-account")} }
            }
        }
    }
}
