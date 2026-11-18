//! `/auth/2fa` (§21, M9.9): the second sign-in step after a correct password.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::AuthLayout;
use super::sign_in::SUBMIT;
use crate::api::auth::verify_2fa;
use crate::app::Route;
use crate::components::ui_extra::form::{FormItem, FormLabel, TextInput};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::AfterLogin;
use crate::i18n::error_text;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::toast;

#[component]
pub fn TwoFactor(redirect: String) -> Element {
    let code = use_signal(String::new);
    let mut recovery = use_signal(|| false);
    let mut busy = use_signal(|| false);

    let onsubmit = move |evt: FormEvent| {
        evt.prevent_default();
        let redirect = redirect.clone();
        async move {
            let mut c = code;
            if c().trim().is_empty() || busy() {
                return;
            }
            busy.set(true);
            match verify_2fa(c()).await {
                Ok(next) => {
                    let target = match next {
                        AfterLogin::ChangePassword => Route::Profile { tab: String::new() },
                        _ => redirect.parse::<Route>().unwrap_or(Route::Dashboard {}),
                    };
                    navigator().replace(target);
                }
                Err(e) => {
                    c.set(String::new());
                    let expired = crate::domain::errors::i18n_key(&e) == "err-twofa-expired";
                    toast(ToastType::Error, error_text(&e));
                    if expired {
                        navigator().replace(Route::SignIn {
                            redirect: redirect.clone(),
                            reason: String::new(),
                        });
                    }
                }
            }
            busy.set(false);
        }
    };

    rsx! {
        AuthLayout {
            Card { class: "max-w-sm gap-4",
                CardHeader {
                    CardTitle { class: "text-lg tracking-tight", {t!("twofa-title")} }
                    CardDescription {
                        if recovery() { {t!("twofa-desc-recovery")} } else { {t!("twofa-desc")} }
                    }
                }
                CardContent {
                    form { class: "grid gap-3", onsubmit,
                        FormItem {
                            FormLabel { html_for: "code", if recovery() { {t!("twofa-recovery-code")} } else { {t!("twofa-code")} } }
                            for r in [recovery()] {
                                TextInput { key: "{r}", id: "code", value: code,
                                    placeholder: if r { "xxxxx-xxxxx" } else { "123456" },
                                    class: "tracking-widest" }
                            }
                        }
                        button { id: "twofa-submit", r#type: "submit", disabled: busy(), class: tw_merge!(SUBMIT, "mt-2"),
                            if busy() { Icon { icon: IconType::LoaderCircle, class: "animate-spin" } } else { Icon { icon: IconType::ShieldCheck } }
                            {t!("twofa-verify")}
                        }
                        button { r#type: "button", class: "text-sm text-muted-foreground underline-offset-4 hover:underline",
                            onclick: move |_| { let mut c = code; c.set(String::new()); recovery.toggle(); },
                            if recovery() { {t!("twofa-use-app")} } else { {t!("twofa-use-recovery")} }
                        }
                    }
                }
            }
        }
    }
}
