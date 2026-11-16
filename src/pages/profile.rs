//! `/profile` (§9.1). M1: change password, required when `must_change_pw` (M1.13).
//! M9 adds the Profile / Security / Sessions / API Tokens / Language tabs.
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::auth::change_password;
use crate::components::layout::UserResource;
use crate::i18n::error_text;
use crate::ui::alert::{Alert, AlertDescription, AlertVariant};
use crate::ui::button::Button;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::input::{Input, InputType};
use crate::ui::label::Label;
use crate::ui::toast::{ToastType, show_toast};

#[component]
pub fn Profile() -> Element {
    let mut user = use_context::<UserResource>();
    let must_change = user
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(|u| u.as_ref())
        .is_some_and(|u| u.must_change_pw);

    let mut current = use_signal(String::new);
    let mut new = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    let submit = move |evt: FormEvent| {
        // Synchronously, before the browser performs its default form submission.
        evt.prevent_default();
        async move {
            if busy() {
                return;
            }
            if new() != confirm() {
                error.set(Some(t!("password-mismatch")));
                return;
            }
            busy.set(true);
            error.set(None);
            match change_password(current(), new()).await {
                Ok(()) => {
                    current.set(String::new());
                    new.set(String::new());
                    confirm.set(String::new());
                    show_toast(ToastType::Success, t!("profile-password-changed"));
                    // `must_change_pw` is now false: refresh the shared user.
                    user.restart();
                }
                Err(e) => error.set(Some(error_text(&e))),
            }
            busy.set(false);
        }
    };

    let field = |id: &'static str, label: String, mut sig: Signal<String>| {
        rsx! {
            div { class: "flex flex-col gap-2",
                Label { html_for: id, "{label}" }
                Input {
                    id,
                    name: id,
                    r#type: InputType::Password,
                    required: true,
                    value: sig(),
                    oninput: move |e: FormEvent| sig.set(e.value()),
                }
            }
        }
    };

    rsx! {
        div { class: "mx-auto flex max-w-md flex-col gap-4",
            if must_change {
                Alert { span { id: "must-change", {t!("profile-must-change")} } }
            }
            Card {
                CardHeader {
                    CardTitle { {t!("profile-change-password")} }
                    CardDescription { {t!("password-hint")} }
                }
                CardContent {
                    form { class: "flex flex-col gap-4", onsubmit: submit,
                        if let Some(msg) = error() {
                            Alert { variant: AlertVariant::Destructive,
                                AlertDescription { span { id: "profile-error", "{msg}" } }
                            }
                        }
                        {field("current-password", t!("field-current-password"), current)}
                        {field("new-password", t!("field-new-password"), new)}
                        {field("confirm-password", t!("field-confirm-password"), confirm)}
                        Button { id: "profile-save", button_type: "submit", disabled: busy(),
                            {t!("profile-save")}
                        }
                    }
                }
            }
        }
    }
}
