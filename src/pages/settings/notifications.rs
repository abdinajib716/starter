use dioxus::prelude::*;
use dioxus_i18n::t;
use serde_json::json;

use super::ContentSection;
use crate::components::ui_extra::form::{
    CheckBox, FormDescription, FormItem, FormLabel, FormMessage, Radio, Switch,
};
use crate::ui::button::Button;
use crate::util::show_submitted_data;

/// `notifications-form.tsx`.
#[component]
pub fn SettingsNotifications() -> Element {
    let kind = use_signal(String::new);
    let communication = use_signal(|| false);
    let marketing = use_signal(|| false);
    let social = use_signal(|| true);
    let security = use_signal(|| true);
    let mut mobile = use_signal(|| false);
    let mut submitted = use_signal(|| false);

    let e_type = kind().is_empty().then(|| t!("v-notify-type"));
    let show = submitted();
    let onsubmit = move |e: FormEvent| {
        e.prevent_default();
        submitted.set(true);
        if !kind().is_empty() {
            show_submitted_data(
                &json!({
                    "type": kind(), "mobile": mobile(), "communication_emails": communication(),
                    "social_emails": social(), "marketing_emails": marketing(), "security_emails": security(),
                }),
                &t!("submitted-title"),
            );
        }
    };

    rsx! {
        ContentSection { title: t!("nav-notifications"), desc: t!("settings-notifications-desc"),
            form { class: "space-y-8", onsubmit,
                FormItem { class: "relative space-y-3",
                    FormLabel { invalid: show && e_type.is_some(), {t!("f-notify-about")} }
                    div { role: "radiogroup", class: "flex flex-col gap-2",
                        for (value, key) in [("all", "f-notify-all"), ("mentions", "f-notify-mentions"), ("none", "f-notify-none")] {
                            div { class: "flex items-center gap-2",
                                Radio { value: kind, item: value, id: "notify-{value}" }
                                FormLabel { html_for: "notify-{value}", class: "font-normal", {t!(key)} }
                            }
                        }
                    }
                    FormMessage { message: e_type.filter(|_| show) }
                }
                div { class: "relative",
                    h3 { class: "mb-4 text-lg font-medium", {t!("f-email-notifications")} }
                    div { class: "space-y-4",
                        EmailRow { title: t!("f-comm-emails"), desc: t!("f-comm-emails-desc"), checked: communication }
                        EmailRow { title: t!("f-marketing-emails"), desc: t!("f-marketing-emails-desc"), checked: marketing }
                        EmailRow { title: t!("f-social-emails"), desc: t!("f-social-emails-desc"), checked: social }
                        EmailRow { title: t!("f-security-emails"), desc: t!("f-security-emails-desc"), checked: security, disabled: true }
                    }
                }
                FormItem { class: "relative flex flex-row items-start gap-3",
                    CheckBox { id: "mobile", checked: mobile(), onchange: move |v| mobile.set(v) }
                    div { class: "space-y-1 leading-none",
                        FormLabel { html_for: "mobile", {t!("f-mobile")} }
                        FormDescription {
                            {t!("f-mobile-desc-1")}
                            " "
                            Link { to: "/settings", class: "underline decoration-dashed underline-offset-4 hover:decoration-solid", {t!("f-mobile-desc-link")} }
                            " "
                            {t!("f-mobile-desc-2")}
                        }
                    }
                }
                Button { button_type: "submit", {t!("f-update-notifications")} }
            }
        }
    }
}

#[component]
fn EmailRow(
    title: String,
    desc: String,
    checked: Signal<bool>,
    #[props(default)] disabled: bool,
) -> Element {
    rsx! {
        div { class: "flex flex-row items-center justify-between rounded-lg border p-4",
            div { class: "space-y-0.5",
                FormLabel { class: "text-base", "{title}" }
                FormDescription { "{desc}" }
            }
            Switch { checked, disabled }
        }
    }
}
