//! `/login` (§7, M1.11). Public; the server guard sends signed-in users to `/`.
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::auth::login;
use crate::app::Route;
use crate::components::layout::LocaleSwitch;
use crate::domain::AfterLogin;
use crate::i18n::error_text;
use crate::ui::alert::{Alert, AlertDescription, AlertVariant};
use crate::ui::button::Button;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::input::{Input, InputType};
use crate::ui::label::Label;
use crate::ui::theme_toggle::ThemeToggle;

#[component]
pub fn Login(next: String) -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        let next = next.clone();
        async move {
            if busy() {
                return;
            }
            busy.set(true);
            error.set(None);
            match login(email(), password()).await {
                Ok(AfterLogin::ChangePassword) => {
                    navigator().replace(Route::Profile {});
                }
                Ok(AfterLogin::Home) => {
                    // Only follow internal paths; anything unparsable goes home.
                    let target = next.parse::<Route>().unwrap_or(Route::Home {});
                    navigator().replace(target);
                }
                Err(e) => {
                    password.set(String::new());
                    error.set(Some(error_text(&e)));
                }
            }
            busy.set(false);
        }
    };

    rsx! {
        div { class: "flex min-h-full flex-col items-center justify-center gap-4 bg-background p-4 text-foreground",
            div { class: "flex w-full max-w-sm justify-end gap-1", LocaleSwitch {} ThemeToggle {} }
            Card { class: "w-full max-w-sm",
                CardHeader {
                    CardTitle { {t!("login-title")} }
                    CardDescription { {t!("login-subtitle")} }
                }
                CardContent {
                    form { class: "flex flex-col gap-4", onsubmit: submit,
                        if let Some(msg) = error() {
                            Alert { variant: AlertVariant::Destructive,
                                AlertDescription { span { id: "login-error", "{msg}" } }
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "email", {t!("field-email")} }
                            Input {
                                id: "email",
                                name: "email",
                                r#type: InputType::Email,
                                required: true,
                                value: email(),
                                oninput: move |e: FormEvent| email.set(e.value()),
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            Label { html_for: "password", {t!("field-password")} }
                            Input {
                                id: "password",
                                name: "password",
                                r#type: InputType::Password,
                                required: true,
                                value: password(),
                                oninput: move |e: FormEvent| password.set(e.value()),
                            }
                        }
                        Button { id: "login-submit", button_type: "submit", disabled: busy(),
                            if busy() { {t!("login-submitting")} } else { {t!("login-submit")} }
                        }
                    }
                }
            }
        }
    }
}
