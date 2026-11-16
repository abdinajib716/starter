use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;

use super::ContentSection;
use crate::components::ui_extra::form::{
    FormDescription, FormItem, FormLabel, FormMessage, SelectBox, TextArea, TextInput,
};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::util::show_submitted_data;

fn is_url(s: &str) -> bool {
    (s.starts_with("http://") || s.starts_with("https://")) && s.len() > 8 && !s.contains(' ')
}

/// `profile-form.tsx` with its zod rules (validated live after the first submit).
#[component]
pub fn SettingsProfile() -> Element {
    let username = use_signal(String::new);
    let email = use_signal(String::new);
    let bio = use_signal(|| "I own a computer.".to_string());
    let mut urls = use_signal(|| {
        vec![
            Signal::new("https://shadcn.com".to_string()),
            Signal::new("http://twitter.com/shadcn".to_string()),
        ]
    });
    let mut submitted = use_signal(|| false);

    let e_user = {
        let n = username().chars().count();
        if n == 0 {
            Some(t!("v-username-required"))
        } else if n < 2 {
            Some(t!("v-username-min"))
        } else if n > 30 {
            Some(t!("v-username-max"))
        } else {
            None
        }
    };
    let e_email = email().is_empty().then(|| t!("v-email-select"));
    let e_bio = {
        let n = bio().chars().count();
        if n < 4 {
            Some(t!("v-bio-min"))
        } else if n > 160 {
            Some(t!("v-bio-max"))
        } else {
            None
        }
    };
    let url_errs: Vec<Option<String>> = urls()
        .iter()
        .map(|u| (!is_url(&u())).then(|| t!("v-url")))
        .collect();
    let show = submitted();
    let valid = e_user.is_none()
        && e_email.is_none()
        && e_bio.is_none()
        && url_errs.iter().all(Option::is_none);

    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if valid {
            let list: Vec<_> = urls().iter().map(|u| json!({ "value": u() })).collect();
            show_submitted_data(
                &json!({ "username": username(), "email": email(), "bio": bio(), "urls": list }),
                &t!("submitted-title"),
            );
        }
    };

    rsx! {
        ContentSection { title: t!("nav-profile"), desc: t!("settings-profile-desc"),
            form { class: "space-y-8", onsubmit,
                FormItem {
                    FormLabel { html_for: "username", invalid: show && e_user.is_some(), {t!("f-username")} }
                    TextInput { id: "username", value: username, placeholder: "shadcn", invalid: show && e_user.is_some() }
                    FormDescription { {t!("f-username-desc")} }
                    FormMessage { message: e_user.filter(|_| show) }
                }
                FormItem {
                    FormLabel { invalid: show && e_email.is_some(), {t!("field-email")} }
                    SelectBox {
                        value: email,
                        invalid: show && e_email.is_some(),
                        placeholder: t!("f-email-placeholder"),
                        options: ["m@example.com", "m@google.com", "m@support.com"]
                            .map(|m| (m.to_string(), m.to_string()))
                            .to_vec(),
                    }
                    FormDescription {
                        {t!("f-email-desc-1")}
                        " "
                        Link { to: "/", {t!("f-email-desc-link")} }
                        "."
                    }
                    FormMessage { message: e_email.filter(|_| show) }
                }
                FormItem {
                    FormLabel { html_for: "bio", invalid: show && e_bio.is_some(), {t!("f-bio")} }
                    TextArea { id: "bio", value: bio, placeholder: t!("f-bio-placeholder"), class: "resize-none", invalid: show && e_bio.is_some() }
                    FormDescription {
                        {t!("f-bio-desc-1")}
                        " "
                        span { "@mention" }
                        " "
                        {t!("f-bio-desc-2")}
                    }
                    FormMessage { message: e_bio.filter(|_| show) }
                }
                div {
                    for (i, url) in urls().into_iter().enumerate() {
                        FormItem { class: if i != 0 { "mt-1.5" } else { "" },
                            FormLabel { class: if i != 0 { "sr-only" } else { "" }, {t!("f-urls")} }
                            FormDescription { class: if i != 0 { "sr-only" } else { "" }, {t!("f-urls-desc")} }
                            TextInput { value: url, invalid: show && url_errs.get(i).cloned().flatten().is_some() }
                            FormMessage { message: url_errs.get(i).cloned().flatten().filter(|_| show) }
                        }
                    }
                    Button {
                        button_type: "button",
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        class: "mt-2",
                        onclick: move |_| urls.write().push(Signal::new(String::new())),
                        {t!("f-add-url")}
                    }
                }
                Button { button_type: "submit", {t!("f-update-profile")} }
            }
        }
    }
}
